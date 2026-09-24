//! What has been done, counted (DESIGN.md §11.5).
//!
//! Every completion goes through one door. [`done`] is
//! [`credit`](super::credit) with the *deed* beside the number: what was made,
//! at which instrument, or what kind of thing happened. Mastery's seven lines
//! read the tally, so a seam that credits and does not count is a room whose
//! line never moves — `tower::mastery`'s tests list the seams so a ninth cannot
//! slip in.
//!
//! Counts, not a currency: a tally key is never spent, never weighted, and never
//! summed into experience. §11.5 keeps experience the one number that buys
//! anything.
//!
//! No stream. A tally is a function of the submissions alone, so it replays from
//! `(seed, submissions)`; a save carries it only so a restore need not replay.

use std::collections::BTreeMap;

use bevy_ecs::prelude::*;

/// The things a room can do that are not a run at an instrument.
///
/// A closed set, checked at load, so a deed authored against an event nothing
/// emits fails `progression.toml` rather than making a station nothing reaches.
/// Every name here has exactly one seam that counts it.
pub const EVENTS: [&str; 5] = [FIGURE, SIEGE, SIEGE_WON, BOUND, SECRET];

/// A beast held at the menagerie's circle (`execute::summon`).
///
/// The chant's name, kept: a tally is carried in a save, so renaming the key
/// would zero every menagerie station a tower had already reached.
pub const FIGURE: &str = "figure";
/// A siege settled, held or fallen (`execute::defend::report::settle`).
pub const SIEGE: &str = "siege";
/// A siege held (the same seam).
pub const SIEGE_WON: &str = "siege_won";
/// A spell bound (`spell::bind`).
pub const BOUND: &str = "bound";
/// A secret recipe found (`tower::learned::discover`).
pub const SECRET: &str = "secret";

/// How many times each key has been counted.
#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct Tally(BTreeMap<String, u32>);

impl Tally {
    /// How many times `key` has been counted.
    #[must_use]
    pub fn count(&self, key: &str) -> u32 {
        self.0.get(key).copied().unwrap_or_default()
    }

    /// Every key and its count, in key order.
    pub fn entries(&self) -> impl Iterator<Item = (&str, u32)> {
        self.0.iter().map(|(key, count)| (key.as_str(), *count))
    }

    /// Put a tally back, for a save.
    pub(crate) fn restore(&mut self, entries: impl IntoIterator<Item = (String, u32)>) {
        self.0 = entries.into_iter().collect();
    }

    fn bump(&mut self, key: &str) {
        let count = self.0.entry(key.to_owned()).or_default();
        *count = count.saturating_add(1);
    }
}

/// One completion, as the tally sees it: the keys it counts under.
///
/// Built by the seam, read by [`done`]. A run at the alembic that made a
/// clarity counts under `at:alembic`, `made:clarity` and `potion` at once, so a
/// deed can ask for any of the three.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Work {
    keys: Vec<String>,
}

impl Work {
    /// A run finished at `instrument` that made nothing nameable.
    #[must_use]
    pub fn at(instrument: &str) -> Self {
        Self {
            keys: vec![format!("at:{instrument}")],
        }
    }

    /// A run finished at `instrument` that made `product`.
    #[must_use]
    pub fn made(instrument: &str, product: &str, potion: bool, scroll: bool) -> Self {
        let mut work = Self::at(instrument).making(product);
        if potion {
            work.keys.push("potion".to_owned());
        }
        if scroll {
            work.keys.push("scroll".to_owned());
        }
        work
    }

    /// Also record that this run put `product` on a shelf.
    ///
    /// For the runs that stock the tower without being a recipe: the menagerie
    /// gives troops to the arsenal, the archive shelves a fragment. Neither goes
    /// through [`made`](Self::made), so both answered [`sold`] no and stocked
    /// the tower for nothing while the laboratory was paid for the same act.
    ///
    /// [`sold`]: Self::sold
    #[must_use]
    pub fn making(mut self, product: &str) -> Self {
        self.keys.push(format!("made:{product}"));
        self
    }

    /// Something a room did that is not a run. One of [`EVENTS`].
    #[must_use]
    pub fn event(name: &str) -> Self {
        debug_assert!(
            EVENTS.contains(&name),
            "`{name}` is not an event the tally knows"
        );
        Self {
            keys: vec![format!("event:{name}")],
        }
    }

