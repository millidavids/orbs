//! `wander` — the arrow keys walking the archive's stacks (§10, §19).
//!
//! # The fourth surface that can own the keyboard, and the smallest
//!
//! The prompt, the editor, the unfurled transcript and the weave screen each own
//! a *pane*. This one owns nothing but the keys: the map draws whenever a maze
//! is open, so walking one by hand and watching a spell walk it are the same
//! picture, and this only decides who the arrows belong to.
//!
//! Which means the asymmetry `weaving` records applies here unchanged — this is
//! gated by a run condition because it consumes keys, and `type_into_line` is
//! not because it has to *run* to throw away what it declines.
//!
//! # An arrow moves the reading the instant it is pressed
//!
//! Two versions of this went through the prompt's queue and both were wrong.
//! Submitting per keystroke walked at the speed of the *keyboard* — `Pending` is
//! drained whole at tick start, so a held arrow was thirty cells at once.
//! Keeping one aim, then a bounded burst, walked at the speed of the *world*,
//! and a maze at 1 Hz is a wait rather than a minigame.
//!
//! The queue was never the problem: the **tick** was. So an arrow now calls
//! [`Sim::walk`](orbs_sim::Sim::walk) directly — a third entry point that moves
//! the reading and consumes no tick, so a player walks as fast as they can
//! press and no brew advances while they do it.
//!
//! There is nothing left here to bound. Key repeat is as fast as the player's
//! keyboard says it is, which is the answer to *"as fast as I can press"*, and a
//! wall still refuses rather than costing anything.

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
/// **It holds no steps.** Two earlier versions kept an aim and then a queue,
/// which is what you need when a move has to wait for a clock. A move no longer
/// waits, so there is nothing to hold — see this module's header.
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
    // **Peeked before it is taken**, exactly as the editor and the weave screen
    // do: `wandering` needs `&mut`, and reaching for it stamps `Tower`'s change
    // tick, which would leave this system re-arming its own run condition every
    // frame and drag the panel and the suggestions back to 60 Hz with it.
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
    // **A held chord is skipped; a *stale* one is not.** `chord_is_stale` says
    // the modifier is a ghost — still latched from an alt-tab the window never
    // saw released — so it means *accept this keystroke*, and reading it the
    // other way swallows one key every time the mode is entered after a pause,
    // which is exactly when it is entered. The weave screen shipped with it
    // inverted; this matches the corrected version rather than the first draft.
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
        // **`break`, not `continue`.** A frame can carry several keystrokes, and
        // going on to the rest of the batch walked the reading *after* the
        // player had left the mode — the exact hazard the comment below worries
        // about, in the one place it was reachable.
        if matches!(&event.logical_key, Key::Escape) {
            walk.close();
            break;
        }
        // The arrow-to-`Way` table is `orbs-shell`'s; it was written here, in
        // `orbs-tui`, and a third time in `dump.rs`.
        //
        // **Everything else is swallowed, not passed on.** A surface that owns
        // the keyboard owns all of it; letting text through would put characters
        // into a prompt the player cannot see a caret in.
        let Some(way) = super::input::pressed(event)
            .as_ref()
            .and_then(orbs_shell::apply_to_maze)
        else {
            continue;
        };
        // **Straight into the world, on this frame.** No message, no queue, no
        // waiting for a tick — see this module's header for the two slower
        // versions this replaced. `Tower` is stamped by the `&mut`, which is
        // what has `refresh_panel` redraw the map on the next frame.
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
/// **Three ways a maze ends, and one condition covers them.** It is solved; it
/// is abandoned by `stop lectern`, which a bound spell may issue; or the player
/// is no longer in the archive. Naming this after the solved case would have
/// left the other two owning the keyboard over a pane with no map on it.
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
        // **The whole point of the third entry point.** Two earlier versions put
        // the step through the prompt's queue, so a press waited for the world's
        // next tick — a second — and a maze at 1 Hz is a wait rather than a
        // minigame. No tick is stepped anywhere in this test.
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
        // **`stop stacks`, and it was `stop lectern`.** The maze moved to its own
        // instrument, so stopping the lectern now abandons an *assembly* and
        // leaves the stacks alone — which is the whole point of splitting
        // them, and is what this test would have gone on asserting the opposite
        // of.
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
