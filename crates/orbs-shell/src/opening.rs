//! A tower out of what reading a save found — one copy for both builds
//! (DESIGN.md §19, *A swap replaces the tower and nothing else*).

use std::path::{Path, PathBuf};

use orbs_sim::Sim;

use crate::install::{Begun, Raise};
use crate::save::{Opened, away_for, read_from};

/// Which tower a swap raises.
///
/// An enum rather than two optional fields: a path beside a length spells four
/// states for three meanings, and the fourth — a load with no file — would be a
/// dead branch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Raising {
    /// Open the tower already in this file.
    ///
    /// A path, not a slot number: the game being left is written to the file it
    /// came from, and a number resolved twice disagrees if `ORBS_SAVE` changes.
    Load(std::path::PathBuf),
    /// Begin a tower at this length — installed first, when the frontend's
    /// setting says so.
    Begin {
        /// Where it will be kept, and written back to — or [`None`] for a
        /// session that keeps nothing (`ORBS_SAVE=off`), which is how every
        /// played scenario and instrument runs.
        keep: Option<crate::save::Keep>,
        /// How long it is to be.
        length: orbs_sim::content::Length,
    },
    /// The installation has woken this tower: raise it, never install it.
    Installed(Raise),
}

/// What a swap needs from the orb it happens in.
#[derive(Debug)]
pub struct Swapping<'a> {
    /// Where the tower being left is kept.
    pub kept: Option<&'a Path>,
    /// Whether writing it there just now worked. A load of that same slot after
    /// a failed write is refused unread: it would put an older autosave over
    /// the live tower.
    pub written: bool,
    /// A new tower's seed.
    pub seed: u64,
    /// A new tower's wizard: the name held, not the file's, so a name chosen
    /// where nothing can be written still names the tower.
    pub wizard: Option<String>,
    /// Whether a new tower is installed first.
    pub installs: bool,
}

/// A tower a swap raised, and the live content files that would not load into
/// it (see [`load`](crate::load)).
pub struct Raised {
    /// The tower, its slot and any installation.
    pub begun: Begun,
    /// What [`load`](crate::load) could not read.
    pub missed: Vec<&'static str>,
}

impl Raising {
    /// The tower this asks for, with `ORBS_CONTENT` poured in. One copy for
    /// both builds; each tears down its own.
    ///
    /// # Errors
    ///
    /// The path and why, when a load gives no tower. The player stays where
    /// they were.
    pub fn raise(self, at: Swapping<'_>) -> Result<Raised, (PathBuf, Refused)> {
        let mut begun = match self {
            Self::Begin { keep, length } => Raise {
                keep,
                length,
                seed: at.seed,
                wizard: at.wizard,
            }
            .begin(at.installs),
            Self::Installed(raise) => raise.wake(),
            Self::Load(path) => match loaded(&path, at.kept, at.written) {
                Ok(sim) => Begun {
                    sim,
                    install: None,
                    keep: Some(path),
                },
                Err(why) => {
                    tracing::error!("the tower in {} was not opened: {why:?}", path.display());
                    return Err((path, why));
                }
            },
        };
        // A new world holds the built-in text, and a swap is the only way to a
        // tower, so without this a writer's edits vanish at the menu.
        let missed = crate::prose::load(&mut begun.sim);
        Ok(Raised { begun, missed })
    }
}

/// Why a load from the menu left the player where they were.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refused {
    /// The file is there and this build cannot read it. It has been set aside.
    Unreadable,
    /// The file has gone since the play page listed it.
    Gone,
    /// The tower in hand could not be written, and this was its own slot.
    Unwritten,
}

/// The tower a launch starts on: the save's, or a fresh one when there is none
/// or it would not read, which is said.
///
/// Startup only. A load from the menu is [`Raising::raise`], which never raises
/// one.
/// `wizard` names a *new* tower's wizard; a restored tower keeps its own.
#[must_use]
pub fn tower_from(opened: Opened, seed: u64, wizard: Option<&str>) -> Sim {
    match opened {
        Opened::Restored(save) => resumed(&save),
        Opened::Unreadable => {
            let mut fresh = fresh_named(seed, wizard);
            fresh.say_save_unreadable();
            fresh
        }
        Opened::New => fresh_named(seed, wizard),
    }
}

