//! The settings file: what the player has chosen about the orb itself.
//!
//! Not in the save. A save is a *tower* — its clock, its stores, what the wizard
//! has learned; a setting is about the machine in front of you. Putting one in
//! the other would carry a preference between slots, and loading somebody else's
//! tower would change how your keyboard behaves. So it is a second file beside
//! the saves, per-machine, and nothing in `orbs-sim` can see it.
//!
//! A `BTreeMap<String, String>` rather than a typed struct, because one of the
//! things a player sets is a phosphor theme, whose names come from the
//! frontend's palette — a list `orbs-shell` must not grow a second copy of.
//! Three things fall out, all wanted: the words are the format, so nothing
//! migrates and the file reads as what a player typed; a key this build does not
//! know is kept rather than dropped on the next write; and a frontend may keep a
//! setting the shell has never heard of, which the Bevy build's tube needs. The
//! type safety a struct bought is bought here by the values being words a player
//! typed, and by the readers below falling back rather than failing.
//!
//! Best-effort in both directions: a failed read is the defaults, a failed write
//! is dropped. `ORBS_SAVE=off` sessions, read-only checkouts and the dump all
//! run with nowhere to keep this, and refusing to change how a line is read
//! because it cannot be remembered is worse than forgetting.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::OnceLock;

/// What the settings file is called, wherever the saves are.
///
/// Named here rather than written into [`path`] because `save::migrate` brings
/// this file across with the towers, and a second spelling of it is a file that
/// quietly does not travel.
pub(crate) const FILE: &str = "orbs-settings.toml";

/// Everything the player has chosen, as words.
///
/// `BTreeMap` rather than `HashMap` so the file is written in a stable order —
/// lines that shuffled on every write make a diff useless.
type Kept = BTreeMap<String, String>;

/// Where the settings live, once a frontend has said.
///
/// Nothing until [`keep`] is called. It was `save::path()` with the file name
/// swapped, so every `cargo test` in the workspace read and wrote the
/// developer's own `~/.local/share/orbs/orbs-settings.toml`.
///
/// `save::chosen` takes its directory as a parameter for the same reason, and
/// `SimPlugin::persist` is an explicit flag: an environment variable is
/// process-global and `cargo test` runs threads. An opt-in is the one shape a
/// test that does not know it needs it cannot get wrong.
static KEPT_AT: OnceLock<Option<PathBuf>> = OnceLock::new();

/// Keep settings beside the saves from now on.
///
/// Called once, by a frontend's `main`, before anything reads a setting. A
/// process that never calls it — every test binary, and `orbs-balance` — keeps
/// no settings and gets the defaults.
pub fn keep() {
    let beside = crate::save::path().map(|save| save.with_file_name(FILE));
    // `set` fails when already called, which is not an error: one process has
    // one frontend, and a test calling it twice means the same thing both times.
    let _ = KEPT_AT.set(beside);
}

/// Where the settings live, or [`None`] if they live nowhere.
fn path() -> Option<PathBuf> {
    KEPT_AT.get().cloned().flatten()
}

/// Everything on file, or nothing.
#[must_use]
pub fn load() -> Kept {
    let Some(path) = path() else {
        return Kept::new();
    };
    read_kept(&path).unwrap_or_default()
}

/// What the player chose for `key`, if anything.
#[must_use]
pub fn get(key: &str) -> Option<String> {
    load().get(key).cloned()
}

