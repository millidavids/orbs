//! The menu's vocabulary, and the prefix rule that governs it.
//!
//! Kept apart from [`state`](super::state) because the rule is the interesting
//! part: every word here runs by its shortest unambiguous prefix, which only
//! works while no two share a first letter — and that is a property a test
//! holds, not a convention a reader has to remember.
//!
//! A word the stance does not offer is not a word, so
//! [`offered`](Word::offered) is consulted by [`word`] itself and not only by
//! the painter. Filtering the listing alone would leave `resume` typeable at the
//! threshold — where it closes a menu with nothing behind it.

use super::stance::Stance;

/// A word the menu answers to.
///
/// Prefix-matched, and no two share a first letter — the rule the editor's and
/// the weave's vocabularies both follow, so `r`, `p`, `s` and `q` all work and
/// the property is [a test](tests::no_two_words_share_a_first_letter). It holds
/// across the other pages too: `back` on all of them, `short`/`medium`/`long` on
/// one, a bare number on another.
///
/// Four words, where there were five. `saves` and `new` moved one level down
/// under [`Play`](Self::Play), which is the shape the top page wanted the moment
/// it had to work in front of no tower: four choices naming both a listing and a
/// new game ask the player to hold that difference before being told there is
/// one. `options` became `settings`, the word people look for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Word {
    /// Go back to the tower.
    Resume,
    /// Open a tower: the ones the orb is keeping, or a new one.
    Play,
    /// What the orb does with a line you type at it, and what it looks like.
    Settings,
    /// The game, explaining itself.
    Manual,
    /// Put the orb down.
    Quit,
}

/// How many words the menu offers.
///
/// A `u16` the array's length is written in terms of, rather than a cast of
/// `WORDS.len()`: `MIN_ROWS` needs the count in cells, and clippy will not take
/// a cast there on faith. Adding a word means changing this, and the array
/// literal will not compile until it is.
pub(super) const COUNT: u16 = 5;

/// Every word, in the order the menu offers them.
pub const WORDS: [(&str, Word); COUNT as usize] = [
    ("resume", Word::Resume),
    ("play", Word::Play),
    ("settings", Word::Settings),
    ("manual", Word::Manual),
    ("quit", Word::Quit),
];

/// The word that steps one page back, on every inner page.
///
/// Not in [`WORDS`], which is the top page's list. It shares no first letter
/// with the lengths (`short`, `medium`, `long`), the drivers, or `new` on the
/// play page, and cannot collide with a slot number — which is the whole of what
/// it has to avoid.
pub(super) const BACK: &str = "back";

/// The word that raises a tower, on the play page.
///
/// Not in [`WORDS`] either: it sits beside the listing rather than above it, so
/// *begin one* and *open one you have* are the same question asked once.
pub(super) const NEW: &str = "new";

/// The word that clears a slot, on the play page.
///
/// The one word in the menu that is not prefix-matched. Everything else runs
/// from its first letter because a mistyped one costs a page you step back from;
/// this sets a tower aside, so it is typed whole and then asked about. `abandon
/// 3` puts the question, a second `abandon 3` answers it, and any other line
/// answers *no* — `quit`'s shape.
///
/// It shares no first letter with `new` or `back`, so the page's other words are
/// unharmed by it.
pub(super) const ABANDON: &str = "abandon";

impl Word {
    /// Whether this word is offered where the menu is standing.
    ///
    /// Only [`Resume`](Self::Resume) has an opinion: it means *go back to the
    /// tower*, and at the threshold there is none. See [`Stance::may_close`].
    #[must_use]
    pub const fn offered(self, stance: Stance) -> bool {
        match self {
            Self::Resume => stance.may_close(),
            Self::Play | Self::Settings | Self::Manual | Self::Quit => true,
        }
    }
}

/// The words this stance offers, in the order the menu offers them.
///
/// The painter draws these and [`word`] matches against these, so the listing
/// and the vocabulary cannot disagree.
pub(super) fn offered(stance: Stance) -> impl Iterator<Item = (&'static str, Word)> {
    WORDS
        .into_iter()
        .filter(move |(_, word)| word.offered(stance))
}

/// The word `typed` names, by unambiguous prefix, among those `stance` offers.
pub(super) fn word(typed: &str, stance: Stance) -> Option<Word> {
    offered(stance)
        .find(|(name, _)| name.starts_with(typed))
        .map(|(_, word)| word)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu::state::Driver;
    use orbs_sim::content::Length;

    #[test]
    fn no_two_words_share_a_first_letter() {
        // The prefix rule the editor and the weave both keep, so `r`, `s`, `n`,
        // `o` and `q` are all unambiguous. Checked per stance, because a subset
        // of a set with no collisions can only have fewer — but the day a word
        // is offered in one stance and not the other, this is what says so.
        for stance in [Stance::InTower, Stance::Threshold] {
            let mut firsts: Vec<char> = offered(stance)
                .filter_map(|(name, _)| name.chars().next())
                .collect();
            firsts.sort_unstable();
            let before = firsts.len();
            firsts.dedup();
            assert_eq!(
                before,
                firsts.len(),
                "two words share a first letter at {stance:?}",
            );
        }
    }

    #[test]
    fn the_threshold_does_not_answer_to_resume() {
        // The half that is not cosmetic: filtering the listing alone would leave
        // `r` closing a menu with nothing behind it — a scratch world that never
        // ticks and is never kept, with no way back and no way out.
        assert_eq!(word("resume", Stance::InTower), Some(Word::Resume));
        assert_eq!(word("r", Stance::InTower), Some(Word::Resume));
        assert_eq!(word("resume", Stance::Threshold), None);
        assert_eq!(word("r", Stance::Threshold), None);

        // Everything else answers the same in both.
        for (typed, want) in [
            ("play", Word::Play),
            ("settings", Word::Settings),
            ("quit", Word::Quit),
        ] {
            for stance in [Stance::InTower, Stance::Threshold] {
                assert_eq!(word(typed, stance), Some(want), "{typed} at {stance:?}");
            }
        }
    }

    #[test]
    fn the_listing_and_the_vocabulary_offer_the_same_words() {
        // The painter draws `offered` and `word` matches against `offered`, so
        // this is true by construction — and it is asserted because the two were
        // separate lists once and that is exactly how `resume` stayed typeable
        // at a threshold that did not draw it.
        for stance in [Stance::InTower, Stance::Threshold] {
            for (name, want) in offered(stance) {
                assert_eq!(word(name, stance), Some(want), "{name} at {stance:?}");
            }
        }
    }

    #[test]
    fn the_inner_pages_have_no_collisions_either() {
        // `back` sits on every inner page beside the lengths, the drivers, `new`
        // and the slot numbers, and it must not prefix-match any of them — `b`
        // has to mean one thing wherever it is typed.
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
        // The play page holds `new`, `back` and a number at once.
        assert_ne!(
            NEW.chars().next(),
            BACK.chars().next(),
            "new and back share a first letter on the play page",
        );
        assert!(NEW.parse::<usize>().is_err(), "new reads as a slot");

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
