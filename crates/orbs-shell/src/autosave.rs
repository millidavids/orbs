//! The tower writing itself out, on one clock for both frontends.

use std::path::Path;

use orbs_sim::Sim;

/// How often the tower writes itself out, in world ticks.
///
/// §8 asks for *"every N ticks and on significant events, not every second"*.
/// A minute of world time: a save is a few hundred lines of TOML and costs less
/// than a frame, and the most a crash can cost is a minute of a game whose
/// slowest single action is 94 ticks. One cadence for both frontends, or a
/// player moving between them loses more in one than the other.
pub const AUTOSAVE_TICKS: u64 = 60;

/// Whether `tick` is one that writes.
///
/// Read from the sim's own clock, not a counter of the frontend's: a counter
/// drifts from world time the moment `meditate` runs three hundred ticks inside
/// one `step`, the trap §19 records the instrument animations falling into.
#[must_use]
pub const fn autosave_due(tick: u64) -> bool {
    tick > 0 && tick.is_multiple_of(AUTOSAVE_TICKS)
}

/// Write the tower to `path`, if it has one, and say whether this is the first
/// failure in a row — the one the caller tells the player about, in voice.
///
/// Once, because there is no `save` verb (§19): a player never asks for one and
/// never sees one refused, so a silent failure is the session's work gone with
/// nothing said. But a read-only directory fails all sixty attempts an hour,
/// which is the noise §19 deleted the editor's per-save announcement over.
#[must_use]
pub fn keep(sim: &Sim, path: Option<&Path>, failing: &mut bool) -> bool {
    let Some(path) = path else {
        *failing = false;
        return false;
    };
    let Err(error) = crate::save::write_to(path, &sim.snapshot()) else {
        *failing = false;
        return false;
    };
    if *failing {
        return false;
    }
    *failing = true;
    tracing::error!("the tower could not be written out: {error}");
    true
}
