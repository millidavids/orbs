//! The save, as a file — the half `orbs-sim` deliberately cannot do.
//!
//! `orbs-sim` turns a world into a [`Save`] document and back and touches no
//! filesystem; `tests/boundaries.rs` there enforces it. Finding the file,
//! writing it, reading it and stamping the wall clock are all here — the split
//! `prose` already uses for content.
//!
//! Nothing here may take the session down, for `shortcuts::export_trace`'s
//! reason: *"a failed export must not take the session down with it — the tester
//! whose run it was recording is still playing."* A save that cannot be read
//! starts a new tower; one that cannot be written says so once and plays on.
//!
//! But a failure must be *said*. There is no `save` verb (§19), so a player never
//! asks for one and never sees one refused — and a write failing every minute for
//! an hour on a read-only Steam directory would cost a whole session in silence.
//! [`write()`] returns the error and the frontend says it once.

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
/// One level, named for the game, because `dirs::data_dir` is the *shared* root
/// — `~/.local/share`, `Application Support`, `AppData\Roaming` — and writing
/// `orbs-save.toml` straight into it would be a file in everybody's way.
pub const DIRECTORY: &str = "orbs";

/// Set by [`seal`]: this process may not reach a save at all.
///
/// An `AtomicBool` rather than a parameter, because what has to be stopped is a
/// *global* read: [`path`] reaches the environment from wherever it is called,
/// and the menu calls it from three places a dump can drive with `ORBS_MENU`. A
/// dump listed the player's real towers into a capture, and `abandon 1` twice
/// renamed one.
static SEALED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// This process keeps nothing, whatever the environment says.
///
/// Called by the dump when `ORBS_SAVE` names no path, which makes `dump.rs`'s
/// claim — *"a dump neither loads nor saves unless `ORBS_SAVE` names a path"* —
/// true of the whole process rather than of the one line that reads the world.
///
/// One-way on purpose: a dump never becomes a game half way through, and a switch
/// that can be turned back off is one something else could turn back off.
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
/// An exported-but-empty `ORBS_SAVE=` is not naming one, said in one place.
/// [`chosen`] has always trimmed and rejected empty; the dump asked
/// `var_os(..).is_some()` instead, so with `ORBS_SAVE=` it skipped [`seal`] *and*
/// resolved through `chosen` to the real app-data directory — reading the
/// player's tower, setting it aside, and writing a scratch save over it.
///
/// The same hole this release closed in `crt::seeded` and `sight::seeded`, found
/// in a third place a day later. One rule now, and both callers use it.
#[must_use]
pub fn named() -> bool {
    std::env::var(SAVE_VAR).is_ok_and(|value| !value.trim().is_empty())
}

/// Where the save is, or `None` if this session keeps none: `ORBS_SAVE`, then the
/// app-data directory, then the working directory.
///
/// The middle leg is where a player's towers live now. They sat in the working
/// directory, which §13 and `PRIVACY_POLICY.md` both promised would change *"with
/// the settings screen"* — and [`write()`] recorded the cost the whole time: a
/// Steam install directory can be read-only.
///
/// The last leg is kept rather than removed. A platform that names no data
/// directory still gets a game that saves — a portable build on a stick — and it
/// is what every `cargo test` falls back to, so nothing in the suite reaches a
/// developer's real towers.
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
/// Split out so it can be *tested*. A test that set `ORBS_SAVE` would set it for
/// every other test in the binary — the environment is per-process, `cargo test`
/// runs threads — and with the app-data leg in place would read and write the
/// developer's own towers. Both halves are parameters here, so nothing in the
/// suite can reach a real machine's save directory.
fn chosen(value: Option<&str>, app_data: Option<PathBuf>) -> Option<PathBuf> {
    match value.map(str::trim) {
        Some(value) if value.eq_ignore_ascii_case(OFF) => None,
        Some(value) if !value.is_empty() => Some(PathBuf::from(value)),
        _ => Some(app_data.map_or_else(|| PathBuf::from(SAVE_PATH), |dir| dir.join(SAVE_PATH))),
    }
}

