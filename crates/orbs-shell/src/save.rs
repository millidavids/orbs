//! The save, as a file — the half `orbs-sim` deliberately cannot do.
//!
//! `orbs-sim` makes the [`Save`] document; the file and the wall clock are
//! here. Nothing here may take the session down: an unreadable save starts a
//! new tower. But a failure is *said*, once — with no `save` verb (§19) a
//! player would never notice an hour of failed writes.

use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use orbs_sim::Save;

/// Environment variable naming where the save lives.
///
/// Three values, and the third is what keeps the instruments honest:
///
/// - unset — the platform's application-data directory, `<data>/orbs/`
/// - a path — that file, which is what a See-it line uses
/// - `off` — **no persistence at all**, which `scripts/dumps.sh` and the played
///   game suite both pin
pub const SAVE_VAR: &str = "ORBS_SAVE";

/// The value of [`SAVE_VAR`] that turns persistence off entirely.
pub const OFF: &str = "off";

/// What the save file is called, wherever it ends up.
pub const SAVE_PATH: &str = "orbs-save.toml";

/// The directory the orb keeps its towers and settings in, under the platform's
/// own application-data root.
///
/// One level down, since `dirs::data_dir` is the shared root.
pub const DIRECTORY: &str = "orbs";

/// Set by [`seal`]: this process may not reach a save at all.
///
/// An `AtomicBool`, since [`path`] is read globally, including from menu paths
/// a dump drives.
static SEALED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// This process keeps nothing, whatever the environment says.
///
/// Called by the dump when `ORBS_SAVE` names no path. One-way, so nothing can
/// turn it back off.
pub fn seal() {
    SEALED.store(true, std::sync::atomic::Ordering::Relaxed);
}

/// Whether this process has been sealed against saves.
#[must_use]
pub fn sealed() -> bool {
    SEALED.load(std::sync::atomic::Ordering::Relaxed)
}

/// Whether [`SAVE_VAR`] names a value at all.
///
/// An exported-but-empty `ORBS_SAVE=` names nothing, said once for [`chosen`]
/// and the dump, which once disagreed and overwrote a player's tower.
#[must_use]
pub fn named() -> bool {
    crate::environment::switched(SAVE_VAR).is_some()
}

/// Where the save is, or `None` if this session keeps none: `ORBS_SAVE`, then the
/// app-data directory, then the working directory.
///
/// App-data, since a Steam install directory can be read-only (§13). The
/// working directory is kept for a platform with no data directory.
#[must_use]
pub fn path() -> Option<PathBuf> {
    resolved(
        sealed(),
        std::env::var(SAVE_VAR).ok().as_deref(),
        app_data(),
    )
}

/// The whole rule, with every input a parameter.
///
/// [`chosen`]'s reason, extended to the seal. The seal is process-global by
/// necessity, and a test that called [`seal`] would seal every other test in the
/// binary — so the *rule* is tested here and the *switch* by `scripts/dumps.sh`.
fn resolved(sealed: bool, value: Option<&str>, app_data: Option<PathBuf>) -> Option<PathBuf> {
    if sealed {
        return None;
    }
    chosen(value, app_data)
}

/// The platform's application-data directory for this game, if it names one.
///
/// Not created here: resolving a path and making a directory are different acts,
/// and this is called from the menu's listing several times a session.
/// [`write_to`] creates it, once, on the way to the first write.
#[must_use]
pub fn app_data() -> Option<PathBuf> {
    Some(dirs::data_dir()?.join(DIRECTORY))
}

/// The rule [`path`] applies, without reaching for the environment or the
/// platform.
///
/// Split out so it can be tested without setting the per-process environment
/// or reaching a real save directory.
fn chosen(value: Option<&str>, app_data: Option<PathBuf>) -> Option<PathBuf> {
    match value.map(str::trim) {
        Some(value) if value.eq_ignore_ascii_case(OFF) => None,
        Some(value) if !value.is_empty() => Some(PathBuf::from(value)),
        _ => Some(app_data.map_or_else(|| PathBuf::from(SAVE_PATH), |dir| dir.join(SAVE_PATH))),
    }
}

/// What was waiting where the save lives.
///
/// Three answers: *no save* is a first launch, *would not open* is owed a
/// reason.
#[derive(Debug)]
pub enum Opened {
    /// No save, or none kept. A first launch, or `ORBS_SAVE=off`.
    New,
    /// A tower to go back to.
    Restored(Box<Save>),
    /// Something was there and could not be read.
    ///
    /// The file is kept, since a later build may read it.
    Unreadable,
}

