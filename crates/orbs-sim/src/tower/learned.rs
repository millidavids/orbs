//! Recipes the player does not know yet, and the roll that finds them.
//!
//! §11 puts discovery in `archive/` — *"decipherment; powers all discovery"* —
//! and this is a second source, in the lens. The two are not the same thing and
//! the split is deliberate:
//!
//! | The archive discovers | The lens discovers |
//! |---|---|
//! | **your own capability** — verbs, hidden directories, what fragments assemble | **other people's knowledge** — what a far wizard knows how to make |
//!
//! *"Archive powers all discovery"* is about the first. You do not decipher a
//! recipe out of your own shelves; you steal it from somebody who already had
//! it, which is what a broken ward is for.
//!
//! # The chance climbs, and resets
//!
//! A flat chance would mean a player could be forty solves in with nothing to
//! show; a certainty would make the lottery a queue. It starts small and rises
//! with every solve that finds nothing, so bad luck is bounded — and it resets
//! the moment something lands, so good luck is not compounding.
//!
//! # It retires when there is nothing left
//!
//! Three secrets ship. The roll stops once all three are known, rather than
//! rolling for ever against an empty pool — a domain that keeps promising
//! something it cannot deliver is worse than one that stops.

use std::collections::BTreeSet;

use bevy_ecs::prelude::*;
use rand::Rng as _;

use crate::content::Recipes;
use crate::rng::{RngStream, Rngs};

/// What the player has learned to make, beyond what they started knowing.
///
/// **Player state, not content.** `Recipes` stays immutable — it is loaded once
/// and never hot-reloaded, because a recipe reaches a decision and swapping one
/// mid-session would break replay from `(seed, submissions)`. This is the
/// mutable half, and it is reproducible from that same pair: a discovery is
/// rolled from a seeded stream on a deterministic tick.
#[derive(Resource, Debug, Default, Clone)]
pub struct Learned {
    known: BTreeSet<String>,
    /// Solves since the last discovery. The climbing half of the chance.
    since: u32,
}

/// The chance of a discovery on the first solve after one lands.
const BASE: u32 = 4;

/// How much each fruitless solve adds, in percentage points.
const STEP: u32 = 4;

/// Where the chance reaches certainty.
///
/// `4 + 4 × 24` is 100, so the **25th** solve after a discovery cannot fail.
/// The mean interval is about six.
///
/// Public because it is a *promise* rather than an implementation detail — a
/// player is owed a bound on bad luck, and the tests below are what hold it.
pub const CERTAIN: u32 = 24;

impl Learned {
    /// Whether the player can make this.
    ///
    /// **A name no recipe marks secret is always known.** The set holds only
    /// what has been *found*, so an ordinary output needs no entry and the save
    /// carries three strings rather than forty.
    #[must_use]
    pub fn knows(&self, recipes: &Recipes, name: &str) -> bool {
        !recipes.is_secret(name) || self.known.contains(name)
    }

    /// Every secret recipe found so far, by name, in **alphabetical** order.
    ///
    /// **Not the same order as [`found`](Self::found)**, which walks
    /// `recipes.toml` — `dreaming, mending, vigour` here against `mending,
    /// dreaming, vigour` there. This is a `BTreeSet` and that is deliberate: a
    /// save's bytes must not depend on insertion order, or two routes to one
    /// world would write different files. Anything a *player* reads wants
    /// `found`; this is for the save.
    pub fn known(&self) -> impl Iterator<Item = &str> {
        self.known.iter().map(String::as_str)
    }

    /// Put a found set back, for a save.
    ///
    /// **Not `discover`**, which rolls: a restore reads what was found rather
    /// than finding it again, so `RngStream::Lens` does not move.
    pub(crate) fn restore(&mut self, known: impl IntoIterator<Item = String>, since: u32) {
        self.known = known.into_iter().collect();
        self.since = since;
    }

    /// Solves since the last discovery.
    #[must_use]
    pub const fn since(&self) -> u32 {
        self.since
    }

    /// Everything found so far, **in `recipes.toml`'s own order**.
    ///
    /// Ordered against the content rather than by the `BTreeSet` it is stored in,
    /// which sorts alphabetically — so this returned `dreaming, mending, vigour`
    /// while the reveal sequence is `mending, dreaming, vigour`. The file's order
    /// is load-bearing everywhere else in this module (`discover` and `learn`
    /// both take the first unfound one), so an accessor answering in a different
    /// order is a trap for its first caller.
    #[must_use]
    pub fn found<'a>(&self, recipes: &'a Recipes) -> Vec<&'a str> {
        recipes
            .secrets()
            .into_iter()
            .filter(|made| self.known.contains(*made))
            .collect()
    }
}

