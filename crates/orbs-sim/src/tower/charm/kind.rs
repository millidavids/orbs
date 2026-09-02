//! What an enchantment does, as a closed table.
//!
//! **A closed table**, the shape `Die::ALL`, `Syllable::ALL` and the lens's
//! `SIGILS` already have — so a new charm is a variant and a row, and nothing
//! derives one. The forge authors *cost* and *duration* in `content/forge.toml`
//! (rule 6, so a tuning pass is a content edit); what a charm **means** is code,
//! because each one is read at a different site and there is nothing to derive.
//!
//! # The words were swept three ways
//!
//! §6's matcher is directional and prefix-first, so a candidate has to be scored
//! against every word the game knows *in both directions*, and against the other
//! candidates. The last pass here skipped that third check and `imbue` → `imbued`
//! came back at **975** — a reading that would have stolen its own verb's
//! abbreviation, and a fourth row in `tests/naming.rs`'s pinned list, whose
//! comment says a fourth *"is a word somebody should look at before shipping
//! it."* `nimble` fell the same way at 667. Both are renamed.
//!
//! What the rejected words cost, so nobody re-proposes them: `temper` scores 625
//! against `tampered`, which is a `verify` verdict; `anvil` ties `until` — a
//! spell control word — at exactly `MIN_SIMILARITY`; `charmed` scores 858
//! against `charged`, an instrument state; and `quickened` would have been a
//! third `qui` word, where §19 already tolerates one collision.

/// What an enchantment does.
///
/// Five, and each is read in a different module — which is the whole reason this
/// is a composition rather than the `if` `tower::dice` warned about.
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
    /// Exact only. §6's fuzzy matcher runs at the *parser*, against the scene's
    /// nouns; by the time a word reaches here it has already been resolved, and
    /// a second round of approximate matching would be a second place for one
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
