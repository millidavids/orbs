//! Which kinds of sound are on — a switch each on the sound page.
//!
//! Beside [`Levels`], not inside it: `voice` is how loud the orb is, and these
//! are whether a kind of sound plays at all. A player who wants the chimes and
//! not the typing should not have to choose a volume to get there.
//!
//! [`Levels`]: super::Levels

use bevy::prelude::*;

/// A kind of sound, switched on or off as one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    /// Typing: the click under a key, and Enter.
    Keys,
    /// Something finished, or went well.
    Chimes,
    /// A breach, or a surface found tampered with.
    Alarms,
    /// The orb did not understand, or could not.
    Refusals,
}

impl Kind {
    /// All four, in the order the page lists them.
    pub(crate) const ALL: [Self; 4] = [Self::Keys, Self::Chimes, Self::Alarms, Self::Refusals];

    /// The row's word, which is also the key the switch is kept under.
    pub(crate) const fn word(self) -> &'static str {
        match self {
            Self::Keys => "keys",
            Self::Chimes => "chimes",
            Self::Alarms => "alarms",
            Self::Refusals => "refusals",
        }
    }

    /// The kind `word` names, whole.
    pub(crate) fn named(word: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.word() == word)
    }

    const fn at(self) -> usize {
        self as usize
    }
}

/// Which kinds are on. Each starts on; a word in the file other than `on` is off.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Kinds([bool; Kind::ALL.len()]);

impl Kinds {
    /// Whether `kind` plays.
    pub(crate) const fn allows(self, kind: Kind) -> bool {
        self.0[kind.at()]
    }

    /// Turn `kind` on or off.
    pub(crate) const fn set(&mut self, kind: Kind, on: bool) {
        self.0[kind.at()] = on;
    }
}

impl Default for Kinds {
    fn default() -> Self {
        // One read of the file, not one per kind.
        let kept = orbs_shell::settings::load();
        Self(Kind::ALL.map(|kind| {
            kept.get(kind.word())
                .is_none_or(|word| orbs_shell::settings::is_on(word))
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_kind_turns_off_alone() {
        let mut kinds = Kinds([true; 4]);
        kinds.set(Kind::Keys, false);
        assert!(!kinds.allows(Kind::Keys));
        for kind in [Kind::Chimes, Kind::Alarms, Kind::Refusals] {
            assert!(kinds.allows(kind), "{} went off with keys", kind.word());
        }
    }

    #[test]
    fn a_kind_is_its_own_word_and_its_own_place() {
        for (place, kind) in Kind::ALL.into_iter().enumerate() {
            assert_eq!(Kind::named(kind.word()), Some(kind));
            assert_eq!(kind.at(), place, "`ALL` is out of declaration order");
        }
        assert_eq!(Kind::named("hum"), None);
    }
}
