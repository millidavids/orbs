//! Turning a world into a file and back.
//!
//! # The rule this module lives under
//!
//! **It touches no filesystem.** `orbs-sim` builds and consumes a *document*;
//! reading and writing the file is a frontend's, through `orbs-shell`. That is
//! the same split content already uses — `crates/orbs-sim/Cargo.toml` says
//! *"this crate never watches a file — a frontend owns the watcher and hands new
//! content in at a tick boundary"* — and it is what keeps rule 1's
//! headless-in-milliseconds promise true of the test suite.
//! `tests/boundaries.rs` holds the line now rather than habit doing it.
//!
//! # What a save is, in one paragraph
//!
//! The tree by path, every component on it, the resources that are player state,
//! the clock, the eight RNG stream positions, and a bounded tail of the record
//! stream. Not the compiled programs, not the scene, not the parse log — each of
//! those is derived, and [`Save`] says so field by field.
//!
//! **`Submissions` does not travel either**, and that is a decision rather than
//! an omission: a journal describes a session, and a resumed session is a new
//! one. So a loaded tower cannot be replayed from its own log or bug-reported by
//! journal — the snapshot is what a report carries instead, which is the readable
//! file §15 asked for. `tests/persistence.rs` holds the journal itself.
//!
//! # Two decisions worth reading before changing anything here
//!
//! **A snapshot is the save, and a journal is not.** `Submissions` and
//! `Sim::replay` could have been the format — *"a replay needs exactly `(seed,
//! submissions)` and nothing else"* — and §13 and §15 chose otherwise: readable
//! TOML, editable by hand, opened in constant time. A journal is invalidated by
//! any content patch, and "editable" would have meant editing a command log.
//!
//! **What catches a component nobody carried is the lockstep test, not the
//! journal one.** That distinction was got wrong first time and is worth stating
//! plainly: `two_routes_to_one_world_write_the_same_save` runs `capture` on two
//! worlds *neither of which was restored*, so a field `capture` omits is absent
//! from both documents and it passes. It is a determinism check, and a good one.
//! The instrument is `a_loaded_tower_keeps_running_the_same_world`, which does
//! restore — a dropped `Burning` makes the loaded athanor cold, and the fire
//! going out shows up within a few hundred ticks in fields the document *does*
//! carry. The two lints beside it are what narrow the rest.
//!
//! **A restore raises the tower first and then adopts it.** Five more domains
//! arrive between here and Phase 9a; a save that owned the whole tree would load
//! into a tower permanently lacking them, and refusing the old save is not an
//! answer because it deletes it. The restore pass matches by path, applies what the
//! save knows, and leaves what it has never heard of alone.

mod adopt;
mod capture;
mod document;
mod error;
mod naming;
mod node;
mod restore;

pub use document::{Away, FORMAT, MarkSave, ProgressSave, RecordSave, RngSave, Save, WorldSave};
pub use error::SaveError;
pub use node::{
    DescentSave, HistorySave, MazeSave, NodeSave, RunningSave, SpanSave, SubstitutedSave, WardSave,
    WorkingSave,
};

pub(crate) use capture::capture;
// The word tables, for the two types whose save conversion lives beside their
// own private fields — `tower::Ward` and `tower::Maze`. One spelling of each
// word, in one place, is the whole point of `naming`.
pub(crate) use naming::{errand_from, errand_word, way_from, way_word};
pub(crate) use restore::restore;