/// What was waiting where the save lives.
///
/// Three answers, not two. *No save* and *a save that would not open* both end in
/// a new tower, but they are not the same thing to say: the first is a first
/// launch, the second a tower that did not come back, and somebody is owed the
/// reason.
#[derive(Debug)]
pub enum Opened {
    /// No save, or none kept. A first launch, or `ORBS_SAVE=off`.
    New,
    /// A tower to go back to.
    Restored(Box<Save>),
    /// Something was there and could not be read.
    ///
    /// The file is kept, not deleted. A save this build cannot read may be one a
    /// later build can — the format refuses a *newer* file precisely so it is not
    /// half-read — and the next autosave overwrites it anyway. Losing a tower is
    /// bad; losing it silently and destroying the evidence is worse.
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
        // Set aside, because the next autosave is sixty ticks away and used to
        // take it — a fresh tower, and the old one overwritten a minute later.
        //
        // Renamed rather than copied, so no window has two files claiming to be
        // the save. A failed rename leaves the original, which is no worse than
        // before — hence best-effort rather than fatal.
        Opened::Unreadable => {
            keep_aside(path);
            Opened::Unreadable
        }
        opened => opened,
    }
}

/// Read `path` and say what was there. **Renames nothing.**
///
/// The split is the point. [`saves`] reads every slot to build a listing, and a
/// listing that renamed what it listed would set a whole directory aside the
/// first time a build could not read one — turning "one tower did not come back"
/// into "none of them are where they were". So the rename lives in
/// [`read_from`], the *loading* door, out of the listing's reach.
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
/// Named rather than dated: one unreadable save is the case that happens, and a
/// directory of `orbs-save.toml.1` through `.9` is a worse answer to a problem
/// the player has once. An existing set-aside file is left alone — the *first*
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
/// The stamp is the frontend's: §19 records that a wall-clock reading inside a
/// sim record *"would make two runs of one seed differ"*, and the sim must not
/// acquire a clock, so `Away::unix` is filled in here.
///
/// Written to one side and renamed. A save lands every sixty ticks, so a crash
/// catching a half-written file is not theoretical; the rename makes the swap
/// atomic everywhere this ships, turning "the tower is corrupt" into "the tower
/// is one minute stale".
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
/// The pair of [`read_from`], and what makes more than one game possible: the
/// path travels with the caller rather than being asked for at the moment of
/// writing. A process-global *current save* would be read by the autosave at a
/// moment nobody chose — which is how loading a second game destroys the first.
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

    // The directory first, because on a fresh machine there is not one.
    // `~/.local/share/orbs` does not exist until something makes it, and without
    // this the first autosave fails, says so once, and then keeps nothing for the
    // rest of the session — which looks exactly like saving.
    //
    // Best-effort: an existing directory is not an error, and a *failure* here
    // falls through to the write below, which names the real problem rather than
    // blaming a directory when the place is read-only.
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
/// A marker rather than a per-slot *copy if absent*, which would resurrect a
/// stale tower from the old directory into a slot `free_slot` had just offered
/// somebody who abandoned theirs. One file, written once, makes the question
/// *has this happened* instead of *does this slot look empty*.
const MIGRATED: &str = ".migrated";

