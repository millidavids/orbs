//! How well stocked the tower is in a thing, measured as a rate (§11.5, §19).
//!
//! A rate, not a timestamp, which kept a thousand potions fresh off one restock
//! (§19). The count held decides whether a supply runs; this decides how fast.
//! It does not decay while the game is closed, since there is no wall clock in
//! the sim.

use std::collections::BTreeMap;

use bevy_ecs::prelude::*;

/// How long a making counts toward the rate, in ticks.
///
/// Half an hour, against a siege cadence of twenty minutes and a clarity that
/// takes under two. Long enough that a player working by hand keeps the two or
/// three things they actually use, short enough that a full shelf wants a spell.
pub const WINDOW: u64 = 1_800;

/// Makings inside [`WINDOW`] to keep a store at full strength.
pub const FRESH_AT: usize = 3;

/// Makings inside [`WINDOW`] to keep a store worth anything at all.
pub const THIN_AT: usize = 1;

/// The most makings worth remembering for one name.
///
/// A ring rather than a growing list: nothing above [`FRESH_AT`] changes any
/// answer, so a spell hammering one recipe cannot grow the save. The oldest is
/// dropped, which is the right end — the rate is about *lately*.
const REMEMBERED: usize = FRESH_AT;

/// How well stocked the tower is in one thing, lately.
///
/// Words, as the domain's readings are, because a fraction is unaskable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Freshness {
    /// Made often enough that a supply may run at any rate.
    Fresh,
    /// Made, but not lately. A supply runs only at its slowest.
    Thin,
    /// Not made in long enough that the stores are out. Nothing flows.
    Spent,
}

impl Freshness {
    /// The word the panel prints and a spell asks for.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Fresh => "fresh",
            Self::Thin => "thin",
            Self::Spent => "spent",
        }
    }

    /// Every word, for the lints that walk them.
    pub const ALL: [Self; 3] = [Self::Fresh, Self::Thin, Self::Spent];

    /// The better stocked of two. A siege reads freshness as at `defend` or
    /// now, whichever is better, so waiting over a round costs nothing.
    #[must_use]
    pub const fn better(self, other: Self) -> Self {
        match (self, other) {
            (Self::Fresh, _) | (_, Self::Fresh) => Self::Fresh,
            (Self::Thin, _) | (_, Self::Thin) => Self::Thin,
            _ => Self::Spent,
        }
    }
}

/// When the tower last made each thing, lately.
///
/// One short, bounded list per name. `BTreeMap` for `Tally`'s reason: a save is
/// read by people, and key order is what makes a diff legible.
#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct Stores(BTreeMap<String, Vec<u64>>);

impl Stores {
    /// Record that one `named` was made at `now`.
    pub fn made(&mut self, named: &str, now: u64) {
        let makings = self.0.entry(named.to_owned()).or_default();
        makings.push(now);
        prune(makings, now);
        // Keep the newest, drop the oldest: the rate is about *lately*, and
        // nothing past `FRESH_AT` changes an answer.
        if makings.len() > REMEMBERED {
            let excess = makings.len() - REMEMBERED;
            makings.drain(..excess);
        }
    }

    /// How many `named` were made inside [`WINDOW`] of `now`.
    #[must_use]
    pub fn rate(&self, named: &str, now: u64) -> usize {
        self.0
            .get(named)
            .map(|makings| makings.iter().filter(|at| inside(**at, now)).count())
            .unwrap_or_default()
    }

    /// How well stocked the tower is in `named`.
    #[must_use]
    pub fn store(&self, named: &str, now: u64) -> Freshness {
        match self.rate(named, now) {
            rate if rate >= FRESH_AT => Freshness::Fresh,
            rate if rate >= THIN_AT => Freshness::Thin,
            _ => Freshness::Spent,
        }
    }

    /// Every name and the ticks it was made at, in key order — for a save.
    pub fn entries(&self) -> impl Iterator<Item = (&str, &[u64])> {
        self.0
            .iter()
            .map(|(name, makings)| (name.as_str(), makings.as_slice()))
    }

    /// Put stores back, for a save.
    pub(crate) fn restore(&mut self, entries: impl IntoIterator<Item = (String, Vec<u64>)>) {
        self.0 = entries.into_iter().collect();
    }