/// Read the save this session keeps, if there is one worth reading.
#[must_use]
pub fn read() -> Opened {
    let Some(path) = path() else {
        return Opened::New;
    };
    read_from(&path)
}

/// Read the save at `path`, setting it aside if it cannot be read.
///
/// The door a save is *loaded* through, as opposed to merely listed. The pair of
/// [`write_to`], and what lets the menu hold more than one game without a
/// process-global notion of *the* save.
#[must_use]
pub fn read_from(path: &Path) -> Opened {
    match load(path) {
        // Set aside before the next autosave takes it — renamed, best-effort.
        Opened::Unreadable => {
            keep_aside(path);
            Opened::Unreadable
        }
        opened => opened,
    }
}

/// Read `path` and say what was there. **Renames nothing.**
///
/// So [`saves`]' listing renames nothing; the rename is [`read_from`]'s, the
/// loading door.
fn load(path: &Path) -> Opened {
    let text = match std::fs::read_to_string(path) {
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
            Opened::Unreadable
        }
    }
}

/// Move a save this build cannot read out of the way of the next autosave.
///
/// Named, not dated; an existing set-aside file is kept, since the first
/// failure holds the tower worth keeping.
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
/// `Away::unix` is stamped here, since the sim has no clock (§19). Written to
/// one side and renamed, so a crash leaves a minute-stale tower, not a corrupt
/// one.
///
/// # Errors
///
/// If the file cannot be written — a read-only directory being the case §13's
/// deferred `dirs` decision leaves open. The caller says so once and plays on.
pub fn write(save: &Save) -> io::Result<()> {
    let Some(path) = path() else {
        return Ok(());
    };
    write_to(&path, save)
}

/// Write the save to `path`, stamping the wall clock on the way past.
///
/// [`read_from`]'s pair: the path travels with the caller, since a global
/// *current save* would let loading a second game destroy the first.
///
/// # Errors
///
/// As [`write()`]: if the file cannot be written.
pub fn write_to(path: &Path, save: &Save) -> io::Result<()> {
    let mut stamped = save.clone();
    stamped.away.unix = now();

    let text = stamped
        .to_toml()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?;

    // The directory first, which a fresh machine lacks. Best-effort: a failure
    // falls through to the write, which names the real problem.
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).ok();
    }

    let scratch = scratch_beside(path);
    std::fs::write(&scratch, text)?;
    std::fs::rename(&scratch, path)
}

/// The file that says the towers beside the binary have already been brought in.
///
/// A marker, not *copy if absent*, which would resurrect an abandoned slot.
const MIGRATED: &str = ".migrated";

/// Bring towers and settings beside the binary into the app-data directory.
///
/// Copied, never moved, so an older build still finds its towers and a failed
/// move cannot split them. Skipped under `ORBS_SAVE`, with no data directory,
/// when the two are one, or once the marker exists.
pub fn migrate() {
    // `ORBS_SAVE` naming a path means the player has said where their towers are;
    // moving other ones in beside them is not what they asked for. `off` means
    // this session keeps nothing. (`named` came from here — the dump had a second
    // spelling of the rule that got it wrong.)
    if named() {
        return;
    }
    let Some(into) = app_data() else {
        return;
    };
    let marker = into.join(MIGRATED);
    if marker.exists() {
        return;
    }
    let from = PathBuf::from(SAVE_PATH);
    let old_settings = PathBuf::from(crate::settings::FILE);

    // Any slot, not slot 1, which `abandon` can have renamed away.
    let anything = (1..=SLOTS).any(|slot| slot_path(&from, slot).exists()) || old_settings.exists();
    if !anything {
        return;
    }
    if std::fs::create_dir_all(&into).is_err() {
        return;
    }

    let mut brought = 0_usize;
    // A copy that *failed* is not a copy that was not needed, and the difference
    // decides whether this ever runs again: the loop used to fall through to the
    // marker, which then stopped it for ever, so an unreadable slot or a full
    // disk left towers in the old directory permanently.
    let mut failed = false;
    for slot in 1..=SLOTS {
        let old = slot_path(&from, slot);
        let new = into.join(slot_path(&PathBuf::from(SAVE_PATH), slot));
        // Never over a tower that is already there. The marker means this runs
        // once, but a directory somebody has copied files into by hand is not a
        // case to overwrite on a guess.
        if old.exists() && !new.exists() {
            match std::fs::copy(&old, &new) {
                Ok(_) => brought += 1,
                Err(error) => {
                    tracing::warn!(
                        "save: could not bring {} across ({error}); will try again next launch",
                        old.display(),
                    );
                    failed = true;
                }
            }
        }
    }
    // The settings file travels with them: same directory, same promise.
    // `settings::path` derives its name from this one.
    //
    // (This used to join `from.parent()` onto the name — `Some("")` for a bare
    // relative path, which the filter rejected, so the branch was dead.)
    let new_settings = into.join(crate::settings::FILE);
    if old_settings.exists()
        && !new_settings.exists()
        && let Err(error) = std::fs::copy(&old_settings, &new_settings)
    {
        tracing::warn!("save: could not bring the settings across ({error})");
        failed = true;
    }

    // The marker last, and only on a clean run, so a run that died half way
    // through tries again rather than declaring itself done. That covered a
    // *crash* and not a failed copy, and the marker's whole job is to make this
    // never happen again.
    if failed {
        return;
    }
    if std::fs::write(&marker, "towers brought in from the working directory\n").is_ok() {
        tracing::info!(
            "save: brought {brought} tower(s) into {} — the originals are untouched",
            into.display(),
        );
    }
}

