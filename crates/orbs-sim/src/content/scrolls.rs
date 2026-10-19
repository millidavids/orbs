//! What a scroll does in the calm layer, as a supply (§19, *Supply*).
//!
//! Separate from [`Spendables`](super::Spendables) because that file says what
//! a scroll does on the wall and this one what it does in a room. Recipes' tier:
//! a draw changes what a run takes, so a mid-session swap would break replay.

use std::collections::BTreeMap;

use bevy_ecs::resource::Resource;
use serde::Deserialize;

use super::rate::Rates;

/// The compiled-in default, so a headless `Sim` needs no filesystem.
const BUILTIN: &str = include_str!("../../content/scrolls.toml");

/// The file's name, for an error a writer can act on.
const FILE: &str = "scrolls.toml";

/// One scroll's calm supply.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Calm {
    /// Ticks between draws, from the moment the supply opened.
    pub every: u64,
    /// How fast it can draw, slowest first.
    pub rates: Rates,
    /// Growth a herb needs to flower. Verdant's alone.
    #[serde(default)]
    pub flowers_at: Option<u32>,
}

/// Every scroll's calm supply, by name.
#[derive(Debug, Clone, Deserialize, Resource)]
#[serde(transparent)]
pub struct Scrolls {
    by_name: BTreeMap<String, Calm>,
}

impl Scrolls {
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

    /// Parse a scrolls file.
    ///
    /// # Errors
    ///
    /// [`ContentError`](super::ContentError) if the text is not valid TOML of the
    /// expected shape, or a row's rates break `content::rate`'s rule.
    pub fn parse(text: &str) -> Result<Self, super::ContentError> {
        super::load::parse(FILE, text)
    }

    /// What `name` does in a room, if it is a scroll with a calm supply.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Calm> {
        self.by_name.get(name)
    }

    /// Every scroll named, alphabetically.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.by_name.keys().map(String::as_str)
    }
}

impl Default for Scrolls {
    fn default() -> Self {
        Self::builtin()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_builtin_file_parses() {
        let scrolls = Scrolls::builtin();
        assert!(scrolls.get("quickening-scroll").is_some());
        assert!(scrolls.get("gleaning-scroll").is_some());
        let flowers_at = scrolls
            .get("verdant-scroll")
            .and_then(|calm| calm.flowers_at);
        assert!(flowers_at.is_some_and(|at| at > 0), "verdant never flowers");
    }

    #[test]
    fn every_scroll_the_lectern_makes_has_a_calm_row() {
        // A scroll with no row would be wielded and refused as unreadable,
        // after four walks of the stacks paid for it.
        let recipes = super::super::Recipes::builtin();
        let scrolls = Scrolls::builtin();
        for name in recipes.outputs() {
            if recipes.kind_of(name) == crate::parser::NounKind::Scroll {
                assert!(scrolls.get(name).is_some(), "{name} has no calm row");
            }
        }
    }

    #[test]
    fn a_scroll_opens_at_one_count_in_a_room_and_on_the_wall() {
        // An item reads `drained` below one threshold, read off `siege.toml`.
        // A calm table opening higher would refuse a scroll that never read
        // drained, and a keeper spell asking `has drained` would never restock.
        let spendables = super::super::Spendables::builtin();
        let scrolls = Scrolls::builtin();
        for name in scrolls.names() {
            let calm = scrolls.get(name).expect("just listed");
            if let Some(wall) = spendables.get(name) {
                assert_eq!(
                    calm.rates.slowest().opens,
                    wall.rates.slowest().opens,
                    "{name} opens at one count in the calm and another on the wall",
                );
            }
        }
    }

    #[test]
    fn every_draw_waits_at_least_a_tick() {
        let scrolls = Scrolls::builtin();
        for name in scrolls.names() {
            let every = scrolls.get(name).map_or(0, |calm| calm.every);
            assert!(every > 0, "{name} draws every tick of nought");
        }
    }

    #[test]
    fn a_rate_that_makes_one_scroll_worth_something_is_refused() {
        let text = "[x]\nevery = 300\nrates = [{ opens = 1, draws = 1, worth = 2 }]\n";
        assert!(Scrolls::parse(text).is_err());
    }
}
