//! Step one of the pipeline: lowercase, strip filler.
//!
//! DESIGN.md §6. Two things make this less trivial than it sounds:
//!
//! - **Filler cannot be stripped before the verb is matched.** `to`, `for`,
//!   `of`, `do`, and `it` are all filler in an argument and all load-bearing in a
//!   verb phrase — `go to`, `look for`, `get rid of`, `how do i`, `take it back`.
//!   So [`tokenise`] keeps everything and [`strip_filler`] runs afterwards, on
//!   the argument tail only.
//! - **Punctuation is not uniformly noise.** `?` is a synonym for `grimoire` and
//!   `./` is one for `invoke`, while `feed.log` and `/tower/alembic` need their
//!   separators intact.

/// Words that carry no meaning in an argument.
///
/// Applied only after a verb phrase has been consumed, never before.
const FILLER: &[&str] = &[
    "the", "a", "an", "some", "my", "please", "that", "this", "of", "to", "for", "at", "in", "on",
    "with", "up", "it",
];

/// Trailing punctuation to shed from a word.
const TRAILING: &[char] = &['.', ',', '!', ';', ':', '?'];

/// Lowercase and tidy, preserving path and flag characters.
#[must_use]
pub fn normalise(input: &str) -> String {
    input.trim().to_lowercase()
}

/// Split normalised input into words, shedding trailing punctuation.
///
/// A word that is *entirely* punctuation is kept whole, so `?` and `./` survive
/// as the synonyms they are.
#[must_use]
pub fn tokenise(normalised: &str) -> Vec<&str> {
    normalised
        .split_whitespace()
        .map(|word| {
            if word.chars().count() <= 1 || word.chars().all(|c| TRAILING.contains(&c) || c == '/')
            {
                word
            } else {
                word.trim_end_matches(TRAILING)
            }
        })
        .filter(|word| !word.is_empty())
        .collect()
}

/// Whether a word carries no meaning on its own.
#[must_use]
pub fn is_filler(word: &str) -> bool {
    FILLER.contains(&word)
}

/// How many leading words to skip before looking for a verb.
///
/// `please go to the alembic` opens with filler, and the verb matcher only looks
/// at the head of the input — so this runs *before* matching while
/// [`strip_filler`] runs *after*. Safe because no synonym phrase begins with a
/// filler word, which `no_synonym_starts_with_filler` asserts.
///
/// Returns 0 when everything is filler, so the input is never emptied.
#[must_use]
pub fn skip_leading_filler(words: &[&str]) -> usize {
    words
        .iter()
        .position(|word| !is_filler(word))
        .unwrap_or_default()
}

/// Drop filler from an argument tail.
#[must_use]
pub fn strip_filler<'a>(words: &[&'a str]) -> Vec<&'a str> {
    let stripped: Vec<&str> = words
        .iter()
        .copied()
        .filter(|word| !FILLER.contains(word))
        .collect();

    // "attend the" should not become "attend". If filler was all there was, the
    // player did name something, and reporting an empty argument would send the
    // parser down the "no argument given" path instead of the "I could not find
    // that" path — a worse message for the same mistake.
    if stripped.is_empty() {
        words.to_vec()
    } else {
        stripped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(input: &str) -> Vec<String> {
        let normalised = normalise(input);
        tokenise(&normalised)
            .into_iter()
            .map(str::to_owned)
            .collect()
    }

    #[test]
    fn case_and_surrounding_space_are_flattened() {
        assert_eq!(words("  GO To The Gates  "), ["go", "to", "the", "gates"]);
    }

    #[test]
    fn trailing_punctuation_is_shed() {
        assert_eq!(words("brew clarity."), ["brew", "clarity"]);
        assert_eq!(words("how do i brew?"), ["how", "do", "i", "brew"]);
    }

    #[test]
    fn standalone_punctuation_synonyms_survive() {
        // `?` is grimoire and `./` is invoke (§6.1).
        assert_eq!(words("?"), ["?"]);
        assert_eq!(words("./ night_watch"), ["./", "night_watch"]);
    }

    #[test]
    fn paths_and_filenames_keep_their_separators() {
        assert_eq!(words("attend /tower/alembic"), ["attend", "/tower/alembic"]);
        assert_eq!(words("peruse feed.log"), ["peruse", "feed.log"]);
        assert_eq!(words("sift march feed.log"), ["sift", "march", "feed.log"]);
    }

    #[test]
    fn apostrophes_survive_for_phrase_matching() {
        // "what's here" is a survey synonym.
        assert_eq!(words("what's here"), ["what's", "here"]);
    }

    #[test]
    fn filler_is_dropped_from_arguments() {
        assert_eq!(
            strip_filler(&["a", "potion", "of", "clarity"]),
            ["potion", "clarity"]
        );
        assert_eq!(
            strip_filler(&["the", "castle", "gates"]),
            ["castle", "gates"]
        );
    }

    #[test]
    fn filler_words_that_are_also_phrase_words_survive_tokenising() {
        // These must still be present when verb matching runs, or `go to`,
        // `look for`, `get rid of`, and `take it back` all stop resolving.
        assert_eq!(words("go to alembic"), ["go", "to", "alembic"]);
        assert_eq!(words("get rid of sludge"), ["get", "rid", "of", "sludge"]);
        assert_eq!(words("take it back"), ["take", "it", "back"]);
    }

    #[test]
    fn an_argument_of_pure_filler_is_kept_rather_than_emptied() {
        assert_eq!(strip_filler(&["the"]), ["the"]);
    }

    #[test]
    fn empty_input_yields_no_tokens() {
        assert!(words("").is_empty());
        assert!(words("   ").is_empty());
    }
}