/// Remember `value` for `key`.
///
/// Read, change, write — rather than writing the one key — so a setting this
/// build does not know about survives. Best-effort: a session with nowhere to
/// write still gets to change how its lines are read. See the module doc.
pub fn set(key: &str, value: &str) {
    let Some(path) = path() else {
        return;
    };
    // Read the file, not `load()`. `load` turns any parse failure into an empty
    // map — right for a reader, a catastrophe for a writer: one unreadable value
    // made the next `set` serialise a one-key map over every other setting. A
    // file that will not parse keeps what it has; the new value is what is lost.
    let mut kept = match read_kept(&path) {
        Some(kept) => kept,
        None if path.exists() => {
            tracing::warn!(
                "settings: {} will not parse; leaving it alone rather than overwriting it",
                path.display(),
            );
            return;
        }
        None => Kept::new(),
    };
    kept.insert(key.to_owned(), value.to_owned());
    if let Ok(text) = toml::to_string_pretty(&kept) {
        // The directory first, for `save::write_to`'s reason: on a fresh machine
        // the app-data folder does not exist until something makes it, and a
        // setting changed before the first autosave would never persist.
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent).ok();
        }
        // Scratch file, then rename, as `save::write_to` does: a bare truncate
        // leaves the file half-written if the process dies, and `load` answers
        // an unparseable file with the defaults — so the player silently loses
        // every accessibility choice they have made.
        //
        // A failed write is dropped rather than said: the choice has already
        // taken effect, and there is no screen here to say it on.
        let scratch = path.with_extension("toml.writing");
        if std::fs::write(&scratch, text).is_ok() && std::fs::rename(&scratch, &path).is_err() {
            std::fs::remove_file(&scratch).ok();
        }
    }
}

/// The file's contents, or `None` if it is absent or will not parse.
///
/// [`load`]'s honest half: `load` folds both answers into the defaults, which a
/// reader wants and a writer must not — see [`set`].
fn read_kept(path: &std::path::Path) -> Option<Kept> {
    toml::from_str(&std::fs::read_to_string(path).ok()?).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_file_is_written_in_a_stable_order() {
        // A `HashMap` would shuffle the lines on every write, making a settings
        // file appear to change when nothing did.
        let mut kept = Kept::new();
        for key in ["theme", "driver", "crt", "sight"] {
            kept.insert(key.to_owned(), "x".to_owned());
        }
        let text = toml::to_string_pretty(&kept).expect("settings serialise");
        let order: Vec<&str> = text
            .lines()
            .filter_map(|line| line.split_whitespace().next())
            .collect();
        assert_eq!(order, ["crt", "driver", "sight", "theme"]);
    }

    #[test]
    fn a_key_this_build_does_not_know_survives_a_write() {
        // The property the map exists for. Play a newer build, go back to an
        // older one, change one setting, and the newer one's choices are still
        // there. A struct with named fields silently discards them.
        let text = "driver = \"plain\"\nsomething_later = \"kept\"\n";
        let mut kept: Kept = toml::from_str(text).expect("a settings file");
        kept.insert("driver".to_owned(), "augury".to_owned());
        let written = toml::to_string_pretty(&kept).expect("settings serialise");
        assert!(
            written.contains("something_later"),
            "a key this build does not know was dropped:\n{written}",
        );
        assert!(written.contains("augury"), "the change did not land");
    }

    #[test]
    fn an_unreadable_file_is_the_defaults_rather_than_a_failure() {
        // `load` is called on the way into a settings page; a corrupt file must
        // not be able to stop a player reaching it.
        let broken: Result<Kept, _> = toml::from_str("this is not toml [[[");
        assert!(broken.is_err(), "the format got looser");
        assert!(Kept::new().is_empty());
    }

    #[test]
    fn a_process_that_never_opted_in_keeps_nothing() {
        // The property that stops a test writing a developer's settings, and it
        // already happened: `path` was `save::path()` with the file name
        // swapped, and the tests that press `F5` left values in a real
        // `~/.local/share/orbs/orbs-settings.toml`. This binary never called
        // `keep`, so the module is inert in it — which is what makes every other
        // test hermetic without knowing it had to be.
        assert_eq!(path(), None, "a test binary resolved a settings file");
        assert!(load().is_empty(), "a test binary read somebody's settings");
        assert_eq!(get("driver"), None);

        // ...and a write is a no-op rather than a file somewhere.
        set("driver", "plain");
        assert_eq!(get("driver"), None, "a test binary wrote a settings file");
    }

    #[test]
    fn settings_are_kept_beside_the_saves_and_not_among_them() {
        // A file named like a slot would be listed as a tower by `save::saves`.
        assert!(
            !FILE.contains("save"),
            "{FILE} would be read as a tower by the listing",
        );
    }
}
