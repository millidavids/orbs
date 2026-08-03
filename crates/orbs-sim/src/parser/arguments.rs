//! Filling a verb's slots from the words that follow it.
//!
//! Two slot kinds never touch the world:
//!
//! - [`NounKind::Pattern`] is free text by definition — `sift march feed.log`
//!   searches for whatever the player typed, existing or not.
//! - [`NounKind::Count`] is a number — `meditate 30`.
//!
//! Everything else resolves against the live [`Scene`], which is what stops the
//! parser promising a brew the sim cannot perform.

use super::fuzzy::EXACT;
use super::intent::Argument;
use super::normalise::Word;
use super::scene::Scene;
use super::verb::{NounKind, Verb};

/// How much each leftover word costs, once every slot is filled.
///
/// Extra words mean the player said something the reading did not account for,
/// which is weak evidence against that reading.
const LEFTOVER_PENALTY: u32 = 120;

/// A required slot that nothing in the input could fill.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Missing {
    /// Its position in the verb's signature.
    ///
    /// Carried so the enumeration in `resolve` can put a candidate noun *into
    /// that slot* rather than rebuilding the argument list from scratch — which
    /// used to discard every slot that had already resolved and shift the rest
    /// into the wrong positions.
    pub index: usize,
    /// What the slot wants.
    pub kind: NounKind,
}

/// The result of trying to fill one verb's signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Filled {
    /// One entry per signature slot, `None` where nothing fitted.
    ///
    /// Positional rather than compacted, so a later slot resolving while an
    /// earlier one does not cannot silently renumber the arguments.
    pub slots: Vec<Option<Argument>>,
    /// Mean fill quality across filled slots, on [`EXACT`]'s scale.
    pub score: u32,
    /// The first required slot that could not be filled, if any.
    pub missing: Option<Missing>,
}

impl Filled {
    /// The slots that resolved, in signature order.
    pub fn arguments(&self) -> Vec<Argument> {
        self.slots.iter().flatten().cloned().collect()
    }
}

