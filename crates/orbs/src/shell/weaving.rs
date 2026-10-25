//! The weave screen's place in the shell: opening it, feeding it keys.
//!
//! The wiring for [`Tapestry`]. Gated by a run condition, where the prompt runs
//! and declines, since a reader that does not run keeps its cursor (§19).

use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

use super::{Tapestry, WeaveOutcome};
use crate::sim::Tower;

/// The weave screen, if it is open.
pub(crate) type Loom = super::modal::Modal<Tapestry>;

/// Open the screen when `weave` has asked for one.
///
/// The sim owns the *decision*; the frontend owns the cursor and the mode.
/// `Sim::weaving` takes rather than reads, so this fires once per `weave` rather
/// than every frame — including the frame after the player quit out of it.
pub(crate) fn open_requested(mut tower: ResMut<Tower>, mut loom: ResMut<Loom>) {
    // Peeked before it is taken, as the editor's `open_requested` does:
    // `weaving` needs `&mut`, which stamps `Tower`'s change tick and would
    // re-arm this system's own run condition every frame, dragging the panel and
    // the suggestions back to 60 Hz.
    if !tower.has_weaving() {
        return;
    }
    if !tower.weaving() {
        return;
    }
    loom.open(Tapestry::default());
}

/// Keep the screen's reading of the world current.
///
/// Unconditional while it is open, because the world ticks behind it: a
/// threshold crossed while a player is looking at the track should land while
/// they are looking.
pub(crate) fn refresh(mut loom: ResMut<Loom>, tower: Res<Tower>) {
    if let Some(screen) = loom.get_mut() {
        screen.read_from(tower.sim());
    }
}

/// Feed keys to the open screen.
pub(crate) fn type_into_loom(
    mut keys: MessageReader<KeyboardInput>,
    held: Res<ButtonInput<KeyCode>>,
    quiet: Res<super::input::Quiet>,
    mut loom: ResMut<Loom>,
    mut tower: ResMut<crate::sim::Tower>,
) {
    // A held chord is skipped; a stale one is not. A stale chord is a ghost —
    // latched from an alt-tab the window never saw released — so it means
    // *accept this keystroke as plain text*. The first version read it as
    // "drop this keystroke", which swallowed one key every time the screen was
    // opened after a pause.
    let chord = super::input::Chord::read(&held, &quiet);

    let mut outcome = None;
    {
        let Some(screen) = loom.get_mut() else {
            keys.clear();
            return;
        };
        for event in keys.read() {
            if event.state != ButtonState::Pressed {
                continue;
            }
            if chord.swallows() {
                continue;
            }
            // `orbs-shell`'s table, which the terminal build also calls.
            let Some(key) = super::input::pressed(event) else {
                continue;
            };
            outcome = orbs_shell::apply_to_weave(&key, screen).or(outcome);
        }
    }
    match outcome {
        Some(WeaveOutcome::Close) => loom.close(),
        // Handed to the sim, which decides. The screen has already refused a
        // marker, a locked node and a spent tier; the world re-checks all three,
        // because a screen's arithmetic is a second opinion about the rules
        // (§19).
        Some(WeaveOutcome::Take(id)) => tower.take(&id),
        None => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An app with the screen's two systems and nothing that wants a window.
    ///
    /// Not the whole `ShellPlugin`, for the reason `editing.rs`'s tests give:
    /// that wants a renderer, and a test needing a GPU is a test nobody runs.
    fn app() -> App {
        let mut app = App::new();
        app.init_resource::<Loom>()
            .insert_resource(Tower::new(1))
            .add_systems(Update, (open_requested, refresh).chain());
        app
    }

    /// Type a line at the sim and give the world a tick to do it.
    fn run(app: &mut App, line: &str) {
        let mut tower = app.world_mut().resource_mut::<Tower>();
        tower.submit(line);
        tower.step();
    }

    /// Let the world run without touching the keyboard.
    fn idle(app: &mut App, ticks: usize) {
        for _ in 0..ticks {
            app.world_mut().resource_mut::<Tower>().step();
            app.update();
        }
    }

    /// What the open screen says its total is.
    fn showing(app: &mut App) -> u64 {
        app.world_mut()
            .resource_mut::<Loom>()
            .get_mut()
            .expect("the screen is not open")
            .experience()
    }

    #[test]
    fn the_verb_opens_it_once() {
        let mut app = app();
        app.update();
        assert!(
            !app.world().resource::<Loom>().is_open(),
            "it opened without being asked",
        );

        run(&mut app, "weave");
        app.update();
        assert!(app.world().resource::<Loom>().is_open());

        // ...and closing it does not reopen on the next frame, which is the
        // whole reason the request is taken rather than read.
        app.world_mut().resource_mut::<Loom>().close();
        app.update();
        assert!(
            !app.world().resource::<Loom>().is_open(),
            "a stale request reopened it",
        );
    }

    #[test]
    fn it_notices_the_world_moving_without_a_keystroke() {
        // The world ticks behind the screen, and a track refreshing only on
        // input would show a total as unreached for as long as the player sat
        // still — exactly when they are looking at it. Driven through a real
        // grind, because there is no public way to hand the sim experience.
        let mut app = app();
        run(&mut app, "attend laboratory");
        run(&mut app, "weave");
        app.update();
        assert_eq!(showing(&mut app), 0);
        assert!(
            !app.world_mut()
                .resource_mut::<Loom>()
                .get_mut()
                .expect("open")
                .ley_line()
                .is_empty(),
            "the screen drew with no track in it",
        );

        run(&mut app, "grind sage");
        idle(&mut app, 12);
        run(&mut app, "empty mortar_and_pestle");
        idle(&mut app, 2);

        assert_eq!(showing(&mut app), 1, "it did not see the work land");
    }
}
