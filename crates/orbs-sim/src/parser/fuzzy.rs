//! Approximate string matching.
//!
//! Integer-only, like everything else the sim scores with. Floats would make
//! candidate ordering depend on rounding, and DESIGN.md §6 requires every
//! resolution be reproducible and explainable — a tie broken by the last bit of
//! an `f32` is neither.
//!
//! Two shapes of near-miss get handled differently on purpose:
//!
//! - **Prefixes are intentional.** A player typing `sur` for `survey` is
//!   abbreviating, not misspelling, and expects it to work. Prefixes score high
//!   and scale gently with how much of the word was given.
//! - **Typos are accidental.** `clarty` for `clarity` scores by edit distance
//!   against the longer word, so a one-character slip in a long word costs less
//!   than a one-character slip in a short one — which is what makes `rm` and `cd`
//!   safe from fuzzy collisions.

/// The score of an exact match. All similarity is on a `0..=EXACT` scale.
pub const EXACT: u32 = 1000;

/// Below this, two strings are not considered related at all.
///
/// Tuned so a single typo in a five-letter word still matches (800) while an
/// unrelated word of the same length does not.
pub const MIN_SIMILARITY: u32 = 600;

/// Longest word this module will compare character by character.
///
/// [`distance`] is O(left x right) and runs once per synonym and once per scene
/// noun, so an unbounded word makes one keystroke-completed line arbitrarily
/// expensive — 100k characters measured at 605 ms in release, a visible freeze
/// on the frame thread that §6 forbids outright. Nothing in the vocabulary or
/// the tower is close to this long; anything longer cannot be a near-miss for
/// something this short, so refusing to score it loses no resolution.
pub const MAX_WORD: usize = 64;

/// Shortest prefix that counts as an abbreviation rather than a coincidence.
///
/// At two characters, `ca` would prefix-match `cast` and `cat` equally, which is
/// a tie the player did not intend to create.
const MIN_PREFIX: usize = 3;

/// Floor for a prefix match, before coverage is added.
const PREFIX_FLOOR: u32 = 850;

/// How much of the remaining range coverage can earn.
const PREFIX_RANGE: u32 = EXACT - PREFIX_FLOOR;

/// How alike two words are, on a `0..=EXACT` scale.
#[must_use]
pub fn similarity(input: &str, target: &str) -> u32 {
    if input == target {
        return EXACT;
    }
    if input.is_empty() || target.is_empty() {
        return 0;
    }

    let input_len = input.chars().take(MAX_WORD + 1).count();
    let target_len = target.chars().take(MAX_WORD + 1).count();

    // Too long to be a typo for anything we know.
    if input_len > MAX_WORD || target_len > MAX_WORD {
        return 0;
    }

    if input_len >= MIN_PREFIX && target.starts_with(input) {
        let coverage = PREFIX_RANGE * u32::try_from(input_len).unwrap_or(u32::MAX)
            / u32::try_from(target_len).unwrap_or(1).max(1);
        return PREFIX_FLOOR + coverage;
    }

    let longest = u32::try_from(input_len.max(target_len))
        .unwrap_or(u32::MAX)
        .max(1);
    let distance = distance(input, target);
    // Saturating: `EXACT * distance` overflows u32 once distance passes ~4.29M,
    // which panics in debug builds. MAX_WORD makes that unreachable, but the
    // arithmetic should not depend on a constant declared elsewhere.
    EXACT.saturating_sub(EXACT.saturating_mul(distance) / longest)
}

/// Whether two words are close enough to be worth considering.
#[must_use]
pub fn is_near(input: &str, target: &str) -> bool {
    similarity(input, target) >= MIN_SIMILARITY
}