/// Roll for a discovery, and return what was found.
///
/// **Called once per broken seal**, and it is the only thing that advances the
/// counter — so a player who never scrys never gets closer, and one who
/// automates the lens gets there at a rate they can feel.
pub fn discover(world: &mut World) -> Option<String> {
    let pool: Vec<String> = {
        let recipes = world.resource::<Recipes>();
        let learned = world.resource::<Learned>();
        recipes
            .secrets()
            .into_iter()
            .filter(|name| !learned.known.contains(*name))
            .map(str::to_owned)
            .collect()
    };

    // **Retired rather than rolled against nothing.** The counter stops too, so
    // a save made after the last discovery does not carry a number that means
    // nothing.
    if pool.is_empty() {
        return None;
    }

    let chance = BASE.saturating_add(STEP.saturating_mul(world.resource::<Learned>().since));
    let roll = world
        .resource_mut::<Rngs>()
        .stream(RngStream::Lens)
        .random_range(0..100);

    let mut learned = world.resource_mut::<Learned>();
    if roll >= chance {
        learned.since = learned.since.saturating_add(1);
        return None;
    }

    // **The file's order, not a uniform draw.** `recipes.toml`'s order is the
    // designer's recommendation (§19 says so of `recall`'s primary route), so
    // the reveal sequence is authored rather than left to a die — and it is
    // deterministic, which a replay needs.
    let found = pool.into_iter().next()?;
    learned.known.insert(found.clone());
    learned.since = 0;
    Some(found)
}

/// Learn a secret outright, for a tester's `debug_learn`.
///
/// **Not a second discovery path.** It writes the same set `discover` writes and
/// resets the same counter, so a tower reached this way and one reached by
/// probing are the same tower — which is the property that makes a See-it line
/// worth anything. What it skips is only the roll.
///
/// `None` means *the next one in the file's order*, which is the order the lens
/// itself reveals them in.
#[must_use]
pub fn learn(world: &mut World, wanted: Option<&str>) -> Option<String> {
    let found = {
        let recipes = world.resource::<Recipes>();
        let learned = world.resource::<Learned>();
        let mut unfound = recipes
            .secrets()
            .into_iter()
            .filter(|name| !learned.known.contains(*name));
        match wanted {
            // A name that is not a secret, or one already known, is refused —
            // both are states the game cannot reach, and testing from one is
            // testing the tool.
            Some(wanted) => unfound.find(|name| *name == wanted).map(str::to_owned),
            None => unfound.next().map(str::to_owned),
        }
    };

    let found = found?;
    let mut learned = world.resource_mut::<Learned>();
    learned.known.insert(found.clone());
    learned.since = 0;
    Some(found)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Sim;

    #[test]
    fn the_chance_reaches_certainty_and_the_arithmetic_says_where() {
        // The bound, as an assertion rather than a comment: a player who has
        // been unlucky twenty-four times over cannot be unlucky again.
        assert_eq!(BASE + STEP * CERTAIN, 100);
    }

    #[test]
    fn a_discovery_arrives_and_resets_the_climb() {
        let mut sim = Sim::new(1);
        let mut solves = 0;
        let found = loop {
            solves += 1;
            assert!(solves <= CERTAIN + 1, "no discovery in {solves} solves");
            if let Some(found) = discover(sim.world_mut()) {
                break found;
            }
        };

        let learned = sim.world().resource::<Learned>();
        assert!(learned.knows(sim.world().resource::<Recipes>(), &found));
        assert_eq!(learned.since(), 0, "the climb did not reset");
    }

    #[test]
    fn what_has_been_found_reads_in_the_files_order() {
        // The order is load-bearing — `discover` and `learn` both take the first
        // unfound secret — so an accessor answering alphabetically would put
        // `dreaming` before `mending` and contradict the sequence a player met.
        let mut sim = Sim::new(1);
        assert!(learn(sim.world_mut(), None).is_some());
        assert!(learn(sim.world_mut(), None).is_some());

        let recipes = sim.world().resource::<Recipes>();
        let found = sim.world().resource::<Learned>().found(recipes);
        let expected: Vec<&str> = recipes.secrets().into_iter().take(2).collect();
        assert_eq!(found, expected, "the reveal order was not preserved");
    }

    #[test]
    fn the_roll_retires_once_every_secret_is_found() {
        // Three secrets ship. Rolling for ever against an empty pool would make
        // the domain keep promising something it cannot deliver.
        let mut sim = Sim::new(1);
        let secrets = sim.world().resource::<Recipes>().secrets().len();
        assert!(secrets > 0, "nothing is authored secret, so nothing gates");

        let mut found = 0;
        for _ in 0..(CERTAIN + 1) * 8 {
            if discover(sim.world_mut()).is_some() {
                found += 1;
            }
        }
        assert_eq!(found, secrets, "the pool did not empty exactly once");
        assert_eq!(
            sim.world().resource::<Learned>().since(),
            0,
            "the counter kept climbing after the pool emptied",
        );
    }

    #[test]
    fn the_interval_averages_near_six() {
        // The number the curve was chosen for. A player should feel a discovery
        // is *due* rather than owed, which is what a mean of six inside a bound
        // of twenty-five buys.
        let mut total = 0u32;
        let mut runs = 0u32;
        for seed in 0..40u64 {
            let mut sim = Sim::new(seed);
            let mut solves = 0;
            while discover(sim.world_mut()).is_none() {
                solves += 1;
            }
            total += solves + 1;
            runs += 1;
        }
        let mean = f64::from(total) / f64::from(runs);
        assert!(
            (3.0..9.0).contains(&mean),
            "the discovery interval averages {mean:.1} solves",
        );
    }
}
