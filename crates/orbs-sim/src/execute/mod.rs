//! What a command actually does.
//!
//! The seam the domains plug into. §10.1's brewing loop is live; the remaining
//! dark verbs acknowledge and do nothing, which is honest rather than lazy — a
//! verb whose domain does not exist has nothing to do, and §6 already guarantees
//! the player was told what the orb understood.
//!
//! Split by concern rather than kept whole. `execute.rs` reached 987 lines
//! holding three unrelated things — the pipeline, navigation, and the log — with
//! only the sixteen-arm `match` touching all of them, so every phase's work
//! landed in the same file and a brewing change conflicted with a log change for
//! no semantic reason.
//!
//! | Module | Verbs |
//! |---|---|
//! | `dispatch` | the `match`, `meditate`, `status`, and the two records every verb can need |
//! | `pipeline` | `move`, `wield`, `stop`, `siphon`, `purge`, `divine` — §10.1's loop |
//! | `grimoire` | `grimoire` — §6.1's manual, read *before* the loop |
//! | `navigate` | `attend`, `survey` — §7's places |
//! | `files` | `peruse`, `sift`, `verify` — §3's log |
//!
//! **No prose in any of them.** Rule 6 and §12 put authored text in content
//! files; these emit facts and let a later layer wrap sentences around them.

mod dispatch;
mod files;
mod grimoire;
mod navigate;
mod pipeline;

#[cfg(test)]
mod tests;

pub use dispatch::{LOG, MAX_MEDITATE, is_live, run_pending};

use dispatch::{acknowledge, missing};
