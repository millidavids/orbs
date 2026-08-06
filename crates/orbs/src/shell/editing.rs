//! The editor's place in the shell: opening it, feeding it keys, saving it.
//!
//! [`Editor`] is the buffer and knows nothing about Bevy. This is
//! the wiring — which is a separate file because the buffer's rules (what
//! Backspace does at column zero) and the shell's rules (which keys are chords,
//! when the prompt is dead) are different concerns that change for different
//! reasons.
//!
//! # The one thing the prompt and the editor must agree on
//!
//! **Exactly one of them takes a keystroke.** They are both text fields on the
//! same screen, and the failure where both consume a key is invisible until a
//! player types `:wq` and finds it in their command history. `type_into_line`
//! refuses to run while an editor is open, and this refuses to run while it is
//! not — the two run conditions are complements, not a convention.

use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;

use super::{Editor, EditorOutcome};
use crate::sim::Tower;

/// The spell being edited, if any.
///
/// A resource rather than a component: there is one editor, it is modal, and an
/// entity would invite a second.
#[derive(Resource, Debug, Default)]
pub(crate) struct Editing(Option<Editor>);

impl Editing {
    /// The open editor.
    ///
    /// **Mutable even for painting.** The viewport follows the caret and how
    /// many lines fit is a fact only the painter has, so `scroll_to` is called
    /// during the draw — see `Editor::scroll_to`. A read-only accessor beside
    /// this one would just be the one nothing could use.
    pub(crate) const fn get_mut(&mut self) -> Option<&mut Editor> {
        self.0.as_mut()
    }

    /// Whether the editor has the keyboard.
    #[must_use]
    pub(crate) const fn is_open(&self) -> bool {
        self.0.is_some()
    }

    /// Put `editor` on screen, replacing whatever was there.
    ///
    /// The one way the modal is entered, so `plugin`'s tests reach the editor
    /// through the same door `open_requested` uses rather than through a second
    /// one that could drift from it.
    pub(crate) fn open(&mut self, editor: Editor) {
        self.0 = Some(editor);
    }

    /// Take the editor off screen.
    pub(crate) fn close(&mut self) {
        self.0 = None;
    }
}

/// Whether the editor currently owns input.
pub(crate) fn editing(editing: Res<Editing>) -> bool {
    editing.is_open()
}

/// Whether the transcript currently owns input.
///
/// # There is no `not_editing` any more
///
/// It was the prompt's run condition, and a run condition was the wrong shape:
/// a system that does not *run* keeps its message cursor, so every keystroke
/// typed while the editor or the transcript had the keyboard was still queued —
/// and arrived at the prompt in a burst the moment it ran again. Throwing a
/// keystroke away means reading it and dropping it, which means running.
///
/// So `type_into_line` runs whenever there are keys and decides for itself; the
/// invariant that exactly one surface consumes a keystroke now lives in one
/// function rather than in a pair of predicates that had to stay complements.
pub(crate) fn reading(scroll: Res<super::input::Scroll>) -> bool {
    scroll.is_reading()
}

/// Open the editor when `scribe` has asked for it.
///
/// The sim owns the *decision* — which spell, and what it holds — and the
/// frontend owns the buffer. `Sim::opening` takes rather than reads, so this
/// fires once per `scribe` rather than every frame.
pub(crate) fn open_requested(mut tower: ResMut<Tower>, mut editing: ResMut<Editing>) {
    // **Peeked before it is taken.** `opening` needs `&mut`, and reaching for it
    // stamps `Tower`'s change tick — so this system re-armed its own run
    // condition every frame and dragged `refresh_panel` and `suggest` back to
    // 60 Hz with it.
    if !tower.has_opening() {
        return;
    }
    let Some(request) = tower.opening() else {
        return;
    };
    editing.open(Editor::open(&request.name, &request.domain, &request.lines));
}