/// Bring towers and settings beside the binary into the app-data directory.
///
/// Copied, never moved, as `abandon` and `keep_aside` do one level down: the old
/// directory is left exactly as it was, so a player who goes back to an older
/// build finds their towers where it looks for them. They exist twice for a
/// while, which beats a move that fails part-way and splits a player's games
/// across two directories with neither complete.
///
/// It does nothing unless there is something to do: skipped when `ORBS_SAVE`
/// names a path or says `off`, when the platform will not name a data directory,
/// when the two are the same place, and when the marker says it has already run.
/// So no instrument in the project reaches it, and neither does a second launch.
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

    // Any slot, not slot 1. This asked `from.exists()`, and `from` *is* slot 1 by
    // construction — `slot_path` returns the base unchanged below 2. So a player
    // whose slot 1 was missing lost the whole migration: slots 2 to 6 and their
    // settings stayed put, the play page said "no towers yet", and no marker was
    // written, so it retried every launch and never succeeded. `abandon` made
    // that easy to reach: it renames slot 1 to `orbs-save.toml.abandoned`.
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
/// Slot 1 *is* the base path, so `orbs-save.toml` keeps being what it has always
/// been and no existing tower moves. The rest sit beside it as `orbs-save-2.toml`
/// and so on — one directory, nothing to migrate.
///
/// The extension is preserved rather than assumed, because `ORBS_SAVE` can name
/// any file and a player who pointed it at `tower.bak` should get `tower-2.bak`.
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
    /// An unreadable slot is listed rather than skipped — the case
    /// [`Opened::Unreadable`] says must be **said**. Dropped, the slot looked
    /// empty and typing its number answered *"there is no tower in 3"* about a
    /// file sitting right there with somebody's game in it, and `new` could then
    /// be offered the slot and write over it.
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
/// Empty when this session keeps no save at all (`ORBS_SAVE=off`), which
/// `scripts/dumps.sh` and the played-game suite pin: a listing that read the
/// working directory anyway would make a dump depend on what was lying beside
/// it, and a baseline that varies is not one.
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
/// Renamed, never unlinked, as `keep_aside` does: a player who types the wrong
/// slot number loses nothing they cannot get back by moving a file. The orb stops
/// listing it, `free_slot` offers the slot again, and the tower is still on disk
/// under `.abandoned`.
///
/// The only destructive word in the menu, which is why it asks twice before
/// reaching here — `quit`'s shape, and §19's argument that the confirmation is a
/// word rather than a screen.
///
/// An existing `.abandoned` file is overwritten, unlike `keep_aside`'s: that
/// guards a tower the *build* could not read, where the first failure is the one
/// worth keeping. This is deliberate, repeated deliberately, and a directory of
/// `.abandoned.1` files is the worse answer.
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
    let base = path()?;
    (1..=SLOTS).find(|slot| matches!(load(&slot_path(&base, *slot)), Opened::New))
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
/// `None` where the stamp is missing (a save written before the field existed, or
/// by a machine with no clock) or where the clock has gone *backwards* — a player
/// who moved timezones is not one who was away for minus three hours.
///
/// Nothing acts on this yet, deliberately. §5 opens *"initially there is no
/// offline progression — the tower ticks only while the window is open"* and puts
/// accrual in Phase 11a. What a frontend does with this today is *say* it.
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
        // Reproduced: a dump with no `ORBS_SAVE` listed the player's real towers
        // into the capture, and `ORBS_MENU=$'play\nabandon 1\nabandon 1'` renamed
        // one — the menu's play page calls `save::path()`, `saves()` and
        // `abandon()` itself, so the dump's "neither loads nor saves" was true of
        // one line and nothing else.
        //
        // The seal outranks every other leg, including a named path, or the
        // switch has a hole shaped like its own argument.
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
        // Got wrong in three places in two days, each the same way: an
        // exported-but-empty variable read as *"somebody asked for this"*.
        // `crt::seeded` and `sight::seeded` discarded a saved accessibility
        // choice over it; `dump.rs` skipped `seal()` and overwrote the player's
        // tower. `named` cannot be tested here without setting the variable for
        // every other test in the binary, so this holds the rule it agrees with.
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
        // The value `scripts/dumps.sh` and the played-game suite both pin. If
        // this stops meaning "no persistence", one captured screen writes a save
        // the next fifty-five read, and a baseline stops being one. `off`
        // outranks the data directory too: an instrument keeping a tower in a
        // real player's save directory is worse than one keeping it in the
        // checkout.
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
