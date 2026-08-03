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
use super::scene::Scene;
use super::verb::{NounKind, Verb};

/// How much each leftover word costs, once every slot is filled.
///
/// Extra words mean the player said something the reading did not account for,
/// which is weak evidence against that reading.
const LEFTOVER_PENALTY: u32 = 120;

/// The result of trying to fill one verb's signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Filled {
    /// Slots that were filled, in signature order.
    pub arguments: Vec<Argument>,
    /// Mean fill quality across required slots, on [`EXACT`]'s scale.
    pub score: u32,
    /// The first required slot that could not be filled, if any.
    pub missing: Option<NounKind>,
}

/// Fill `verb`'s signature from `words`.
pub(super) fn fill(verb: Verb, words: &[&str], scene: &Scene) -> Filled {
    let signature = verb.signature();
    if signature.is_empty() {
        // A verb that takes nothing still pays for words it cannot explain, so
        // `status alembic` does not outrank a reading that uses "alembic".
        return Filled {
            arguments: Vec::new(),
            score: penalise(EXACT, words.len()),
            missing: None,
        };
    }

    let mut arguments = Vec::with_capacity(signature.len());
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
                arguments.push(argument);
                scores.push(score);
                remaining = tail;
            }
            None => {
                if slot.required && missing.is_none() {
                    missing = Some(slot.kind);
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
        arguments,
        score: penalise(mean, remaining.len()),
        missing,
    }
}

/// Fill a single slot, or report that nothing here fits it.
fn fill_one(kind: NounKind, words: &[&str], scene: &Scene) -> Option<(Argument, u32)> {
    if words.is_empty() {
        return None;
    }

    match kind {
        // Free text: whatever was typed is the pattern.
        NounKind::Pattern => Some((
            Argument {
                kind,
                value: words.join(" "),
            },
            EXACT,
        )),
        NounKind::Count => {
            let value: u64 = words.first()?.parse().ok()?;
            Some((
                Argument {
                    kind,
                    value: value.to_string(),
                },
                EXACT,
            ))
        }
        _ => {
            let found = scene.best_match(kind, words)?;
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

    fn tower() -> Scene {
        Scene::new()
            .with(NounKind::Place, "/tower/alembic")
            .with(NounKind::File, "feed.log")
            .with(NounKind::Essence, "clarity")
            .with(NounKind::Script, "night_watch")
    }

    #[test]
    fn a_single_slot_takes_every_remaining_word() {
        let filled = fill(Verb::Attend, &["alembic"], &tower());
        assert_eq!(filled.arguments.len(), 1);
        assert_eq!(filled.arguments[0].value, "/tower/alembic");
        assert_eq!(filled.score, EXACT);
        assert!(filled.missing.is_none());
    }

    #[test]
    fn sift_splits_a_free_text_pattern_from_a_real_file() {
        let filled = fill(Verb::Sift, &["march", "feed.log"], &tower());
        assert_eq!(filled.arguments.len(), 2);
        assert_eq!(filled.arguments[0].kind, NounKind::Pattern);
        assert_eq!(filled.arguments[0].value, "march");
        assert_eq!(filled.arguments[1].kind, NounKind::File);
        assert_eq!(filled.arguments[1].value, "feed.log");
        assert!(filled.missing.is_none());
    }

    #[test]
    fn a_pattern_need_not_exist_but_a_file_must() {
        // The pattern is what you are looking for; the file is where you look.
        let filled = fill(Verb::Sift, &["nonexistent", "nowhere.log"], &tower());
        assert_eq!(filled.arguments[0].value, "nonexistent");
        assert_eq!(filled.missing, Some(NounKind::File));
    }

    #[test]
    fn counts_parse_as_numbers() {
        let filled = fill(Verb::Meditate, &["30"], &tower());
        assert_eq!(filled.arguments[0].value, "30");
        assert!(filled.missing.is_none());
    }

    #[test]
    fn a_count_that_is_not_a_number_is_missing() {
        let filled = fill(Verb::Meditate, &["awhile"], &tower());
        assert_eq!(filled.missing, Some(NounKind::Count));
    }

    #[test]
    fn a_required_slot_with_nothing_to_fill_it_is_reported() {
        // This is what turns into the numbered prompt of §6.
        let filled = fill(Verb::Decoct, &[], &tower());
        assert_eq!(filled.missing, Some(NounKind::Essence));
        assert!(filled.arguments.is_empty());
    }

    #[test]
    fn an_optional_slot_left_empty_is_not_missing() {
        let filled = fill(Verb::Survey, &[], &tower());
        assert!(filled.missing.is_none());
    }

    #[test]
    fn words_a_verb_cannot_explain_cost_it() {
        // `status` takes nothing, so trailing words are evidence against it.
        let clean = fill(Verb::Status, &[], &tower());
        let noisy = fill(Verb::Status, &["the", "alembic"], &tower());
        assert!(noisy.score < clean.score);
    }

    #[test]
    fn an_essence_that_does_not_exist_does_not_fill_its_slot() {
        let filled = fill(Verb::Decoct, &["haste"], &tower());
        assert_eq!(filled.missing, Some(NounKind::Essence));
    }
}
