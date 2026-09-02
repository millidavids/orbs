//! Quintessence: the tower's one spendable resource.
//!
//! §11.5's **mana**, and its table has always read *"Produced by: passive
//! regeneration, Ley Line steps"*. What shipped at `0.8.15` was the siege's
//! narrowing of that — a fixed pool granted on `defend`, spent on dice, gone
//! when the fight ended. Enchanting spends the same resource, so the pool comes
//! up to the tower where §11.5 always had it, and the siege becomes one of two
//! rooms that draw on it rather than the only one.
//!
//! **This is a return, not a reversal**, and §19 records it as such.
//!
//! # What the ceiling means
//!
//! Integrity and the ley line used to decide *what a siege granted*. They now
//! decide **how much the tower can hold**, so repairing the barrier is what buys
//! enchanting capacity — and a worn tower is smaller rather than slower. The
//! curve is unchanged, [`FLOOR_PCT`] and all; only the question it answers is.
//!
//! # Regeneration, and why the siege's is a lump
//!
//! In the calm it trickles: [`REGEN_TICKS`] apart, one at a time, capped.
//!
//! **Under siege it does not trickle at all** — a round resolved by `hold`
//! grants [`REGEN_PER_ROUND`] and waiting inside a turn earns nothing. That is
//! §14 satisfied by construction rather than by exception: the screen-reader
//! accommodation advances siege ticks *on player input*, so anything measured in
//! ticks would mean **more typing produces more resource** for the players that
//! mode exists to serve. A lump granted per round reads no clock, so a patient
//! chant and a played one grant identically.
//!
//! It also gets the shape the design wants for free: dawdling in a siege earns
//! nothing, so the only way to more quintessence is to advance the fight and
//! take what the enemy does to you.

use bevy_ecs::prelude::*;

use super::erosion::STANDING;

/// The ceiling a tower at full [`STANDING`] holds, before the ley line.
///
/// **A first-pass number.** Against the dice costs in `siege.toml` (`d6` 1, `d8`
/// 2, `d20` 5) a full allocation is eight a round, so twenty-four is three
/// unrestrained rounds of a siege that runs six to thirteen — enough to matter
/// every round, never enough to stop choosing. It now also has to buy charms,
/// which is exactly the tension the shared pool exists to create, and
/// `orbs-balance` is what settles whether it is the right number.
pub const QUINTESSENCE_BASE: u32 = 24;

/// What one ley-line step granting quintessence adds to the ceiling.
pub const PER_LEY_STEP: u32 = 6;

/// The share of the ceiling a tower worn to nothing still holds, as a percentage.
///
/// **The anti-spiral.** A lost siege takes `DEFEAT_WEAR` off the barrier, so an
/// unfloored ceiling would make each defeat cheapen the next fight until it
/// could not be won — punishment compounding into a dead end, which §11.5
/// forbids: *"never ruinous, only slower"*. At a half, neglect is expensive and
/// never fatal.
pub const FLOOR_PCT: u32 = 50;

/// How often a point comes back in the calm, in ticks.
///
/// **Thirty seconds**, at §5.0's one tick a second — so a full ceiling from
/// empty is about twelve minutes of an unattended tower. That is deliberately
/// slower than erosion's thirty-tick wear: a tower left alone should lose
/// standing faster than it recovers the means to mend it, or walking away would
/// be a strategy. A placeholder like every duration here, and `orbs-balance` is
/// what sweeps it.
pub const REGEN_TICKS: u64 = 30;

/// What resolving one siege round grants.
///
/// **Two, and it was four — measured, not argued.** A full allocation of the
/// three shipped dice is eight a round, so four made a fight pay for half of
/// itself and the pool last twice as long. That is not a tuning nicety: the
/// reachability sweep in `tests/scripting_the_siege.rs` came back saying
/// **`outnumbered` was no longer published on any seed**, because a garrison
/// that can afford its dice every round is never overtaken — a whole reading,
/// and the solver rungs that ask for it, quietly dead.
///
/// At two the pool drains six a round against a base of 24, which is four
/// unrestrained rounds where the fixed pool bought three. Advancing the siege
/// still pays, dawdling still earns nothing, and the enemy can still get ahead.
pub const REGEN_PER_ROUND: u32 = 2;

/// What the tower holds, and what it may hold.
///
/// A `Resource` rather than a component: it belongs to the tower rather than to
/// any room, and both the forge and the bailey spend from it.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct Quintessence {
    held: u32,
}

impl Quintessence {
    /// A tower holding `held`.
    #[must_use]
    pub const fn new(held: u32) -> Self {
        Self { held }
    }

