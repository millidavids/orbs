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
//! The chance climbs with every fruitless solve and resets when one lands:
//! flat would leave a player forty solves in with nothing, certainty would make
//! it a queue. It retires once every shipped secret is known.

use std::collections::BTreeSet;

use bevy_ecs::prelude::*;
use rand::RngExt as _;

use crate::content::Recipes;
use crate::rng::{RngStream, Rngs};

/// What the player has learned to make, beyond what they started knowing.
///
/// Player state, the mutable half beside the immutable `Recipes`, still
/// reproducible from `(seed, submissions)`.
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
/// `4 + 4 × 24` is 100, so the 25th solve after a discovery cannot fail; the
/// mean is about six. Public, because a bound on bad luck is a promise.
pub const CERTAIN: u32 = 24;

impl Learned {
    /// Whether the player can make this.
    ///
    /// A name no recipe marks secret is always known: the set holds only what
    /// has been *found*, so the save carries three strings rather than forty.
    #[must_use]
    pub fn knows(&self, recipes: &Recipes, name: &str) -> bool {
        !recipes.is_secret(name) || self.known.contains(name)
    }

    /// Every secret recipe found so far, by name, in alphabetical order.
    ///
    /// For the save, whose bytes must not depend on insertion order; anything a
    /// player reads wants [`found`](Self::found).
    pub fn known(&self) -> impl Iterator<Item = &str> {
        self.known.iter().map(String::as_str)
    }

    /// Put a found set back, for a save.
    ///
    /// Not `discover`, which rolls: a restore reads what was found rather than
    /// finding it again, so `RngStream::Lens` does not move.
    pub(crate) fn restore(&mut self, known: impl IntoIterator<Item = String>, since: u32) {
        self.known = known.into_iter().collect();
        self.since = since;
    }

    /// Solves since the last discovery.
    #[must_use]
    pub const fn since(&self) -> u32 {
        self.since
    }

    /// Everything found so far, in `recipes.toml`'s own order.
    ///
    /// The file's order is load-bearing — `discover` and `learn` take the first
    /// unfound — so alphabetical would be a trap.
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
/// Called once per broken seal, and the only thing that advances the counter,
/// so a player who never scrys never gets closer.
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

    // Retired rather than rolled against nothing. The counter stops too, so a
    // save made after the last discovery carries no meaningless number.
    if pool.is_empty() {
        return None;
    }

    let chance = BASE.saturating_add(STEP.saturating_mul(world.resource::<Learned>().since));
    let roll = world
        .resource_mut::<Rngs>()
        .stream(RngStream::Lens)
        .random_range(0..100);

    let found = {
        let mut learned = world.resource_mut::<Learned>();
        if roll >= chance {
            learned.since = learned.since.saturating_add(1);
            return None;
        }

        // The file's order, not a uniform draw: the reveal sequence is
        // authored (§19), and deterministic, which a replay needs.
        let found = pool.into_iter().next()?;
        learned.known.insert(found.clone());
        learned.since = 0;
        found
    };
    // A secret found is what the lens's mastery line counts.
    super::note(world, super::SECRET);
    Some(found)
}

/// Learn a secret outright, for a tester's `debug_learn`.
///
/// The same set and counter as a discovery, skipping only the roll. `None` is
/// the next in the file's order, the lens's own.
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
            // A name that is not a secret, or one already known, is refused:
            // both are states the game cannot reach.
            Some(wanted) => unfound.find(|name| *name == wanted).map(str::to_owned),
            None => unfound.next().map(str::to_owned),
        }
    };

    let found = found?;
    {
        let mut learned = world.resource_mut::<Learned>();
        learned.known.insert(found.clone());
        learned.since = 0;
    }
    // Counted as a find, so a tower reached this way and one reached by probing
    // are the same tower.
    super::note(world, super::SECRET);
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
        // `discover` and `learn` both take the first unfound secret, so an
        // accessor answering alphabetically would put `dreaming` before
        // `mending` and contradict the sequence a player met.
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
        // The number the curve was chosen for: a discovery should feel *due*
        // rather than owed.
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
