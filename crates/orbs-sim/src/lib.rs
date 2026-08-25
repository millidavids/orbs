//! O.R.B.S. simulation core.
//!
//! Headless, deterministic, and free of any rendering dependency. Frontends are
//! *callers*: they construct a [`Sim`] and drive it with [`Sim::step`]. They never
//! host the schedule.
//!
//! See `CLAUDE.md` for the architectural rules this crate exists to uphold.

pub mod content;
pub mod parser;

pub mod execute;
pub mod session;
pub mod tower;

mod rng;
pub mod save;
mod schedule;
mod sim;
mod tick;

pub use content::{ContentError, Prose};
pub use execute::{LOG, MAX_MEDITATE, Request, spell_expect, spell_vocabulary};
pub use rng::{RngStream, Rngs};
pub use save::{Save, SaveError};
pub use schedule::{SimSchedule, new_sim_schedule};
pub use session::{
    Choices, DEFAULT_WIZARD, Pending, Queued, Scrollback, Skip, Submission, Submissions, Wizard,
};
pub use sim::Sim;
pub use tick::Tick;
pub use tower::spell::{Fault, Reading};
pub use tower::{Cwd, Name, Node, NodeId, Standing, children_of};
