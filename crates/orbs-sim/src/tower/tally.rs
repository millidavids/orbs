//! What has been done, counted (DESIGN.md §11.5).
//!
//! # Every completion goes through one door
//!
//! There were eight sites in the game where work succeeded and each called
//! [`credit`](super::credit) with what it earned. [`done`] is that call with
//! the *deed* beside the number: what was made, at which instrument, or what
//! kind of thing happened. The tally is what Mastery's seven lines read, so a
//! seam that credits and does not count is a room whose line never moves — and
//! the seams are listed in `tower::mastery`'s tests so a ninth cannot be added
//! without being noticed.
//!
//! # Counts, not a currency
//!
//! A tally key is a count of things done and nothing else: it is never spent,
//! never weighted, and never summed into experience. §11.5 keeps experience the
//! one number that buys anything.
//!
//! # No stream
//!
//! Nothing here is drawn. A tally is a function of the submissions alone, so it
//! replays from `(seed, submissions)` without a stream of its own — and a save
//! carries it only so a restore need not replay to know it.

use std::collections::BTreeMap;

use bevy_ecs::prelude::*;

/// The things a room can do that are not a run at an instrument.
///
/// **A closed set, checked at load**, so a deed authored against an event
/// nothing emits fails `progression.toml` rather than authoring a station
/// nothing can reach. Every name here has exactly one seam that counts it.
pub const EVENTS: [&str; 5] = [FIGURE, SIEGE, SIEGE_WON, BOUND, SECRET];

/// A chant sung to its end (`execute::sing::settle`).
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
    /// **For the runs that stock the tower without being a recipe.** The
    /// menagerie's chant closes a *figure* — an event, which is what its mastery
    /// line counts — and in the same breath gives troops to the arsenal; the
    /// archive's maze finishes at an instrument and shelves a fragment. Both put
    /// a nameable thing on a shelf, which is exactly the question [`sold`] asks,
    /// and both answered no because neither went through [`made`](Self::made).
    ///
    /// So the menagerie and the archive stocked the arsenal and the shelves for
    /// nothing, while the laboratory was paid for the same act — and troops are
    /// what a siege spends. This is the seam that says *a thing was made here*
    /// independently of how the run is counted.
    ///
    /// [`sold`]: Self::sold
    /// [`made`]: Self::made
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
    /// **The question renown asks, and the reason it is asked here.** `done` is
    /// the one door for *events* as well as makings, so a mint on the door
    /// itself would pay for binding a spell and would pay a siege twice — escrow
    /// arrives through the same call, and escrow pays on a loss.
    ///
    /// **A `made:` key is the answer, and three seams set one.**
    /// [`made`](Self::made) is the recipe case; [`making`](Self::making) is for
    /// the two runs that stock the tower without being a recipe. It used to read
    /// *only* `made`, which meant the archive shelved fragments and the
    /// menagerie shelved troops for no standing — this doc said "on a shelf" the
    /// whole time and both of those reach `tower::give`.
    ///
    /// **What still answers no is anything that shelves nothing**, and that is
    /// most of the tower: the lens finds knowledge, the sanctum's pylon solves a
    /// course, the forge's lattice lays a charm on a tool. None of them makes
    /// stock, so none of them is a sale.
    #[must_use]
    pub fn sold(&self) -> bool {
        self.keys.iter().any(|key| key.starts_with("made:"))
    }

    /// What this run put on a shelf, if it put anything there.
    ///
    /// **The same `made:` key [`sold`](Self::sold) asks about**, read for its
    /// name rather than its presence — so the two questions cannot come to
    /// disagree about what a making is. A run makes at most one nameable thing,
    /// which is why this is an `Option` and not a list: the byproduct goes to
    /// the instrument and is not a sale.
    #[must_use]
    pub fn product(&self) -> Option<&str> {
        self.keys.iter().find_map(|key| key.strip_prefix("made:"))
    }
}

/// Count a completion, credit what it earned, and move every line it moved.
///
/// **The order is the order a player reads it in**: the run's own sentence has
/// already been said by the seam; then the level it bought, if any; then what it
/// was worth in standing; then the station it reached, if any; then what that
/// opened. Called where a run *succeeded*, never where one merely ended — see
/// `credit`.
///
/// **Renown is minted only for a making**, which is what [`Work::sold`] asks.
/// The door carries events too, and paying for those would pay a siege twice.
pub fn done(world: &mut World, work: &Work, earned: u64) {
    {
        let mut tally = world.resource_mut::<Tally>();
        for key in work.keys() {
            tally.bump(key);
        }
    }
    super::credit(world, earned);
    if work.sold() {
        // **What the tower is stocked in, measured before the mint reads it.**
        // A store's standing is a *rate* — how many were made lately — so this
        // is the one door that has to see every making, and it is already that
        // door for renown.
        //
        // **Surplus sells**, which is why the order matters: a making of a name
        // that was *already* at full strength is stock the tower did not need,
        // so it is sold rather than shelved and pays a second time. Asked before
        // this making is recorded, or every making would look like its own
        // surplus.
        //
        // **This is the steady state of any sustained loop, not an occasional
        // bonus, and the number says so**: a bound brewing spell holds `Fresh`
        // permanently, so every making past the third inside the window pays
        // double. Measured at `0.11.14`, `clarity` mints 1,103 renown against
        // 996 experience — a little over twice `worth`.
        //
        // That is the mechanic read literally rather than a defect: a loop that
        // outruns its own stores *is* selling the excess. But it means renown
        // from production is uniformly doubled, which **rescales the currency
        // rather than rewarding a behaviour** — so the ten rank thresholds in
        // `progression.toml` are now priced against a number twice what they
        // were authored for, and `orbs-balance` is what settles whether they
        // move or this does.
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
