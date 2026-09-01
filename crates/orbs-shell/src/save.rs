//! The save, as a file — the half `orbs-sim` deliberately cannot do.
//!
//! `orbs-sim` turns a world into a [`Save`] document and back and touches no
//! filesystem; `tests/boundaries.rs` there enforces it. Finding the file,
//! writing it, reading it and stamping the wall clock on the way past are all
//! here, which is the same split `prose` already uses for content: *"finding and
//! parsing that file is plain `std::fs`, and every frontend needs it."*
//!
//! # Nothing here may take the session down
//!
//! `shortcuts::export_trace` sets the posture and the reason is the same one:
//! *"a failed export must not take the session down with it — the tester whose
//! run it was recording is still playing."* A save that cannot be read starts a
//! new tower; a save that cannot be written says so once and the game carries on.
//!
//! # ...but a failure must be *said*
//!
//! There is no `save` verb (§19), so a player never asks for one and never sees
//! one refused. That makes silence the wrong default in the one place it would
//! otherwise be tempting: a write that fails every minute for an hour, on a
//! read-only Steam directory, would cost a whole session and report nothing.
//! [`write`] returns the error and the frontend says it once.

use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use orbs_sim::Save;

/// Environment variable naming where the save lives.
///
/// Three values, and the third is what keeps the instruments honest:
///
/// - unset — [`SAVE_PATH`], beside the binary
/// - a path — that file, which is what a See-it line uses
/// - `off` — **no persistence at all**, which `scripts/dumps.sh` and the played
///   game suite both pin
pub const SAVE_VAR: &str = "ORBS_SAVE";

/// The value of [`SAVE_VAR`] that turns persistence off entirely.
pub const OFF: &str = "off";

/// Where the save lives when nothing says otherwise.
///
/// **Beside the binary, like the parse trace**, and for a version at least. §13
/// names `dirs` in the stack and Phase 11's settings screen is where it arrives
/// with Steam Cloud; until then this is one function to change rather than a
/// path scattered through two frontends. The honest cost is written down in
/// [`write()`]: an install directory can be read-only.
pub const SAVE_PATH: &str = "orbs-save.toml";

/// Where the save is, or `None` if this session keeps none.
#[must_use]
pub fn path() -> Option<PathBuf> {
    chosen(std::env::var(SAVE_VAR).ok().as_deref())
}

/// The rule [`path`] applies, without the environment.
///
/// Split out so it can be *tested*: a test that set `ORBS_SAVE` would set it for
/// every other test in the binary, because the environment is per-process and
/// `cargo test` runs threads. The three cases are the whole of the rule and none
/// of them needs a real machine to check.
fn chosen(value: Option<&str>) -> Option<PathBuf> {
    match value.map(str::trim) {
        Some(value) if value.eq_ignore_ascii_case(OFF) => None,
        Some(value) if !value.is_empty() => Some(PathBuf::from(value)),
        _ => Some(PathBuf::from(SAVE_PATH)),
    }
}

/// What was waiting where the save lives.
///
/// **Three answers, not two.** *No save* and *a save that would not open* both
/// end in a new tower, but they are not the same thing to say to a player: the
/// first is a first launch and the second is a tower that did not come back, and
/// somebody is owed the reason for that.
#[derive(Debug)]
pub enum Opened {
    /// No save, or none kept. A first launch, or `ORBS_SAVE=off`.
    New,
    /// A tower to go back to.
    Restored(Box<Save>),
    /// Something was there and could not be read.
    ///
    /// **The file is kept, not deleted.** A save this build cannot read may be
    /// one a later build can — the format refuses a *newer* file precisely so it
    /// is not half-read — and overwriting it is what the next autosave does
    /// anyway. Losing a tower is bad; losing it silently and destroying the
    /// evidence is worse.
    Unreadable,
}

/// Read the save, if there is one worth reading.
#[must_use]
pub fn read() -> Opened {
    let Some(path) = path() else {
        return Opened::New;
    };
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        // Not an error and not worth a line: the first launch has no save.
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Opened::New,
        Err(error) => {
            tracing::warn!("save: cannot read {} ({error})", path.display());
            return Opened::Unreadable;
        }
    };
    match Save::from_toml(&text) {
        Ok(save) => Opened::Restored(Box::new(save)),
        Err(error) => {
            tracing::warn!(
                "save: {} is not a save this build reads ({error})",
                path.display()
            );
            // **Set aside, because the next autosave is sixty ticks away.**
            // `Opened::Unreadable`'s own doc says *"the file is kept … losing a
            // tower is bad; losing it silently and destroying the evidence is
            // worse"* — and the code did exactly that: the player got a fresh
            // tower and the autosave overwrote the old one a minute later.
            //
            // Renaming rather than copying, so there is no window in which two
            // files claim to be the save. If the rename fails the original is
            // still there and the next autosave will take it, which is no worse
            // than before and is why this is best-effort rather than fatal.
            keep_aside(&path);
            Opened::Unreadable
        }
    }
}

