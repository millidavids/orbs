//! What the arsenal is worth in a siege (DESIGN.md §5.1, §11.5).
//!
//! Separate from [`Materials`](super::Materials) because that file says what a
//! thing *is* and this one says what it *does* in one room. Folding them
//! together would put a combat number on a reagent that will never see one.
//!
//! **Authored rather than hardcoded** (rule 6). Balancing this domain means
//! moving these numbers dozens of times, and in Rust that is a recompile per
//! guess.

use std::collections::BTreeMap;

use bevy_ecs::resource::Resource;
use serde::Deserialize;

use crate::parser::Verb;
use crate::tower::dice::{Die, Effect};

/// The compiled-in default, so a headless `Sim` needs no filesystem.
const BUILTIN: &str = include_str!("../../content/siege.toml");

/// The file's name, for an error a writer can act on.
const FILE: &str = "siege.toml";

/// What one arsenal item does when it is spent in a siege.
#[derive(Debug, Clone, Deserialize)]
pub struct Spendable {
    /// Which word spends it — `deploy`, `quaff` or `wield`.
    pub verb: String,
    /// Which of the five shapes this is.
    pub kind: String,
    /// Troops, for `kind = "troops"`.
    #[serde(default)]
    pub count: u32,
    /// Fight, for `kind = "vigour"`.
    #[serde(default)]
    pub points: u32,
    /// The adjustment, for `kind = "bonus"`. May be negative.
    #[serde(default)]
    pub amount: i32,
    /// The die, for `kind = "upgrade"`.
    #[serde(default)]
    pub die: String,
}

/// Everything the arsenal can spend in a siege, by name.
#[derive(Debug, Clone, Deserialize, Resource)]
#[serde(transparent)]
pub struct Spendables {
    by_name: BTreeMap<String, Spendable>,
}

impl Spendables {
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

    /// Parse a siege file.
    ///
    /// # Errors
    ///
    /// [`ContentError`](super::ContentError) if the text is not valid TOML of the
    /// expected shape.
    pub fn parse(text: &str) -> Result<Self, super::ContentError> {
        super::load::parse(FILE, text)
    }

    /// What `name` does, if it can be spent at all.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Spendable> {
        self.by_name.get(name)
    }

    /// Every spendable name, alphabetically.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.by_name.keys().map(String::as_str)
    }

    /// Which verb spends `name`.
    #[must_use]
    pub fn verb_for(&self, name: &str) -> Option<Verb> {
        let word = &self.get(name)?.verb;
        Verb::ALL.into_iter().find(|verb| verb.canonical() == word)
    }
}

impl Spendable {
    /// The [`Effect`] this is, if it is one of the three that edit a roll.
    ///
    /// Returns `None` for `troops` and `vigour`, which change the *band* rather
    /// than the dice — the caller distinguishes them, because a bonus and a
    /// reinforcement are not the same kind of thing however similar they look
    /// in a table.
    #[must_use]
    pub fn effect(&self) -> Option<Effect> {
        match self.kind.as_str() {
            "bonus" => Some(Effect::Bonus(self.amount)),
            "advantage" => Some(Effect::Advantage),
            "disadvantage" => Some(Effect::Disadvantage),
            "upgrade" => Die::named(&self.die).map(Effect::Upgrade),
            _ => None,
        }
    }
}

impl Default for Spendables {
    fn default() -> Self {
        Self::builtin()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_builtin_file_parses() {
        let spendables = Spendables::builtin();
        assert!(spendables.names().count() >= 8);
        assert!(spendables.get("troop").is_some());
    }

    #[test]
    fn every_entry_names_a_kind_the_game_understands() {
        // **An unknown kind fails the build**, the way an unknown tint does in
        // `materials.toml`: a silent fallback would make a typo indistinguishable
        // from a deliberate choice, and the item would sit in the arsenal doing
        // nothing with every test green.
        let spendables = Spendables::builtin();
        for name in spendables.names() {
            let entry = spendables.get(name).expect("just listed");
            assert!(
                matches!(
                    entry.kind.as_str(),
                    "troops" | "vigour" | "bonus" | "advantage" | "disadvantage" | "upgrade"
                ),
                "{name} has an unknown kind {:?}",
                entry.kind,
            );
            if entry.kind == "upgrade" {
                assert!(
                    Die::named(&entry.die).is_some(),
                    "{name} upgrades to {:?}, which is not one of the seven",
                    entry.die,
                );
            }
        }
    }

    #[test]
    fn every_entry_names_a_verb_that_can_spend_it() {
        // A `verb` nobody has is an item that can never be spent — the same
        // silent-nothing an unknown kind would be.
        let spendables = Spendables::builtin();
        for name in spendables.names() {
            let verb = spendables.verb_for(name);
            assert!(
                matches!(verb, Some(Verb::Deploy | Verb::Quaff | Verb::Wield)),
                "{name} is spent by {:?}, which is not a word that spends anything",
                spendables.get(name).map(|s| &s.verb),
            );
        }
    }

    #[test]
    fn a_roll_editing_entry_produces_an_effect_and_the_others_do_not() {
        let spendables = Spendables::builtin();
        for name in spendables.names() {
            let entry = spendables.get(name).expect("just listed");
            let effect = entry.effect();
            match entry.kind.as_str() {
                "troops" | "vigour" => assert!(
                    effect.is_none(),
                    "{name} changes a band and also produced a dice effect",
                ),
                _ => assert!(effect.is_some(), "{name} produced no effect"),
            }
        }
    }

    #[test]
    fn the_troop_is_deployed_and_the_potions_are_quaffed() {
        // The split §19 records: a scroll keeps `wield`, a troop is deployed,
        // and drinking is its own word. A table that got this wrong would offer
        // `quaff troop`.
        let spendables = Spendables::builtin();
        assert_eq!(spendables.verb_for("troop"), Some(Verb::Deploy));
        assert_eq!(spendables.verb_for("mending"), Some(Verb::Quaff));
        assert_eq!(spendables.verb_for("quickening-scroll"), Some(Verb::Wield));
    }
}
