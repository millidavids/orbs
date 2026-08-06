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
    let settle_incomplete = |candidates: Vec<Candidate>| {
        incomplete.as_ref().map_or_else(
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
                resolution: Resolution::Incomplete {
                    verb: incomplete.verb,
                    register: incomplete.register,
                    missing: incomplete.missing,
                    filled: incomplete.filled.clone(),
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
) -> (Vec<Candidate>, Option<Incomplete>, Option<(u32, Verb)>) {
    let mut candidates = Vec::new();
    let mut incomplete = None;
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
                    candidates.push(score(intent, verb_score, filled.score));
                }

                // `get_or_insert_with`, not `get_or_insert`: the eager form built
                // the whole `Incomplete` — including `filled.arguments()`, which
                // allocates a `Vec` — on **every** synonym past the first, then
                // threw it away because the slot was already taken. This runs once
                // per vocabulary entry per keystroke.
                if fillers(missing.kind, missing.index, scene).is_empty() {
                    incomplete.get_or_insert_with(|| Incomplete {
                        verb: synonym.verb,
                        register: synonym.register,
                        missing: missing.kind,
                        filled: filled.arguments(),
                    });
                }
            }
            None => {
                let intent = Intent {
                    verb: synonym.verb,
                    register: synonym.register,
                    arguments: filled.arguments(),
                };
                candidates.push(score(intent, verb_score, filled.score));
            }
        }
    }

    (dedupe(candidates), incomplete, elsewhere)
}

/// A verb that matched but whose empty slot cannot be offered as a list.
struct Incomplete {
    verb: Verb,
    register: Register,
    missing: NounKind,
    filled: Vec<super::intent::Argument>,
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
fn score(intent: Intent, verb_score: u32, argument_score: u32) -> Candidate {
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