/// Fill `verb`'s signature from `words`.
pub(super) fn fill(verb: Verb, words: &[Word<'_>], scene: &Scene) -> Filled {
    let signature = verb.signature();
    if signature.is_empty() {
        // A verb that takes nothing still pays for words it cannot explain, so
        // `status alembic` does not outrank a reading that uses "alembic".
        return Filled {
            slots: Vec::new(),
            score: penalise(EXACT, words.len()),
            missing: None,
        };
    }

    let mut slots = vec![None; signature.len()];
    let mut scores = Vec::with_capacity(signature.len());
    let mut missing = None;
    let mut remaining = words;

    for (index, slot) in signature.iter().enumerate() {
        let is_last = index + 1 == signature.len();
        // Every slot but the last takes a single word; the last takes the rest,
        // so `attend castle gates` keeps both words for the place.
        let take = if is_last { remaining.len() } else { 1 };
        let (head, tail) = remaining.split_at(take.min(remaining.len()));

        match fill_one(slot.kind, head, scene) {
            Some((argument, score)) => {
                slots[index] = Some(argument);
                scores.push(score);
                remaining = tail;
            }
            None => {
                if slot.required && missing.is_none() {
                    missing = Some(Missing {
                        index,
                        kind: slot.kind,
                    });
                }
                // Do not consume words a slot could not use — a later slot may
                // still want them.
            }
        }
    }

    // An optional slot left empty is not a failure — `survey` with no place is
    // the commonest thing a player types. Scoring it zero dragged the whole
    // reading below the acceptance floor and made bare `survey` unresolvable the
    // moment the verb itself was fuzzy-matched.
    let mean = if scores.is_empty() {
        if missing.is_none() { EXACT } else { 0 }
    } else {
        let total: u32 = scores.iter().sum();
        total / u32::try_from(scores.len()).unwrap_or(1).max(1)
    };

    Filled {
        slots,
        score: penalise(mean, remaining.len()),
        missing,
    }
}

/// Fill a single slot, or report that nothing here fits it.
fn fill_one(kind: NounKind, words: &[Word<'_>], scene: &Scene) -> Option<(Argument, u32)> {
    if words.is_empty() {
        return None;
    }

    match kind {
        // Free text: whatever was typed, in the case and punctuation they typed
        // it in. Folding it for matching and then storing the folded form made
        // `sift ERROR feed.log` search for `error`.
        NounKind::Pattern => Some((
            Argument {
                kind,
                value: words
                    .iter()
                    .map(|word| word.raw)
                    .collect::<Vec<_>>()
                    .join(" "),
            },
            EXACT,
        )),
        NounKind::Count => {
            let value: u64 = words.first()?.matching.parse().ok()?;
            Some((
                Argument {
                    kind,
                    value: value.to_string(),
                },
                EXACT,
            ))
        }
        _ => {
            let folded: Vec<&str> = words.iter().map(|word| word.matching).collect();
            let found = scene.best_match(kind, &folded)?;
            Some((
                Argument {
                    kind: found.kind,
                    value: found.name,
                },
                found.score,
            ))
        }
    }
}

fn penalise(score: u32, leftovers: usize) -> u32 {
    let leftovers = u32::try_from(leftovers).unwrap_or(u32::MAX);
    score.saturating_sub(LEFTOVER_PENALTY.saturating_mul(leftovers))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests state input as plain words; the parser sees `Word`s.
    fn words(items: &[&'static str]) -> Vec<Word<'static>> {
        items
            .iter()
            .map(|item| Word {
                raw: item,
                matching: item,
            })
            .collect()
    }

    fn tower() -> Scene {
        Scene::new()
            .with(NounKind::Place, "/tower/alembic")
            .with(NounKind::File, "feed.log")
            .with(NounKind::Essence, "clarity")
            .with(NounKind::Script, "night_watch")
    }

    #[test]
    fn a_single_slot_takes_every_remaining_word() {
        let filled = fill(Verb::Attend, &words(&["alembic"]), &tower());
        assert_eq!(filled.arguments().len(), 1);
        assert_eq!(filled.arguments()[0].value, "/tower/alembic");
        assert_eq!(filled.score, EXACT);
        assert!(filled.missing.is_none());
    }

    #[test]
    fn sift_splits_a_free_text_pattern_from_a_real_file() {
        let filled = fill(Verb::Sift, &words(&["march", "feed.log"]), &tower());
        assert_eq!(filled.arguments().len(), 2);
        assert_eq!(filled.arguments()[0].kind, NounKind::Pattern);
        assert_eq!(filled.arguments()[0].value, "march");
        assert_eq!(filled.arguments()[1].kind, NounKind::File);
        assert_eq!(filled.arguments()[1].value, "feed.log");
        assert!(filled.missing.is_none());
    }

    #[test]
    fn a_pattern_need_not_exist_but_a_file_must() {
        // The pattern is what you are looking for; the file is where you look.
        let filled = fill(
            Verb::Sift,
            &words(&["nonexistent", "nowhere.log"]),
            &tower(),
        );
        assert_eq!(filled.arguments()[0].value, "nonexistent");
        let missing = filled.missing.expect("the file slot is empty");
        assert_eq!(missing.kind, NounKind::File);
        // Position matters: the enumeration puts a filler *here*, and getting it
        // wrong shifted the file into the pattern's slot.
        assert_eq!(missing.index, 1);
        assert!(filled.slots[0].is_some(), "the pattern must survive");
        assert!(filled.slots[1].is_none());
    }

    #[test]
    fn counts_parse_as_numbers() {
        let filled = fill(Verb::Meditate, &words(&["30"]), &tower());
        assert_eq!(filled.arguments()[0].value, "30");
        assert!(filled.missing.is_none());
    }

    #[test]
    fn a_count_that_is_not_a_number_is_missing() {
        let filled = fill(Verb::Meditate, &words(&["awhile"]), &tower());
        assert_eq!(filled.missing.map(|m| m.kind), Some(NounKind::Count));
    }

    #[test]
    fn a_required_slot_with_nothing_to_fill_it_is_reported() {
        // This is what turns into the numbered prompt of §6.
        let filled = fill(Verb::Decoct, &words(&[]), &tower());
        assert_eq!(filled.missing.map(|m| m.kind), Some(NounKind::Essence));
        assert!(filled.arguments().is_empty());
    }

    #[test]
    fn an_optional_slot_left_empty_is_not_missing() {
        let filled = fill(Verb::Survey, &words(&[]), &tower());
        assert!(filled.missing.is_none());
    }

    #[test]
    fn words_a_verb_cannot_explain_cost_it() {
        // `status` takes nothing, so trailing words are evidence against it.
        let clean = fill(Verb::Status, &words(&[]), &tower());
        let noisy = fill(Verb::Status, &words(&["the", "alembic"]), &tower());
        assert!(noisy.score < clean.score);
    }

    #[test]
    fn an_essence_that_does_not_exist_does_not_fill_its_slot() {
        let filled = fill(Verb::Decoct, &words(&["haste"]), &tower());
        assert_eq!(filled.missing.map(|m| m.kind), Some(NounKind::Essence));
    }
}