    /// Whether anything has ever been recorded.
    ///
    /// Tells a fresh tower from an old save: reading a pre-this document's
    /// silence as *every store is spent* would empty a returning arsenal.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Whether a making at `at` still counts at `now`.
const fn inside(at: u64, now: u64) -> bool {
    now.saturating_sub(at) < WINDOW
}

/// Drop makings that have fallen out of the window.
fn prune(makings: &mut Vec<u64>, now: u64) {
    makings.retain(|at| inside(*at, now));
}

/// Record that one `named` was made, now.
pub fn made(world: &mut World, named: &str) {
    let now = world.resource::<crate::tick::Tick>().get();
    world.resource_mut::<Stores>().made(named, now);
}

/// How well stocked the tower is in `named`, now.
#[must_use]
pub fn freshness_of(world: &World, named: &str) -> Freshness {
    let now = world.resource::<crate::tick::Tick>().get();
    world.resource::<Stores>().store(named, now)
}

/// The set an arsenal item belongs to, so a spell can walk them.
///
/// Singular, because the word names the cursor as well as the set: `for each
/// store` binds `store`, and the body reads `if store has thin`.
pub const STORE: &str = "store";

/// The words an arsenal item answers `has` with.
#[must_use]
pub fn readings() -> Vec<&'static str> {
    Freshness::ALL.into_iter().map(Freshness::word).collect()
}

/// Keep every arsenal item's group and reading in step with its store.
///
/// A system, because a store changes as time passes. It writes only when the
/// word changes, so a watched hour and a collapsed one issue the same ids. The
/// group marker goes on regardless, so a spell can walk an arsenal unsurveyed.
pub fn stocktake(world: &mut World) {
    let Some(arsenal) = super::keep(world) else {
        return;
    };
    let now = world.resource::<Stores>().clone();
    let tick = world.resource::<crate::tick::Tick>().get();
    for node in super::children_of(world, arsenal) {
        let Some(named) = world.get::<super::Name>(node).map(|name| name.0.clone()) else {
            continue;
        };
        // A log is a child of the arsenal too, and it has no store.
        if world.get::<super::Stock>(node).is_none() {
            continue;
        }
        world
            .entity_mut(node)
            .insert(super::Grouped(STORE.to_owned()));

        let want = now.store(&named, tick).word();
        super::sync_readings(world, node, &readings(), &[(want, None)]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_restock_is_never_fresh_however_much_it_restocks() {
        // The hole this file exists to close: §19 withdrew a perishable arsenal
        // because a timer keeps a thousand potions fresh off one restock. A
        // rate cannot — one making is one, whatever it stocked.
        let mut stores = Stores::default();
        stores.made("warding", 0);
        assert_eq!(stores.store("warding", 0), Freshness::Thin);
        assert_eq!(stores.store("warding", 10), Freshness::Thin);
    }

    #[test]
    fn the_rate_is_what_reads_and_the_pile_is_never_asked_about() {
        // Total stock never appears in this file's arithmetic, which is what
        // lets `Stock::Counted` stay as it is.
        let mut stores = Stores::default();
        for at in 0..FRESH_AT as u64 {
            stores.made("warding", at);
        }
        assert_eq!(stores.store("warding", 0), Freshness::Fresh);
    }

    #[test]
    fn a_store_thins_and_then_runs_out_as_the_window_passes() {
        let mut stores = Stores::default();
        for at in 0..FRESH_AT as u64 {
            stores.made("warding", at);
        }
        assert_eq!(stores.store("warding", 0), Freshness::Fresh);
        // The oldest makings fall out first, so a store passes through `Thin`
        // on its way and a player gets warning rather than a cliff.
        assert_eq!(stores.store("warding", WINDOW), Freshness::Thin);
        // ...and past the newest making's own window it is out. `WINDOW + 1` is
        // not far enough: the last making was at tick 2, counted until 1,802.
        assert_eq!(stores.store("warding", WINDOW + 10), Freshness::Spent);
    }

    #[test]
    fn a_thing_never_made_is_spent() {
        let stores = Stores::default();
        assert_eq!(stores.store("warding", 0), Freshness::Spent);
    }

    #[test]
    fn what_is_remembered_is_bounded_however_hard_a_spell_hammers_it() {
        // A bound spell can make thousands. The save may not grow with them.
        let mut stores = Stores::default();
        for at in 0..1_000 {
            stores.made("clarity", at);
        }
        let (_, makings) = stores.entries().next().expect("something was made");
        assert!(
            makings.len() <= REMEMBERED,
            "a thousand makings kept {} entries",
            makings.len(),
        );
        assert_eq!(stores.store("clarity", 1_000), Freshness::Fresh);
    }

    #[test]
    fn the_better_store_is_the_one_made_more_lately() {
        use Freshness::{Fresh, Spent, Thin};
        assert_eq!(Spent.better(Thin), Thin);
        assert_eq!(Thin.better(Fresh), Fresh);
        assert_eq!(Spent.better(Spent), Spent);
    }
}
