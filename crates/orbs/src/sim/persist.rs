//! Keeping the tower, and putting it back.
//!
//! The frontend half of the save. `orbs-sim` turns a world into a document,
//! `orbs_shell::save` turns a document into a file, and this decides *when* —
//! which is the only part of it that is Bevy's.
//!
//! Sibling to [`content`](super::content) and for the same reason: I/O the sim
//! must not do, landing on a tick boundary because §8 says a save may land
//! nowhere else.

use std::path::{Path, PathBuf};

use bevy::app::AppExit;
use bevy::prelude::*;

use super::driver::Tower;

/// The file the tower in front of the player is kept in.
///
/// State, not a call, travelling with the `Sim` through `menuing::swap`, or the
/// game left is written into the file of the one opened. `None` for
/// `ORBS_SAVE=off`.
#[derive(Resource, Debug, Default, Clone)]
pub(crate) struct Kept(Option<PathBuf>);

impl Kept {
    /// Keep the tower at `path` from now on.
    pub(crate) const fn at(path: Option<PathBuf>) -> Self {
        Self(path)
    }

    /// Where the tower is kept, if anywhere.
    pub(crate) fn path(&self) -> Option<&Path> {
        self.0.as_deref()
    }
}

/// Write `tower` to `kept` immediately, before anything else moves, and say
/// whether it was written.
///
/// The half of a swap that must happen first: a load of the slot in hand reads
/// back what this wrote, and is refused when it wrote nothing.
pub(crate) fn keep_now(tower: &Tower, kept: &Kept) -> bool {
    let Some(path) = kept.path() else {
        return false;
    };
    let written = orbs_shell::write_save_to(path, &tower.sim().snapshot());
    if let Err(error) = &written {
        bevy::log::error!("the tower could not be written out: {error}");
    }
    written.is_ok()
}

/// Write the tower out, every `orbs_shell::AUTOSAVE_TICKS` of world time. §8's
/// *significant events* half is one event today — leaving — and it is not on
/// this clock at all: see [`keep_on_the_way_out`].
///
/// Ordered after [`advance`](super::driver::advance) in `FixedUpdate`, so the
/// world it describes is the one the tick just finished — §8's boundary, and the
/// moment `Pending` and `Skip` are both empty, which is what lets the document
/// leave them out.
pub(super) fn autosave(mut tower: ResMut<Tower>, kept: Res<Kept>, mut said: Local<bool>) {
    // Peeked before it is taken: reaching for `ResMut` unconditionally stamps
    // `Tower`'s change tick every second, re-arming its own `resource_changed`
    // run condition for ever and dragging the panel, the suggestions and the
    // four surface watchers back to frame rate. `driver` records it.
    if !orbs_shell::autosave_due(tower.sim().tick().get()) {
        return;
    }
    keep(&mut tower, &kept, &mut said);
}

/// Write the tower out because the game is closing.
///
/// Every way out: `AppExit`, in `Last`, where `F10`, the close button and
/// `quit` converge (§19).
pub(super) fn keep_on_the_way_out(
    leaving: MessageReader<AppExit>,
    mut tower: ResMut<Tower>,
    kept: Res<Kept>,
    mut said: Local<bool>,
) {
    if leaving.is_empty() {
        return;
    }
    keep(&mut tower, &kept, &mut said);
}

/// Write it, and complain once if it will not go — see `orbs_shell::keep`.
///
/// `Local<bool>` rather than a resource, because the two callers fail for the
/// same reason and neither needs to know about the other.
fn keep(tower: &mut Tower, kept: &Kept, said: &mut bool) {
    // The path this game came from, never `save_path()` — see `Kept`. A path
    // resolved here is resolved after a swap, which is how loading a second
    // tower writes the first one into the second one's file.
    if orbs_shell::keep(tower.sim(), kept.path(), said) {
        // And in voice, not only in the log: §3 makes the record stream the
        // output, a player never reads `tracing`, and with no `save` verb they
        // have no reason to look for a failure they did not ask for.
        tower.say_save_failed();
    }
}
