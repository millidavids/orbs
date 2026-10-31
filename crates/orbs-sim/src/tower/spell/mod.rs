//! Spells the orb is working through (DESIGN.md §8).
//!
//! Under `tower/`, not `execute/`, because a running spell is world state that
//! ticks, like `work/`; `execute/` holds no per-tick systems.

mod bind;
mod block;
mod compile;
mod guard;
mod invoke;
mod line;
mod names;
mod parts;
mod program;
mod reading;
mod run;
mod saying;
mod speech;
mod strand;
mod subject;
mod watch;
mod whole;

#[cfg(test)]
mod tests;

pub use bind::{Bound, bind, held, stand};
pub use block::{Blocked, Hold, would_block};
pub use compile::{compile, read, reads_cleanly, source};
pub use invoke::{invoke, stop_spell};
pub use names::{SPELL_MARGIN, SPELL_SIMILARITY, candidates};
pub use program::{Block, Complaint, Draft, Kind, Loop, Program, Step};
pub use reading::{Fault, Reading, interpret};
#[cfg(test)]
pub(crate) use run::dragged_for_test;
pub use run::{
    Caller, Casting, Descent, LONGEST_BIDE, MAX_DEPTH, MAX_PARTS, MAX_STRANDS, PATIENCE, Running,
    SCRIPT_BUDGET, Strand, advance, budget, line_of,
};
pub use speech::{may_issue, may_say};
pub use watch::{Event, holds, watch};
pub use whole::{TOWER_CONCENTRATION, brief as tower_brief};
pub(crate) use whole::{may_write as may_write_for_the_tower, names_tower};