/// How many towers the orb will keep at once.
///
/// Bounded, because [`saves`] reads every slot to build a listing and an
/// unbounded scan is a directory walk with a stat per entry each time the menu
/// opens. Six is more games than anyone runs at once and fits a short pane.
pub const SLOTS: usize = 6;

/// Where slot `slot` lives, given the path slot 1 is at.
///
/// Slot 1 *is* the base path; the rest sit beside it as `orbs-save-2.toml`.
/// The extension is kept, since `ORBS_SAVE` can name any file.
#[must_use]
pub fn slot_path(base: &Path, slot: usize) -> PathBuf {
    if slot <= 1 {
        return base.to_path_buf();
    }
    let stem = base.file_stem().unwrap_or_default().to_string_lossy();
    let mut name = format!("{stem}-{slot}");
    if let Some(extension) = base.extension() {
        name.push('.');
        name.push_str(&extension.to_string_lossy());
    }
    base.with_file_name(name)
}

/// Where a new tower is to be kept: a slot, and slot 1's path beside it.
///
/// Both read once, when the play page opened, so a slot asked again later is
/// asked of the same folder and name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keep {
    /// Slot 1's path, which names the rest ([`slot_path`]).
    pub base: PathBuf,
    /// Which slot, from 1.
    pub slot: usize,
}

impl Keep {
    /// The file this slot is.
    #[must_use]
    pub fn path(&self) -> PathBuf {
        slot_path(&self.base, self.slot)
    }

    /// This slot while it is empty, else the lowest empty one beside it, or
    /// `None` when every slot is taken.
    #[must_use]
    pub fn still_free(&self) -> Option<Self> {
        if !self.path().exists() {
            return Some(self.clone());
        }
        free_slot_in(&self.base).map(|slot| Self {
            base: self.base.clone(),
            slot,
        })
    }
}

/// What a slot holds, when this build can read it.
///
/// Everything here is already in the document — wizard, seed, tick, length and
/// the wall-clock stamp — which is why more than one save needs no `FORMAT` bump.
/// Nothing was added; it was all being written and read back by one caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Held {
    /// The wizard's name.
    pub wizard: String,
    /// How long the game is.
    pub length: orbs_sim::content::Length,
    /// How many ticks the tower has run.
    pub tick: u64,
    /// Experience earned — how far along the game is, in the number the weave
    /// draws.
    pub experience: u64,
    /// How long the orb has been dark, if the save can say.
    pub away: Option<u64>,
}

/// One slot of the orb's, as the menu lists it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slot {
    /// Which slot, from 1. What the player types to open it.
    pub slot: usize,
    /// The file it is in.
    pub path: PathBuf,
    /// The tower in it — or [`None`] when there is a file there this build
    /// cannot read.
    ///
    /// An unreadable slot is listed, not skipped, or it looks empty and `new`
    /// writes over it ([`Opened::Unreadable`]).
    pub held: Option<Held>,
}

impl Slot {
    /// Whether this build could read what is in it.
    #[must_use]
    pub const fn is_readable(&self) -> bool {
        self.held.is_some()
    }
}

