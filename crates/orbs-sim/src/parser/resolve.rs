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

/// Extra credit for a verb whose instrument is standing right here, since
/// where you are is evidence about what you meant (§7).
///
/// Sized like [`PHRASE_BONUS`]: it settles a tie without overturning a real
/// difference.
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
    /// The verb was typed, not guessed — [`fuzzy::EXACT`] or better — and no
    /// word was left over ([`Candidate::leftover`]); a fuzzy noun is fine.
    ///
    /// `Elsewhere`, `InSpell` and `Incomplete` are outright though none runs a
    /// command: *"I do not know that word"* would lie about a word the game
    /// taught (§19).
    #[must_use]
    pub fn reads_outright(&self) -> bool {
        match &self.resolution {
            Resolution::Elsewhere { .. } | Resolution::InSpell { .. } => true,
            // Wanting one more thing is outright; a verb followed by words it
            // could not use is `Incomplete` too (§19), and must reach a reader.
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
            // The first choice runs when it used every word it was handed, bare
            // verbs included.
            None if uses_every_word => return Some(reading),
            None => first = Some(reading),
            // A later reading must use words itself, or `[grind sage now,
            // quit]` runs `quit`.
            Some(_) if uses_every_word && names_something => return Some(reading),
            Some(_) => {}
        }
    }
    first
}

/// Resolve, keeping every scored reading for instrumentation.
#[must_use]
pub fn analyse(input: &str, scene: &Scene, mode: Mode) -> Analysis {
    // Spell words are answered before the matcher sees the line, exactly, so
    // `waat` falls through and is answered as a typo.
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
    // another room: "not here" is the honest answer.
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

    // A verb matched but its slot has no list to offer; saying so beats
    // Unresolved. The winning verb's own reason, not the first synonym's.
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
    // order, then the arguments themselves.
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

    // A verb that explained none of what followed it does not run: `grind
    // gibberish` is *"grind what?"*. Tied to the `Incomplete` `collect`
    // recorded, so its exemptions carry across. A siege waives the no-slot
    // refusal (§6), and a reading that filled something diverts only to its
    // own `Incomplete`.
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
        // A per-instrument verb is only a word where its instrument is (§7), so
        // out of its domain it never wins, captures a typo or joins a prompt.
        // Remembered, so the answer can be `Resolution::Elsewhere`.
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
            // One candidate per filler, so the tie machinery produces §6's
            // numbered prompt rather than a special case.
            Some(missing) => {
                for filler in fillers(missing.kind, missing.index, scene) {
                    // Into the empty slot, keeping every slot that resolved.
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

                // Once per verb, and only where it is wanted.
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
                // bare verb, or `verify gibberish` audits everything (§19). The
                // candidate is still pushed, so a fuzzy reading of another verb
                // cannot swallow the line and `ParseLog` keeps the near-miss.
                //
                // Exempt: naming the instrument you operate (`light athanor`);
                // naming where the verb acts, per `Verb::anchor` — the tower,
                // for a tower-wide verb; a plain-English phrase running past
                // the command. Filler and punctuation say nothing: a word needs
                // a letter or a digit.
                let explained = filled.slots.iter().any(Option::is_some);
                let said_something = tail.iter().any(|word| {
                    word.matching.chars().any(char::is_alphanumeric)
                        && !normalise::is_filler(word.matching)
                });
                if !explained && said_something {
                    let folded: Vec<&str> = tail.iter().map(|word| word.matching).collect();
                    // The whole tail, not a word of it, or `undo laboratory
                    // move` names a place.
                    let place = scene
                        .best_match(NounKind::Place, &folded)
                        .filter(|place| place.words == folded.len());
                    let names_a_place = place.is_some();
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
                // dropped, where the verb bare is a different act (§19).
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
/// considered. Argument fit still decides within a tier.
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