/// Levenshtein edit distance, in characters.
///
/// Two rolling rows rather than a full matrix: inputs are single words, so the
/// allocation matters more than the asymptotics.
#[must_use]
pub fn distance(left: &str, right: &str) -> u32 {
    if left == right {
        return 0;
    }

    // Bounded for the reason on [`MAX_WORD`]: this is the quadratic step.
    let left: Vec<char> = left.chars().take(MAX_WORD).collect();
    let right: Vec<char> = right.chars().take(MAX_WORD).collect();

    if left.is_empty() {
        return u32::try_from(right.len()).unwrap_or(u32::MAX);
    }
    if right.is_empty() {
        return u32::try_from(left.len()).unwrap_or(u32::MAX);
    }

    let mut previous: Vec<u32> = (0..=u32::try_from(right.len()).unwrap_or(u32::MAX)).collect();
    let mut current = vec![0u32; right.len() + 1];

    for (row, &left_char) in left.iter().enumerate() {
        current[0] = u32::try_from(row + 1).unwrap_or(u32::MAX);
        for (column, &right_char) in right.iter().enumerate() {
            let substitution = u32::from(left_char != right_char);
            current[column + 1] = (previous[column] + substitution)
                .min(previous[column + 1] + 1)
                .min(current[column] + 1);
        }
        core::mem::swap(&mut previous, &mut current);
    }

    previous[right.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_words_are_exact() {
        assert_eq!(similarity("survey", "survey"), EXACT);
        assert_eq!(distance("survey", "survey"), 0);
    }

    #[test]
    fn distance_counts_single_edits() {
        assert_eq!(distance("brew", "brow"), 1);
        assert_eq!(distance("brew", "brews"), 1);
        assert_eq!(distance("brew", "bew"), 1);
        assert_eq!(distance("kitten", "sitting"), 3);
    }

    #[test]
    fn distance_is_symmetric() {
        for (left, right) in [("decoct", "decant"), ("sift", "sit"), ("", "purge")] {
            assert_eq!(distance(left, right), distance(right, left));
        }
    }

    #[test]
    fn abbreviations_score_above_the_threshold() {
        // A player typing a prefix is abbreviating and expects it to work.
        for (input, target) in [
            ("sur", "survey"),
            ("dec", "decoct"),
            ("grim", "grimoire"),
            ("medit", "meditate"),
        ] {
            assert!(
                is_near(input, target),
                "{input} should abbreviate {target} (scored {})",
                similarity(input, target)
            );
        }
    }

    #[test]
    fn a_longer_prefix_scores_higher_than_a_shorter_one() {
        assert!(similarity("surve", "survey") > similarity("sur", "survey"));
    }

    #[test]
    fn single_typos_are_forgiven() {
        for (input, target) in [
            ("clarty", "clarity"),
            ("prge", "purge"),
            ("invok", "invoke"),
            ("inscribr", "inscribe"),
        ] {
            assert!(
                is_near(input, target),
                "{input} should reach {target} (scored {})",
                similarity(input, target)
            );
        }
    }

    #[test]
    fn unrelated_words_do_not_match() {
        for (input, target) in [
            ("survey", "decoct"),
            ("bind", "purge"),
            ("gates", "clarity"),
        ] {
            assert!(
                !is_near(input, target),
                "{input} should not reach {target} (scored {})",
                similarity(input, target)
            );
        }
    }

    #[test]
    fn short_shell_synonyms_do_not_collide() {
        // `cd`, `ls`, `rm`, and `vi` are two characters apart from many things.
        // Fuzzy matching them would make every typo a destructive command.
        for (input, target) in [("rm", "vi"), ("ls", "cd"), ("cd", "rm")] {
            assert!(
                !is_near(input, target),
                "{input} should not reach {target} (scored {})",
                similarity(input, target)
            );
        }
    }

    #[test]
    fn a_two_character_prefix_is_not_treated_as_an_abbreviation() {
        // "ca" prefixes both "cast" and "cat"; accepting it would manufacture a
        // tie the player never intended.
        assert!(similarity("ca", "cast") < PREFIX_FLOOR);
    }

    #[test]
    fn empty_input_matches_nothing() {
        assert_eq!(similarity("", "survey"), 0);
        assert_eq!(similarity("survey", ""), 0);
    }

    #[test]
    fn absurdly_long_words_are_refused_rather_than_scored() {
        // The cost guard. Without it a pasted paragraph is compared against
        // every synonym and every noun in the tower.
        let long = "a".repeat(MAX_WORD + 1);
        assert_eq!(similarity(&long, "attend"), 0);
        assert_eq!(similarity("attend", &long), 0);
    }

    #[test]
    fn distance_is_bounded_by_the_word_limit() {
        let long = "a".repeat(100_000);
        // Would be ~100k without the cap, and would overflow the score maths.
        assert!(distance(&long, "attend") <= u32::try_from(MAX_WORD).expect("small"));
    }

    #[test]
    fn scoring_never_exceeds_the_scale() {
        for (input, target) in [("a", "abcdefghijklmnop"), ("survey", "s"), ("x", "y")] {
            assert!(similarity(input, target) <= EXACT);
        }
    }
}
