//! Spells the orb is working through (DESIGN.md §8).
//!
//! Under `tower/` rather than `execute/` because a running spell is **world
//! state that ticks**, like `work/` and `heat.rs` beside it. `execute/` holds no
//! per-tick systems at all — `run_pending` is exclusive and stateless — and
//! `execute::dispatch`'s own docs call it *"the one file every phase must
//! edit"*, which is not a place to put a second responsibility.

mod block;
mod invoke;
mod program;
mod run;
mod watch;

#[cfg(test)]
mod tests;

pub use block::{Blocked, would_block};
pub use invoke::{invoke, stop_spell};
pub use program::{Block, Complaint, Kind, Loop, Program, Step, parse};
pub use run::{Depth, MAX_DEPTH, PATIENCE, Running, SCRIPT_BUDGET, advance, line_of};
pub use watch::{Event, holds, watch};
