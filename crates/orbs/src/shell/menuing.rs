//! The orb's menu in the Bevy shell: opening it, feeding it keys.
//!
//! [`Menu`] is the screen and knows nothing about Bevy. This is the wiring, and
//! it is a separate file for the reason `weaving.rs` is.
//!
//! `menu` opens it; `quit` leaves, asking once. `quit` did both for one
//! iteration — superseded (§19).
//!
//! [`type_into_menu`] runs whenever a key arrives, not only while the menu is
//! up, and is ordered before [`open_requested`]: a gated reader keeps its
//! cursor, so the first time this ran it typed the word that opened the menu
//! back into it and `quit` left the orb. It runs, declines and *clears*.

use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

use orbs_shell::{Menu, MenuOutcome};

/// The menu, if it is up.
///
/// A resource rather than a component, for the reason `Loom` gives: there is one
/// screen, it is modal, and an entity would invite a second.
#[derive(Resource, Debug, Default)]
pub(crate) struct Standing(Option<Menu>);

impl Standing {
    /// The open menu.
    pub(crate) const fn get(&self) -> Option<&Menu> {
        self.0.as_ref()
    }

    /// The open menu, to type into.
    pub(crate) const fn get_mut(&mut self) -> Option<&mut Menu> {
        self.0.as_mut()
    }

    /// Whether the menu has the keyboard.
    #[must_use]
    pub(crate) const fn is_open(&self) -> bool {
        self.0.is_some()
    }

    /// Put it up, replacing whatever was there.
    ///
    /// `stance` is whether there is a tower behind it, which decides whether the
    /// menu can be closed at all. `driver` is what is in effect right now, held
    /// by the frontend rather than the settings file, so the page can mark it.
    pub(crate) fn open(
        &mut self,
        stance: orbs_shell::Stance,
        driver: orbs_shell::Driver,
        rows: Vec<orbs_shell::settings::Row>,
    ) {
        let mut menu = Menu::at(stance);
        menu.show_driver(driver);
        // What is in effect, asked of the things that hold it, never read back
        // from the settings file: `ORBS_SAVE=off` has no file and still has a
        // tube, and `ORBS_CRT=off` outranks it.
        menu.show_settings(rows);
        self.0 = Some(menu);
    }

    /// Take it down.
    pub(crate) fn close(&mut self) {
        self.0 = None;
    }
}

// There is no `menuing` run condition, and its absence is the fix (see the
// module doc). One here would invite gating `type_into_menu` on it again.

/// Open the menu when `menu` has asked for it.
///
/// Peeked before it is taken, as the other five handshakes are: reaching through
/// `ResMut` stamps `Tower`'s change tick and re-arms this system's own
/// `resource_changed::<Tower>` for ever.
pub(crate) fn open_requested(
    mut tower: ResMut<crate::sim::Tower>,
    mut standing: ResMut<Standing>,
    settable: super::setting::Settable,
) {
    if !tower.has_menuing() {
        return;
    }
    if !tower.menuing() {
        return;
    }
    // The `menu` verb is typed at a prompt, so there is always a tower behind
    // this one. The threshold's menu is opened by `thresholding` instead.
    standing.open(
        orbs_shell::Stance::InTower,
        settable.driver(),
        settable.rows(),
    );
}

/// Which tower the menu asked for.
///
/// An enum rather than two optional fields: `Option<PathBuf>` beside
/// `Option<Length>` spells four states for three meanings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Raising {
    /// Open the tower already in this file.
    ///
    /// A path, not a slot number: the game being left is written to the file it
    /// came from, and a number resolved twice disagrees if `ORBS_SAVE` changes.
    Load(std::path::PathBuf),
    /// Begin a tower at this length.
    Begin {
        /// Where it will be kept, and written back to — or [`None`] for a
        /// session that keeps nothing (`ORBS_SAVE=off`), which is the ordinary
        /// case for every instrument here.
        keep: Option<std::path::PathBuf>,
        /// How long it is to be.
        length: orbs_sim::content::Length,
    },
}

/// A setting the menu changed, waiting to be honoured.
///
/// A message because the tube is a component on the camera, so applying one
/// needs the `&mut World` that [`apply_setting`] has.
#[derive(Message, Debug, Clone)]
pub(crate) struct SettingMessage {
    /// Its key in the settings file.
    pub(crate) setting: String,
    /// What it is now, as a word.
    pub(crate) value: String,
}

