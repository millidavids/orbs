//! The simulation schedule.
//!
//! Single-threaded, deliberately. `bevy_ecs`'s multi-threaded executor gives no
//! ordering guarantee between systems that lack explicit constraints, and this
//! sim must replay identically and match offline to online. At 1 Hz with this
//! entity count, parallelism buys nothing and would cost the one property
//! everything else rests on. See CLAUDE.md, architectural rule 3.

use bevy_ecs::prelude::*;
use bevy_ecs::schedule::{ScheduleLabel, SingleThreadedExecutor};

/// Label for the one schedule the simulation runs each tick.
#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SimSchedule;

/// Build the sim schedule with the deterministic executor installed.
///
/// Always construct schedules through this — a schedule built directly with
/// `Schedule::default()` gets the multi-threaded executor and silently
/// reintroduces non-determinism.
#[must_use]
pub fn new_sim_schedule() -> Schedule {
    let mut schedule = Schedule::new(SimSchedule);
    schedule.set_executor(SingleThreadedExecutor::new());
    schedule
}
