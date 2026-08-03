//! Driving the simulation from the frontend.
//!
//! Architectural rule 3: **the frontend is a caller, not a host.** Bevy's
//! scheduler never runs sim systems. It runs exactly one system, [`advance`],
//! which calls [`Sim::step`] once. Everything inside the sim runs on the sim's
//! own single-threaded schedule, in its own deterministic order.
//!
//! One tick is one real second (DESIGN.md §5.0), so the driver lives in
//! `FixedUpdate` at 1 Hz rather than in `Update`. Frame rate must never change
//! how fast the world moves.

use bevy::prelude::*;
use orbs_sim::{Sim, Tick};

/// The simulated tower, owned by the frontend.
#[derive(Resource)]
pub(crate) struct Tower(Sim);

impl Tower {
    /// A tower from a master seed.
    pub(crate) fn new(seed: u64) -> Self {
        Self(Sim::new(seed))
    }

    /// The current world time.
    pub(crate) fn tick(&self) -> Tick {
        self.0.tick()
    }

    /// The seed this world was built from.
    pub(crate) fn seed(&self) -> u64 {
        self.0.seed()
    }
}

/// Advance the world by exactly one tick.
///
/// `FixedUpdate` may run this several times in one frame after a stall, which is
/// correct: the world owes that time regardless of how long the GPU took.
pub(crate) fn advance(mut tower: ResMut<Tower>) {
    tower.0.step();
}
