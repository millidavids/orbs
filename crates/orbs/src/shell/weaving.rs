//! The weave screen's place in the shell: opening it, feeding it keys.
//!
//! [`Tapestry`] is the screen and knows nothing about Bevy. This is the wiring,
//! and it is a separate file for the reason `editing.rs` is: what an arrow key
//! *means* on a tree and which keys are stale chords are different concerns.
//!
//! # Exactly one surface takes a keystroke
//!
//! There are now four that can own the keyboard — the prompt, the editor, the
//! unfurled transcript, and this. §19 records the shape that rule has to take:
//! the prompt **runs and declines**, because *"a system that does not run keeps
//! its message cursor"* and every key typed while another surface held the
//! keyboard otherwise arrived in a burst the moment it ran again. So this is
//! gated by a run condition and `type_into_line` is not — the asymmetry is
//! deliberate and is the same one the editor has.

use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

use super::{Tapestry, WeaveOutcome};
use crate::sim::Tower;

/// The weave screen, if it is open.
///
/// A resource rather than a component, for the reason `Editing` gives: there is
/// one screen, it is modal, and an entity would invite a second.
#[derive(Resource, Debug, Default)]
pub(crate) struct Loom(Option<Tapestry>);

impl Loom {
    /// The open screen.
    pub(crate) const fn get_mut(&mut self) -> Option<&mut Tapestry> {
        self.0.as_mut()
    }

    /// Whether the screen has the keyboard.
    #[must_use]
    pub(crate) const fn is_open(&self) -> bool {
        self.0.is_some()
    }

    /// Put a screen up, replacing whatever was there.
    ///
    /// The one way the modal is entered, so tests reach it through the same door
    /// `open_requested` uses rather than a second one that could drift.
    pub(crate) fn open(&mut self, screen: Tapestry) {
        self.0 = Some(screen);
    }

    /// Take it down.
    pub(crate) fn close(&mut self) {
        self.0 = None;
    }
}

/// Whether the screen has the keyboard.
pub(crate) fn weaving(loom: Res<Loom>) -> bool {
    loom.is_open()
}

/// Open the screen when `weave` has asked for one.
///
/// The sim owns the *decision*; the frontend owns the cursor and the mode.
/// `Sim::weaving` takes rather than reads, so this fires once per `weave` rather
/// than every frame — including the frame after the player quit out of it.
pub(crate) fn open_requested(mut tower: ResMut<Tower>, mut loom: ResMut<Loom>) {
    // **Peeked before it is taken**, exactly as `open_requested` does for the
    // editor: `weaving` needs `&mut` and reaching for it stamps `Tower`'s change
    // tick, which would leave this system re-arming its own run condition every
    // frame and drag the panel and the suggestions back to 60 Hz with it.
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
/// **Unconditional while it is open**, because the world ticks behind it: a
/// threshold crossed while a player is looking at the track should land while
/// they are looking, not the next time they open it. Pushed in rather than
/// pulled, because the screen knows nothing about the sim.
pub(crate) fn refresh(mut loom: ResMut<Loom>, tower: Res<Tower>) {
    let Some(screen) = loom.get_mut() else {
        return;
    };
    let sim = tower.sim();
    screen.refresh(
        sim.experience(),
        sim.renown(),
        sim.scale(),
        sim.ley_line(),
        sim.mastery(),
    );
}

/// Feed keys to the open screen.
pub(crate) fn type_into_loom(
    mut keys: MessageReader<KeyboardInput>,
    held: Res<ButtonInput<KeyCode>>,
    quiet: Res<super::input::Quiet>,
    mut loom: ResMut<Loom>,
    mut tower: ResMut<crate::sim::Tower>,
) {
    // **A held chord is skipped; a *stale* one is not.** `chord_is_stale` says
    // the modifier is a ghost — still latched from an alt-tab the window never
    // saw released — so it means *accept this keystroke as plain text*, and the
    // first version read it as "drop this keystroke". That is one swallowed key
    // every time the screen is opened after a pause, which is exactly when it is
    // opened. The prompt and the editor both have this the right way round; this
    // now matches them rather than paraphrasing them.
    let stale_chord = super::input::chord_is_stale(quiet.gap());
    let chord = held.any_pressed([
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
    ]);

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
            // A chord the player aimed at their operating system must not reach
            // this screen on the way past.
            if chord && !stale_chord {
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
        // **Handed to the sim, which decides.** The screen has already refused a
        // marker, a locked node and a spent tier; the world re-checks all three
        // before granting, because a screen's arithmetic is a second opinion
        // about the rules and §19 records what happens when two of those drift.
        Some(WeaveOutcome::Take(id)) => tower.take(&id),
        None => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An app with the screen's two systems and nothing that wants a window.
    ///
    /// **Not the whole `ShellPlugin`**, for the reason `editing.rs`'s tests give:
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
        // **The world ticks behind the screen**, and a track that only refreshed
        // on input would show a total as unreached for as long as the player sat
        // still — which is exactly when they are looking at it. Driven through a
        // real grind, because there is no public way to hand the sim experience
        // and a test that skipped the work would test a state the game cannot
        // reach.
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
