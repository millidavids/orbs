//! The spells the tower ships with (DESIGN.md §8).
//!
//! A `.spell` is the first file in the game with **stored text**. Every other
//! readable is a *view* over the record stream — `orb.log` is the whole of it
//! and a domain log is that stream filtered by where each line happened (§3) —
//! which is what survives the eldritch renderer corrupting output. A spell is
//! not that: it is lines a player wrote, kept verbatim until they change them.
//!
//! # Not hot-reloadable, for the reason recipes are not
//!
//! [`Prose`](super::Prose) reloads because no line of it reaches a decision.
//! A spell is *nothing but* decisions, so swapping one mid-session would break
//! replay from `(seed, submissions)` unless the content were versioned into the
//! submission log. `Sim` loads these once at construction and never again.

use std::collections::BTreeMap;

use bevy_ecs::resource::Resource;
use serde::Deserialize;

/// The compiled-in default, so a headless `Sim` needs no filesystem.
const BUILTIN: &str = include_str!("../../content/spells.toml");

/// The file's name, for an error a writer can act on.
const FILE: &str = "spells.toml";

/// The extension every spell answers to.
///
/// One constant rather than a literal at each site: `scribe` appends it when the
/// player leaves it off, `build` spawns with it, and `tower::scene` decides what
/// is a [`Script`](crate::parser::NounKind::Script) by it.
pub const EXTENSION: &str = ".spell";

/// One authored spell.
#[derive(Debug, Clone, Deserialize)]
pub struct Spell {
    /// Which domain it is written for, and runs in.
    ///
    /// A spell does not walk. It is written where the work is and it works
    /// there, which is what makes `attend` meaningless inside one — and what
    /// makes canonicalisation decidable once control flow arrives, since a
    /// branch would otherwise make "which room is line 9 resolved against"
    /// unanswerable at authoring time.
    pub domain: String,
    /// Its lines, as a player would have typed them.
    pub lines: Vec<String>,
}

/// Every spell the tower starts with, by name (without the extension).
#[derive(Debug, Clone, Deserialize, Resource)]
#[serde(transparent)]
pub struct Spells {
    by_name: BTreeMap<String, Spell>,
}

impl Spells {
    /// The spellbook compiled into the binary.
    ///
    /// # Panics
    ///
    /// If the built-in file is malformed — a build-time authoring error, covered
    /// by `the_builtin_file_parses`.
    #[must_use]
    pub fn builtin() -> Self {
        super::load::builtin(FILE, BUILTIN)
    }

    /// Parse a spellbook file.
    ///
    /// # Errors
    ///
    /// [`ContentError`](super::ContentError) if the text is not valid TOML of the
    /// expected shape.
    pub fn parse(text: &str) -> Result<Self, super::ContentError> {
        super::load::parse(FILE, text)
    }

    /// Every spell, in name order.
    ///
    /// `BTreeMap`, so the order is the file's names sorted rather than a hash
    /// order that varies per run. The tower spawns them in this order and §6
    /// resolves a scoring tie to whichever noun was registered first, which
    /// makes this part of the world's determinism — see `tower::node`.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Spell)> {
        self.by_name
            .iter()
            .map(|(name, spell)| (name.as_str(), spell))
    }

    /// Whether anything is authored at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_name.is_empty()
    }
}

impl Default for Spells {
    fn default() -> Self {
        Self::builtin()
    }
}

/// `name` with the spell extension, however the player wrote it.
///
/// `first_light` and `first_light.spell` are the same spell. §6's matcher
/// accepts either because the file is registered under its full name and the
/// leaf logic never applies to a non-place — so the normalising happens here,
/// once, rather than at each of `scribe`, `invoke` and `bind`.
#[must_use]
pub fn with_extension(name: &str) -> String {
    if name.ends_with(EXTENSION) {
        name.to_owned()
    } else {
        format!("{name}{EXTENSION}")
    }
}

/// `name` without the spell extension.
#[must_use]
pub fn without_extension(name: &str) -> &str {
    name.strip_suffix(EXTENSION).unwrap_or(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_builtin_file_parses() {
        let spells = Spells::builtin();
        assert!(!spells.is_empty(), "the tower ships with no spells at all");
    }

    #[test]
    fn the_shipped_spell_is_the_opening_of_the_brewing_loop() {
        // §12 wants the first automation to be something the player has already
        // done by hand. If this stops being true the onboarding beat is gone,
        // and nothing else would notice.
        let spells = Spells::builtin();
        let (name, spell) = spells.iter().next().expect("a shipped spell");
        assert_eq!(name, "first_light");
        assert!(
            spell.lines.iter().any(|line| line.contains("kindle")),
            "the shipped spell should light the athanor: {:?}",
            spell.lines,
        );
    }

    #[test]
    fn the_extension_is_optional_wherever_a_spell_is_named() {
        assert_eq!(with_extension("night_watch"), "night_watch.spell");
        assert_eq!(with_extension("night_watch.spell"), "night_watch.spell");
        assert_eq!(without_extension("night_watch.spell"), "night_watch");
        assert_eq!(without_extension("night_watch"), "night_watch");
    }

    #[test]
    fn a_broken_file_is_an_error_rather_than_a_panic() {
        // A writer mid-edit saves broken TOML constantly, and a frontend
        // hot-reloading must keep what it has rather than go mute.
        assert!(Spells::parse("[first_light]\nlines = 3").is_err());
    }
}
