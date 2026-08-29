//! Spells the orb is working through (DESIGN.md §8).
//!
//! Under `tower/` rather than `execute/` because a running spell is **world
//! state that ticks**, like `work/` and `heat.rs` beside it. `execute/` holds no
//! per-tick systems at all — `run_pending` is exclusive and stateless — and
//! `execute::dispatch`'s own docs call it *"the one file every phase must
//! edit"*, which is not a place to put a second responsibility.

mod bind;
mod block;
mod compile;
mod invoke;
mod program;
mod run;
mod watch;

#[cfg(test)]
mod tests;

pub use bind::{Bound, bind, held, stand};
pub use block::{Blocked, would_block};
pub use compile::{
    Fault, Reading, SPELL_MARGIN, SPELL_SIMILARITY, candidates, compile, interpret, read,
};
pub use invoke::{invoke, stop_spell};
pub use program::{Block, Complaint, Draft, Kind, Loop, Program, Step};
pub use run::{
    Caller, Casting, Descent, MAX_DEPTH, MAX_PARTS, MAX_STRANDS, PATIENCE, Running, SCRIPT_BUDGET,
    Strand, advance, budget, line_of, may_issue,
};
pub use watch::{Event, holds, watch};
