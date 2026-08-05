//! Step one of the pipeline: split, lowercase, strip filler.
//!
//! DESIGN.md §6. Three things make this less trivial than it sounds:
//!
//! - **Filler cannot be stripped before the verb is matched.** `to`, `for`,
//!   `of`, `do`, and `it` are all filler in an argument and all load-bearing in a
//!   verb phrase — `go to`, `look for`, `get rid of`, `how do i`, `take it back`.
//!   So [`tokenise`] keeps everything and [`strip_filler`] runs afterwards, on
//!   the argument tail only.
//! - **Punctuation is not uniformly noise.** `?` is a synonym for `grimoire` and
//!   `./` is one for `invoke`, while `feed.log` and `/tower/laboratory` need their
//!   separators intact.
//! - **Lowercasing must not destroy the input.** [`NounKind::Pattern`] is free
//!   text by definition, so `sift ERROR feed.log` has to search for `ERROR` and
//!   not `error`. Every token therefore carries **both** forms: `raw` as typed,
//!   and `matching` folded for comparison. They travel together as a [`Word`] so
//!   that filtering filler out of one cannot desynchronise it from the other.
//!
//! Quoted runs are held together, so `sift "march north" feed.log` searches for
//! a two-word phrase rather than for `"march`.

/// Words that carry no meaning in an argument.
///
/// Applied only after a verb phrase has been consumed, never before.
const FILLER: &[&str] = &[
    "the", "a", "an", "some", "my", "please", "that", "this", "of", "to", "for", "at", "in", "on",
    "with", "up", "it",
];

/// Trailing punctuation to shed from a word before matching it.
const TRAILING: &[char] = &['.', ',', '!', ';', ':', '?'];

/// The longest line the parser will consider.
///
/// Beyond this the input is not a command, and scoring it against every synonym
/// and every noun in the tower is work §6 has no budget for.
pub const MAX_INPUT: usize = 512;

/// The most words a line may contain, for the same reason.
pub const MAX_WORDS: usize = 32;

/// One token, in both the form the player typed and the form we compare.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Word<'a> {
    /// Exactly as typed, minus any surrounding quotes.
    pub raw: &'a str,
    /// Lowercased and stripped of trailing punctuation, for matching.
    pub matching: &'a str,
}

/// A tokenised line. Owns both forms; hand out [`Word`]s with [`Tokens::words`].
#[derive(Debug, Default, Clone)]
pub struct Tokens {
    raw: Vec<String>,
    matching: Vec<String>,
}

impl Tokens {
    /// Split a line into tokens, capped at [`MAX_INPUT`] and [`MAX_WORDS`].
    #[must_use]
    pub fn split(input: &str) -> Self {
        let trimmed: String = input.trim().chars().take(MAX_INPUT).collect();

        let mut raw = Vec::new();
        let mut rest = trimmed.as_str();
        while let Some(start) = rest.find(|c: char| !c.is_whitespace()) {
            rest = &rest[start..];
            let (token, remainder) = if let Some(body) = rest.strip_prefix('"') {
                match body.find('"') {
                    // A quoted run is one token, however many spaces it holds.
                    Some(end) => (&body[..end], &body[end + 1..]),
                    None => (body, ""),
                }
            } else {
                let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
                (&rest[..end], &rest[end..])
            };
            rest = remainder;

            if token.is_empty() {
                continue;
            }
            raw.push(token.to_owned());
            if raw.len() == MAX_WORDS {
                break;
            }
        }

        let matching = raw.iter().map(|token| fold(token)).collect();
        Self { raw, matching }
    }

    /// The tokens, in order.
    #[must_use]
    pub fn words(&self) -> Vec<Word<'_>> {
        self.raw
            .iter()
            .zip(&self.matching)
            .map(|(raw, matching)| Word {
                raw: raw.as_str(),
                matching: matching.as_str(),
            })
            .collect()
    }
}

/// Lowercase and shed trailing punctuation, for comparison only.
///
/// A token that is *entirely* punctuation is kept whole, so `?` and `./` survive
/// as the synonyms they are.
fn fold(token: &str) -> String {
    let lowered = token.to_lowercase();
    if lowered.chars().count() <= 1
        || lowered
            .chars()
            .all(|c| TRAILING.contains(&c) || c == '/' || c == '.')
    {
        return lowered;
    }
    lowered.trim_end_matches(TRAILING).to_owned()
}

/// Whether a word carries no meaning on its own.
#[must_use]
pub fn is_filler(word: &str) -> bool {
    FILLER.contains(&word)
}

