//! What the player might be about to type — **the prompt's view of it**.
//!
//! The answer itself is [`expect`](mod@super::expect)'s, which three surfaces share.
//! What is left here is the shape the prompt's Tab *listing* wants — strings
//! rather than kinds — and §6's numbered answer.
//!
//! Lives here rather than in a frontend because it needs the vocabulary and the
//! live [`Scene`], both of which are the parser's — and because rule 2 says both
//! frontends must be able to offer the same thing. It is also the only part of
//! the prompt's editing behaviour that can be tested without a window.
//!
//! # The shape is `rustyline`'s, for its reasons
//!
//! **A range, not an append.** `rustyline`'s `complete` returns *where the
//! completable word starts*; `reedline`'s `Suggestion` carries the span it
//! replaces. That matters more here than it does there, because §6's parser is
//! deliberately fuzzy — `clarty` resolves to `clarity` — and a completer that can
//! only extend what was typed is useless the moment the player has typed a
//! near-miss, which is the player this game is built for.
//!
//! **One string, not `display` apart from `insert`.** `rustyline` and `reedline`
//! both split them, and the split was taken here on the theory that a place is
//! registered by full path and shown by leaf. It is not: the leaf is what the
//! list shows *and* what goes into the line, because §6's matcher accepts it and
//! the echo shows it back. Two fields that every construction site sets equal are
//! two fields that can drift — the shared prefix read one and the Tab listing the
//! other — so there is one until something genuinely needs two.
//!
//! # What it does not do
//!
//! No debouncing, no caching, no background thread. `fish` has all three and
//! `zsh` recommends async, because their corpus is a filesystem and a history of
//! tens of thousands of lines. Ours is a few dozen scene nouns, in memory, with
//! no I/O — and rule 8 forbids async here regardless. Taking that machinery would
//! be pure cost, which is worth writing down because the prior art all points at
//! it.

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
/// The prompt's view of [`expect`](super::expect()): the same answer with the
/// kinds dropped, because a Tab listing shows strings. `spell` is false — the
/// prompt cannot run `repeat` or `end`, and offering them would teach a word
/// that is refused the moment it is used.
///
/// `prompt_open` turns completion off entirely: while §6's numbered prompt is
/// waiting, the orb wants a **digit**, and offering verbs there — or ghosting one
/// — walks the player into a dead end. §15 weighs the dead-end rate above the raw
/// resolution rate.
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
        // §6 claims all three registers reach the same command, so completing
        // only the arcane form would teach that the other two are lesser.
        let found = inserts("sur");
        assert!(found.contains(&"survey".to_owned()), "{found:?}");

        let shell = inserts("l");
        assert!(shell.contains(&"ls".to_owned()), "{shell:?}");
        assert!(shell.contains(&"look".to_owned()), "{shell:?}");
    }

    #[test]
    fn a_later_word_completes_to_the_nouns_that_slot_accepts() {
        // `wield` takes a place; `move` takes a reagent first. Offering the
        // wrong kind would suggest a command that cannot resolve.
        let places = inserts("wield mo");
        assert_eq!(places, ["mortar_and_pestle"]);

        let reagents = inserts("move sa");
        assert_eq!(reagents, ["sage"]);
    }

    #[test]
    fn a_place_completes_to_its_leaf_not_its_path() {
        // The scene holds `/tower/laboratory/mortar_and_pestle`; inserting that
        // would be valid, unreadable, and not what anyone typed.
        assert_eq!(inserts("wield mortar"), ["mortar_and_pestle"]);
    }

    #[test]
    fn free_text_slots_complete_nothing() {
        // A pattern is whatever the player is searching for and a count is a
        // number. Offering scene nouns for either lies about the slot.
        assert!(inserts("sift ma").is_empty());
        assert!(inserts("meditate 3").is_empty());
    }

    #[test]
    fn the_replaced_range_is_the_partial_word_only() {
        // A range rather than an append, so a candidate can *correct* a
        // near-miss rather than only extend a prefix — which is what §6's fuzzy
        // matching means for a completer.
        let completion = complete("wield mo", 8, &tower(), false);
        assert_eq!(completion.replaces, 6..8);
        assert_eq!(&"wield mo"[completion.replaces], "mo");
    }

    #[test]
    fn nothing_is_offered_while_a_numbered_prompt_is_open() {
        // The orb wants a digit. Offering verbs — or ghosting one — walks the
        // player into a dead end, which §15 weighs above the resolution rate.
        let completion = complete("sur", 3, &tower(), true);
        assert!(completion.is_empty());
    }

    #[test]
    fn a_multi_word_verb_completes_its_own_slot() {
        // `look for` is `sift`, whose first slot is free text — so it must offer
        // nothing. A single-word scan read the head `look` as `survey` and
        // offered places: Tab suggesting arguments for a verb Enter would not
        // run. §6's whole claim is that the plain register is not second class.
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

    /// Verbs come back sorted and nouns come back in scene order.
    ///
    /// **Nine tests and only two of them asserted an order**, both by comparing
    /// a one-element list — which pins nothing. `common()` reads the whole list
    /// and Tab's listing shows it in the order it arrives, so the order is
    /// player-visible behaviour with no gate on it.
    ///
    /// This is written *before* [`expect`](super::expect) grows ranking, so that
    /// the day ranking lands the change is a failing assertion here rather than
    /// a listing that quietly reshuffled.
    #[test]
    fn the_order_a_completion_arrives_in_is_part_of_the_answer() {
        // Verbs: alphabetical, from `verbs`'s `sort`. Not table order, not
        // score order — a listing a player scans wants one they can predict.
        let mut sorted = inserts("s");
        assert!(sorted.len() > 2, "too few to prove an order: {sorted:?}");
        let unsorted = sorted.clone();
        sorted.sort();
        assert_eq!(unsorted, sorted, "verbs stopped arriving alphabetically");

        // Nouns: **scene order**, which is the tower's own raise order and is
        // emphatically not alphabetical — `balneum_mariae` is registered after
        // `mortar_and_pestle` and comes back after it. Pinned whole rather than
        // by a pair of indices, because the thing that would rot is the
        // sequence.
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
