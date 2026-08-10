//! What the athanor burns, and for how long (DESIGN.md §10.1).
//!
//! Separate from [`Recipes`](super::Recipes) because the athanor transforms
//! nothing: it takes fuel and gives heat. Forcing it into the recipe shape would
//! have meant inventing an output for it.

use std::collections::BTreeMap;

use bevy_ecs::resource::Resource;
use serde::Deserialize;

/// The compiled-in default, so a headless `Sim` needs no filesystem.
const BUILTIN: &str = include_str!("../../content/fuel.toml");

/// One thing the athanor will burn.
#[derive(Debug, Clone, Deserialize)]
pub struct Fuel {
    /// How many ticks one of these burns for.
    pub ticks: u64,
    /// What it leaves behind when it is spent.
    pub leaves: String,
}

/// Everything burnable, by reagent name.
#[derive(Debug, Clone, Deserialize, Resource)]
#[serde(transparent)]
pub struct Fuels {
    by_name: BTreeMap<String, Fuel>,
}

/// The file's name, for an error a writer can act on.
const FILE: &str = "fuel.toml";

impl Fuels {
    /// The fuel table compiled into the binary.
    ///
    /// # Panics
    ///
    /// If the built-in file is malformed — a build-time authoring error, covered
    /// by `the_builtin_file_parses`.
    #[must_use]
    pub fn builtin() -> Self {
        super::load::builtin(FILE, BUILTIN)
    }

    /// Parse a fuel file.
    ///
    /// # Errors
    ///
    /// [`ContentError`](super::ContentError) if the text is not valid TOML of the
    /// expected shape.
    pub fn parse(text: &str) -> Result<Self, super::ContentError> {
        super::load::parse(FILE, text)
    }

    /// What `name` burns like, if it burns at all.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Fuel> {
        self.by_name.get(name)
    }

    /// Every burnable name, alphabetically.
    ///
    /// A `BTreeMap`, so the order is the names' own and **not the file's** —
    /// stable across runs either way, which is what matters: this feeds a list a
    /// tester reads and, through `debug_spawn`, a name they can type.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.by_name.keys().map(String::as_str)
    }
}

impl Default for Fuels {
    fn default() -> Self {
        Self::builtin()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_builtin_file_parses() {
        let fuels = Fuels::builtin();
        let charcoal = fuels.get("charcoal").expect("charcoal burns");
        assert!(
            charcoal.ticks > 0,
            "fuel that burns for no time is not fuel"
        );
        assert_eq!(charcoal.leaves, "ash");
    }

    #[test]
    fn a_reagent_that_is_not_fuel_does_not_burn() {
        assert!(Fuels::builtin().get("sage").is_none());
    }

    #[test]
    fn a_broken_file_is_an_error_rather_than_a_panic() {
        assert!(Fuels::parse("not toml [[[").is_err());
    }
}
