//! The menu's vocabulary, and the prefix rule that governs it.
//!
//! Kept apart from [`state`](super::state) because the rule is the interesting
//! part: every word here runs by its shortest unambiguous prefix, which only
//! works while no two share a first letter — and that is a property a test
//! holds, not a convention a reader has to remember.

/// A word the menu answers to.
///
/// **Prefix-matched, and no two share a first letter** — the rule the editor's
/// and the weave's vocabularies both follow, so `r`, `s`, `n`, `o` and `q` all
/// work and the property is [a test](tests::no_two_words_share_a_first_letter)
/// rather than a convention. It holds across the other pages as well: `back` on
/// all of them, `short`/`medium`/`long` on one, and a bare number on another.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Word {
    /// Go back to the tower.
    Resume,
    /// List the towers the orb is keeping.
    Saves,
    /// Begin one.
    New,
    /// What the orb does with a line you type at it.
    Options,
    /// Put the orb down.
    Quit,
}

/// How many words the menu offers.
///
/// **A `u16` the array's length is written in terms of**, rather than a cast of
/// `WORDS.len()`: `MIN_ROWS` needs the count in cells, and a cast there is one
/// clippy will not take on faith. Adding a word means changing this, and the
/// array literal will not compile until it is.
pub(super) const COUNT: u16 = 5;

/// Every word, in the order the menu offers them.
pub const WORDS: [(&str, Word); COUNT as usize] = [
    ("resume", Word::Resume),
    ("saves", Word::Saves),
    ("new", Word::New),
    ("options", Word::Options),
    ("quit", Word::Quit),
];

/// The word that steps one page back, on every inner page.
///
/// Not in [`WORDS`], which is the top page's list. It shares no first letter
/// with the lengths (`short`, `medium`, `long`) or the drivers, and cannot
/// collide with a slot number, which is the whole of what it has to avoid.
pub(super) const BACK: &str = "back";

/// The word `typed` names, by unambiguous prefix.
pub(super) fn word(typed: &str) -> Option<Word> {
    WORDS
        .iter()
        .find(|(name, _)| name.starts_with(typed))
        .map(|(_, word)| *word)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu::state::Driver;
    use orbs_sim::content::Length;

    #[test]
    fn no_two_words_share_a_first_letter() {
        // The prefix rule the editor and the weave both keep, so `r`, `s`, `n`,
        // `o` and `q` are all unambiguous.
        let mut firsts: Vec<char> = WORDS
            .iter()
            .filter_map(|(name, _)| name.chars().next())
            .collect();
        firsts.sort_unstable();
        let before = firsts.len();
        firsts.dedup();
        assert_eq!(before, firsts.len(), "two words share a first letter");
    }

    #[test]
    fn the_inner_pages_have_no_collisions_either() {
        // `back` sits on every inner page beside the lengths, the drivers and
        // the slot numbers, and it must not prefix-match any of them — `b` has
        // to mean one thing wherever it is typed.
        for length in Length::OFFERED {
            assert!(
                !length.word().starts_with(&BACK[..1]),
                "`b` is ambiguous between back and {}",
                length.word(),
            );
        }
        for driver in Driver::ALL {
            assert!(
                !driver.word().starts_with(&BACK[..1]),
                "`b` is ambiguous between back and {}",
                driver.word(),
            );
        }
        assert!(BACK.parse::<usize>().is_err(), "back reads as a slot");

        let mut firsts: Vec<char> = Length::OFFERED
            .iter()
            .filter_map(|length| length.word().chars().next())
            .collect();
        firsts.sort_unstable();
        let before = firsts.len();
        firsts.dedup();
        assert_eq!(before, firsts.len(), "two lengths share a first letter");

        let mut firsts: Vec<char> = Driver::ALL
            .iter()
            .filter_map(|driver| driver.word().chars().next())
            .collect();
        firsts.sort_unstable();
        let before = firsts.len();
        firsts.dedup();
        assert_eq!(before, firsts.len(), "two drivers share a first letter");
    }
}
