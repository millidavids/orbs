//! The pipeline. DESIGN.md §6, steps 1–6.
//!
//! 1. Normalise. 2. Fuzzy match against vocabulary *and* entities that exist.
//! 3. Resolve against world state. 4. Score. 5. Disambiguate on ties.
//! 6. Suggest when nothing scores — **never a bare error**.
//!
//! # Determinism
//!
//! Ranking is a total order: score first, then the verb's position in
//! [`Verb::ALL`]. Nothing here consults an RNG. §6 anticipated a
//! [`RngStream::Parser`](crate::RngStream::Parser) for breaking exact ties, but a
//! random tie-break would make replay depend on how many times the parser had
//! been called, and would make "the parser must explain itself" impossible to
//! honour — the explanation for a coin flip is a coin flip. A total order gives
//! the same stability for free.

use super::arguments;
use super::fuzzy::{self, MIN_SIMILARITY};
use super::intent::{Candidate, Confidence, Intent, Mode, Resolution};
use super::normalise::{self, Word};
use super::scene::Scene;
use super::verb::{NounKind, Verb};
use super::vocabulary::{LONGEST_PHRASE, Register, SYNONYMS, Synonym};

/// Below this, a reading is not offered at all.
const MIN_ACCEPT: u32 = MIN_SIMILARITY;

/// Two readings this close are a tie, and a tie is a question.
const TIE_WINDOW: u32 = 60;

/// Extra credit per additional word in a matched phrase, so `go to` beats `go`.
const PHRASE_BONUS: u32 = 40;

/// Extra credit for a verb whose instrument is standing right here.
///
/// **Where you are is evidence about what you meant.** `grind` shares a prefix
/// with `grimoire` and sits two edits from `bind`; on the page those are three
/// words that could be confused, but in the laboratory — the only place `grind`
/// is a word at all — a player reaching for it is reaching for the mortar. §7
/// already makes place decide which *nouns* resolve; this is the same evidence
/// applied to the verb.
///
/// Sized like [`PHRASE_BONUS`] and for the same reason: it settles a tie without
/// overturning a real difference. An exactly-typed `grimoire` still beats a
/// two-edit `grind` by 400, and no bonus this side of absurd should change that.
/// What it does decide is the case where both readings are equally plausible,
/// and there the tool in front of you is the better guess.
const DOMAIN_BONUS: u32 = 40;

/// The most readings a numbered prompt will offer.
const MAX_PROMPT: usize = 4;

/// The most verbs suggested when nothing resolves.
const MAX_SUGGESTIONS: usize = 3;

/// A resolution together with every reading that was considered.
///
/// §6 requires that *"full input, resolution, and candidate scores"* be logged
/// for every resolution — including the ones that succeeded. A clean win and a
/// narrow win look identical in [`Resolution`] alone, and the difference is
/// exactly what the Phase 0 gate needs in order to cluster near-misses before
/// they become misses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Analysis {
    /// What the parser concluded.
    pub resolution: Resolution,
    /// Every scored reading, best first.
    pub candidates: Vec<Candidate>,
}

