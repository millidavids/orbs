//! Settings that are one of two words, the game as it ships first.
//!
//! `contrast`, `weight` and `echo` are three. The on/off rows are
//! [`SWITCH`](super::SWITCH), which reads the other way: anything but `on` is
//! off.

use super::{Category, Row, get};

/// A setting that is one of two words, the first being the game as it ships.
///
/// Anything but the second word reads as the first, a later build's word
/// included: a word this build does not know is no reason to change the
/// screen.
pub trait Pair: Copy + From<bool> {
    /// The key it is kept under, which is also the row's word.
    const KEY: &'static str;
    /// Both words, as shipped first, in the order the row cycles them.
    const WORDS: [&'static str; 2];

    /// Whether it is the second word.
    fn raised(self) -> bool;

    /// What `word` names.
    #[must_use]
    fn named(word: &str) -> Self {
        Self::from(word == Self::WORDS[1])
    }

    /// The word for it.
    #[must_use]
    fn word(self) -> &'static str {
        Self::WORDS[usize::from(self.raised())]
    }

    /// What the settings file says, or the first word.
    #[must_use]
    fn remembered() -> Self {
        get(Self::KEY).map_or_else(|| Self::from(false), |word| Self::named(&word))
    }

    /// Its row on `page`, showing `self`.
    #[must_use]
    fn row(self, page: Category) -> Row {
        Row::new(page, Self::KEY, self.word(), &Self::WORDS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Loud(bool);

    impl From<bool> for Loud {
        fn from(raised: bool) -> Self {
            Self(raised)
        }
    }

    impl Pair for Loud {
        const KEY: &'static str = "loud";
        const WORDS: [&'static str; 2] = ["quiet", "loud"];

        fn raised(self) -> bool {
            self.0
        }
    }

    #[test]
    fn a_word_this_build_does_not_know_is_the_first() {
        assert_eq!(Loud::named("deafening"), Loud(false));
        for word in Loud::WORDS {
            assert_eq!(Loud::named(word).word(), word);
        }
        let row = Loud(true).row(Category::Sound);
        assert_eq!((row.word.as_str(), row.value.as_str()), ("loud", "loud"));
    }
}
