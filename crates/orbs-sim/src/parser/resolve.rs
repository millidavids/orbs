//! The pipeline. DESIGN.md §6, steps 1–6.
//!
//! 1. Normalise. 2. Fuzzy match against vocabulary *and* entities that exist.
//! 3. Resolve against world state. 4. Score. 5. Disambiguate on ties.
//! 6. Suggest when nothing scores — **never a bare error**.
//!
//! Ranking is a total order: score first, then the verb's position in
//! [`Verb::ALL`], and nothing here consults an RNG. §6 anticipated a
//! [`RngStream::Parser`](crate::RngStream::Parser) for exact ties, but that
//! would make replay depend on how often the parser had been called.

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
/// Where you are is evidence about what you meant: in the laboratory, the only
/// place `grind` is a word at all, a player reaching for it means the mortar.
/// §7 already makes place decide which *nouns* resolve.
///
/// Sized like [`PHRASE_BONUS`]: it settles a tie without overturning a real
/// difference. An exactly-typed `grimoire` still beats a two-edit `grind` by
/// 400.
const DOMAIN_BONUS: u32 = 40;

/// The most readings a numbered prompt will offer.
const MAX_PROMPT: usize = 4;

/// The most verbs suggested when nothing resolves.
const MAX_SUGGESTIONS: usize = 3;

/// A resolution together with every reading that was considered.
///
/// §6 requires that *"full input, resolution, and candidate scores"* be logged
/// for every resolution, including the ones that succeeded: a clean win and a
/// narrow win look identical in [`Resolution`] alone.
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
    /// The augury's router (§6): a line this answers `true` for never reaches
    /// the model.
    ///
    /// Two conditions. The verb was typed, not guessed — [`fuzzy::EXACT`] or
    /// better, the domain bonus lifting an in-domain operation above the
    /// ceiling. And nothing was left over: a fuzzy *noun* is fine (`brew clarty`
    /// reaches `clarity` at 819), but an unexplained *word* means the reading
    /// did not account for what the player said. [`Candidate::argument_score`]
    /// folds both into one number, so [`Candidate::leftover`] is separate.
    ///
    /// `Elsewhere`, `InSpell` and `Incomplete` are outright though none runs a
    /// command: *"I do not know that word"* would lie about a word the game
    /// taught the player (§19). `Incomplete` had to be found by testing — a bare
    /// `sift` carries no candidates, so a rule consulting only those sent it to
    /// a reader that answers everything.
    #[must_use]
    pub fn reads_outright(&self) -> bool {
        match &self.resolution {
            Resolution::Elsewhere { .. } | Resolution::InSpell { .. } => true,
            // Two different `Incomplete`s, and only one is outright. Wanting one
            // more thing is not a failure to understand; a verb followed by
            // words it could not use is `Incomplete` too (§19), and `take the
            // husks out and throw them away` must reach a reader. The leftover
            // count separates them, and `TakesNothing` always has a word over.
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

/// The reading to run, of several a reader offered for one line: the reader's
/// first choice that resolves — unless that one left words unused, and a later
/// reading both uses every word it was handed and names something.
///
/// Nothing asked a reader's readings what [`Analysis::reads_outright`] asks a
/// typed line, so *"stir the alembic"* ran as `distil alembic` with the alembic
/// named and then ignored. [`Candidate::leftover`] is the answer.
///
/// `Sim::submit_reading` and the parser bench both choose with this, so the
/// bench measures the rule the game plays by.
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
            // The first choice runs when it used every word it was handed.
            // Refusing a bare verb this cost 33 corpus and 72 holdout lines —
            // `weave` for *"open the loom"* lost to `survey loom`.
            None if uses_every_word => return Some(reading),
            None => first = Some(reading),
            // A later reading replaces one that left words unused only by using
            // words itself: a bare verb is handed none, so letting it jump made
            // `[grind sage now, quit]` run `quit`.
            Some(_) if uses_every_word && names_something => return Some(reading),
            Some(_) => {}
        }
    }
    first
}

