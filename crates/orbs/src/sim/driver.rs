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
use orbs_sim::Sim;

/// The simulated tower, owned by the frontend.
#[derive(Resource)]
pub(crate) struct Tower(Sim);

impl Tower {
    /// A tower from a master seed.
    pub(crate) fn new(seed: u64) -> Self {
        Self(Sim::new(seed))
    }

    /// Hand a finished line to the sim.
    ///
    /// The **only** other way the frontend touches the world, and it does not
    /// advance it: `submit` echoes immediately and queues any resolved command
    /// for the next `step()`. See `orbs_sim::session`.
    pub(crate) fn submit(&mut self, line: &str) {
        self.0.submit(line);
    }

    /// The world, for painting.
    pub(crate) fn sim(&self) -> &Sim {
        &self.0
    }
}

/// Advance the world by exactly one tick.
///
/// `FixedUpdate` may run this several times in one frame after a stall, which is
/// correct: the world owes that time regardless of how long the GPU took.
pub(crate) fn advance(mut tower: ResMut<Tower>) {
    tower.0.step();
}
