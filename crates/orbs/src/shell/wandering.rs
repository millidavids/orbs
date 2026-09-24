//! `wander` — the arrow keys walking the archive's stacks (§10, §19).
//!
//! The smallest surface that can own the keyboard: the prompt, the editor, the
//! unfurled transcript and the weave screen each own a pane, and this owns only
//! the keys. The map draws whenever a maze is open, so walking one by hand and
//! watching a spell walk it are the same picture.
//!
//! So `weaving`'s asymmetry applies unchanged — gated by a run condition because
//! it consumes keys, where `type_into_line` has to *run* to throw away what it
//! declines.
//!
//! An arrow moves the reading the instant it is pressed. Two versions went
//! through the prompt's queue and both were wrong: per keystroke walked at the
//! speed of the keyboard (`Pending` drains whole at tick start, so a held arrow
//! was thirty cells), and one aim plus a bounded burst walked at the speed of the
//! world, which at 1 Hz is a wait rather than a minigame. The tick was the
//! problem, not the queue — so an arrow calls
//! [`Sim::walk`](orbs_sim::Sim::walk) directly, moving the reading and costing
//! no tick. Nothing is left to bound; a wall still refuses for free.

use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
#[cfg(test)]
use orbs_sim::tower::Way;

use crate::sim::Tower;

/// Whether the arrows have the stacks.
///
/// A resource rather than a component, for the reason `Editing` and `Loom` both
/// give: there is one maze, the mode is modal, and an entity would invite a
/// second.
///
/// It holds no steps. Two earlier versions kept an aim and then a queue, which
/// is what a move waiting on a clock needs. A move no longer waits.
#[derive(Resource, Debug, Default)]
pub(crate) struct Walk(bool);

impl Walk {
    /// Whether the arrows have the maze.
    pub(crate) const fn is_open(&self) -> bool {
        self.0
    }

    /// Take the keys.
    pub(crate) const fn open(&mut self) {
        self.0 = true;
    }

    /// Give them back.
    pub(crate) const fn close(&mut self) {
        self.0 = false;
    }
}

/// Whether the arrows have the maze.
pub(crate) fn walking(walk: Res<Walk>) -> bool {
    walk.is_open()
}

/// Take the keys when `wander` asks for them.
pub(crate) fn open_requested(mut tower: ResMut<Tower>, mut walk: ResMut<Walk>) {
    // Peeked before it is taken, as the editor and the weave screen do:
    // `wandering` needs `&mut`, which stamps `Tower`'s change tick and would
    // re-arm this system's own run condition every frame, dragging the panel and
    // the suggestions back to 60 Hz.
    if !tower.has_wandering() {
        return;
    }
    if !tower.wandering() {
        return;
    }
    walk.open();
}

/// Feed keys to the maze.
pub(crate) fn type_into_maze(
    mut keys: MessageReader<KeyboardInput>,
    held: Res<ButtonInput<KeyCode>>,
    quiet: Res<super::input::Quiet>,
    mut walk: ResMut<Walk>,
    mut tower: ResMut<Tower>,
) {
    // A held chord is skipped; a stale one is not. `chord_is_stale` says the
    // modifier is a ghost — latched from an alt-tab the window never saw
    // released — so it means *accept this keystroke*. Read the other way it
    // swallows one key every time the mode is entered after a pause, which is
    // exactly when it is entered. The weave screen shipped with it inverted.
    let stale_chord = super::input::chord_is_stale(quiet.gap());
    let chord = held.any_pressed([
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
    ]);

    for event in keys.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }
        // A chord the player aimed at their operating system must not reach the
        // maze on the way past.
        if chord && !stale_chord {
            continue;
        }
        // `break`, not `continue`: a frame can carry several keystrokes, and
        // going on through the batch walked the reading after the player had
        // left the mode.
        if matches!(&event.logical_key, Key::Escape) {
            walk.close();
            break;
        }
        // The arrow-to-`Way` table is `orbs-shell`'s; it was written here, in
        // `orbs-tui`, and a third time in `dump.rs`.
        //
        // Everything else is swallowed, not passed on: a surface that owns the
        // keyboard owns all of it, and letting text through would type into a
        // prompt with no visible caret.
        let Some(way) = super::input::pressed(event)
            .as_ref()
            .and_then(orbs_shell::apply_to_maze)
        else {
            continue;
        };
        // Straight into the world, on this frame: no message, no queue, no tick
        // (see the module header). The `&mut` stamps `Tower`, which is what has
        // `refresh_panel` redraw the map next frame.
        if !tower.walk(way) {
            // The maze went while the keys were held — solved by the step that
            // took it, or closed by a spell. `close_when_gone` says so too, but
            // it runs after this and a whole key repeat could land first.
            walk.close();
        }
    }
}