/// A tower the menu asked for, waiting for the swap.
///
/// A message rather than the key handler doing it: a different `Sim` means
/// resetting every shell resource, which needs [`swap`]'s `&mut World`.
#[derive(Message, Debug, Clone)]
pub(crate) struct SwapMessage {
    /// Which tower, and where it is kept.
    pub(crate) raising: Raising,
    /// The seed a new tower is built from — asked for, or found in an empty slot.
    ///
    /// Carried here rather than chosen in `swap`, so a test can name one.
    pub(crate) seed: u64,
}

/// Feed keys to the open menu.
pub(crate) fn type_into_menu(
    mut keys: MessageReader<KeyboardInput>,
    held: Res<ButtonInput<KeyCode>>,
    quiet: Res<super::input::Quiet>,
    mut standing: ResMut<Standing>,
    mut exit: MessageWriter<AppExit>,
    mut swapping: MessageWriter<SwapMessage>,
    mut setting_changed: MessageWriter<SettingMessage>,
    mut opening: MessageWriter<super::manualling::OpenManualMessage>,
    // `Option`, for `commanding::submit`'s reason: half the tests here build the
    // shell alone, and a bare `Res` fails parameter validation in one.
    readers: Option<ResMut<crate::sim::Readers>>,
    mut reading: Option<ResMut<super::manualling::Reading>>,
) {
    // The manual sits over the menu and owns the keyboard: without the guard,
    // `q` at a chapter prefix-matched the menu's `quit` and left the game.
    //
    // Open *or* it answered this frame and then shut — `type_into_manual` runs
    // first, so `is_open` alone missed the Escape that leaves the manual and the
    // menu closed too. `Reading::took_the_keys` carries that across the gap.
    //
    // Cleared, not skipped, which is this system's whole shape — see below.
    let manual_answered = reading
        .as_mut()
        .is_some_and(|reading| reading.is_open() || reading.took_the_keys());
    if manual_answered {
        keys.clear();
        return;
    }
    // A held chord is skipped; a *stale* one is not — `type_into_loom` records
    // the keystroke swallowed by reading this the other way round.
    let stale_chord = super::input::chord_is_stale(quiet.gap());
    let chord = held.any_pressed([
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
    ]);

    let mut outcome = None;
    {
        // Cleared, not skipped: running always keeps the cursor from falling
        // behind, and ordering before `open_requested` empties the first frame.
        let Some(menu) = standing.get_mut() else {
            keys.clear();
            return;
        };
        for event in keys.read() {
            if event.state != ButtonState::Pressed {
                continue;
            }
            if chord && !stale_chord {
                continue;
            }
            let Some(key) = super::input::pressed(event) else {
                continue;
            };
            outcome = orbs_shell::apply_to_menu(&key, menu).or(outcome);
        }
    }
    match outcome {
        Some(MenuOutcome::Close) => standing.close(),
        // The only place in the Bevy build where a word ends the session;
        // `persist::keep_on_the_way_out` reads this `AppExit` and saves.
        Some(MenuOutcome::PutDown) => {
            exit.write(AppExit::Success);
        }
        // Not done here. A swap needs `&mut World`; `swap` below has it.
        Some(MenuOutcome::Load(path)) => {
            swapping.write(SwapMessage {
                raising: Raising::Load(path),
                seed: orbs_shell::new_game_seed(),
            });
        }
        Some(MenuOutcome::Begin { path, length }) => {
            swapping.write(SwapMessage {
                raising: Raising::Begin { keep: path, length },
                seed: orbs_shell::new_game_seed(),
            });
        }
        // Honoured now, not at the next launch: a player who turns the tube off
        // and cannot see it go off thinks nothing happened. Through the same
        // exclusive door the swap uses, because the tube is a camera component.
        Some(MenuOutcome::Set { setting, value }) => {
            setting_changed.write(SettingMessage { setting, value });
        }
        // Raised, not done here: the book is assembled from the `Sim` and this
        // system holds none, which is what lets the menu survive a swap.
        Some(MenuOutcome::OpenManual) => {
            opening.write(super::manualling::OpenManualMessage);
        }
        // Takes effect on the next line typed, not on the next launch, which is
        // the whole reason the outcome exists.
        Some(MenuOutcome::Drive(driver)) => {
            // Both registers, because both hang off the one driver. A kept
            // reading belongs to the reader that made it (`tower::Read::by`).
            if let Some(mut readers) = readers {
                readers.drive(driver);
            }
        }
        None => {}
    }
}

/// Honour the settings the menu has just changed.
///
/// `&mut World` because the tube is a camera component and the rest are
/// resources — the one signature that reaches both.
pub(crate) fn apply_setting(world: &mut World) {
    let Some(changed) = world
        .get_resource_mut::<Messages<SettingMessage>>()
        .map(|mut asked| asked.drain().collect::<Vec<_>>())
    else {
        return;
    };
    for change in changed {
        super::setting::apply(world, &change.setting, &change.value);
    }
}