    /// The keys this counts under.
    #[must_use]
    pub fn keys(&self) -> &[String] {
        &self.keys
    }

    /// Whether this run put a nameable thing on a shelf.
    ///
    /// The question renown asks, and asked here because `done` is the one door
    /// for events too: a mint on the door itself would pay for binding a spell
    /// and pay a siege twice, since escrow arrives the same way and pays on a
    /// loss.
    ///
    /// A `made:` key is the answer, set by [`made`](Self::made) for recipes and
    /// [`making`](Self::making) for the runs that stock the tower without being
    /// one. Anything that shelves nothing answers no — the lens, the pylon, the
    /// lattice — because none of them makes stock.
    #[must_use]
    pub fn sold(&self) -> bool {
        self.keys.iter().any(|key| key.starts_with("made:"))
    }

    /// What this run put on a shelf, if it put anything there.
    ///
    /// The same `made:` key [`sold`](Self::sold) asks about, read for its name
    /// rather than its presence, so the two cannot disagree. A run makes at most
    /// one nameable thing — the byproduct goes to the instrument and is not a
    /// sale.
    #[must_use]
    pub fn product(&self) -> Option<&str> {
        self.keys.iter().find_map(|key| key.strip_prefix("made:"))
    }
}

/// Count a completion, credit what it earned, and move every line it moved.
///
/// The order is the order a player reads it in: the seam's own sentence, the
/// level it bought, what it was worth in standing, the station it reached, what
/// that opened. Called where a run *succeeded*, never where one merely ended.
///
/// Renown is minted only for a making ([`Work::sold`]); the door carries events
/// too, and paying for those would pay a siege twice.
pub fn done(world: &mut World, work: &Work, earned: u64) {
    {
        let mut tally = world.resource_mut::<Tally>();
        for key in work.keys() {
            tally.bump(key);
        }
    }
    super::credit(world, earned);
    if work.sold() {
        // A store's standing is a rate, so this door has to see every making.
        // Surplus sells: a making of a name already at full strength is stock
        // the tower did not need, so it pays twice. Asked before this making is
        // recorded, or every making would look like its own surplus.
        //
        // That is the steady state of any sustained loop, not an occasional
        // bonus — a bound brewing spell holds `Fresh` permanently, so at
        // `0.11.14` `clarity` minted 1,103 renown against 996 experience. It
        // rescales the currency rather than rewarding a behaviour, so
        // `progression.toml`'s ten thresholds are priced against twice what
        // they were authored for; `orbs-balance` settles whether they move or
        // this does.
        let over = work
            .product()
            .is_some_and(|named| super::supply_of(world, named) == super::Supply::Fresh);
        if let Some(named) = work.product() {
            let named = named.to_owned();
            super::made(world, &named);
        }
        let worth = super::renown::worth(earned);
        super::renown::earn(world, if over { worth * 2 } else { worth });
    }
    super::mastery::advance(world);
}

/// Count an event that earns nothing.
pub fn note(world: &mut World, event: &str) {
    done(world, &Work::event(event), 0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_run_that_made_a_potion_counts_three_ways() {
        let work = Work::made("alembic", "clarity", true, false);
        assert_eq!(work.keys(), ["at:alembic", "made:clarity", "potion"]);
        let scroll = Work::made("lectern", "gleaning-scroll", false, true);
        assert_eq!(
            scroll.keys(),
            ["at:lectern", "made:gleaning-scroll", "scroll"]
        );
    }

    #[test]
    fn the_tally_counts_and_reads_back() {
        let mut tally = Tally::default();
        tally.bump("potion");
        tally.bump("potion");
        assert_eq!(tally.count("potion"), 2);
        assert_eq!(tally.count("scroll"), 0);
        let entries: Vec<(&str, u32)> = tally.entries().collect();
        assert_eq!(entries, [("potion", 2)]);
    }

    #[test]
    fn every_event_has_a_name_the_deed_parser_accepts() {
        // The set is what `Deed::check` reads, so a name here that a deed could
        // not spell would be a seam counting toward nothing.
        for event in EVENTS {
            assert!(!event.is_empty());
            assert!(!event.contains(':'), "{event} would split as a key");
        }
    }
}
