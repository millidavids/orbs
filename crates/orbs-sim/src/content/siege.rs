//! What the arsenal is worth in a siege (DESIGN.md §5.1, §11.5).
//!
//! Separate from [`Materials`](super::Materials) because that file says what a
//! thing *is* and this one says what it *does* in one room. Folding them
//! together would put a combat number on a reagent that will never see one.
//!
//! Authored rather than hardcoded (rule 6): balancing this domain means moving
//! these numbers dozens of times, and in Rust that is a recompile per guess.
//!
//! Every row is a standing supply (§19, *Supply*): its [`Rates`] say how many
//! must be held to run, how many each round draws, and what that buys.

use std::collections::BTreeMap;

use bevy_ecs::resource::Resource;
use serde::Deserialize;

use super::rate::{Rate, Rates};
use crate::parser::Verb;
use crate::tower::dice::{Die, Effect};

/// The compiled-in default, so a headless `Sim` needs no filesystem.
const BUILTIN: &str = include_str!("../../content/siege.toml");

/// The file's name, for an error a writer can act on.
const FILE: &str = "siege.toml";

/// What a supply's draw does to the siege: see `siege.toml`'s header.
///
/// An enum so an unknown word fails at load, rather than falling through a
/// string match into a roll modifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// Refills the line.
    Troops,
    /// Puts fight back.
    Vigour,
    /// Holds room above what was levied.
    Fortify,
    /// Ends the siege with both sides standing.
    Still,
    /// Adds to every garrison roll.
    Bonus,
    /// Draw twice, keep the better.
    Advantage,
    /// Draw twice, keep the worse.
    Disadvantage,
    /// Roll a bigger die.
    Upgrade,
}

impl Kind {
    /// Whether this edits the round's rolls rather than the band.
    #[must_use]
    pub const fn modifies_rolls(self) -> bool {
        matches!(
            self,
            Self::Bonus | Self::Advantage | Self::Disadvantage | Self::Upgrade
        )
    }
}

/// What one arsenal item does as a supply in a siege.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spendable {
    /// Which word opens it — `deploy`, `quaff` or `wield`.
    pub verb: String,
    /// What a draw does.
    pub kind: Kind,
    /// The die, for `kind = "upgrade"`.
    #[serde(default)]
    pub die: String,
    /// How fast it can draw, slowest first.
    pub rates: Rates,
    /// Set on a demon: what a hold bound as one puts in the arsenal (§19,
    /// *Supply*). Absent on everything a recipe makes.
    #[serde(default)]
    pub summoned: Option<Summoned>,
}

/// What a beast held at the circle yields when it is bound as this demon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Summoned {
    /// How many a hold within par binds.
    pub yields: u32,
    /// How many a hold past par binds.
    pub past_par: u32,
}

/// Everything the arsenal can spend in a siege, by name.
#[derive(Debug, Clone, Deserialize, Resource)]
#[serde(from = "BTreeMap<String, Spendable>")]
pub struct Spendables {
    by_name: BTreeMap<String, Spendable>,
    /// The rows a hold can be bound as, listed once at load: the scene asks
    /// every tick.
    demons: Vec<String>,
}

impl From<BTreeMap<String, Spendable>> for Spendables {
    fn from(by_name: BTreeMap<String, Spendable>) -> Self {
        let demons = by_name
            .iter()
            .filter(|(_, entry)| entry.summoned.is_some())
            .map(|(name, _)| name.clone())
            .collect();
        Self { by_name, demons }
    }
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
    /// expected shape, or a row's rates break the rule that no single item does
    /// anything (`content::rate`).
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

    /// Every demon the circle can bind, alphabetically.
    pub fn demons(&self) -> impl Iterator<Item = &str> {
        self.demons.iter().map(String::as_str)
    }
}

impl Spendable {
    /// Whether this edits the round's rolls rather than the band.
    #[must_use]
    pub const fn modifies_rolls(&self) -> bool {
        self.kind.modifies_rolls()
    }

    /// The [`Effect`] a draw at `rate` stages, if this edits a roll.
    #[must_use]
    pub fn effect_at(&self, rate: &Rate) -> Option<Effect> {
        match self.kind {
            Kind::Bonus => Some(Effect::Bonus(rate.worth)),
            Kind::Advantage => Some(Effect::Advantage),
            Kind::Disadvantage => Some(Effect::Disadvantage),
            Kind::Upgrade => Die::named(&self.die).map(Effect::Upgrade),
            Kind::Troops | Kind::Vigour | Kind::Fortify | Kind::Still => None,
        }
    }

    /// What a draw at `rate` buys, never below nought.
    #[must_use]
    pub const fn worth_at(&self, rate: &Rate) -> u32 {
        rate.worth.unsigned_abs()
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
        // A hold refills the standing garrison now (§19, *Supply*).
        assert!(spendables.get("troop").is_none());
    }

    #[test]
    fn an_unknown_kind_is_refused_at_load() {
        // A silent fallback would leave the item in the arsenal doing nothing
        // with every test green.
        let text = r#"
            [warding]
            verb = "quaff"
            kind = "fireball"
            rates = [{ opens = 6, draws = 2, worth = 2 }]
        "#;
        assert!(Spendables::parse(text).is_err());
        assert!(Spendables::parse(&text.replace("fireball", "bonus")).is_ok());
    }

    #[test]
    fn every_upgrade_names_one_of_the_seven_dice() {
        let spendables = Spendables::builtin();
        for name in spendables.names() {
            let entry = spendables.get(name).expect("just listed");
            if entry.kind == Kind::Upgrade {
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
            let effect = entry.effect_at(entry.rates.slowest());
            assert_eq!(
                effect.is_some(),
                entry.modifies_rolls(),
                "{name} ({:?}) and its effect disagree",
                entry.kind
            );
        }
    }

    #[test]
    fn a_magnitude_is_worth_something_at_every_rate() {
        // A heal of nought or a bonus of nought would draw for nothing.
        let spendables = Spendables::builtin();
        for name in spendables.names() {
            let entry = spendables.get(name).expect("just listed");
            if matches!(
                entry.kind,
                Kind::Bonus | Kind::Vigour | Kind::Fortify | Kind::Troops
            ) {
                for rate in entry.rates.iter() {
                    assert!(rate.worth > 0, "{name} draws {rate:?} for nothing");
                }
            }
        }
    }

    #[test]
    fn the_potions_are_quaffed_and_the_scrolls_wielded() {
        // The split §19 records: a scroll keeps `wield`, and drinking is its
        // own word.
        let spendables = Spendables::builtin();
        assert_eq!(spendables.verb_for("mending"), Some(Verb::Quaff));
        assert_eq!(spendables.verb_for("quickening-scroll"), Some(Verb::Wield));
    }
}