/// Resolve, keeping every scored reading for instrumentation.
#[must_use]
pub fn analyse(input: &str, scene: &Scene, mode: Mode) -> Analysis {
    // Spell words are answered before the matcher sees the line: `wait for the
    // mortar` opened the editor on an empty `mortar.spell`, and `repeat 3`
    // resolved to `undo`.
    //
    // Exact, never fuzzy: these are not in §6's vocabulary. A typo like `waat`
    // falls through and is answered as a typo.
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

    // The verb matcher only looks at the head, so leading filler goes first —
    // "please go to the laboratory" opens on a word no phrase starts with.
    // `skip_leading_filler` yields 0 when all is filler, so this is never empty.
    let words = &all[normalise::skip_leading_filler(&all)..];

    let (mut candidates, incomplete, elsewhere) = collect(words, scene);

    // A verb typed exactly beats a fuzzy reading of a different one, even from
    // another room: `grind sage` in the archive offered `sift sage archive.log`,
    // `grind` being two edits from `find`. "Not here" is the honest answer.
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
    // list to offer. Saying so beats falling through to Unresolved.
    //
    // The one belonging to the verb that won: a single `Option` kept whichever
    // synonym reached it first, so an earlier entry's fuzzy reading claimed the
    // slot and `quit gibberish` ran `quit` with the word thrown away.
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
    // allocated two Strings per comparison.
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

    // A verb that explained none of what followed it does not run. The sort
    // puts an exactly-typed verb first, so `grind gibberish` diverts here rather
    // than to the `sift` reading below it and is answered *"grind what?"*;
    // `status gibberish` diverts as `TakesNothing`, having no slot.
    //
    // Bare commands are untouched. Tying this to the `Incomplete` `collect`
    // recorded, rather than re-deriving the test, carries the exemption across:
    // `light athanor` records none, so it resolves as bare `kindle`.
    //
    // A siege runs it rather than refusing it — §6 gives the mode its own answer
    // to ambiguity. Only the no-slot refusal is waived.
    //
    // A reading that filled something diverts only to its own `Incomplete`:
    // `limn keystone xyzzy` fills `keystone` and its last slot refuses `xyzzy`.
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
    // One per verb, not one for the line: which verb wins is decided after this
    // returns, and the winner's own reason is the one worth answering with.
    let mut incomplete: Vec<Incomplete> = Vec::new();
    // The best-scoring verb that would have matched if its instrument were here.
    let mut elsewhere: Option<(u32, Verb)> = None;

    for synonym in SYNONYMS {
        // A per-instrument verb is only a word where its instrument is (§7,
        // `Scene::offers`). Skipped here rather than refused later, the reading
        // never exists out of its domain, so it can neither win a tie, capture a
        // typo, nor be offered in a numbered prompt.
        //
        // It is still *remembered*, so the answer can be "not here" — see
        // `Resolution::Elsewhere`.
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
                    // and left the file in the pattern's position.
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

                // Built once per verb, and only where it is wanted: the eager
                // form built the whole `Incomplete` — including the `Vec` from
                // `filled.arguments()` — on every synonym of every verb.
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
                // A verb whose argument explained nothing is `Incomplete`, not a
                // bare verb: an optional slot that cannot use its word sets no
                // `missing` and consumes nothing, so `verify gibberish` audited
                // the whole tower with the word discarded (§19).
                //
                // `Incomplete` is what makes refusing safe — dropping the
                // candidate lets a *fuzzy* reading of another verb swallow the
                // line. It is still pushed, so `ParseLog` keeps the near-miss
                // §6 wants.
                //
                // The exemptions. Naming the instrument you are operating is
                // not an unexplained word: `light athanor` fills nothing —
                // `kindle` takes fuel, the athanor is a place — yet bare
                // `kindle` is right. A verb that takes nothing is the same case
                // with no slot to name (`Resolution::TakesNothing`).
                //
                // Naming where it acts is exempt, `Verb::anchor` saying where
                // that is: a fixture's verb has a place to name, one the whole
                // tower answers to has the tower and nothing narrower.
                // Unbounded, it let `status laboratory` run with the word
                // discarded.
                //
                // A plain-English *phrase* is not refused, talk running past the
                // command. A phrase, not the register — exempting every plain
                // synonym let `decode gibberish` run `research`.
                //
                // Neither filler nor punctuation is a word handed over (`?` and
                // `./` are synonyms in their own right), so *"status please"*
                // and `status .` were both refused. A word says something when
                // it has a letter or a digit in it.
                let explained = filled.slots.iter().any(Option::is_some);
                let said_something = tail.iter().any(|word| {
                    word.matching.chars().any(char::is_alphanumeric)
                        && !normalise::is_filler(word.matching)
                });
                if !explained && said_something {
                    let folded: Vec<&str> = tail.iter().map(|word| word.matching).collect();
                    // The whole tail, not a word of it: `best_match` tries the
                    // joined phrase *and* each word, so `undo laboratory move`
                    // found `laboratory` and called the line a place.
                    // `NounMatch::words` is what tells those apart.
                    let place = scene
                        .best_match(NounKind::Place, &folded)
                        .filter(|place| place.words == folded.len());
                    let names_a_place = place.is_some();
                    // Named so the guard below reads as the rule. Where a verb
                    // acts is its anchor's, or the whole tower — *"overview of
                    // the tower"* was refused as a word discarded.
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
                // A word the last slot could not use is asked about rather than
                // dropped, where the verb bare is a different act — `limn
                // keystone xyzzy` stepped the glyph to a humour nobody named
                // (§19). Filled slots are kept, so the prompt echoes back.
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
/// By shape, not by spelling: an absolute path of one segment. A bare leaf
/// (`alembic`, as `corpus_scene` registers an instrument) has no leading slash.
fn is_the_tower(path: &str) -> bool {
    path.strip_prefix('/')
        .is_some_and(|rest| !rest.is_empty() && !rest.contains('/'))
}

/// Everything in the scene that could fill a slot of `kind`.
///
/// [`NounKind::Any`] matches every noun — `verify` and `purge` reach all four
/// sabotage surfaces (§8.1). [`NounKind::Pattern`] and [`NounKind::Count`] are
/// free text and a number, so they yield no fillers and the caller reports
/// [`Resolution::Incomplete`] instead.
fn fillers(kind: NounKind, slot: usize, scene: &Scene) -> Vec<super::intent::Argument> {
    // `Name` joins them: a spell being coined does not exist, so a numbered
    // prompt would list things the player is explicitly *not* naming.
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
    // Keyed on the intent rather than its rendered echo: identical echoes mean
    // identical (verb, arguments), and building the String allocated twice.
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
/// considered. Without this `take clarity` resolved to `decoct clarity`, though
/// `take` *is* siphon. Argument fit still decides within a tier.
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
