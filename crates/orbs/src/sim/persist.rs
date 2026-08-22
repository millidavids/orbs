//! Keeping the tower, and putting it back.
//!
//! The frontend half of the save. `orbs-sim` turns a world into a document,
//! `orbs_shell::save` turns a document into a file, and this decides *when* —
//! which is the only part of it that is Bevy's.
//!
//! Sibling to [`content`](super::content) and for the same reason: I/O the sim
//! must not do, landing on a tick boundary because §8 says a save may land
//! nowhere else.

use bevy::app::AppExit;
use bevy::prelude::*;

use super::driver::Tower;

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
/// **Read from the sim's own clock, not from a counter of our own.** A counter
/// here would drift from world time the moment `meditate` ran three hundred
/// ticks inside one `step` — which is the same trap §19 records the instrument
/// animations falling into, and the reason the tick is a *sample* rather than a
/// quantity.
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
pub(super) fn autosave(mut tower: ResMut<Tower>, mut said: Local<bool>) {
    // **Peeked before it is taken.** Reaching for `ResMut` unconditionally
    // stamps `Tower`'s change tick every second, which re-arms its own
    // `resource_changed` run condition for ever and drags the panel, the
    // suggestions and the four surface watchers back to frame rate. `quit`
    // shipped without this peek once; `driver` records it.
    if !due(&tower) {
        return;
    }
    keep(&mut tower, &mut said);
}

/// Write the tower out because the game is closing.
///
/// # Every way out, not the one with a word
///
/// `quit` is a verb and it is *not* the only exit: `F10` writes an `AppExit`
/// directly, the window's close button produces one from `bevy_window`, and
/// neither touches the `Quitting` flag. Ordering this against `quit_requested`
/// would therefore have covered one exit in three.
///
/// So it reads `AppExit` itself, in `Last`, which is the one place every route
/// out has converged by. `quit`'s own §19 entry records the mirror-image bug —
/// the first attempt checked the flag at `submit` time and did nothing.
pub(super) fn keep_on_the_way_out(
    leaving: MessageReader<AppExit>,
    mut tower: ResMut<Tower>,
    mut said: Local<bool>,
) {
    if leaving.is_empty() {
        return;
    }
    keep(&mut tower, &mut said);
}

/// Write it, and complain **once** if it will not go.
///
/// # Why once
///
/// There is no `save` verb (§19), so a player never asks for one and never sees
/// one refused — which makes a silent failure the whole session's worth of work
/// gone with nothing said. But a save is attempted every sixty ticks, and a
/// read-only directory fails every one of them: a line per attempt would be
/// sixty an hour, which is the same noise §19 deleted the editor's per-save
/// announcement over.
///
/// One line, then quiet. `Local<bool>` rather than a resource because the two
/// callers fail for the same reason and neither needs to know about the other.
fn keep(tower: &mut Tower, said: &mut bool) {
    let Err(error) = orbs_shell::write_save(&tower.sim().snapshot()) else {
        *said = false;
        return;
    };
    if !*said {
        *said = true;
        bevy::log::error!("the tower could not be written out: {error}");
        // **And in voice, not only in the log.** §3 makes the record stream the
        // output; a player never reads `tracing`, and with no `save` verb they
        // have no reason to go looking for a failure they did not ask for.
        tower.say_save_failed();
    }
}
