//! The orb's menu in the Bevy shell: opening it, feeding it keys.
//!
//! [`Menu`] is the screen and knows nothing about Bevy. This is the wiring, and
//! it is a separate file for the reason `weaving.rs` is.
//!
//! # `menu` opens it, and `quit` does not
//!
//! `quit` opened this for one iteration, on the argument that the word means
//! *leave the thing you are in*. **Superseded** (§19): leaving the game and
//! stepping out to a screen are two things, and one word for both made stopping
//! a two-step operation for a player who only wanted to stop. `quit` leaves,
//! asking once; `menu` opens this.
//!
//! # The keystrokes that opened it are not typed into it
//!
//! [`type_into_menu`] runs **whenever a key arrives**, not only while the menu
//! is up, and is ordered **before** [`open_requested`]. Both halves are
//! load-bearing and the second was a shipped defect:
//!
//! A gated reader keeps its cursor — `weaving.rs` records that — so the first
//! time this ran it read whatever `Messages` still retained, which on the frame
//! the menu opened was the word that opened it. Typed back in, `quit` reached
//! the menu's own `quit` and left the orb, which looked exactly like `quit`
//! never having stopped ending the session. `focus.rs` states the rule this
//! breaks: *"a surface that grabs the keyboard on open eats the player's first
//! keystroke"*.
//!
//! So this runs, declines and **clears** while the menu is shut, and the ordering
//! guarantees that the clearing happens on the opening frame rather than after
//! it.

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
    /// `driver` is what is in effect right now, so the options page can mark it
    /// — the frontend holds that, not the settings file.
    pub(crate) fn open(&mut self, driver: orbs_shell::Driver) {
        let mut menu = Menu::default();
        menu.show_driver(driver);
        self.0 = Some(menu);
    }

    /// Take it down.
    pub(crate) fn close(&mut self) {
        self.0 = None;
    }
}

// **There is no `menuing` run condition, and its absence is the fix.**
// `type_into_menu` was gated on one, and a gated reader keeps its cursor — so
// the first time it ran it typed the word that opened the menu back into it. It
// runs on a keystroke arriving and declines inside, exactly as `type_into_line`
// does; see the module doc. A run condition here would be an invitation to use
// it, so it is gone rather than left unused.

/// Open the menu when `menu` has asked for it.
///
/// **Peeked before it is taken**, exactly as the other five handshakes are:
/// `menuing` needs `&mut`, and reaching through `ResMut` for it stamps `Tower`'s
/// change tick — which, against this system's own `resource_changed::<Tower>`
/// run condition, re-arms it for ever and drags the panel and the suggestions
/// back to frame rate. `commanding::quit_requested` records that defect
/// happening once already.
pub(crate) fn open_requested(
    mut tower: ResMut<crate::sim::Tower>,
    mut standing: ResMut<Standing>,
    // `Option` for `commanding::submit`'s reason; absent means nothing is
    // reading lines any other way, which is exactly `Driver::default`.
    readers: Option<Res<crate::sim::Readers>>,
) {
    if !tower.has_menuing() {
        return;
    }
    if !tower.menuing() {
        return;
    }
    standing.open(readers.map_or_else(Default::default, |readers| readers.driver()));
}

/// A tower the menu asked for, waiting for the swap.
///
/// **A message rather than the key handler doing it**, because putting a
/// different `Sim` in front of the player needs `&mut World` — every shell
/// resource has to go back to its default, and that is a list the type system
/// cannot hand to an ordinary system's parameters. The exclusive system that
/// reads this is [`swap`].
#[derive(Message, Debug, Clone)]
pub(crate) struct SwapMessage {
    /// Where the incoming tower is kept, and where it will be written back.
    pub(crate) path: std::path::PathBuf,
    /// The length a *new* tower is to be, or `None` to load what is at `path`.
    pub(crate) length: Option<orbs_sim::content::Length>,
}

