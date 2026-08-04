//! O.R.B.S. simulation core.
//!
//! Headless, deterministic, and free of any rendering dependency. Frontends are
//! *callers*: they construct a [`Sim`] and drive it with [`Sim::step`]. They never
//! host the schedule.
//!
//! See `CLAUDE.md` for the architectural rules this crate exists to uphold.

pub mod parser;

pub mod execute;
pub mod session;

mod rng;
mod schedule;
mod sim;
mod tick;

pub use execute::{LOG, MAX_MEDITATE};
pub use rng::{RngStream, Rngs};
pub use schedule::{SimSchedule, new_sim_schedule};
pub use session::{Pending, Scrollback, Skip, Submissions};
pub use sim::Sim;
pub use tick::Tick;
