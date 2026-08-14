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
//! | `recall` | `recall` — §6.1's manual, read *before* the loop |
//! | `navigate` | `attend`, `survey` — §7's places |
//! | `files` | `peruse`, `sift`, `verify` — §3's log |
//!
//! **No prose in any of them.** Rule 6 and §12 put authored text in content
//! files; these emit facts and let a later layer wrap sentences around them.

#[cfg(debug_assertions)]
mod debug;
mod dispatch;
mod files;
mod navigate;
mod pipeline;
mod recall;
mod research;
mod scribe;
mod unfurl;
mod wander;
mod weave;

#[cfg(test)]
mod tests;

#[cfg(debug_assertions)]
pub use debug::{Order as SpawnOrder, SPAWN, order as spawn_order};
pub use dispatch::{LOG, MAX_MEDITATE, execute_one, is_gated, is_live, offered, run_pending};
// Crate-internal: `Sim::labyrinth` needs it and `divine` is a private module, so
// the re-export is what makes it nameable rather than what makes it public.
pub use navigate::find_domain;
pub(crate) use research::{lectern, tread};
pub(crate) use scribe::Reloaded;
pub use scribe::{Opening, Request, write};
pub use unfurl::Unfurling;
pub use wander::Wandering;
pub use weave::Weaving;

use dispatch::{acknowledge, missing};