/// Let go when the stacks are no longer open to walk.
///
/// Three ways a maze ends, one condition: solved, abandoned by `stop lectern`
/// (which a bound spell may issue), or the player left the archive. Naming this
/// after the solved case leaves the other two owning the keyboard over a pane
/// with no map on it.
pub(crate) fn close_when_gone(tower: Res<Tower>, mut walk: ResMut<Walk>) {
    if tower.sim().stacks().is_none() {
        walk.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The systems, without a renderer.
    ///
    /// Deliberately not `ShellPlugin`: that wants a GPU, and a test needing one
    /// is a test nobody runs.
    fn app() -> App {
        let mut app = App::new();
        app.init_resource::<Walk>()
            .insert_resource(Tower::new(1))
            .add_systems(Update, (open_requested, close_when_gone).chain());
        app
    }

    /// Submit a line and step the world.
    fn run(app: &mut App, line: &str) {
        app.world_mut().resource_mut::<Tower>().submit(line);
        app.world_mut().resource_mut::<Tower>().step();
        app.update();
    }

    /// Where the reading stands.
    fn head(app: &App) -> usize {
        app.world()
            .resource::<Tower>()
            .sim()
            .stacks()
            .expect("the stacks are shut")
            .at
    }

    /// A way that is actually open from where the reading stands.
    fn a_way_out(app: &App) -> Way {
        let maze = app
            .world()
            .resource::<Tower>()
            .sim()
            .stacks()
            .expect("the stacks are shut");
        Way::ALL
            .into_iter()
            .enumerate()
            .find_map(|(index, way)| maze.open(index).then_some(way))
            .expect("a square with no way out of it")
    }

    #[test]
    fn the_verb_takes_the_keys_once() {
        let mut app = app();
        run(&mut app, "attend archive");
        run(&mut app, "research");
        assert!(!app.world().resource::<Walk>().is_open());

        run(&mut app, "wander");
        assert!(app.world().resource::<Walk>().is_open());

        // ...and a stale request does not take them back after Escape.
        app.world_mut().resource_mut::<Walk>().close();
        app.update();
        assert!(
            !app.world().resource::<Walk>().is_open(),
            "the arrows seized the keyboard again on the frame after it was let go",
        );
    }

    #[test]
    fn a_press_moves_the_reading_on_the_frame_it_lands() {
        // The whole point of the third entry point: two earlier versions put the
        // step through the prompt's queue, so a press waited a second. No tick
        // is stepped anywhere in this test.
        let mut app = app();
        run(&mut app, "attend archive");
        run(&mut app, "research");
        run(&mut app, "wander");

        let before = head(&app);
        let way = a_way_out(&app);
        assert!(app.world_mut().resource_mut::<Tower>().walk(way));

        let after = head(&app);
        assert_ne!(
            after, before,
            "the reading waited for a tick that never came"
        );
    }

    #[test]
    fn pressing_as_fast_as_you_like_walks_as_fast_as_you_press() {
        // Ten presses with no tick between them are ten steps, which is what
        // "as fast as I can press" has to mean. Under the queue this was six a
        // second and under the aim before it, one.
        let mut app = app();
        run(&mut app, "attend archive");
        run(&mut app, "research");
        run(&mut app, "wander");

        let mut seen = 1;
        for _ in 0..10 {
            let way = a_way_out(&app);
            app.world_mut().resource_mut::<Tower>().walk(way);
            let (walked, _) = app
                .world()
                .resource::<Tower>()
                .sim()
                .stacks()
                .map_or((seen, 0), |maze| maze.explored());
            seen = seen.max(walked);
        }
        assert!(
            seen > 1,
            "ten presses with no tick between them walked nowhere",
        );
    }

    #[test]
    fn walking_out_of_the_archive_gives_the_keys_back() {
        // Unreachable by typing — the prompt is dead while this is open — but a
        // bound spell can `stop stacks`, and `Sim::stacks` is `None` outside
        // the archive too. One condition, three ways in.
        //
        // `stop stacks`, and it was `stop lectern`: the maze moved to its own
        // instrument, so stopping the lectern abandons an assembly and leaves
        // the stacks alone.
        let mut app = app();
        run(&mut app, "attend archive");
        run(&mut app, "research");
        run(&mut app, "wander");
        assert!(app.world().resource::<Walk>().is_open());

        run(&mut app, "stop stacks");
        assert!(
            !app.world().resource::<Walk>().is_open(),
            "the arrows held stacks that had closed",
        );
    }

    #[test]
    fn a_press_with_no_stacks_left_gives_the_keys_back_at_once() {
        // `close_when_gone` also says so, but it runs *after* the key handler
        // and a whole key repeat can land inside one frame. The handler has to
        // notice for itself or the frame after a solve walks into nothing.
        let mut app = app();
        run(&mut app, "attend archive");
        run(&mut app, "research");
        run(&mut app, "wander");

        app.world_mut()
            .resource_mut::<Tower>()
            .submit("stop stacks");
        app.world_mut().resource_mut::<Tower>().step();
        assert!(!app.world_mut().resource_mut::<Tower>().walk(Way::East));
    }
}
