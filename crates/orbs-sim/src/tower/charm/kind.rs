//! What an enchantment does, as a closed table.
//!
//! A new charm is a variant and a row. Cost and duration are authored in
//! `content/forge.toml`; what a charm means is code, read at different sites.
//!
//! Rejected names, so nobody re-proposes them: `imbued` (975 against its own
//! verb), `temper` (625 against `tampered`), `anvil` (ties `until`), `charmed`
//! (858 against `charged`), `quickened` (a third `qui` word).

/// What an enchantment does.
///
/// Five, each read in a different module — a composition rather than the `if`
/// `tower::dice` warned about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    /// Work takes half as long.
    ///
    /// What a `quickening-scroll` has always bought, now with a second source.
    Hurried,
    /// A run yields one more than the recipe says.
    Fruitful,
    /// A walk of the stacks pays an extra fragment.
    Bountiful,
    /// A die rolls one size up.
    Whetted,
    /// A sabotage strike lands and is turned aside.
    Shielded,
}

impl Kind {
    /// Every charm the forge can lay.
    pub const ALL: [Self; 5] = [
        Self::Hurried,
        Self::Fruitful,
        Self::Bountiful,
        Self::Whetted,
        Self::Shielded,
    ];

    /// The word a player types and a spell reads.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Hurried => "hurried",
            Self::Fruitful => "fruitful",
            Self::Bountiful => "bountiful",
            Self::Whetted => "whetted",
            Self::Shielded => "shielded",
        }
    }

    /// Read one back from its word.
    ///
    /// Exact only: the parser has already resolved it, and a second fuzzy round
    /// would let one name mean two things.
    #[must_use]
    pub fn from_word(word: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.word() == word)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two charms sharing a word would make `imbue` ambiguous in the one place
    /// the parser has already stopped looking.
    #[test]
    fn every_charm_reads_back_from_a_word_of_its_own() {
        crate::testing::assert_round_trips(&Kind::ALL, Kind::word, Kind::from_word);
    }
}
