//! Quintessence: the tower's one spendable resource.
//!
//! §11.5's mana, the tower's and not the siege's, since enchanting spends it
//! too (§19). Integrity and the ley line decide how much the tower holds, so
//! repairing the barrier buys capacity and a worn tower is smaller, not slower.
//!
//! In the calm it trickles, [`REGEN_TICKS`] apart. Under siege only a resolved
//! round grants it ([`REGEN_PER_ROUND`]): a tick-measured grant would pay more
//! typing under §14's screen-reader pacing, and the only way to more is to
//! advance the fight.

use bevy_ecs::prelude::*;

use super::erosion::STANDING;

/// The ceiling a tower at full [`STANDING`] holds, before the ley line.
///
/// Three unrestrained rounds of eight against a siege of six to thirteen —
/// enough to matter, never enough to stop choosing — and charms besides. A
/// first pass; `orbs-balance` settles it.
pub const QUINTESSENCE_BASE: u32 = 24;

/// What one ley-line step granting quintessence adds to the ceiling.
pub const PER_LEY_STEP: u32 = 6;

/// The share of the ceiling a tower worn to nothing still holds, as a percentage.
///
/// The anti-spiral: unfloored, each defeat would cheapen the next until none
/// could be won (§11.5).
pub const FLOOR_PCT: u32 = 50;

/// How often a point comes back in the calm, in ticks.
///
/// About twelve minutes to fill from empty — slower than erosion, or walking
/// away would be a strategy. `orbs-balance` sweeps it.
pub const REGEN_TICKS: u64 = 30;

/// What resolving one siege round grants.
///
/// Measured: at four, a garrison that can always afford its dice is never
/// overtaken, and `outnumbered` was published on no seed. At two the enemy can
/// still get ahead.
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

    /// Spend `cost`, or say there is not enough. Never partial.
    pub const fn spend(&mut self, cost: u32) -> bool {
        if cost > self.held {
            return false;
        }
        self.held -= cost;
        true
    }

    /// Put `amount` back, never raising the pool *past* `ceiling`.
    ///
    /// A pool above the ceiling keeps what it has: the ceiling caps regeneration
    /// only, and a full tower whose barrier wore down must not lose the surplus.
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
/// step raises what is being scaled. Integer throughout — a percentage of a
/// small number is exact, and a float is one more thing replay has to trust.
#[must_use]
pub fn ceiling_for(integrity: u32, steps: usize) -> u32 {
    ceiling_with(integrity, steps, FLOOR_PCT, 0)
}

/// [`ceiling_for`], with the Ley Line's forks applied.
///
/// `floor` is the floor in percent — [`FLOOR_PCT`] plus the `floor` grant's
/// tiers — and `bonus` is the `pool` grant's quintessence, added to the base
/// so integrity scales it like the rest.
#[must_use]
pub fn ceiling_with(integrity: u32, steps: usize, floor: u32, bonus: u32) -> u32 {
    let steps = u32::try_from(steps).unwrap_or(u32::MAX);
    let base = QUINTESSENCE_BASE
        .saturating_add(steps.saturating_mul(PER_LEY_STEP))
        .saturating_add(bonus);
    // Clamped, because a value past `STANDING` would scale the ceiling *above*
    // the base rather than up to it — and a floor past a hundred would too.
    let floor = floor.min(100);
    let standing = integrity.min(STANDING);
    let scale = floor + (100 - floor) * standing / STANDING;
    base.saturating_mul(scale) / 100
}

/// This tower's ceiling right now.
#[must_use]
pub fn ceiling(world: &World) -> u32 {
    ceiling_with(
        world.resource::<super::Integrity>().get(),
        super::quintessence_steps(world),
        FLOOR_PCT.saturating_add(super::grant::floor_bonus(world)),
        super::grant::pool_bonus(world),
    )
}

/// The calm trickle.
///
/// Draws nothing, so it could be appended to the schedule. A modulo on the
/// tick, not a countdown, for `erode`'s reason.
pub fn regenerate(world: &mut World) {
    if !world
        .resource::<crate::tick::Tick>()
        .get()
        .is_multiple_of(REGEN_TICKS)
    {
        return;
    }
    if super::besieged(world) {
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