/// Put a different tower in front of the player.
///
/// The order is the whole of it:
///
/// 1. The game being left is written first, to the path it came from, or the
///    autosave writes the outgoing tower into the incoming one's file (`Kept`).
/// 2. The incoming tower is read before anything is torn down, so a save that
///    will not open leaves the player where they were.
/// 3. Every shell resource goes back to its default — see
///    [`reset_for_swap`](super::plugin::reset_for_swap). `Reveal` holds indices
///    into the old record stream, `Passing` a snapshot of the old screen.
/// 4. `Tower` and `Kept` are replaced together.
/// 5. `ORBS_CONTENT`'s prose is re-applied: a new world holds the built-in text
///    and the watcher only refreshes on an edit.
/// 6. The menu closes itself, so a failure above leaves it open with the reason.
/// 7. The threshold is crossed last, and only here.
///
/// Ordered before `ShellSystems::Drive` in `plugin.rs` — `Panel` self-heals from
/// the new world only if it is refreshed after this has run.
pub(crate) fn swap(world: &mut World) {
    // The last one, and the queue is drained either way: a leftover message
    // would swap again next frame and put a stale tower on top.
    let Some(asked) = world
        .get_resource_mut::<Messages<SwapMessage>>()
        .and_then(|mut asked| asked.drain().last())
    else {
        return;
    };

    // 2. Read first, so a refusal costs nothing.
    let waiting = match &asked.raising {
        // A new tower: nothing to read, and the file is written on the first
        // autosave. `free_slot` already found it empty.
        Raising::Begin { .. } => orbs_shell::Opened::New,
        Raising::Load(path) => orbs_shell::read_save_from(path),
    };
    if let (orbs_shell::Opened::Unreadable, Raising::Load(path)) = (&waiting, &asked.raising) {
        // Left where they were. `read_from` has set the save aside; arriving at
        // a fresh tower nobody asked for would be the worse answer.
        bevy::log::error!("the tower in {} could not be read", path.display());
        return;
    }

    // 1. ...but the outgoing tower is written before anything is replaced.
    // `Kept` is `SimPlugin`'s, and a build without it keeps nothing.
    if let Some(kept) = world.get_resource::<crate::sim::Kept>().cloned() {
        crate::sim::keep_now(world.resource::<crate::sim::Tower>(), &kept);
    }

    let seed = asked.seed;
    let wizard = orbs_shell::wizard();
    let (mut tower, kept) = match asked.raising {
        Raising::Begin { keep, length } => (
            crate::sim::Tower::begun(seed, length, wizard.as_deref()),
            keep,
        ),
        Raising::Load(path) => (
            crate::sim::raise(waiting, seed, wizard.as_deref()),
            Some(path),
        ),
    };
    // 5. A new world holds the built-in prose; the watcher only fires on an edit.
    crate::sim::apply_content(&mut tower);

    // 3.
    super::plugin::reset_for_swap(world);
    // 4. Together, and nothing between them can observe one without the other.
    world.insert_resource(tower);
    world.insert_resource(crate::sim::Kept::at(kept));
    // 6.
    world.resource_mut::<Standing>().close();
    // 7. Not in `shell_resources!`: `reset_for_swap` blanks that list, and a
    //    `Threshold` reset would put the player back out of the chosen tower.
    world.insert_resource(orbs_shell::Threshold::Playing);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::{Kept, Tower};

    #[test]
    fn the_menu_opens_shut_and_reports_who_has_the_keyboard() {
        let mut standing = Standing::default();
        assert!(!standing.is_open());
        assert!(standing.get().is_none());

        standing.open(
            orbs_shell::Stance::InTower,
            orbs_shell::Driver::default(),
            Vec::new(),
        );
        assert!(standing.is_open(), "the menu did not take the keyboard");

        standing.close();
        assert!(!standing.is_open(), "the menu kept the keyboard");
    }

    /// A shell with the swap system in it and a tower in front of the player.
    fn app(seed: u64, kept: std::path::PathBuf) -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            bevy::input::InputPlugin,
            bevy::window::WindowPlugin::default(),
            crate::shell::ShellPlugin,
        ))
        .insert_resource(Tower::new(seed))
        .insert_resource(Kept::at(Some(kept)));
        app.update();
        app
    }

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("orbs-swap-{name}-{}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).expect("a temp directory");
        dir
    }

    #[test]
    fn loading_a_second_tower_writes_the_first_to_its_own_file() {
        // Before `Kept`, the path was resolved at the moment of writing, so the
        // tower being left went into the file of the one being opened.
        let dir = scratch("two-towers");
        let one = dir.join("orbs-save.toml");
        let two = dir.join("orbs-save-2.toml");

        // A second tower already on disk, recognisably not the first.
        orbs_shell::write_save_to(&two, &orbs_sim::Sim::new(909).snapshot()).expect("slot two");

        let mut app = app(101, one.clone());
        app.world_mut().write_message(SwapMessage {
            raising: Raising::Load(two.clone()),
            seed: 1,
        });
        app.update();

        // The second tower is what the player is looking at...
        assert_eq!(
            app.world().resource::<Tower>().sim().snapshot().world.seed,
            909,
            "the menu did not put the chosen tower in front of the player",
        );
        // ...it is what the autosave will write to from now on...
        assert_eq!(app.world().resource::<Kept>().path(), Some(two.as_path()));
        // ...and the first is still in its own file, unharmed.
        let orbs_shell::Opened::Restored(left) = orbs_shell::read_save_from(&one) else {
            panic!("the tower that was left did not come back");
        };
        assert_eq!(
            left.world.seed, 101,
            "the tower being left was written into the file of the one being opened",
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_swap_puts_every_derived_surface_back() {
        // `Reveal` holds indices into the old record stream and `Loom` a screen
        // built from a world that is gone. Neither crashes; both mislead.
        let dir = scratch("reset");
        let one = dir.join("orbs-save.toml");
        let two = dir.join("orbs-save-2.toml");
        orbs_shell::write_save_to(&two, &orbs_sim::Sim::new(4).snapshot()).expect("slot two");

        let mut app = app(5, one);
        app.world_mut()
            .resource_mut::<crate::shell::Loom>()
            .open(orbs_shell::Tapestry::default());
        app.world_mut().resource_mut::<Standing>().open(
            orbs_shell::Stance::InTower,
            orbs_shell::Driver::default(),
            Vec::new(),
        );
        assert!(app.world().resource::<crate::shell::Loom>().is_open());

        app.world_mut().write_message(SwapMessage {
            raising: Raising::Load(two),
            seed: 1,
        });
        app.update();

        assert!(
            !app.world().resource::<crate::shell::Loom>().is_open(),
            "the weave screen survived into a world it was not built from",
        );
        assert!(
            !app.world().resource::<Standing>().is_open(),
            "the menu did not close itself after the swap",
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_new_game_is_begun_at_the_length_that_was_chosen() {
        let dir = scratch("begun");
        let one = dir.join("orbs-save.toml");
        let two = dir.join("orbs-save-2.toml");

        let mut app = app(11, one.clone());
        app.world_mut().write_message(SwapMessage {
            raising: Raising::Begin {
                keep: Some(two),
                length: orbs_sim::content::Length::Long,
            },
            seed: 482_913,
        });
        app.update();

        let snapshot = app.world().resource::<Tower>().sim().snapshot();
        assert_eq!(
            snapshot.world.length,
            orbs_sim::content::Length::Long,
            "the new game was not begun at the length the player chose",
        );
        // A world of its own, built from the seed the menu chose rather than the
        // tower it replaced.
        assert_eq!(
            snapshot.world.seed, 482_913,
            "the new game was not built from the seed it was asked for",
        );
        // ...and the tower it replaced was still written out first.
        assert!(one.exists(), "the tower being left was not kept");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_tower_that_will_not_open_leaves_the_player_where_they_were() {
        // Read before anything is torn down: arriving at a fresh tower, having
        // thrown away the one they had, is the worse answer.
        let dir = scratch("unreadable");
        let one = dir.join("orbs-save.toml");
        let two = dir.join("orbs-save-2.toml");
        std::fs::write(&two, "this is not a save").expect("a file");

        let mut app = app(77, one);
        app.world_mut().resource_mut::<Standing>().open(
            orbs_shell::Stance::InTower,
            orbs_shell::Driver::default(),
            Vec::new(),
        );
        app.world_mut().write_message(SwapMessage {
            raising: Raising::Load(two),
            seed: 1,
        });
        app.update();

        assert_eq!(
            app.world().resource::<Tower>().sim().snapshot().world.seed,
            77,
            "an unreadable save took the player's tower away",
        );
        assert!(
            app.world().resource::<Standing>().is_open(),
            "the menu closed over a swap that did not happen",
        );

        std::fs::remove_dir_all(&dir).ok();
    }
}
