//! What an enchantment does, as a closed table.
//!
//! The shape `Die::ALL`, `Humour::ALL` and the lens's `SIGILS` already have: a
//! new charm is a variant and a row. The forge authors *cost* and *duration* in
//! `content/forge.toml` (rule 6); what a charm means is code, because each one
//! is read at a different site.
//!
//! The words were swept against every word the game knows, in both directions,
//! and against each other — the third check is the one a pass here once skipped,
//! which let `imbued` through at 975 against its own verb's abbreviation.
//! Rejected, so nobody re-proposes them: `temper` (625 against `tampered`),
//! `anvil` (ties `until` at `MIN_SIMILARITY`), `charmed` (858 against
//! `charged`), and `quickened` (a third `qui` word, where §19 already tolerates
//! one collision).

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
    /// Exact only. §6's fuzzy matcher runs at the parser, so a word arriving
    /// here is already resolved; a second round would be a second place for one
    /// name to mean two things.
    #[must_use]
    pub fn from_word(word: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.word() == word)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_charm_reads_back_from_its_word() {
        for kind in Kind::ALL {
            assert_eq!(
                Kind::from_word(kind.word()),
                Some(kind),
                "{} does not read back",
                kind.word(),
            );
        }
    }

    /// Two charms sharing a word would make `imbue` ambiguous in the one place
    /// the parser has already stopped looking.
    #[test]
    fn no_two_charms_share_a_word() {
        let mut seen: Vec<&str> = Kind::ALL.into_iter().map(Kind::word).collect();
        seen.sort_unstable();
        let count = seen.len();
        seen.dedup();
        assert_eq!(count, seen.len(), "two charms share a word: {seen:?}");
    }
}