/// Every slot the orb is keeping something in, in slot order.
///
/// Empty under `ORBS_SAVE=off`, so a dump is a baseline.
///
/// Reads through the private `load`, never [`read_from`] — see `load` for why.
#[must_use]
pub fn saves() -> Vec<Slot> {
    let Some(base) = path() else {
        return Vec::new();
    };
    (1..=SLOTS)
        .filter_map(|slot| {
            let path = slot_path(&base, slot);
            let held = match load(&path) {
                Opened::Restored(save) => Some(Held {
                    wizard: save.progress.wizard.clone(),
                    length: save.world.length,
                    tick: save.world.tick,
                    experience: save.progress.experience,
                    away: away_for(&save),
                }),
                // Listed, and listed as what it is. See `Slot::held`.
                Opened::Unreadable => None,
                // Genuinely empty. Nothing to list and `free_slot` will offer it.
                Opened::New => return None,
            };
            Some(Slot { slot, path, held })
        })
        .collect()
}

/// Set the tower in `path` aside, so the slot it was in comes free.
///
/// Renamed to `.abandoned`, never unlinked, so a wrong slot is recoverable.
/// Asked twice first (§19). An existing `.abandoned` is overwritten, unlike
/// `keep_aside`'s: this one is deliberate.
///
/// # Errors
///
/// If the file cannot be renamed — a read-only directory, or a slot whose file
/// has gone since the listing was read.
pub fn abandon(path: &Path) -> io::Result<()> {
    let aside = path.with_extension("toml.abandoned");
    std::fs::rename(path, &aside)?;
    tracing::info!("save: set {} aside as {}", path.display(), aside.display(),);
    Ok(())
}

/// The lowest slot with no tower in it, if the orb has room for another.
///
/// Lowest rather than next, so a game deleted from the middle leaves a hole the
/// next new game fills — the alternative marches up to [`SLOTS`] and refuses
/// while five slots stand empty.
#[must_use]
pub fn free_slot() -> Option<usize> {
    free_slot_in(&path()?)
}