/// Feed keys to the open menu.
pub(crate) fn type_into_menu(
    mut keys: MessageReader<KeyboardInput>,
    held: Res<ButtonInput<KeyCode>>,
    quiet: Res<super::input::Quiet>,
    mut standing: ResMut<Standing>,
    mut exit: MessageWriter<AppExit>,
    mut swapping: MessageWriter<SwapMessage>,
    // `Option`, for `commanding::submit`'s reason: half the tests here build the
    // shell alone, and a bare `Res` fails parameter validation in one.
    readers: Option<ResMut<crate::sim::Readers>>,
) {
    // **A held chord is skipped; a *stale* one is not** — see `type_into_loom`,
    // whose comment records the swallowed keystroke that comes of reading this
    // the other way round.
    let stale_chord = super::input::chord_is_stale(quiet.gap());
    let chord = held.any_pressed([
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
    ]);

    let mut outcome = None;
    {
        // **Cleared, not skipped, and this is the whole fix.** A reader that
        // does not run keeps its cursor, so a gated version of this system read
        // the word that opened the menu straight back into it — see the module
        // doc. Running always and clearing here means the cursor is never
        // behind, and being ordered before `open_requested` means the frame the
        // menu opens is a frame this has already emptied.
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
        // **The only place in the Bevy build where a *word* ends the session.**
        // `persist::keep_on_the_way_out` reads the `AppExit` this writes, which
        // is how the save happens whichever of the four ways out was taken.
        Some(MenuOutcome::PutDown) => {
            exit.write(AppExit::Success);
        }
        // **Not done here.** A swap needs `&mut World`; `swap` below has it.
        Some(MenuOutcome::Load(path)) => {
            swapping.write(SwapMessage { path, length: None });
        }
        Some(MenuOutcome::Begin { path, length }) => {
            swapping.write(SwapMessage {
                path,
                length: Some(length),
            });
        }
        // **Takes effect on the next line typed, not on the next launch.** The
        // menu has already written the choice down; this is the running session
        // catching up with it, which is the whole reason the outcome exists.
        Some(MenuOutcome::Drive(driver)) => {
            // **Both registers**, because both hang off the one driver — see
            // `Readers`. A spell saved after the switch is read the new way on
            // the next settle beat, every line of it: a kept reading belongs to
            // the reader that made it (`tower::Read::by`), so none survives a
            // change of reader.
            if let Some(mut readers) = readers {
                readers.drive(driver);
            }
        }
        None => {}
    }
}

