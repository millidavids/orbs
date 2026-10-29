//! Spells the orb is working through (DESIGN.md §8).
//!
//! Under `tower/`, not `execute/`, because a running spell is world state that
//! ticks, like `work/`; `execute/` holds no per-tick systems.

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
pub use block::{Blocked, Hold, would_block};
pub use compile::{
    Fault, Reading, SPELL_MARGIN, SPELL_SIMILARITY, candidates, compile, interpret, read,
    reads_cleanly, source,
};
pub use invoke::{invoke, stop_spell};
pub use program::{Block, Complaint, Draft, Kind, Loop, Program, Step};
#[cfg(test)]
pub(crate) use run::dragged_for_test;
pub use run::{
    Caller, Casting, Descent, LONGEST_BIDE, MAX_DEPTH, MAX_PARTS, MAX_STRANDS, PATIENCE, Running,
    SCRIPT_BUDGET, Strand, advance, budget, line_of, may_issue,
};
pub use watch::{Event, holds, watch};
