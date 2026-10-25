//! What the player might be about to type — the prompt's view of it.
//!
//! The answer is [`expect`](mod@super::expect)'s; this is the shape the Tab
//! listing wants — strings rather than kinds — and §6's numbered answer. In the
//! parser, because it needs the vocabulary and the live [`Scene`], and both
//! frontends must offer the same thing (rule 2).
//!
//! `rustyline`'s shape: a range replaced rather than an append, since the
//! parser is fuzzy. One string where those libraries split `display` from
//! `insert`, because here they are always equal. No debouncing, caching or
//! background thread: a few dozen nouns in memory, and rule 8.

use core::ops::Range;

use super::scene::Scene;

/// One thing the player might have meant: what a list shows and what a Tab
/// inserts, which are the same string.
pub type Suggestion = String;

/// Everything that could finish the word under the caret.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Completion {
    /// The byte range of the line this replaces — the partial word.
    pub replaces: Range<usize>,
    /// The candidates, in scene order.
    pub candidates: Vec<Suggestion>,
}

impl Completion {
    /// Whether there is anything to offer.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.candidates.is_empty()
    }
}

/// What could finish the word ending at `caret`.
///
/// The prompt's view of [`expect`](super::expect()), with the kinds dropped.
/// `prompt_open` turns it off while §6's numbered prompt waits for a digit.
#[must_use]
pub fn complete(line: &str, caret: usize, scene: &Scene, prompt_open: bool) -> Completion {
    let found = super::expect::expect(
        line,
        caret,
        &super::expect::Situation {
            scene,
            spell: false,
            prompt_open,
            // The prompt has no lines above it and cannot run a block anyway,
            // nor a `for each`.
            open: &[],
            sets: &[],
        },
    );
    Completion {
        candidates: found.texts(),
        replaces: found.replaces,
    }
}

/// Whether a line is only a number — an answer to §6's numbered prompt.
#[must_use]
pub fn is_answer(line: &str) -> bool {
    line.trim().parse::<usize>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::NounKind;

    fn tower() -> Scene {
        Scene::new()
            .with(NounKind::Place, "/tower/laboratory")
            .with(NounKind::Place, "/tower/laboratory/mortar_and_pestle")
            .with(NounKind::Place, "/tower/laboratory/balneum_mariae")
            .with(NounKind::Place, "/tower/archive")
            .with(NounKind::Reagent, "sage")
            .with(NounKind::Reagent, "rock-salt")
            .with(NounKind::File, "feed.log")
    }

    fn inserts(line: &str) -> Vec<String> {
        let caret = line.chars().count();
        complete(line, caret, &tower(), false).candidates
    }

    #[test]
    fn the_first_word_completes_to_verbs_in_every_register() {
        // §6: no register is lesser.
        let found = inserts("sur");
        assert!(found.contains(&"survey".to_owned()), "{found:?}");

        let shell = inserts("l");
        assert!(shell.contains(&"ls".to_owned()), "{shell:?}");
        assert!(shell.contains(&"look".to_owned()), "{shell:?}");
    }

    #[test]
    fn a_later_word_completes_to_the_nouns_that_slot_accepts() {
        // `wield` takes a place; `move` takes a reagent first.
        let places = inserts("wield mo");
        assert_eq!(places, ["mortar_and_pestle"]);

        let reagents = inserts("move sa");
        assert_eq!(reagents, ["sage"]);
    }

    #[test]
    fn a_place_completes_to_its_leaf_not_its_path() {
        assert_eq!(inserts("wield mortar"), ["mortar_and_pestle"]);
    }

    #[test]
    fn free_text_slots_complete_nothing() {
        assert!(inserts("sift ma").is_empty());
        assert!(inserts("meditate 3").is_empty());
    }

    #[test]
    fn the_replaced_range_is_the_partial_word_only() {
        // So a candidate can correct a near-miss, not only extend it.
        let completion = complete("wield mo", 8, &tower(), false);
        assert_eq!(completion.replaces, 6..8);
        assert_eq!(&"wield mo"[completion.replaces], "mo");
    }

    #[test]
    fn nothing_is_offered_while_a_numbered_prompt_is_open() {
        let completion = complete("sur", 3, &tower(), true);
        assert!(completion.is_empty());
    }

    #[test]
    fn a_multi_word_verb_completes_its_own_slot() {
        // `look for` is `sift`, whose first slot is free text, not `survey`.
        assert!(
            inserts("look for ").is_empty(),
            "{:?}",
            inserts("look for ")
        );

        // ...and the register is reachable from an empty line at all.
        let plain = inserts("go");
        assert!(plain.contains(&"go to".to_owned()), "{plain:?}");
    }

    #[test]
    fn filler_between_the_verb_and_the_caret_is_not_a_slot() {
        // `move sage to ` has filled one slot, not two. Counting `to` would
        // offer the destination's nouns for the source.
        assert_eq!(inserts("move sage to mo"), ["mortar_and_pestle"]);
    }

    #[test]
    fn an_unknown_verb_offers_nothing_for_its_arguments() {
        assert!(inserts("xyzzy pl").is_empty());
    }

    /// Verbs come back sorted and nouns in scene order — player-visible, since
    /// Tab lists in arrival order.
    #[test]
    fn the_order_a_completion_arrives_in_is_part_of_the_answer() {
        // Verbs: alphabetical, an order a player can predict.
        let mut sorted = inserts("s");
        assert!(sorted.len() > 2, "too few to prove an order: {sorted:?}");
        let unsorted = sorted.clone();
        sorted.sort();
        assert_eq!(unsorted, sorted, "verbs stopped arriving alphabetically");

        // Nouns: scene order, not alphabetical, pinned whole.
        assert_eq!(
            inserts("wield "),
            [
                "laboratory",
                "mortar_and_pestle",
                "balneum_mariae",
                "archive"
            ],
            "nouns stopped arriving in scene order",
        );
    }

    #[test]
    fn a_digit_is_recognised_as_an_answer() {
        assert!(is_answer("2"));
        assert!(is_answer("  10 "));
        assert!(!is_answer("survey"));
        assert!(!is_answer(""));
    }
}