/// Put a different tower in front of the player.
///
/// # The order is the whole of it
///
/// 1. **The game being left is written first**, to the path it came from, before
///    anything else moves. `Kept`'s doc has the reason at length: the autosave
///    writes to whatever path it is holding, so a swap that replaced the path
///    first would write the outgoing tower into the incoming one's file and the
///    game you left would be gone. This is the exit criterion of the whole
///    feature and it is one line, in this position.
/// 2. **The incoming tower is read before anything is torn down**, so a save
///    that will not open leaves the player where they were rather than in a
///    half-built world.
/// 3. Every shell resource goes back to its default — see
///    [`reset_for_swap`](super::plugin::reset_for_swap). `Reveal` holds raw
///    indices into the *old* record stream; `Passing` holds a cell snapshot of
///    the old screen; `HeldOver` is the guard that stops a keystroke leaking
///    into the prompt, which matters when a menu is left by pressing a key.
/// 4. `Tower` and `Kept` are replaced **together**.
/// 5. `ORBS_CONTENT`'s prose is re-applied, because a new world holds the
///    built-in text and the watcher only refreshes on an edit.
/// 6. The menu closes itself, last, so a failure above leaves it open with the
///    reason on it.
///
/// # Ordered before `ShellSystems::Drive`
///
/// `Panel` self-heals from the new world, but only if it is refreshed after this
/// has run. Registered accordingly in `plugin.rs`.
pub(crate) fn swap(world: &mut World) {
    // **The last one, and the queue is drained either way.** Two swaps in one
    // frame cannot happen — the menu closes on the first — but a leftover
    // message would swap again next frame, and a *stale* swap is a tower
    // arriving on top of the one the player just chose.
    let Some(asked) = world
        .get_resource_mut::<Messages<SwapMessage>>()
        .and_then(|mut asked| asked.drain().last())
    else {
        return;
    };

    // 2. Read first, so a refusal costs nothing.
    let waiting = match asked.length {
        // A new tower: nothing to read, and the file is written on the first
        // autosave. `free_slot` already found it empty.
        Some(_) => orbs_shell::Opened::New,
        None => orbs_shell::read_save_from(&asked.path),
    };
    if matches!(waiting, orbs_shell::Opened::Unreadable) {
        // **Left where they were.** The save has been set aside by `read_from`
        // and the log says so; tearing the world down to arrive at a fresh tower
        // the player did not ask for would be the worse answer.
        bevy::log::error!("the tower in {} could not be read", asked.path.display());
        return;
    }

    // 1. ...but the outgoing tower is written before anything is replaced.
    //
    // `Kept` is `SimPlugin`'s, and a build without it keeps nothing — which is
    // the honest reading, not a case to panic over.
    if let Some(kept) = world.get_resource::<crate::sim::Kept>().cloned() {
        crate::sim::keep_now(world.resource::<crate::sim::Tower>(), &kept);
    }

    let seed = orbs_shell::seed();
    let wizard = orbs_shell::wizard();
    let mut tower = match asked.length {
        Some(length) => crate::sim::Tower::begun(seed, length, wizard.as_deref()),
        None => crate::sim::raise(waiting, seed, wizard.as_deref()),
    };
    // 5. A new world holds the built-in prose; the watcher only fires on an edit.
    crate::sim::apply_content(&mut tower);

    // 3.
    super::plugin::reset_for_swap(world);
    // 4. Together, and nothing between them can observe one without the other.
    world.insert_resource(tower);
    world.insert_resource(crate::sim::Kept::at(Some(asked.path)));
    // 6.
    world.resource_mut::<Standing>().close();
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

        standing.open(orbs_shell::Driver::default());
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
        // **The exit criterion of the whole feature, and the thing autosave
        // would have broken.** Before `Kept`, the path was resolved at the
        // moment of writing — so the tower being *left* would have been written
        // into the file of the tower being *opened*, and the one you left would
        // be gone. The assertion that matters is the last one.
        let dir = scratch("two-towers");
        let one = dir.join("orbs-save.toml");
        let two = dir.join("orbs-save-2.toml");

        // A second tower already on disk, recognisably not the first.
        orbs_shell::write_save_to(&two, &orbs_sim::Sim::new(909).snapshot()).expect("slot two");

        let mut app = app(101, one.clone());
        app.world_mut().write_message(SwapMessage {
            path: two.clone(),
            length: None,
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
        // `Reveal` holds raw indices into the *old* record stream and `Loom`
        // holds a screen built from a world that is gone. Neither crashes when
        // it survives a swap; both quietly describe the wrong tower, which is
        // why this is asserted rather than argued.
        let dir = scratch("reset");
        let one = dir.join("orbs-save.toml");
        let two = dir.join("orbs-save-2.toml");
        orbs_shell::write_save_to(&two, &orbs_sim::Sim::new(4).snapshot()).expect("slot two");

        let mut app = app(5, one);
        app.world_mut()
            .resource_mut::<crate::shell::Loom>()
            .open(orbs_shell::Tapestry::default());
        app.world_mut()
            .resource_mut::<Standing>()
            .open(orbs_shell::Driver::default());
        assert!(app.world().resource::<crate::shell::Loom>().is_open());

        app.world_mut().write_message(SwapMessage {
            path: two,
            length: None,
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
            path: two,
            length: Some(orbs_sim::content::Length::Long),
        });
        app.update();

        let snapshot = app.world().resource::<Tower>().sim().snapshot();
        assert_eq!(
            snapshot.world.length,
            orbs_sim::content::Length::Long,
            "the new game was not begun at the length the player chose",
        );
        // ...and the tower it replaced was still written out first.
        assert!(one.exists(), "the tower being left was not kept");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_tower_that_will_not_open_leaves_the_player_where_they_were() {
        // **Read before anything is torn down.** Arriving at a fresh tower the
        // player did not ask for, having thrown away the one they had, is the
        // worse answer to a save this build cannot read.
        let dir = scratch("unreadable");
        let one = dir.join("orbs-save.toml");
        let two = dir.join("orbs-save-2.toml");
        std::fs::write(&two, "this is not a save").expect("a file");

        let mut app = app(77, one);
        app.world_mut()
            .resource_mut::<Standing>()
            .open(orbs_shell::Driver::default());
        app.world_mut().write_message(SwapMessage {
            path: two,
            length: None,
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
