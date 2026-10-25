//! How big a siege arrives, and what buying it down costs (§11.5, §19).
//!
//! Renown decides the ceiling and `petition` lowers it — one mechanic, kept in
//! one file. Nothing here draws, so the ceiling can be asked before a siege
//! exists, which lets `petition` quote a price and refuse without spending.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use super::{BASE_MOST, FEWEST, MOST, RANKS_PER_FOE};

/// What one foe fewer costs in standing: exactly what that foe would have paid.
///
/// Self-limiting: spending drops your rank, which lowers the ceiling anyway.
pub const PETITION_PER_FOE: u64 = super::RENOWN_PER_FOE;

/// How many foes have been bought off the next siege to arrive.
///
/// A resource, because it outlives every siege. Absent in a save reads as
/// nought, the honest reading of a document from before petitioning.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Petitioned(u32);

impl Petitioned {
    /// How many foes are currently bought off.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }

    /// Put a count back, for a save.
    pub const fn restore(&mut self, count: u32) {
        self.0 = count;
    }

    /// Buy one more foe off the next siege.
    pub const fn add(&mut self) {
        self.0 = self.0.saturating_add(1);
    }

    /// Spend the whole allowance, which is what opening a siege does.
    pub const fn take(&mut self) -> u32 {
        let held = self.0;
        self.0 = 0;
        held
    }
}

/// The largest enemy this tower would meet, at `ranks` standing, having
/// petitioned `bought` foes away.
///
/// The floor never moves, so fame lengthens the tail: a famous tower can still
/// draw a quiet night, and an unknown one never meets the worst.
#[must_use]
pub fn most_at(ranks: usize, bought: u32) -> u32 {
    let earned = u32::try_from(ranks / RANKS_PER_FOE).unwrap_or(u32::MAX);
    BASE_MOST
        .saturating_add(earned)
        .min(MOST)
        .saturating_sub(bought)
        .max(FEWEST)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tail_lengthens_with_standing_and_the_floor_never_moves() {
        // Ten ranks buys three foes of tail, and the floor moves with none of
        // them — which is what keeps a bad night survivable.
        assert_eq!(most_at(0, 0), BASE_MOST);
        assert_eq!(most_at(2, 0), BASE_MOST);
        assert_eq!(most_at(3, 0), BASE_MOST + 1);
        assert_eq!(most_at(6, 0), BASE_MOST + 2);
        assert_eq!(most_at(9, 0), MOST);
    }

    #[test]
    fn the_written_ceiling_holds_however_famous_the_tower() {
        // The garrison is six and `outnumbered` is `enemy >= garrison * 2`, so
        // twelve is where that reading turns over — past it, the rung three
        // shipped solvers branch on is true in every fight.
        for ranks in 0..100 {
            assert!(
                most_at(ranks, 0) <= MOST,
                "{ranks} ranks drew past the written ceiling",
            );
        }
    }

    #[test]
    fn petitioning_buys_the_tail_down_and_stops_at_the_floor() {
        assert_eq!(most_at(9, 1), MOST - 1);
        assert_eq!(most_at(9, 3), BASE_MOST);
        // ...and no further, however much is paid.
        assert_eq!(most_at(9, 99), FEWEST);
        assert_eq!(most_at(0, 99), FEWEST);
    }

    #[test]
    fn the_allowance_is_spent_whole_by_the_siege_that_opens() {
        let mut bought = Petitioned::default();
        bought.add();
        bought.add();
        assert_eq!(bought.get(), 2);
        assert_eq!(bought.take(), 2);
        assert_eq!(bought.get(), 0, "the allowance outlived its siege");
    }
}
