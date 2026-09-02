//! What a charm costs, how long it lasts, and what a settle takes.
//!
//! Rule 6: balancing enchanting means moving these numbers dozens of times, and
//! in Rust that is a recompile per guess. `siege.toml` is the precedent one room
//! over, and for the same reason.
//!
//! What is **not** authored is what a charm *does* — that is `charm::Kind`, a
//! closed table in Rust, because each one is read at a different site and there
//! is nothing to derive. A key here with no matching variant fails the load
//! rather than being ignored, which is what stops a typo shipping as a charm
//! nobody can make.

use std::collections::BTreeMap;

use bevy_ecs::prelude::Resource;
use serde::Deserialize;

use crate::tower::charm::Kind;

const FILE: &str = "forge.toml";
const BUILTIN: &str = include_str!("../../content/forge.toml");

/// What one charm costs and how long it runs.
#[derive(Debug, Clone, Deserialize)]
pub struct Charm {
    /// Quintessence to bind it.
    pub costs: u32,
    /// How long it lasts once bound, in ticks.
    pub lasts: u64,
    /// Ticks of the production slot one settle takes.
    pub takes: u64,
}

/// What the surcharge is, and when it applies.
#[derive(Debug, Clone, Deserialize)]
pub struct Surcharge {
    /// What a charm costs while a siege is being fought, as a multiplier.
    pub besieged: u32,
}

/// Everything the forge can lay, by name.
#[derive(Debug, Clone, Deserialize, Resource)]
pub struct Charms {
    surcharge: Surcharge,
    #[serde(flatten)]
    by_name: BTreeMap<String, Charm>,
}

impl Charms {
    /// The table compiled into the binary.
    ///
    /// # Panics
    ///
    /// If the built-in file is malformed — a build-time authoring error, covered
    /// by `the_builtin_file_parses`.
    #[must_use]
    pub fn builtin() -> Self {
        super::load::builtin(FILE, BUILTIN)
    }

    /// Parse a forge file.
    ///
    /// # Errors
    ///
    /// [`ContentError`](super::ContentError) if the text is not valid TOML of the
    /// expected shape.
    pub fn parse(text: &str) -> Result<Self, super::ContentError> {
        super::load::parse(FILE, text)
    }

    /// What `kind` costs and lasts.
    #[must_use]
    pub fn get(&self, kind: Kind) -> Option<&Charm> {
        self.by_name.get(kind.word())
    }

    /// What `kind` costs to bind, given whether a siege is being fought.
    ///
    /// **The surcharge is a multiplier and it is authored**, not a branch: a
    /// wizard at his forge while the wall is under attack is neglecting the
    /// wall, and scaling with the charm means hurrying a mortar mid-fight is a
    /// small indulgence where shielding a spell is a large one.
    #[must_use]
    pub fn cost(&self, kind: Kind, besieged: bool) -> u32 {
        let base = self.get(kind).map_or(u32::MAX, |charm| charm.costs);
        if besieged {
            base.saturating_mul(self.surcharge.besieged)
        } else {
            base
        }
    }

    /// Every charm named in the file, alphabetically.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.by_name.keys().map(String::as_str)
    }
}

impl Default for Charms {
    fn default() -> Self {
        Self::builtin()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_builtin_file_parses() {
        let charms = Charms::builtin();
        assert!(charms.names().count() > 0, "the forge has no charms");
    }

    /// **Both directions, because either gap is silent.**
    ///
    /// A charm in the table with no variant is a word a player can read in a
    /// file and never type; a variant with no row is a charm the forge offers
    /// and cannot price. `siege.toml`'s own lint is the precedent — an item with
    /// no entry there cannot be spent at all.
    #[test]
    fn every_charm_the_game_has_is_one_the_file_prices() {
        let charms = Charms::builtin();
        for kind in Kind::ALL {
            assert!(
                charms.get(kind).is_some(),
                "{} is a charm the forge can lay and the file does not price",
                kind.word(),
            );
        }
        let unknown: Vec<&str> = charms
            .names()
            .filter(|name| Kind::from_word(name).is_none())
            .collect();
        assert!(
            unknown.is_empty(),
            "the file prices charms the game does not have: {unknown:?}",
        );
    }

    /// A free charm is one you always lay, which is the decision switched off.
    #[test]
    fn no_charm_is_free_or_instant() {
        let charms = Charms::builtin();
        for kind in Kind::ALL {
            let charm = charms.get(kind).expect("priced");
            assert!(charm.costs > 0, "{} is free", kind.word());
            assert!(charm.lasts > 0, "{} lasts no time", kind.word());
            assert!(charm.takes > 0, "{} takes no slot", kind.word());
        }
    }

    /// The surcharge has to bite, or enchanting mid-siege is free in the one
    /// way the design says it must not be.
    #[test]
    fn a_siege_makes_every_charm_dearer() {
        let charms = Charms::builtin();
        for kind in Kind::ALL {
            assert!(
                charms.cost(kind, true) > charms.cost(kind, false),
                "{} costs the same in a siege",
                kind.word(),
            );
        }
    }
}
