//! The keys the shell itself asks about, and the typed readers over them.
//!
//! `store` is the file and `values` is the page; this is the handful of
//! questions the *shell* has of its own settings — which driver reads a line,
//! whether the boot sequence is skipped. A frontend's settings are the
//! frontend's: it builds the [`Row`](super::Row)s from what it has and writes
//! the answers back here.
//!
//! Not in `settings/mod.rs`, which CLAUDE.md reserves for `mod` declarations
//! and re-exports.

use crate::menu::Driver;

use super::{get, is_on, set};

/// The key the parse driver is kept under.
///
/// The name the old one-key file already used, so nothing migrates: a
/// `driver = "augury"` line written by `0.13.17` is read by this unchanged.
pub const DRIVER: &str = "driver";

/// The key §4's sticky boot skip is kept under.
///
/// *"Skip is a sticky setting, not a per-launch keypress"*, owed since Phase
/// 0.5 — §19 records the keypress version being removed rather than kept.
pub const SKIP: &str = "skip";

/// The key the installation is kept under: whether a new tower is installed
/// before it wakes. On unless a player turned it off.
pub const INSTALL: &str = "install";

/// The key §9's focus mode is kept under.
///
/// Here rather than in a frontend because both builds bind `F4` and both must
/// write the same key — a second spelling is a setting that half persists, and
/// `Screen::default` reads it for both.
pub const FOCUS: &str = "focus";

/// The key §14's motion switch is kept under.
pub const MOTION: &str = "motion";

/// The key the picture's fit is kept under. Only a window has a fit, but
/// `Screen` is the shell's and reads it.
pub const SCALE: &str = "scale";

/// The key echo verbosity is kept under (§6, §14).
pub const ECHO: &str = "echo";

/// The two words `echo` takes: every echo, or only one that says something
/// the line did not.
pub const ECHOES: [&str; 2] = ["always", "changed"];

/// The key that records §14's health warning has been shown on this machine.
pub const WARNED: &str = "warned";

/// Which driver the player last chose.
#[must_use]
pub fn driver() -> Driver {
    get(DRIVER)
        .and_then(|word| Driver::named(&word))
        .unwrap_or_default()
}

/// Remember this driver for next time.
pub fn set_driver(driver: Driver) {
    set(DRIVER, driver.word());
}

/// Whether the player has asked to skip the boot sequence.
///
/// `ORBS_BOOT=0` still outranks it — ~200 See-it lines depend on that meaning
/// exactly one thing — but a player on their fortieth launch now has an answer
/// that is not an environment variable.
///
/// Read once, at startup: `Boot::default` is the only caller, and the settings
/// *row* comes from a resource seeded from this, because reading the file per
/// paint is a blocking read and a TOML parse every frame.
#[must_use]
pub fn skips_boot() -> bool {
    get(SKIP).is_some_and(|word| is_on(&word))
}

/// Whether a new tower is installed before it wakes, as far as the file says.
///
/// `ORBS_INSTALL` outranks it; see [`install_wanted`](crate::install_wanted).
/// Read once per frontend, at startup, for `skips_boot`'s reason.
#[must_use]
pub fn installs() -> bool {
    get(INSTALL).is_none_or(|word| is_on(&word))
}

/// Whether the player wants motion, as far as the file says.
///
/// On unless turned off, and a word this build does not know reads as off:
/// §14's rule is that a player who asked for stillness gets it, so the quieter
/// reading is the safe one. Read once per frontend, at startup.
#[must_use]
pub fn moves() -> bool {
    get(MOTION).is_none_or(|word| is_on(&word))
}

/// Whether an echo that only repeats the line is left out, as the file says.
///
/// A word this build does not know shows every echo: hiding is the choice
/// that loses something, so it is never the fallback.
#[must_use]
pub fn terse_echo() -> bool {
    get(ECHO).is_some_and(|word| word == ECHOES[1])
}

/// Whether §14's health warning is still owed here, marking it paid if so.
///
/// Once per machine, and only where settings are kept: a dump or a test keeps
/// none, and a warning on every capture would move every threshold capture to
/// say a thing nobody needs twice.
#[must_use]
pub fn warn_once() -> bool {
    if !super::store::is_kept() || get(WARNED).is_some() {
        return false;
    }
    set(WARNED, super::ON);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_driver_survives_the_round_trip_through_the_file_format() {
        // The words *are* the format — never variant names — so this is the one
        // place a rename would show.
        for driver in Driver::ALL {
            assert_eq!(Driver::named(driver.word()), Some(driver));
        }
    }

    #[test]
    fn the_written_form_is_the_word_a_player_typed() {
        // A settings file somebody opens in an editor should say `augury`,
        // which is what they chose, not a variant name.
        for driver in Driver::ALL {
            assert!(
                driver
                    .word()
                    .chars()
                    .all(|glyph| glyph.is_ascii_lowercase()),
                "{:?} is not a word a player types",
                driver.word(),
            );
        }
    }

    #[test]
    fn a_setting_nobody_has_chosen_is_the_default() {
        // Every reader falls back rather than failing, which is what lets a
        // session with nowhere to keep a settings file still have settings.
        for word in ["nonsense", ""] {
            assert_eq!(
                Driver::named(word),
                None,
                "{word:?} named a driver it should not have",
            );
        }
        assert!(!is_on("nonsense"), "a stray word read as on");
    }

    #[test]
    fn the_keys_are_distinct() {
        // Lines in one file: a collision would make one setting silently
        // overwrite another.
        let keys = [DRIVER, SKIP, INSTALL, FOCUS, MOTION, SCALE, ECHO, WARNED];
        for (at, key) in keys.iter().enumerate() {
            assert!(
                !keys[..at].contains(key),
                "`{key}` is the name of two settings",
            );
        }
    }
}
