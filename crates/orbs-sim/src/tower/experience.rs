//! What the work is worth, and what it buys (DESIGN.md §11.5, §19).
//!
//! # It only ever rises
//!
//! Experience accumulates and is never spent: an upgrade opens when the total
//! passes its number and stays open. That is what makes it a `u64` and a save a
//! single value — there is no balance to keep, and nothing a player can spend
//! and then wish they had not.
//!
//! What it is *worth* and what it *buys* are both authored
//! ([`Progression`](crate::content::Progression)); this module knows only when
//! to add and how to say so.
//!
//! # Under `tower/`, because it is world state that ticks
//!
//! The same test that put `spell/` here rather than in `execute/`: `execute/`
//! holds no per-tick state at all. This is written by a run *completing*, which
//! happens in a system, on a tick boundary, whether or not anybody typed
//! anything.
//!
//! # Concentration is derived from it, never stored
//!
//! [`concentration`] is a function of the total against the authored table, so
//! there is no second number to fall out of step with the first — the same shape
//! as a spell's `Program` being derived from its text rather than kept beside
//! it. A save that carried both could disagree with itself; this one cannot.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::{Progression, Prose};
use crate::session::Scrollback;

/// Everything the player has earned by working.
///
/// **Never decreases.** Nothing in the game removes experience, and no code path
/// here can: the only mutator adds.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Experience(u64);

impl Experience {
    /// Put a total back, for a save.
    ///
    /// **Not `credit`**, which emits records and can announce an unlock. A
    /// restore is not the player earning anything; it is the world being what it
    /// already was, and a load that congratulated you on work you did yesterday
    /// would be reporting a lie in voice.
    pub(crate) const fn restore(&mut self, total: u64) {
        self.0 = total;
    }

    /// The total earned.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// How many spells the orb can hold, at the current total.
#[must_use]
pub fn concentration(world: &World) -> usize {
    let earned = world.resource::<Experience>().get();
    world.resource::<Progression>().concentration(earned)
}

/// What one completed run at `named` is worth.
///
/// **Read before the run reports itself, credited after** — see
/// [`credit`]. Splitting the two is what puts the sentences in the order they
/// happened: the alembic yields a clarity, *and then* the orb can hold a spell.
/// Done in one call it announced the reward before the work.
#[must_use]
pub fn worth(world: &World, named: &str) -> u64 {
    world.resource::<Progression>().earns(named)
}

/// Add what a completed run earned, and say so if it bought something.
///
/// **Called where a run *succeeded*, never where one ended.** `finish` also runs
/// for a scour and for a run that matched no recipe, and neither is work the
/// tower has anything to show for — see `work::produce::transmute`, which is
/// where the successful branch is.
pub fn credit(world: &mut World, earned: u64) {
    if earned == 0 {
        return;
    }

    let before = concentration(world);
    world.resource_mut::<Experience>().0 += earned;
    let after = concentration(world);

    // **Said once, at the tick that bought it.** A level is derived, so it would
    // otherwise be true silently and for ever — the player would find out by
    // trying `bind` and being refused, or not refused, with nothing having
    // announced the difference. `Reloaded` in `scribe` is the same shape: the
    // fact is continuous, the sentence is an edge.
    if after > before {
        say_gained(world, after);
    }
}

/// What a run at `named` earned, credited — for callers with nothing to say.
///
/// The tests below, and nothing in the game: every real site reports what it
/// made, so it wants [`worth`] and [`credit`] either side of that sentence.
#[cfg(test)]
fn earn(world: &mut World, named: &str) -> u64 {
    let earned = worth(world, named);
    credit(world, earned);
    earned
}

/// The orb can hold one more than it could.
///
/// §6 forbids a bare fact where a sentence would teach, and this is the first
/// thing in the game that *opens* rather than refuses — the moment §11.5 calls
/// the game's turn. It names the verb, because a capability nobody can find is a
/// capability nobody has.
fn say_gained(world: &mut World, level: usize) {
    let message = world
        .resource::<Prose>()
        .line("concentration_gained", &[("count", &level.to_string())]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, "concentration")
        .count(FieldName::Quantity, level as u64)
        .text(FieldName::Message, &message)
        .role(Role::Success)
        .finish();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Sim;

    #[test]
    fn a_completed_run_earns_what_the_curve_says() {
        let mut sim = Sim::new(1);
        assert_eq!(sim.experience(), 0, "the tower started with something");

        assert_eq!(earn(sim.world_mut(), "mortar_and_pestle"), 1);
        assert_eq!(earn(sim.world_mut(), "alembic"), 8);
        assert_eq!(sim.experience(), 9);
    }

    #[test]
    fn a_name_the_curve_does_not_price_earns_nothing() {
        // Not an error — `check` has already refused any *instrument* missing
        // from the table at load. This is everything else that might ask.
        let mut sim = Sim::new(1);
        assert_eq!(earn(sim.world_mut(), "dispensary"), 0);
        assert_eq!(sim.experience(), 0);
    }

    #[test]
    fn crossing_a_threshold_is_said_once() {
        let mut sim = Sim::new(1);
        for _ in 0..40 {
            earn(sim.world_mut(), "alembic");
        }
        let said = sim
            .scrollback()
            .records()
            .iter()
            .filter_map(|record| record.field(FieldName::Message))
            .filter(
                |value| matches!(value, orbs_render::Value::Text(text) if text.contains("hold")),
            )
            .count();

        assert_eq!(said, 1, "the level was announced {said} times");
        assert!(sim.experience() > 16, "the sweep never crossed it");
    }
}