/// Write the buffer out once the player has stopped typing, and keep the
/// running-line marker current.
///
/// **Unconditional while the editor is open**, because both halves are clocks
/// rather than reactions: the settle timer has to run on the frames where
/// nothing was typed — those are the only frames it can *finish* on — and a
/// spell's marker moves on ticks the player is not touching the keyboard for.
pub(crate) fn autosave(time: Res<Time>, mut editing: ResMut<Editing>, mut tower: ResMut<Tower>) {
    let Some(editor) = editing.get_mut() else {
        return;
    };

    // Where the orb has reached in this spell, if it is running one. Pushed in
    // rather than pulled, because the buffer knows nothing about the sim.
    let line = tower.sim().running_line(editor.name());
    editor.set_running_line(line);

    if editor.settle(time.delta_secs()) {
        let (name, lines) = (editor.name().to_owned(), editor.lines().to_vec());
        editor.saved();
        tower.write_spell(&name, &lines);
    }
}

/// Feed keys to the open editor.
pub(crate) fn type_into_editor(
    mut keys: MessageReader<KeyboardInput>,
    mut held: ResMut<ButtonInput<KeyCode>>,
    mut editing: ResMut<Editing>,
    mut tower: ResMut<Tower>,
    quiet: Res<super::input::Quiet>,
) {
    // Set when a keystroke proves the held chord is a ghost — see
    // `input::chord_is_stale`. The editor needs this as much as the prompt does,
    // and for the identical reason: they share the guard, so they shared the
    // freeze.
    let mut stale = false;
    let stale_chord = super::input::chord_is_stale(quiet.gap());
    let Some(editor) = editing.get_mut() else {
        return;
    };

    // Same guard as the prompt's, and for the same reason: a chord the player
    // aimed at their operating system must not reach the text field on the way
    // past. Alt stays out of the list — AltGr is how European layouts type `@`,
    // `#` and `\`, which a spell needs.
    let chord = held.any_pressed([
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
    ]);
    let editing_chord = held.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight])
        && !held.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);

    let mut outcome = None;
    for event in keys.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }
        if chord && !stale_chord {
            // `Cmd+←/→` is Home/End on a keyboard with neither.
            if editing_chord {
                match &event.logical_key {
                    Key::ArrowLeft => editor.home(),
                    Key::ArrowRight => editor.end(),
                    _ => {}
                }
            }
            continue;
        }
        stale |= chord && stale_chord;
        match &event.logical_key {
            // Both of these mean something in both states, and **the editor
            // decides which** — see `Editor::enter` and `Editor::escape`. This
            // loop branching on the mode itself would be a second copy of the
            // state machine, in the one place neither the tests nor `ORBS_DUMP`
            // can reach.
            // **`or`, not `=`.** Two Enters in one frame — key repeat, a frame
            // hitch, a practiced `wq<Enter>` — had the second overwrite the
            // first: `run_command` takes the command string, so the second call
            // ran on an empty line, returned `None`, and discarded a pending
            // `SaveAndClose`. The editor stayed open on a `quit` that looked
            // ignored, with the buffer unflushed.
            Key::Enter => outcome = editor.enter().or(outcome),
            Key::Escape => editor.escape(),
            Key::Backspace => editor.backspace(),
            Key::ArrowLeft => editor.left(),
            Key::ArrowRight => editor.right(),
            Key::ArrowUp => editor.up(),
            Key::ArrowDown => editor.down(),
            Key::Home => editor.home(),
            Key::End => editor.end(),
            _ => {
                let Some(text) = &event.text else {
                    continue;
                };
                editor.type_text(text);
            }
        }
    }

    if stale {
        held.reset_all();
    }
    apply(outcome, &mut editing, &mut tower);
}