/// How many leading words to skip before looking for a verb.
///
/// `please go to the laboratory` opens with filler, and the verb matcher only looks
/// at the head of the input — so this runs *before* matching while
/// [`strip_filler`] runs *after*. Safe because no synonym phrase begins with a
/// filler word, which `no_synonym_starts_with_filler` asserts.
///
/// Returns 0 when everything is filler, so the input is never emptied.
#[must_use]
pub fn skip_leading_filler(words: &[Word<'_>]) -> usize {
    words
        .iter()
        .position(|word| !is_filler(word.matching))
        .unwrap_or_default()
}

/// Drop filler from an argument tail.
#[must_use]
pub fn strip_filler<'a>(words: &[Word<'a>]) -> Vec<Word<'a>> {
    let stripped: Vec<Word<'a>> = words
        .iter()
        .copied()
        .filter(|word| !is_filler(word.matching))
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

    fn matching(input: &str) -> Vec<String> {
        Tokens::split(input)
            .words()
            .iter()
            .map(|w| w.matching.to_owned())
            .collect()
    }

    fn raw(input: &str) -> Vec<String> {
        Tokens::split(input)
            .words()
            .iter()
            .map(|w| w.raw.to_owned())
            .collect()
    }

    #[test]
    fn case_and_surrounding_space_are_flattened_for_matching() {
        assert_eq!(
            matching("  GO To The Gates  "),
            ["go", "to", "the", "gates"]
        );
    }

    #[test]
    fn the_raw_form_survives_lowercasing() {
        // `sift ERROR feed.log` must search for ERROR, not error.
        assert_eq!(raw("sift ERROR feed.log"), ["sift", "ERROR", "feed.log"]);
        assert_eq!(
            matching("sift ERROR feed.log"),
            ["sift", "error", "feed.log"]
        );
    }

    #[test]
    fn the_raw_form_keeps_trailing_punctuation() {
        assert_eq!(raw("sift ok. feed.log"), ["sift", "ok.", "feed.log"]);
        assert_eq!(matching("sift ok. feed.log"), ["sift", "ok", "feed.log"]);
    }

    #[test]
    fn quoted_runs_stay_together() {
        assert_eq!(
            raw(r#"sift "march north" feed.log"#),
            ["sift", "march north", "feed.log"]
        );
    }

    #[test]
    fn an_unterminated_quote_takes_the_rest_of_the_line() {
        assert_eq!(raw(r#"sift "march north"#), ["sift", "march north"]);
    }

    #[test]
    fn trailing_punctuation_is_shed_for_matching() {
        assert_eq!(matching("brew clarity."), ["brew", "clarity"]);
        assert_eq!(matching("how do i brew?"), ["how", "do", "i", "brew"]);
    }

    #[test]
    fn standalone_punctuation_synonyms_survive() {
        // `?` is grimoire and `./` is invoke (§6.1).
        assert_eq!(matching("?"), ["?"]);
        assert_eq!(matching("./ night_watch"), ["./", "night_watch"]);
    }

    #[test]
    fn paths_and_filenames_keep_their_separators() {
        assert_eq!(
            matching("attend /tower/laboratory"),
            ["attend", "/tower/laboratory"]
        );
        assert_eq!(matching("peruse feed.log"), ["peruse", "feed.log"]);
    }

    #[test]
    fn apostrophes_survive_for_phrase_matching() {
        assert_eq!(matching("what's here"), ["what's", "here"]);
    }

    #[test]
    fn filler_words_that_are_also_phrase_words_survive_tokenising() {
        assert_eq!(matching("go to laboratory"), ["go", "to", "laboratory"]);
        assert_eq!(
            matching("get rid of sludge"),
            ["get", "rid", "of", "sludge"]
        );
        assert_eq!(matching("take it back"), ["take", "it", "back"]);
    }

    #[test]
    fn filler_is_dropped_from_arguments() {
        let tokens = Tokens::split("a potion of clarity");
        let kept: Vec<_> = strip_filler(&tokens.words())
            .iter()
            .map(|w| w.matching)
            .collect();
        assert_eq!(kept, ["potion", "clarity"]);
    }

    #[test]
    fn an_argument_of_pure_filler_is_kept_rather_than_emptied() {
        let tokens = Tokens::split("the");
        assert_eq!(strip_filler(&tokens.words()).len(), 1);
    }

    #[test]
    fn empty_input_yields_no_tokens() {
        assert!(matching("").is_empty());
        assert!(matching("   ").is_empty());
    }

    #[test]
    fn absurd_input_is_capped_rather_than_parsed() {
        // §6 has no budget for scoring a pasted paragraph against the whole
        // vocabulary and every noun in the tower.
        let long = "word ".repeat(1000);
        assert!(Tokens::split(&long).words().len() <= MAX_WORDS);

        let single = "a".repeat(10_000);
        let tokens = Tokens::split(&single);
        assert!(tokens.words()[0].raw.chars().count() <= MAX_INPUT);
    }
}