/// [`free_slot`] beside `base`, so the rule can be tested without `ORBS_SAVE`.
#[must_use]
pub fn free_slot_in(base: &Path) -> Option<usize> {
    (1..=SLOTS).find(|slot| matches!(load(&slot_path(base, *slot)), Opened::New))
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
/// `None` with no stamp or a clock gone backwards. Nothing accrues offline yet
/// (§5); a frontend only says it.
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
/// Beside the save, never in a temp directory: `rename` is only atomic within a
/// filesystem, and `/tmp` is a different one often enough to matter — where it is
/// not, the fallback is a copy, the half-written file this exists to prevent.
fn scratch_beside(path: &Path) -> PathBuf {
    let mut scratch = path.as_os_str().to_owned();
    scratch.push(".writing");
    PathBuf::from(scratch)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A stand-in for the platform's data directory.
    ///
    /// Never the real one: `chosen` takes it as a parameter so a test cannot
    /// reach a developer's own towers.
    fn app_data_is(dir: &str) -> Option<PathBuf> {
        Some(PathBuf::from(dir).join(DIRECTORY))
    }

    #[test]
    fn a_sealed_process_reaches_no_save_whatever_the_environment_says() {
        // The seal outranks every leg, a named path included.
        let somewhere = app_data_is("/home/somebody/.local/share");
        for value in [None, Some("off"), Some("/tmp/named.toml"), Some("")] {
            assert_eq!(
                resolved(true, value, somewhere.clone()),
                None,
                "a sealed process resolved a path for {value:?}",
            );
        }
        // ...and unsealed, the rule is exactly `chosen`'s, unchanged.
        for value in [None, Some("off"), Some("/tmp/named.toml"), Some("")] {
            assert_eq!(
                resolved(false, value, somewhere.clone()),
                chosen(value, somewhere.clone()),
                "the seal changed the answer for {value:?} while off",
            );
        }
    }

    #[test]
    fn an_empty_value_is_not_a_path_anybody_named() {
        // A blank variable asks for nothing. `named` reads the environment, so
        // this holds the rule it agrees with.
        let somewhere = app_data_is("/home/somebody/.local/share");
        for value in ["", "   ", "\t"] {
            assert_eq!(
                chosen(Some(value), somewhere.clone()),
                chosen(None, somewhere.clone()),
                "an empty `ORBS_SAVE={value:?}` resolved differently from an unset one",
            );
        }
    }

    #[test]
    fn off_is_the_only_value_that_keeps_no_save() {
        // What every instrument pins, outranking the data directory.
        for app_data in [None, app_data_is("/home/somebody/.local/share")] {
            assert_eq!(chosen(Some("off"), app_data.clone()), None);
            assert_eq!(
                chosen(Some("OFF"), app_data.clone()),
                None,
                "the value is not case-sensitive",
            );
            assert_eq!(
                chosen(Some("  off  "), app_data),
                None,
                "nor whitespace-sensitive",
            );
        }

        // A named path outranks the data directory: it is the player saying
        // where, and a See-it line saying where.
        assert_eq!(
            chosen(Some("/tmp/x.toml"), app_data_is("/home/somebody/share")),
            Some(PathBuf::from("/tmp/x.toml")),
        );
        // ...and a file that happens to be *called* off is still a file.
        assert_eq!(chosen(Some("./off"), None), Some(PathBuf::from("./off")),);
    }

    #[test]
    fn an_unnamed_save_goes_to_the_data_directory_and_falls_back_to_the_cwd() {
        // The whole of the move. Unset and blank both land in the platform's
        // application-data directory, where a player's towers live now — §13's
        // recorded cost for the working directory was a Steam install that can
        // refuse to be written to.
        let into = app_data_is("/home/somebody/.local/share");
        for value in [None, Some(""), Some("   ")] {
            assert_eq!(
                chosen(value, into.clone()),
                Some(PathBuf::from(
                    "/home/somebody/.local/share/orbs/orbs-save.toml"
                )),
                "{value:?} did not land in the data directory",
            );
        }

        // The fall-back is kept rather than removed. A platform that names no
        // data directory still saves — a portable build on a stick — and it is
        // what every test takes, so nothing in the suite reaches real towers.
        for value in [None, Some("")] {
            assert_eq!(
                chosen(value, None),
                Some(PathBuf::from(SAVE_PATH)),
                "a platform with no data directory lost its save entirely",
            );
        }
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
    fn slot_one_is_the_save_that_was_always_there() {
        // Nothing migrates: a player who had a tower before the menu existed
        // still has it, and it is the first thing the listing shows.
        let base = PathBuf::from(SAVE_PATH);
        assert_eq!(slot_path(&base, 1), base);
        assert_eq!(
            slot_path(&base, 0),
            base,
            "there is no slot below the first"
        );
        assert_eq!(slot_path(&base, 2), PathBuf::from("orbs-save-2.toml"));
        assert_eq!(slot_path(&base, 6), PathBuf::from("orbs-save-6.toml"));
    }

    #[test]
    fn a_slot_keeps_the_extension_and_the_directory_it_was_given() {
        // `ORBS_SAVE` can name any file at all, so the numbering has to go
        // before the extension rather than after the whole name.
        assert_eq!(
            slot_path(Path::new("/somewhere/tower.bak"), 3),
            PathBuf::from("/somewhere/tower-3.bak"),
        );
        // ...and a file with no extension keeps having none.
        assert_eq!(
            slot_path(Path::new("/somewhere/tower"), 3),
            PathBuf::from("/somewhere/tower-3"),
        );
    }

    #[test]
    fn listing_a_save_it_cannot_read_does_not_rename_it() {
        // The constraint the split exists for. A listing reads every slot, so one
        // that set unreadable files aside would move a whole directory the first
        // time this build met a save it could not open — turning "one tower did
        // not come back" into "none of them are where they were".
        let dir = std::env::temp_dir().join(format!("orbs-listing-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("a temp directory");
        let path = dir.join("orbs-save.toml");
        std::fs::write(&path, "this is not a save").expect("a file");

        assert!(matches!(load(&path), Opened::Unreadable));
        assert!(path.exists(), "`load` renamed a file it only read");

        // ...and the loading door does set it aside, which is the other half.
        assert!(matches!(read_from(&path), Opened::Unreadable));
        assert!(
            !path.exists(),
            "`read_from` left an unreadable save in place"
        );
        assert!(path.with_extension("toml.unreadable").exists());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_tower_written_to_a_slot_reads_back_from_it() {
        // The pair `write`/`read` are wrappers over, and what lets the menu hold
        // more than one game: the path travels with the caller.
        let dir = std::env::temp_dir().join(format!("orbs-slots-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("a temp directory");
        let base = dir.join("orbs-save.toml");

        let first = orbs_sim::Sim::new(7).snapshot();
        let second = orbs_sim::Sim::new(9).snapshot();
        write_to(&slot_path(&base, 1), &first).expect("slot one");
        write_to(&slot_path(&base, 2), &second).expect("slot two");

        // The dangerous case in miniature: writing the second did not touch the
        // first, because the path is an argument rather than a global.
        let Opened::Restored(back) = read_from(&slot_path(&base, 1)) else {
            panic!("slot one did not come back");
        };
        assert_eq!(back.world.seed, 7, "writing slot two moved slot one");

        std::fs::remove_dir_all(&dir).ok();
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