    /// What is in hand.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.held
    }

    /// Spend `cost`, or say there is not enough. **Never partial.**
    pub const fn spend(&mut self, cost: u32) -> bool {
        if cost > self.held {
            return false;
        }
        self.held -= cost;
        true
    }

    /// Put `amount` back, never raising the pool *past* `ceiling`.
    ///
    /// **A pool already above the ceiling keeps what it has.** The obvious
    /// spelling — add, then clamp — confiscates the surplus, and that
    /// contradicts the invariant the load path states in as many words: *"the
    /// ceiling caps regeneration and nothing else: erosion lowers it without
    /// confiscating what the tower already holds, so a pool above the ceiling is
    /// a legal state a live world reaches by wearing down while full."*
    ///
    /// So the two disagreed, silently and in the player's favour on load and
    /// against them on the very next tick: a full tower whose barrier then wore
    /// down lost the difference at the next multiple of [`REGEN_TICKS`], with no
    /// message and nothing to notice.
    pub const fn restore(&mut self, amount: u32, ceiling: u32) {
        if self.held >= ceiling {
            return;
        }
        self.held = self.held.saturating_add(amount);
        if self.held > ceiling {
            self.held = ceiling;
        }
    }
}

/// What the tower may hold, given the barrier and the ley line.
///
/// Integrity scales the whole ceiling between [`FLOOR_PCT`] and full; each ley
/// step raises what is being scaled. **Integer throughout** — a percentage of a
/// small number is exact here and a float would be one more thing replay has to
/// trust.
#[must_use]
pub fn ceiling_for(integrity: u32, steps: usize) -> u32 {
    let steps = u32::try_from(steps).unwrap_or(u32::MAX);
    let base = QUINTESSENCE_BASE.saturating_add(steps.saturating_mul(PER_LEY_STEP));
    // Clamped, because a value past `STANDING` would scale the ceiling *above*
    // the base rather than up to it.
    let standing = integrity.min(STANDING);
    let scale = FLOOR_PCT + (100 - FLOOR_PCT) * standing / STANDING;
    base.saturating_mul(scale) / 100
}

/// This tower's ceiling right now.
#[must_use]
pub fn ceiling(world: &World) -> u32 {
    ceiling_for(
        world.resource::<super::Integrity>().get(),
        super::quintessence_steps(world),
    )
}

/// Whether a siege is being fought anywhere in the tower.
///
/// What suspends the calm trickle. Asked of the world rather than of `Cwd`,
/// because a siege runs whether or not the player is standing in the bailey.
#[must_use]
fn besieged(world: &mut World) -> bool {
    world
        .query::<&super::Siege>()
        .iter(world)
        .any(super::Siege::running)
}

/// The calm trickle.
///
/// **Appended to the schedule and drawing nothing**, which is the licence
/// `settling`, `erode` and `lapse_chant` already hold: a system that only reads
/// the clock cannot perturb any `RngStream`, so it can go on the end without
/// touching a single existing replay.
///
/// A modulo on the tick rather than a countdown, which is `erode`'s shape and
/// the same reason — `meditate` collapses hundreds of ticks inside one `step`,
/// and a counter would advance once where a clock reading advances properly.
pub fn regenerate(world: &mut World) {
    if !world
        .resource::<crate::tick::Tick>()
        .get()
        .is_multiple_of(REGEN_TICKS)
    {
        return;
    }
    if besieged(world) {
        return;
    }
    let ceiling = ceiling(world);
    world.resource_mut::<Quintessence>().restore(1, ceiling);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ceiling_runs_from_half_to_whole() {
        assert_eq!(ceiling_for(STANDING, 0), QUINTESSENCE_BASE);
        assert_eq!(
            ceiling_for(0, 0),
            QUINTESSENCE_BASE * FLOOR_PCT / 100,
            "a ruined tower is not on the floor",
        );
        assert_eq!(
            ceiling_for(STANDING / 2, 0),
            QUINTESSENCE_BASE * 75 / 100,
            "the curve is not linear between the two ends",
        );
    }

    #[test]
    fn a_ley_step_raises_what_integrity_scales() {
        assert_eq!(
            ceiling_for(STANDING, 1),
            QUINTESSENCE_BASE + PER_LEY_STEP,
            "a ley step did not reach a whole tower",
        );
        assert_eq!(
            ceiling_for(0, 1),
            (QUINTESSENCE_BASE + PER_LEY_STEP) * FLOOR_PCT / 100,
            "a ley step is not scaled by a worn barrier",
        );
    }

    /// Integrity past `STANDING` must not scale the ceiling above the base.
    #[test]
    fn a_barrier_beyond_whole_grants_no_more() {
        assert_eq!(ceiling_for(STANDING * 4, 0), QUINTESSENCE_BASE);
    }

    #[test]
    fn spending_is_never_partial() {
        let mut held = Quintessence::new(3);
        assert!(!held.spend(4), "an unaffordable spend went through");
        assert_eq!(held.get(), 3, "a refused spend took something anyway");
        assert!(held.spend(3));
        assert_eq!(held.get(), 0);
    }

    #[test]
    fn restoring_never_passes_the_ceiling() {
        let mut held = Quintessence::new(20);
        held.restore(10, 24);
        assert_eq!(held.get(), 24);
    }
}
