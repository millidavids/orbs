//! What the player has chosen about the orb itself, kept between sessions.
//!
//! # Not in the save, and that is the whole distinction
//!
//! A save is a *tower* — its clock, its stores, what the wizard has learned. A
//! setting is about the machine in front of you: which driver reads your lines,
//! and later which backend and which colours. Putting one in the other would
//! mean carrying a preference between slots, and loading somebody else's tower
//! would change how your keyboard behaves.
//!
//! So this is a second file beside the saves, per-machine, and **nothing in
//! `orbs-sim` can see it**. `Sim` takes a reader or does not; how that was
//! decided is the shell's business and never the world's.
//!
//! # One key so far, and a file anyway
//!
//! A single `driver = "augury"` line does not need TOML. It gets it because the
//! roadmap's open `Options, remapping, all toggles` is the rest of this file,
//! and a bespoke one-line format is a thing to migrate later — the same argument
//! §13 makes for the save.
//!
//! # Best-effort, in both directions
//!
//! A read that fails is the default, and a write that fails is dropped. Neither
//! is worth a dead end: `ORBS_SAVE=off` sessions, read-only checkouts and the
//! dump all run with nowhere to keep this, and a game that refused to let you
//! change how it reads a line because it could not remember the answer would be
//! worse than one that simply forgets.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::menu::Driver;

/// What the settings file holds.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
struct Settings {
    /// How the orb reads a typed line.
    #[serde(default)]
    driver: Kept,
}

/// [`Driver`] as it is written down.
///
/// **A separate type, deliberately.** `Driver` is a menu word; this is a file
/// format, and §19 records what it costs when one type is asked to be both — a
/// renamed variant becomes an unreadable file. The mapping is one `match` and it
/// is the only place the two meet.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Kept {
    /// The trained reader works out what you meant.
    #[default]
    Augury,
    /// Only the words the orb already knows.
    Plain,
}

impl From<Driver> for Kept {
    fn from(driver: Driver) -> Self {
        match driver {
            Driver::Augury => Self::Augury,
            Driver::Plain => Self::Plain,
        }
    }
}

impl From<Kept> for Driver {
    fn from(kept: Kept) -> Self {
        match kept {
            Kept::Augury => Self::Augury,
            Kept::Plain => Self::Plain,
        }
    }
}

/// Where the settings live: beside the saves, not among them.
///
/// [`None`] when this session keeps nothing at all, which is `ORBS_SAVE=off` and
/// is how every dump runs.
fn path() -> Option<PathBuf> {
    let save = crate::save::path()?;
    Some(save.with_file_name("orbs-settings.toml"))
}

/// Everything on file, or the defaults.
fn load() -> Settings {
    let Some(path) = path() else {
        return Settings::default();
    };
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| toml::from_str(&text).ok())
        .unwrap_or_default()
}

/// Which driver the player last chose.
#[must_use]
pub fn driver() -> Driver {
    load().driver.into()
}

/// Remember this driver for next time.
///
/// **Best-effort**, for the reason the module doc gives: a session with nowhere
/// to write still gets to change how its lines are read.
pub fn set_driver(driver: Driver) {
    let Some(path) = path() else {
        return;
    };
    let settings = Settings {
        driver: driver.into(),
    };
    if let Ok(text) = toml::to_string_pretty(&settings) {
        // A failed write is dropped rather than said: the choice has already
        // taken effect for this session, and there is no screen here to say it
        // on.
        let _ = std::fs::write(path, text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_driver_survives_the_round_trip_through_the_file_format() {
        // The one place `Driver` and `Kept` meet. A renamed variant on either
        // side would make a written file unreadable, which is the defect the
        // two types exist to keep visible.
        for driver in Driver::ALL {
            assert_eq!(Driver::from(Kept::from(driver)), driver);
        }
    }

    #[test]
    fn the_written_form_is_the_word_a_player_typed() {
        // Not required by anything, and worth holding: a settings file somebody
        // opens in an editor should say `augury`, which is what they chose, and
        // not a variant name.
        for driver in Driver::ALL {
            let text = toml::to_string_pretty(&Settings {
                driver: driver.into(),
            })
            .expect("settings serialise");
            assert!(
                text.contains(driver.word()),
                "{text:?} does not name {}",
                driver.word(),
            );
        }
    }

    #[test]
    fn an_unreadable_file_is_the_default_rather_than_a_failure() {
        // `load` is called on the way into a menu page; a corrupt file must not
        // be able to stop a player reaching it.
        let broken: Result<Settings, _> = toml::from_str("driver = \"nonsense\"");
        assert!(broken.is_err(), "the format got looser");
        assert_eq!(Settings::default().driver, Kept::Augury);
    }

    #[test]
    fn settings_are_kept_beside_the_saves_and_not_among_them() {
        // A file named like a slot would be listed as a tower by `save::saves`.
        let Some(path) = path() else {
            return;
        };
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        assert!(!name.contains("save"), "{name} would be read as a tower");
    }
}