impl Analysis {
    /// Whether the orb read the line outright, or is guessing at it.
    ///
    /// **The augury's router** (§6). A line this answers `true` for is answered
    /// by the deterministic pipeline and the model never sees it; everything
    /// else is a phrasing the model gets a chance at, and tier three is this
    /// same analysis waiting behind it.
    ///
    /// # Two conditions, and both had to be found by building it
    ///
    /// **The verb was typed, not guessed.** [`fuzzy::EXACT`] or better — better,
    /// because the domain bonus lifts an in-domain operation above the ceiling,
    /// which is why this is the same `>=` the exactness tier-break already uses.
    ///
    /// **Nothing was left over.** A fuzzy *noun* is fine and must stay here:
    /// `brew clarty` reaches `clarity` at 819 and the matcher is better at that
    /// than any model trained on phrasings will be. An unexplained *word* is not
    /// fine, because it means the reading did not account for what the player
    /// said. [`Candidate::argument_score`] folds both into one number, which is
    /// why [`Candidate::leftover`] is carried separately.
    ///
    /// # Three answers are outright even though none of them runs a command
    ///
    /// `Elsewhere`, `InSpell` and `Incomplete` are all *right*, and §19 records
    /// each existing for the same reason: *"I do not know that word"* would lie
    /// about a word the game taught the player — in the room next door, in the
    /// editor, or in the very line they are typing. Handing one to a model
    /// trades a good answer for a guess.
    ///
    /// **`Incomplete` is the one that had to be found by testing.** A bare
    /// `sift` carries no candidates at all — the verb matched at full score and
    /// its free-text slot cannot be enumerated — so a rule that only consulted
    /// `candidates` sent it to a reader, and a reader that answers everything
    /// answered. The orb knowing the verb and wanting one more thing is not a
    /// failure to understand.
    #[must_use]
    pub fn reads_outright(&self) -> bool {
        match &self.resolution {
            Resolution::Elsewhere { .. } | Resolution::InSpell { .. } => true,
            // **Two different `Incomplete`s, and only one is an outright
            // reading.** A bare `sift` carries no candidates at all — the verb
            // matched and its free-text slot cannot be enumerated — and the orb
            // wanting one more thing is not a failure to understand. But a verb
            // followed by words it could not use is now `Incomplete` too (§19),
            // and that is a sentence: `take the husks out and throw them away`
            // must reach a reader rather than be answered *"take what?"*.
            //
            // The leftover count is what separates them, and it is the same
            // question the arm below asks. `TakesNothing` always has a word
            // over, so it is never outright: *"status report"* is a sentence a
            // reader should see before the orb says `status` takes nothing.
            Resolution::Incomplete { .. } | Resolution::TakesNothing { .. } => self
                .candidates
                .first()
                .is_none_or(|best| best.leftover == 0),
            Resolution::Resolved { .. }
            | Resolution::Ambiguous { .. }
            | Resolution::Unresolved { .. } => self
                .candidates
                .first()
                .is_some_and(|best| best.verb_score >= fuzzy::EXACT && best.leftover == 0),
        }
    }
}

/// Resolve one line of player input.
///
/// Never fails and never returns a bare error: the worst outcome is
/// [`Resolution::Unresolved`] carrying suggestions.
///
/// Use [`analyse`] when the candidate scores are wanted too.
#[must_use]
pub fn resolve(input: &str, scene: &Scene, mode: Mode) -> Resolution {
    analyse(input, scene, mode).resolution
}

/// The reading to run, of several a reader offered for one line: **the reader's
/// first choice that resolves** — unless that one left words unused, and a later
/// reading both uses every word it was handed and names something.
///
/// # The question `reads_outright` asks, asked of the readings too
///
/// A reader offers several because it cannot see the world, and the caller took
/// the first that resolved. [`Analysis::reads_outright`] already refuses to call
/// a *typed* line outright when a word is left over; nothing asked the same of
/// a reader's readings, so one that resolved with a word unused beat one behind
/// it that used them all — *"stir the alembic"* ran as `distil alembic`, the
/// alembic named and then ignored. [`Candidate::leftover`] is the answer.
///
/// `Sim::submit_reading` and the parser bench both choose with this, so the
/// bench measures the rule the game plays by rather than a looser one.
#[must_use]
pub fn reading_to_run<'a>(readings: &'a [String], scene: &Scene, mode: Mode) -> Option<&'a String> {
    // The reader's first choice that resolves, once one has and left words over.
    let mut first: Option<&'a String> = None;
    for reading in readings {
        let analysis = analyse(reading, scene, mode);
        if !analysis.resolution.is_resolved() {
            continue;
        }
        let best = analysis.candidates.first();
        let uses_every_word = best.is_some_and(|best| best.leftover == 0);
        let names_something = best.is_some_and(|best| !best.intent.arguments.is_empty());
        match first {
            // **The first choice runs when it used every word it was handed**,
            // argument or none — there is nothing unused for a later reading to
            // do better on. Refusing a bare verb this cost 33 corpus lines and
            // 72 of five trained readers' holdout lines to a later reading that
            // jumped it: `weave` for *"open the loom"* lost to `survey loom`.
            None if uses_every_word => return Some(reading),
            None => first = Some(reading),
            // **A later reading replaces a first that left words unused only by
            // using words itself.** A bare verb uses every word it was handed by
            // being handed none, so letting it jump was a sink: `[grind sage
            // now, quit]` ran `quit`. Every such jump was measured — none right
            // on any population, six of them displacing a right reading.
            Some(_) if uses_every_word && names_something => return Some(reading),
            Some(_) => {}
        }
    }
    first
}