/// Act on what the editor asked for.
fn apply(outcome: Option<EditorOutcome>, editing: &mut Editing, tower: &mut Tower) {
    let Some(outcome) = outcome else {
        return;
    };
    let Some(editor) = editing.get_mut() else {
        return;
    };

    match outcome {
        EditorOutcome::Save | EditorOutcome::SaveAndClose => {
            // One call, carrying the whole buffer — the editor's entire
            // contribution to a replay. `Sim::write_spell` records the
            // submission now and queues the write for the next tick, because
            // effects land on tick boundaries.
            let (name, lines) = (editor.name().to_owned(), editor.lines().to_vec());
            editor.saved();
            tower.write_spell(&name, &lines);
            if outcome == EditorOutcome::SaveAndClose {
                editing.close();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    /// An app with just the two resources and the one system under test.
    ///
    /// **Not the whole `ShellPlugin`.** That wants a window and a renderer, and
    /// a test that needed a GPU would be a test nobody runs. What is exercised
    /// here is the wiring the unit tests on `Editor` cannot reach: a `Time` that
    /// really advances, driving a save that really lands in the world.
    fn app() -> App {
        let mut app = App::new();
        app.insert_resource(Time::<()>::default())
            .insert_resource(Editing::default())
            .insert_resource(Tower::new(1))
            .add_systems(Update, autosave.run_if(editing));
        app
    }

    /// Let `seconds` pass and run one frame.
    fn tick(app: &mut App, seconds: f32) {
        app.world_mut()
            .resource_mut::<Time<()>>()
            .advance_by(Duration::from_secs_f32(seconds));
        app.update();
    }

    #[test]
    fn a_pause_in_the_typing_writes_the_spell_out() {
        // The user-facing behaviour, end to end: type, stop, and it is saved —
        // with no word typed and no key pressed to make it happen.
        //
        // Driven through a real `App` because the clock is the mechanism. The
        // unit test on `Editor::settle` proves the arithmetic; this proves the
        // system is registered, takes `Time`, and reaches `Sim::write_spell`.
        let mut app = app();
        app.world_mut().resource_mut::<Editing>().open(Editor::open(
            "morning.spell",
            "laboratory",
            &[],
        ));
        app.world_mut()
            .resource_mut::<Editing>()
            .get_mut()
            .expect("the editor was just opened")
            .type_text("edit");
        // Out of command state and into the buffer, the way a player gets there.
        {
            let mut editing = app.world_mut().resource_mut::<Editing>();
            let editor = editing.get_mut().expect("still open");
            editor.enter();
            editor.type_text("survey");
        }

        tick(&mut app, 0.05);
        assert!(
            app.world_mut()
                .resource_mut::<Editing>()
                .get_mut()
                .is_some_and(|editor| editor.is_dirty()),
            "the buffer was written out mid-word",
        );

        tick(&mut app, 5.0);
        assert!(
            app.world_mut()
                .resource_mut::<Editing>()
                .get_mut()
                .is_some_and(|editor| !editor.is_dirty()),
            "the pause never produced a save",
        );

        // ...and it reached the world, rather than only clearing a flag. The
        // write queues for the next tick like every other effect.
        app.world_mut().resource_mut::<Tower>().step();
        assert_eq!(
            app.world().resource::<Tower>().sim().spell("morning"),
            Some(vec!["survey".to_owned()]),
            "the save never reached the sim",
        );
    }

    #[test]
    fn the_marker_follows_a_running_spell_without_a_keystroke() {
        // The other half of the same system, and the half no keystroke drives:
        // the orb moves down the file while the player sits still, so the marker
        // has to be refreshed on frames where nothing was typed.
        let mut app = app();
        {
            let mut tower = app.world_mut().resource_mut::<Tower>();
            tower.submit("attend laboratory");
            tower.step();
            tower.write_spell(
                "slow",
                &[
                    "survey".to_owned(),
                    "survey".to_owned(),
                    "survey".to_owned(),
                ],
            );
            tower.step();
            tower.submit("invoke slow");
            tower.step();
        }
        app.world_mut().resource_mut::<Editing>().open(Editor::open(
            "slow.spell",
            "laboratory",
            &[],
        ));

        tick(&mut app, 0.016);
        let first = app
            .world_mut()
            .resource_mut::<Editing>()
            .get_mut()
            .and_then(|editor| editor.running_line());
        assert!(first.is_some(), "the marker never found the invocation");

        app.world_mut().resource_mut::<Tower>().step();
        tick(&mut app, 0.016);
        let second = app
            .world_mut()
            .resource_mut::<Editing>()
            .get_mut()
            .and_then(|editor| editor.running_line());
        assert_ne!(first, second, "the marker did not follow the orb");
    }
}
