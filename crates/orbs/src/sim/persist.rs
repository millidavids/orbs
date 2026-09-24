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
/// The path is state rather than a function call, and that is what stops loading
/// a second game destroying the first. The autosave fires every sixty ticks and
/// again on the way out; while there was one game that path could be
/// `save_path()`, read afresh. With a menu that can put a different tower in
/// front of the player, a path read at the moment of writing is read *after* the
/// swap — so the game you just left is written into the file of the one you just
/// opened.
///
/// So the path travels with the `Sim`: `shell::menuing::swap` replaces both in
/// one operation and nothing between can observe one without the other.
///
/// `None` when this session keeps nothing at all — `ORBS_SAVE=off`, which
/// `scripts/dumps.sh` and the played-game suite both pin.
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

/// Write `tower` to `kept` immediately, before anything else moves.
///
/// The half of a swap that must happen first, and why it is a free function
/// rather than a method: it is called with the outgoing pair, at a point where
/// the incoming one has already been read off disk and is waiting.
pub(crate) fn keep_now(tower: &Tower, kept: &Kept) {
    let Some(path) = kept.path() else {
        return;
    };
    if let Err(error) = orbs_shell::write_save_to(path, &tower.sim().snapshot()) {
        bevy::log::error!("the tower could not be written out: {error}");
    }
}

/// How often the tower writes itself out.
///
/// §8 asks for *"every N ticks and on significant events, not every second"*.
/// One minute of world time is the whole of the answer for now: a save is a few
/// hundred lines of TOML and writing it costs less than a frame, and the most a
/// crash can cost is a minute of a game whose slowest single action is 94 ticks.
///
/// The *significant events* half is one event today — leaving — and it is not on
/// this clock at all. See [`keep_on_the_way_out`].
const AUTOSAVE_TICKS: u64 = 60;

/// Whether this tick is one that writes.
///
/// Read from the sim's own clock, not a counter of our own: a counter drifts
/// from world time the moment `meditate` runs three hundred ticks inside one
/// `step`, which is the trap §19 records the instrument animations falling into.
fn due(tower: &Tower) -> bool {
    let tick = tower.sim().tick().get();
    tick > 0 && tick.is_multiple_of(AUTOSAVE_TICKS)
}

/// Write the tower out, a minute of world time at a time.
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
    if !due(&tower) {
        return;
    }
    keep(&mut tower, &kept, &mut said);
}

/// Write the tower out because the game is closing.
///
/// Every way out, not the one with a word: `F10` writes an `AppExit` directly
/// and the window's close button produces one from `bevy_window`, neither
/// touching the `Quitting` flag — so ordering against `quit_requested` covers
/// one exit in three.
///
/// So it reads `AppExit` itself, in `Last`, the one place every route out has
/// converged by. `quit`'s §19 entry records the mirror-image bug.
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

/// Write it, and complain once if it will not go.
///
/// Once, because there is no `save` verb (§19): a player never asks for one and
/// never sees one refused, so a silent failure is the session's work gone with
/// nothing said. But a read-only directory fails all sixty attempts an hour,
/// which is the noise §19 deleted the editor's per-save announcement over.
///
/// `Local<bool>` rather than a resource, because the two callers fail for the
/// same reason and neither needs to know about the other.
fn keep(tower: &mut Tower, kept: &Kept, said: &mut bool) {
    // The path this game came from, never `save_path()` — see `Kept`. A path
    // resolved here is resolved after a swap, which is how loading a second
    // tower writes the first one into the second one's file.
    let Some(path) = kept.path() else {
        *said = false;
        return;
    };
    let Err(error) = orbs_shell::write_save_to(path, &tower.sim().snapshot()) else {
        *said = false;
        return;
    };
    if !*said {
        *said = true;
        bevy::log::error!("the tower could not be written out: {error}");
        // And in voice, not only in the log: §3 makes the record stream the
        // output, a player never reads `tracing`, and with no `save` verb they
        // have no reason to look for a failure they did not ask for.
        tower.say_save_failed();
    }
}