/// Move a save this build cannot read out of the way of the next autosave.
///
/// Named for what it is rather than dated: one unreadable save is the case that
/// happens, and a directory filling with `orbs-save.toml.1` through `.9` is a
/// worse answer to a problem the player has once. An existing set-aside file is
/// left alone for the same reason — the *first* failure holds the tower worth
/// keeping, and a later one is a fresh tower this build wrote and could not read
/// back, which is a bug report rather than a loss.
fn keep_aside(path: &std::path::Path) {
    let aside = path.with_extension("toml.unreadable");
    if aside.exists() {
        tracing::warn!("save: {} is already set aside", aside.display());
        return;
    }
    match std::fs::rename(path, &aside) {
        Ok(()) => tracing::warn!(
            "save: kept the old tower at {} — this build could not read it",
            aside.display()
        ),
        Err(error) => tracing::warn!("save: could not set {} aside ({error})", path.display()),
    }
}

/// Write the save, stamping the wall clock on the way past.
///
/// # The stamp is the frontend's, and that is architectural
///
/// §19 records that a wall-clock reading inside a sim record *"would make two
/// runs of one seed differ"*. The sim has no clock but its own tick and must not
/// acquire one, so `Away::unix` is filled in here — the one place that already
/// knows what time it is because it is the one place that talks to the machine.
///
/// # Written to one side and renamed
///
/// A save is written every sixty ticks for as long as the game is open, so the
/// window in which a crash could catch a half-written file is not theoretical.
/// The temporary file and the rename make the swap atomic on every platform this
/// ships to, which turns "the tower is corrupt" into "the tower is one minute
/// stale".
///
/// # Errors
///
/// If the file cannot be written — a read-only directory being the case §13's
/// deferred `dirs` decision leaves open. The caller says so once and plays on.
pub fn write(save: &Save) -> io::Result<()> {
    let Some(path) = path() else {
        return Ok(());
    };

    let mut stamped = save.clone();
    stamped.away.unix = now();

    let text = stamped
        .to_toml()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?;

    let scratch = scratch_beside(&path);
    std::fs::write(&scratch, text)?;
    std::fs::rename(&scratch, &path)
}

/// Seconds since the Unix epoch, or nought if the machine will not say.
///
/// A clock before 1970 is a machine whose time is wrong, and the honest answer
/// to *"how long was the orb dark"* on such a machine is *"no idea"* — which is
/// what nought means to [`away_for`].
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs())
}

/// How long the orb was dark, if the save can say.
///
/// `None` where the stamp is missing (a save written before the field existed,
/// or by a machine with no clock) or where the clock has gone *backwards* — a
/// player who moved timezones, or a machine that corrected itself, is not a
/// player who was away for minus three hours.
///
/// **Nothing acts on this yet, deliberately.** §5 opens *"initially there is no
/// offline progression — the tower ticks only while the window is open"* and
/// puts accrual in Phase 9a. What a frontend does with this today is *say* it.
#[must_use]
pub fn away_for(save: &Save) -> Option<u64> {
    let departed = save.away.unix;
    if departed == 0 {
        return None;
    }
    now().checked_sub(departed).filter(|gap| *gap > 0)
}

/// The temporary file a save is written to before it is renamed into place.
///
/// **Beside the save, never in a temp directory.** `rename` is only atomic
/// within a filesystem, and `/tmp` is a different one often enough to matter —
/// on the platform where it is not, the fallback is a copy, which is the
/// half-written file this exists to prevent.
fn scratch_beside(path: &Path) -> PathBuf {
    let mut scratch = path.as_os_str().to_owned();
    scratch.push(".writing");
    PathBuf::from(scratch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn off_is_the_only_value_that_keeps_no_save() {
        // The value `scripts/dumps.sh` and the played-game suite both pin. If
        // this ever stops meaning "no persistence", one captured screen can
        // write a save that the next fifty-five read, and a baseline stops being
        // one.
        assert_eq!(chosen(Some("off")), None);
        assert_eq!(chosen(Some("OFF")), None, "the value is not case-sensitive");
        assert_eq!(chosen(Some("  off  ")), None, "nor whitespace-sensitive");

        assert_eq!(
            chosen(None),
            Some(PathBuf::from(SAVE_PATH)),
            "unset is the default path"
        );
        assert_eq!(
            chosen(Some("")),
            Some(PathBuf::from(SAVE_PATH)),
            "so is blank"
        );
        assert_eq!(
            chosen(Some("/tmp/x.toml")),
            Some(PathBuf::from("/tmp/x.toml"))
        );

        // ...and a file that happens to be *called* off is still a file.
        assert_eq!(chosen(Some("./off")), Some(PathBuf::from("./off")));
    }

    #[test]
    fn the_scratch_file_is_a_sibling_of_the_save() {
        // The rename is only atomic within a filesystem, so this must never
        // wander off to a temp directory.
        let scratch = scratch_beside(Path::new("/somewhere/orbs-save.toml"));
        assert_eq!(
            scratch.parent(),
            Path::new("/somewhere/orbs-save.toml").parent()
        );
        assert_ne!(scratch, PathBuf::from("/somewhere/orbs-save.toml"));
    }

    #[test]
    fn a_save_with_no_stamp_reports_no_absence() {
        // A save written before the stamp existed, or by a machine with no
        // clock. `Away::default()` is nought, which must not read as 1970.
        let mut save = orbs_sim::Sim::new(0).snapshot();
        save.away.unix = 0;
        assert_eq!(away_for(&save), None);
    }

    #[test]
    fn a_clock_that_went_backwards_is_not_an_absence() {
        let mut save = orbs_sim::Sim::new(0).snapshot();
        save.away.unix = now() + 3_600;
        assert_eq!(away_for(&save), None, "a future stamp is a wrong clock");
    }
}