/// Resolve, keeping every scored reading for instrumentation.
#[must_use]
pub fn analyse(input: &str, scene: &Scene, mode: Mode) -> Analysis {
    // **Spell words are answered before the matcher sees the line**, because the
    // matcher's answer is worse than no answer. `wait for the mortar` used to
    // open the editor on a new empty `mortar.spell` — `for`/`the` are filler,
    // `wait` is a `meditate` synonym whose `Count` slot cannot take `mortar`, so
    // the reading lost to `scribe <Name>`, which takes free text. `repeat 3`
    // resolved to `undo`.
    //
    // Exact, never fuzzy: these are not in §6's vocabulary and must not compete
    // with it. A typo like `waat` falls through and is answered as a typo.
    if let Some(word) = super::spellword::leading(input) {
        return Analysis {
            resolution: Resolution::InSpell { word },
            candidates: Vec::new(),
        };
    }

    let tokens = normalise::Tokens::split(input);
    let all = tokens.words();

    if all.is_empty() {
        return Analysis {
            resolution: Resolution::Unresolved {
                suggestions: Vec::new(),
            },
            candidates: Vec::new(),
        };
    }

    // The verb matcher only looks at the head, so leading filler has to go
    // first — "please go to the laboratory" opens on a word no phrase starts with.
    // `skip_leading_filler` yields 0 when everything is filler, so this slice is
    // never empty given `all` is not.
    let words = &all[normalise::skip_leading_filler(&all)..];

    let (mut candidates, incomplete, elsewhere) = collect(words, scene);

    // **A verb typed exactly beats a fuzzy reading of a different one**, even
    // when the one typed belongs to another room. Without this, `grind sage` in
    // the archive offered `sift sage archive.log` — `grind` is two edits from
    // `find`, which `sift` claims — so scoping the verb to its domain would have
    // *created* the silent misreading §19's naming pass exists to prevent
    // instead of preventing it. Saying "not here" is the honest answer to a word
    // the player knows.
    if let Some((score, verb)) = elsewhere
        && candidates
            .iter()
            .all(|candidate| candidate.verb_score < score)
    {
        return Analysis {
            resolution: Resolution::Elsewhere { verb },
            candidates,
        };
    }

    // A verb matched but its slot takes free text or a number, so there is no
    // list to offer. Saying so beats falling through to Unresolved, which used
    // to answer "I do not know that word" and then suggest the word just typed.
    // **The one belonging to the verb that won**, of however many `collect`
    // recorded. One `Option` kept whichever synonym reached it first, so an
    // earlier entry's fuzzy reading claimed the slot and the exactly-typed verb
    // below it had none: `quit gibberish` ran `quit` with the word thrown away,
    // because `verify`'s `audit` scores 600 against `quit` and sits above it.
    let settle_incomplete = |candidates: Vec<Candidate>| {
        let wanted = candidates
            .first()
            .and_then(|best| incomplete.iter().find(|held| held.verb == best.intent.verb))
            .or_else(|| incomplete.first());
        wanted.map_or_else(
            || Analysis {
                // A verb the player knows, in a room that does not answer to it,
                // beats suggesting three words they did not type.
                resolution: elsewhere.map_or_else(
                    || Resolution::Unresolved {
                        suggestions: suggest(words),
                    },
                    |(_, verb)| Resolution::Elsewhere { verb },
                ),
                candidates: candidates.clone(),
            },
            |incomplete| Analysis {
                resolution: match incomplete.missing {
                    Some(missing) => Resolution::Incomplete {
                        verb: incomplete.verb,
                        register: incomplete.register,
                        missing,
                        filled: incomplete.filled.clone(),
                    },
                    None => Resolution::TakesNothing {
                        verb: incomplete.verb,
                        register: incomplete.register,
                        extra: incomplete.extra.clone(),
                    },
                },
                candidates: candidates.clone(),
            },
        )
    };

    if candidates.is_empty() {
        return settle_incomplete(candidates);
    }

    // Total order: exact verb matches first, then score, then verb declaration
    // order, then the arguments themselves. Comparing rendered echoes here
    // allocated two Strings per comparison on a path documented as
    // sub-millisecond.
    candidates.sort_by(|a, b| {
        named_exactly(b)
            .cmp(&named_exactly(a))
            .then_with(|| b.score.cmp(&a.score))
            .then_with(|| verb_order(a.intent.verb).cmp(&verb_order(b.intent.verb)))
            .then_with(|| a.intent.arguments.cmp(&b.intent.arguments))
    });

    let best_score = candidates[0].score;
    if best_score < MIN_ACCEPT {
        return settle_incomplete(candidates);
    }

    // **A verb that explained none of what followed it does not run.** The sort
    // above puts an exactly-typed verb first, which is what makes this safe:
    // `grind gibberish` diverts here rather than falling to the `sift` reading
    // sitting below it, and `settle_incomplete` answers *"grind what?"* instead
    // of *"I do not know that word — perhaps grind"*. `status gibberish` diverts
    // the same way and is answered as `TakesNothing`, having no slot to ask for.
    //
    // Bare commands are untouched: `survey` has no arguments **and** nothing
    // left over. So is a reading that used part of what it was handed, which is
    // how `attend laboratory and then start the mortar` still reads.
    // **Tied to the `Incomplete` `collect` recorded**, rather than re-deriving
    // the test here. That is what carries the exemption across: `light athanor`
    // records none, so it resolves as bare `kindle` exactly as it always has.
    //
    // **A siege runs it rather than refusing it.** §6 gives the mode its own
    // answer to ambiguity — *"a modal prompt would make ambiguous phrasing cost
    // siege time"* — and a verb that takes nothing has the same shape: `muster
    // the troops` and `hold fast` are what a player types with the wall coming
    // down, and the words over cost them a turn to be told about. Only the
    // no-slot refusal is waived; a verb still asks for a slot it needs.
    //
    // **A reading that filled something diverts only to its own `Incomplete`.**
    // `limn keystone xyzzy` fills `keystone` and its last slot refuses `xyzzy`,
    // and the `Incomplete` recorded for it kept `keystone` — so the reading that
    // would step the glyph asks instead. A reading of the same verb that filled
    // something else is a different reading and is left to run.
    let diverts = incomplete.iter().any(|wanted| {
        wanted.verb == candidates[0].intent.verb
            && (candidates[0].intent.arguments.is_empty()
                || wanted.filled == candidates[0].intent.arguments)
            && !(wanted.missing.is_none() && mode == Mode::Siege)
    });
    if diverts && candidates[0].leftover > 0 {
        return settle_incomplete(candidates);
    }

    // A tie is only a tie within one exactness tier: an approximate reading is
    // never "close enough" to argue with a verb the player actually named.
    let best_exact = named_exactly(&candidates[0]);
    let tied = candidates
        .iter()
        .take_while(|candidate| {
            named_exactly(candidate) == best_exact
                && best_score.saturating_sub(candidate.score) <= TIE_WINDOW
        })
        .count();

    if tied <= 1 {
        return Analysis {
            resolution: Resolution::Resolved {
                intent: candidates[0].intent.clone(),
                confidence: Confidence::Clear,
            },
            candidates,
        };
    }

    let resolution = match mode {
        // §6: a modal prompt would make ambiguous phrasing cost siege time.
        Mode::Siege => Resolution::Resolved {
            intent: candidates[0].intent.clone(),
            confidence: Confidence::Forced,
        },
        Mode::Calm => Resolution::Ambiguous {
            candidates: candidates
                .iter()
                .take(tied.min(MAX_PROMPT))
                .cloned()
                .collect(),
        },
    };

    Analysis {
        resolution,
        candidates,
    }
}

