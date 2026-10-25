//! What the work is worth, and what it buys (DESIGN.md §11.5, §19).
//!
//! It only ever rises, so an upgrade opens when the total passes its number and
//! stays open, and nothing is spent and regretted. What it is worth and what it
//! buys are authored ([`Progression`]); this knows when to add and how to say
//! so. Concentration is derived from the total, never stored, so a save cannot
//! disagree with itself.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::{Progression, Prose};
use crate::session::Scrollback;

/// Everything the player has earned by working. Never decreases.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Experience(u64);

impl Experience {
    /// Put a total back, for a save — not `credit`, or a load would congratulate
    /// you on yesterday's work.
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

/// How deep a siege's quintessence pool is, before the tower's repair scales it
/// — [`concentration`]'s twin, so which table answers stays this module's.
#[must_use]
pub fn quintessence_steps(world: &World) -> usize {
    let earned = world.resource::<Experience>().get();
    world.resource::<Progression>().quintessence(earned)
}

/// What one completed run at `named` is worth.
///
/// Read before the run reports itself and credited after (see [`credit`]), so
/// the work is said before the reward.
#[must_use]
pub fn worth(world: &World, named: &str) -> u64 {
    world.resource::<Progression>().earns(named)
}

/// What a solve at `named` pays, given how many tries it took against `par`.
///
/// Full at or under par, three quarters beyond it, never nothing: a blind
/// ladder already pays in ticks, so a steeper yield penalty would double-count.
/// One rule for the lens and the menagerie.
#[must_use]
pub fn worth_within_par(world: &World, named: &str, spent: u32, par: u32) -> u64 {
    let full = worth(world, named);
    if spent <= par {
        full
    } else {
        full.saturating_mul(3) / 4
    }
}

/// Add what a completed run earned, and say so if it bought something.
///
/// Where a run succeeded, never where one ended: a scour and an unmatched run
/// also finish. See `work::produce::transmute`.
pub fn credit(world: &mut World, earned: u64) {
    if earned == 0 {
        return;
    }

    let was = world.resource::<Experience>().get();
    let before = concentration(world);
    world.resource_mut::<Experience>().0 += earned;
    let now = world.resource::<Experience>().get();
    let after = concentration(world);

    // Said on the edge, or a derived level would be true silently for ever.
    if after > before {
        say_gained(world, after);
    }

    // A room opened is not derived, so crossing a step writes it
    // (`ley::cross`), after the level.
    super::ley::cross(world, was, now);
}

/// What a run at `named` earned, credited — for the tests; every real site
/// says what it made between [`worth`] and [`credit`].
#[cfg(test)]
fn earn(world: &mut World, named: &str) -> u64 {
    let earned = worth(world, named);
    credit(world, earned);
    earned
}

/// The orb can hold one more than it could — §11.5's turn. It names the verb,
/// because a capability nobody can find is one nobody has.
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

        // Forty alembic runs cross three levels now that the line runs to the
        // soft ending; each is said exactly once.
        let crossed = Progression::builtin().concentration(sim.experience());
        assert!(crossed > 1, "the sweep never crossed a second level");
        assert_eq!(
            said, crossed,
            "{crossed} levels were crossed and {said} announced"
        );
    }
}