/// The tower in `path`, which a load from the menu asked for, or why the player
/// stays where they were. A load never raises a tower.
///
/// Called after the tower in hand was written to `kept`, `written` saying
/// whether that worked. Loading that same slot after a failed write is refused
/// unread: it would put an older autosave over the live tower.
///
/// # Errors
///
/// [`Refused`], when the file will not give a tower.
fn loaded(path: &Path, kept: Option<&Path>, written: bool) -> Result<Sim, Refused> {
    if !written && kept == Some(path) {
        return Err(Refused::Unwritten);
    }
    match read_from(path) {
        Opened::Restored(save) => Ok(resumed(&save)),
        Opened::Unreadable => Err(Refused::Unreadable),
        Opened::New => Err(Refused::Gone),
    }
}

/// A saved tower, saying how long it was dark. Read here because the sim has no
/// wall clock and §19 forbids it one; nothing accrues while it is away.
fn resumed(save: &orbs_sim::Save) -> Sim {
    let mut resumed = Sim::restored(save);
    resumed.say_resumed(away_for(save));
    resumed
}

/// A fresh tower: sealed unless `ORBS_SEALED=0`, as long as `ORBS_LENGTH` says.
fn fresh_named(seed: u64, wizard: Option<&str>) -> Sim {
    let mut fresh = crate::environment::fresh(seed, true, orbs_sim::content::Length::Medium);
    if let Some(wizard) = wizard {
        fresh.rename(wizard);
    }
    fresh
}

#[cfg(test)]
mod tests {
    use super::*;

    fn said(sim: &Sim, key: &str) -> bool {
        let line = sim.prose().line(key, &[]);
        sim.scrollback().records().iter().any(|record| {
            matches!(
                record.field(orbs_render::FieldName::Message),
                Some(orbs_render::Value::Text(text)) if *text == line
            )
        })
    }

    #[test]
    fn a_new_tower_takes_the_wizard_it_is_given() {
        let sim = tower_from(Opened::New, 7, Some("morgana"));
        assert!(
            sim.prompt().contains("morgana"),
            "the prompt reads {:?}",
            sim.prompt()
        );
    }

    #[test]
    fn an_unreadable_save_says_so_and_a_fresh_tower_starts() {
        let sim = tower_from(Opened::Unreadable, 7, None);
        assert!(
            said(&sim, "save_unreadable"),
            "nothing said the save was set aside"
        );
        assert_eq!(sim.tick().get(), 0);
    }

    #[test]
    fn a_restored_tower_keeps_its_own_name() {
        let mut kept = Sim::new(3);
        kept.rename("merlin");
        let save = Box::new(kept.snapshot());
        let sim = tower_from(Opened::Restored(save), 7, Some("morgana"));
        assert!(
            sim.prompt().contains("merlin"),
            "a save took the new tower's wizard: {:?}",
            sim.prompt(),
        );
    }

    #[test]
    fn a_load_never_raises_a_tower() {
        let dir = std::env::temp_dir().join(format!("orbs-opening-{}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).expect("a temp directory");
        let (one, two) = (dir.join("orbs-save.toml"), dir.join("orbs-save-2.toml"));

        assert_eq!(
            loaded(&two, Some(&one), true).err(),
            Some(Refused::Gone),
            "a vanished file raised a tower"
        );
        std::fs::write(&two, "this is not a save").expect("a file");
        assert_eq!(
            loaded(&two, Some(&one), true).err(),
            Some(Refused::Unreadable),
            "an unreadable file raised a tower"
        );
        crate::save::write_to(&one, &Sim::new(3).snapshot()).expect("a save");
        assert!(loaded(&one, Some(&one), true).is_ok());

        // The write of the tower in hand failed: its own slot holds an older
        // autosave, which must not be read back over it. Any other slot is
        // still a slot.
        assert_eq!(
            loaded(&one, Some(&one), false).err(),
            Some(Refused::Unwritten)
        );
        assert!(loaded(&one, Some(&two), false).is_ok());

        std::fs::remove_dir_all(&dir).ok();
    }
}