/// Every reading worth scoring.
fn collect(
    words: &[Word<'_>],
    scene: &Scene,
) -> (Vec<Candidate>, Vec<Incomplete>, Option<(u32, Verb)>) {
    let mut candidates = Vec::new();
    // **One per verb, not one for the line.** Which verb wins is decided after
    // this returns, and the winner's own reason is the one worth answering with.
    let mut incomplete: Vec<Incomplete> = Vec::new();
    // The best-scoring verb that would have matched if its instrument were here.
    let mut elsewhere: Option<(u32, Verb)> = None;

    for synonym in SYNONYMS {
        // A per-instrument verb is only a word where its instrument is (§7, and
        // `Scene::offers`). Skipping it *here* rather than refusing later is what
        // makes the saving real: out of its domain the reading never exists, so
        // it can neither win a tie, capture a typo meant for another domain's
        // verb, nor be offered in a numbered prompt. That property is what keeps
        // the vocabulary safe to grow as §10's five further domains land.
        //
        // It is still *remembered*, so the answer can be "not here" rather than
        // "I do not know that word" — see `Resolution::Elsewhere`.
        if !scene.offers(synonym.verb) {
            if let Some((score, _)) = match_phrase(synonym, words)
                && elsewhere.is_none_or(|(best, _)| score > best)
            {
                elsewhere = Some((score, synonym.verb));
            }
            continue;
        }
        let Some((verb_score, consumed)) = match_phrase(synonym, words) else {
            continue;
        };
        // The instrument is here, so the verb that names it is the likelier
        // reading — see `DOMAIN_BONUS`.
        let verb_score = if synonym.verb.is_operation() {
            verb_score.saturating_add(DOMAIN_BONUS)
        } else {
            verb_score
        };

        let tail = normalise::strip_filler(&words[consumed..]);
        let filled = arguments::fill(synonym.verb, &tail, scene);

        match filled.missing {
            // §6's worked example: "start potion" -> "brew --recipe=?" -> a
            // numbered list of what could fill it. One candidate per filler, so
            // the tie machinery produces the prompt rather than a special case.
            Some(missing) => {
                for filler in fillers(missing.kind, missing.index, scene) {
                    // Into the empty slot, keeping every slot that resolved.
                    // Rebuilding the list from scratch dropped `sift`'s pattern
                    // and left the file sitting in the pattern's position.
                    let mut slots = filled.slots.clone();
                    if let Some(slot) = slots.get_mut(missing.index) {
                        *slot = Some(filler);
                    }
                    let intent = Intent {
                        verb: synonym.verb,
                        register: synonym.register,
                        arguments: slots.iter().flatten().cloned().collect(),
                    };
                    candidates.push(score(intent, verb_score, filled.score, filled.leftover));
                }

                // **Built once per verb**, and only where it is wanted: the
                // eager form built the whole `Incomplete` — including
                // `filled.arguments()`, which allocates a `Vec` — on every
                // synonym of every verb, then threw it away. This runs once per
                // vocabulary entry per keystroke.
                if fillers(missing.kind, missing.index, scene).is_empty()
                    && !incomplete.iter().any(|held| held.verb == synonym.verb)
                {
                    incomplete.push(Incomplete {
                        verb: synonym.verb,
                        register: synonym.register,
                        missing: Some(missing.kind),
                        filled: filled.arguments(),
                        extra: String::new(),
                    });
                }
            }
            None => {
                // **A verb whose argument explained nothing is `Incomplete`, not
                // a bare verb.** An optional slot that cannot use the word it
                // was handed sets no `missing` and consumes nothing, so `fill`
                // scores it as though the player had typed the verb alone — and
                // `score` weights the verb two thirds, so an exactly-typed one
                // clears `MIN_SIMILARITY` by itself. `verify gibberish` audited
                // the whole tower for twenty-one ticks and `digest husks` ran the
                // balneum, both with the player's word discarded (§19).
                //
                // **`Incomplete` is what makes refusing safe**, and its own doc
                // says why: dropping the candidate instead sends the line to
                // `Unresolved`, which answers *"I do not know that word"* and
                // then suggests the word just typed. It also lets a *fuzzy*
                // reading of a different verb take the line — `grind gibberish`
                // became `sift gibberish arsenal.log`, because `sift`'s free-text
                // slot swallows anything. Both were measured on the first
                // attempt at this and are why it was reverted.
                //
                // **The candidate is still pushed**, so `ParseLog` and `--tsv`
                // keep the reading. §6 wants the near-misses recorded, and this
                // is exactly the class that gets clustered.
                //
                // **Naming the instrument you are operating is not an
                // unexplained word.** `light athanor` fills nothing — `kindle`
                // takes fuel, and the athanor is a place — but bare `kindle` is
                // the right reading, and `light_the_athanor_lights_it_rather_
                // than_listing_it` pins it.
                //
                // **A verb that takes nothing is the same case with no slot to
                // name** (`Resolution::TakesNothing`): `status gibberish` ran
                // `status` and `undo gibberish` acknowledged, each with the word
                // thrown away.
                //
                // **Naming where it acts is exempt — where the verb acts
                // somewhere.** `Verb::anchor` is that question already: a verb a
                // fixture declares has a place to be named (`probe lens`,
                // `wander stacks`, `research lectern`), and one the whole tower
                // answers to has the tower and nothing narrower. Without it the
                // exemption covered `status
                // laboratory`, `quit laboratory` and `logout archive`, each of
                // which ran with the word discarded — the defect this is here to
                // end, wearing a place name.
                //
                // **And a plain-English *phrase* is not refused.** A sentence in
                // the plain register is how a newcomer talks, and talk runs past
                // the command: *"how are things going"* is the `status` synonym
                // *"how are things"* with a word after it, and refusing it
                // teaches nothing. The arcane and shell registers are exact, so
                // a word over one of those is a mistake worth naming.
                //
                // **A phrase, not the register.** Exempting every plain synonym
                // put the defect straight back: `decode` is one plain word for
                // `research`, and `decode gibberish` ran it with the word thrown
                // away. What carries the chatter is the sentence — so the
                // exemption is for a synonym the matcher took more than one word
                // of.
                //
                // **Filler is not a word handed over.** `strip_filler` never
                // empties what it is given, so a tail of nothing but filler —
                // *"status please"* — comes back whole, and asking only whether
                // the tail was empty refused a polite line as a stray word.
                // **Punctuation is not a word handed over either.** `fold` sheds
                // a *trailing* stop but keeps a token that is nothing else whole
                // — `?` and `./` are synonyms in their own right — so `status .`
                // arrived carrying `.`, which is no filler, and a line that had
                // always run was refused. A word says something when it has a
                // letter or a digit in it.
                let explained = filled.slots.iter().any(Option::is_some);
                let said_something = tail.iter().any(|word| {
                    word.matching.chars().any(char::is_alphanumeric)
                        && !normalise::is_filler(word.matching)
                });
                if !explained && said_something {
                    let folded: Vec<&str> = tail.iter().map(|word| word.matching).collect();
                    // **The whole tail, not a word of it.** `best_match` tries
                    // the joined phrase *and* each word, so `undo laboratory
                    // move` found `laboratory` and called the line a place;
                    // `NounMatch::words` is what tells those apart.
                    let place = scene
                        .best_match(NounKind::Place, &folded)
                        .filter(|place| place.words == folded.len());
                    let names_a_place = place.is_some();
                    // The two exemptions, named so the guard below reads as the
                    // rule: a verb naming where it acts — and a plain sentence
                    // running past its command.
                    //
                    // **Where a verb acts is its anchor's, or the whole tower.**
                    // One nothing anchors answers to every room at once, so the
                    // tower is the one place it can name: *"overview of the
                    // tower"* is `status` said about exactly what it reports on,
                    // and was refused as a word thrown away.
                    let acts_there = match synonym.verb.anchor() {
                        Some(_) => names_a_place,
                        None => place
                            .as_ref()
                            .is_some_and(|place| is_the_tower(&place.name)),
                    };
                    let a_sentence = synonym.register == Register::Plain && consumed > 1;
                    let remember = |incomplete: &mut Vec<Incomplete>, held: Incomplete| {
                        if !incomplete.iter().any(|kept| kept.verb == held.verb) {
                            incomplete.push(held);
                        }
                    };
                    match synonym.verb.signature().first() {
                        Some(slot) if !(synonym.verb.is_operation() && names_a_place) => {
                            remember(
                                &mut incomplete,
                                Incomplete {
                                    verb: synonym.verb,
                                    register: synonym.register,
                                    missing: Some(slot.kind),
                                    filled: Vec::new(),
                                    extra: String::new(),
                                },
                            );
                        }
                        None if !(acts_there || a_sentence) => {
                            remember(
                                &mut incomplete,
                                Incomplete {
                                    verb: synonym.verb,
                                    register: synonym.register,
                                    missing: None,
                                    filled: Vec::new(),
                                    extra: tail
                                        .iter()
                                        .map(|word| word.raw)
                                        .collect::<Vec<_>>()
                                        .join(" "),
                                },
                            );
                        }
                        _ => {}
                    }
                }
                // **A word the last slot could not use is asked about, not
                // dropped** — where the verb bare is a different act. `limn
                // keystone xyzzy` ran `limn keystone`, which steps the glyph to a
                // humour nobody named, and `dial first qqqq` turned the socket
                // (§19). The slots that did fill are kept, so the prompt shows
                // the player their own command back.
                if let Some(refused) = filled.refused
                    && !incomplete.iter().any(|kept| kept.verb == synonym.verb)
                {
                    incomplete.push(Incomplete {
                        verb: synonym.verb,
                        register: synonym.register,
                        missing: Some(refused.kind),
                        filled: filled.arguments(),
                        extra: String::new(),
                    });
                }
                let intent = Intent {
                    verb: synonym.verb,
                    register: synonym.register,
                    arguments: filled.arguments(),
                };
                candidates.push(score(intent, verb_score, filled.score, filled.leftover));
            }
        }
    }

    (dedupe(candidates), incomplete, elsewhere)
}

/// A verb that matched but cannot run as it stands: its empty slot cannot be
/// offered as a list, or it takes nothing and was handed words anyway.
struct Incomplete {
    verb: Verb,
    register: Register,
    /// What the empty slot wants — [`None`] for a verb with no slot to name.
    missing: Option<NounKind>,
    filled: Vec<super::intent::Argument>,
    /// The words handed to a verb that takes nothing, as the player typed them.
    extra: String,
}

/// Whether `path` names the place every other place is inside — `/tower`.
///
/// **By shape, not by spelling**: an absolute path of one segment. A bare leaf
/// (`alembic`, as `corpus_scene` registers an instrument) has no leading slash
/// and is not it.
fn is_the_tower(path: &str) -> bool {
    path.strip_prefix('/')
        .is_some_and(|rest| !rest.is_empty() && !rest.contains('/'))
}

/// Everything in the scene that could fill a slot of `kind`.
///
/// [`NounKind::Any`] matches every noun — `verify` and `purge` reach all four
/// sabotage surfaces (§8.1), and comparing kinds for equality silently excluded
/// every one of them. [`NounKind::Pattern`] and [`NounKind::Count`] are free text
/// and a number: nothing in the world enumerates them, so they yield no fillers
/// and the caller reports [`Resolution::Incomplete`] instead.
fn fillers(kind: NounKind, slot: usize, scene: &Scene) -> Vec<super::intent::Argument> {
    // `Name` joins them: a spell being coined does not exist, so the world has
    // nothing to offer and a numbered prompt would list things the player is
    // explicitly *not* naming.
    if matches!(kind, NounKind::Pattern | NounKind::Count | NounKind::Name) {
        return Vec::new();
    }
    scene
        .nouns()
        .iter()
        .filter(|noun| kind.accepts(noun.kind))
        .map(|noun| super::intent::Argument {
            kind: noun.kind,
            slot,
            value: noun.name.clone(),
        })
        .collect()
}

/// Combine the two halves into the number candidates are ranked by.
///
/// The verb is weighted double: a confident verb with a shaky argument is a
/// better guess than a shaky verb with a confident argument, because the
/// argument can be asked about and the verb cannot.
fn score(intent: Intent, verb_score: u32, argument_score: u32, leftover: usize) -> Candidate {
    let combined = if intent.verb.signature().is_empty() {
        verb_score.min(argument_score)
    } else {
        (verb_score.saturating_mul(2).saturating_add(argument_score)) / 3
    };
    Candidate {
        intent,
        verb_score,
        argument_score,
        score: combined,
        leftover,
    }
}

/// Match a synonym against the head of the input.
///
/// Returns its score and how many words it consumed. Longer phrases earn a bonus
/// so `go to` outranks `go` on the same input.
pub(super) fn match_phrase(synonym: &Synonym, words: &[Word<'_>]) -> Option<(u32, usize)> {
    let span = synonym.words.len();
    if span > words.len() || span > LONGEST_PHRASE {
        return None;
    }

    let mut total = 0u32;
    for (expected, actual) in synonym.words.iter().zip(words) {
        let score = fuzzy::similarity(actual.matching, expected);
        if score < MIN_SIMILARITY {
            return None;
        }
        total = total.saturating_add(score);
    }

    let mean = total / u32::try_from(span).unwrap_or(1).max(1);
    let bonus = PHRASE_BONUS.saturating_mul(u32::try_from(span - 1).unwrap_or(0));
    Some((mean.saturating_add(bonus).min(fuzzy::EXACT), span))
}

/// Collapse readings that would run the identical command, keeping the best.
fn dedupe(mut candidates: Vec<Candidate>) -> Vec<Candidate> {
    // Keyed on the intent itself rather than its rendered echo: identical echoes
    // mean identical (verb, arguments), and building the String to find that out
    // allocated twice per comparison.
    candidates.sort_by(|a, b| {
        a.intent
            .verb
            .cmp(&b.intent.verb)
            .then_with(|| a.intent.arguments.cmp(&b.intent.arguments))
            .then(b.score.cmp(&a.score))
    });
    candidates.dedup_by(|a, b| {
        a.intent.verb == b.intent.verb && a.intent.arguments == b.intent.arguments
    });
    candidates
}

/// What to offer when nothing resolved.
///
/// §6: never a bare error. Ranked by how close the first word came to each verb,
/// so a wild miss still gets pointed somewhere sensible.
fn suggest(words: &[Word<'_>]) -> Vec<Verb> {
    let Some(first) = words.first() else {
        return Vec::new();
    };

    let mut scored: Vec<(u32, Verb)> = Verb::ALL
        .iter()
        .map(|&verb| (fuzzy::similarity(first.matching, verb.canonical()), verb))
        .collect();

    scored.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| verb_order(a.1).cmp(&verb_order(b.1)))
    });
    scored
        .into_iter()
        .take(MAX_SUGGESTIONS)
        .map(|(_, verb)| verb)
        .collect()
}

/// Whether the player's words *named* this verb rather than approximating it.
///
/// An exact phrase match outranks every approximate one, before score is even
/// considered. Without this, a word that exactly names one verb could lose to a
/// word that merely resembles another, on the strength of the argument: `take
/// clarity` resolved to `decoct clarity` — brewing — because `take` reaches
/// `make` at 750 and `clarity` is an essence, even though `take` *is* siphon.
///
/// Argument fit still decides between readings of equal exactness.
const fn named_exactly(candidate: &Candidate) -> bool {
    candidate.verb_score >= fuzzy::EXACT
}

/// A verb's position in [`Verb::ALL`], for stable ordering.
fn verb_order(verb: Verb) -> usize {
    Verb::ALL
        .iter()
        .position(|&other| other == verb)
        .unwrap_or(usize::MAX)
}
