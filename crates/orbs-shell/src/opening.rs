//! A tower out of what reading a save found — one copy for both builds
//! (DESIGN.md §19, *A swap replaces the tower and nothing else*).

use std::path::Path;

use orbs_sim::Sim;

use crate::save::{Opened, away_for, read_from};

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
/// Startup only. A load from the menu is [`loaded`], which never raises one.
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
pub fn loaded(path: &Path, kept: Option<&Path>, written: bool) -> Result<Sim, Refused> {
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
