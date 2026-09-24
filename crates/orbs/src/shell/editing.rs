//! The editor's place in the shell: opening it, feeding it keys, saving it.
//!
//! [`Editor`] is the buffer and knows nothing about Bevy; this is the wiring.
//! A separate file because the buffer's rules (Backspace at column zero) and
//! the shell's rules (which keys are chords) change for different reasons.
//!
//! Exactly one of the prompt and the editor takes a keystroke. Both consuming
//! one is invisible until a player types `:wq` and finds it in their command
//! history, so the two run conditions are complements, not a convention.

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
    /// Mutable even for painting: only the painter knows how many lines fit, so
    /// it calls `Editor::scroll_to` during the draw.
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
    /// The one way the modal is entered, so `plugin`'s tests use the same door
    /// `open_requested` does rather than a second one that could drift.
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
/// There is no `not_editing` any more: a system that does not run keeps its
/// message cursor, so keys typed while the editor had the keyboard arrived at
/// the prompt in a burst. `type_into_line` runs whenever there are keys and
/// decides for itself.
pub(crate) fn reading(scroll: Res<orbs_shell::Scroll>) -> bool {
    scroll.is_reading()
}

/// Open the editor when `scribe` has asked for it.
///
/// The sim owns the *decision* — which spell, and what it holds — and the
/// frontend owns the buffer. `Sim::opening` takes rather than reads, so this
/// fires once per `scribe` rather than every frame.
pub(crate) fn open_requested(
    mut tower: ResMut<Tower>,
    mut editing: ResMut<Editing>,
    // `Option` for `commanding::submit`'s reason: half the tests build the
    // shell alone, where a bare `Res` fails validation. Absent means empty.
    readers: Option<Res<crate::sim::Readers>>,
) {
    // Peeked before it is taken: `opening` needs `&mut`, which stamps `Tower`'s
    // change tick and so re-armed this system's own run condition every frame,
    // dragging `refresh_panel` and `suggest` to 60 Hz with it.
    if !tower.has_opening() {
        return;
    }
    let Some(request) = tower.opening() else {
        return;
    };
    let mut editor = Editor::open(&request.name, &request.domain, &request.lines);
    // Read before the first keystroke, so a spell opened with a fault in it
    // says so on the way in rather than after the first pause.
    let scrivener = readers.as_deref().and_then(crate::sim::Readers::scrivener);
    editor.set_reading(tower.read_spell_with(
        &request.name,
        &request.domain,
        &request.lines,
        scrivener,
    ));
    // And the guide, for the same reason: it opens on the vocabulary, and a
    // pane that filled in only after the first keystroke would look broken.
    editor.refresh(tower.sim());
    editing.open(editor);
}

/// Write the buffer out once the player has stopped typing, and keep the
/// running-line marker current.
///
/// Unconditional while the editor is open, because both halves are clocks: the
/// settle timer can only finish on a frame where nothing was typed, and the
/// marker moves on ticks nobody pressed a key for.
pub(crate) fn autosave(
    time: Res<Time>,
    mut editing: ResMut<Editing>,
    mut tower: ResMut<Tower>,
    readers: Option<Res<crate::sim::Readers>>,
) {
    let Some(editor) = editing.get_mut() else {
        return;
    };

    // Where the orb has reached in this spell, if it is running one. Pushed in
    // rather than pulled, because the buffer knows nothing about the sim.
    let line = tower.sim().running_line(editor.name());
    editor.set_running_line(line);

    if editor.settle(time.delta_secs()) {
        let (name, lines) = (editor.name().to_owned(), editor.lines().to_vec());
        let domain = editor.domain().to_owned();
        editor.saved();
        // On the save's beat, not every frame: reading parses the whole buffer,
        // and only a keystroke can change what it says.
        //
        // Write first, then read, so the editor's reading is the one the write
        // queued rather than a second trip to the reader.
        let scrivener = readers.as_deref().and_then(crate::sim::Readers::scrivener);
        tower.write_spell_with(&name, &lines, scrivener);
        let reading = tower.read_spell_with(&name, &domain, &lines, scrivener);
        if let Some(editor) = editing.get_mut() {
            editor.set_reading(reading);
        }
    }
}

/// Feed keys to the open editor.
pub(crate) fn type_into_editor(
    mut keys: MessageReader<KeyboardInput>,
    mut held: ResMut<ButtonInput<KeyCode>>,
    mut editing: ResMut<Editing>,
    mut tower: ResMut<Tower>,
    quiet: Res<super::input::Quiet>,
    readers: Option<Res<crate::sim::Readers>>,
) {
    // Set when a keystroke proves the held chord is a ghost — see
    // `input::chord_is_stale`. The editor shares the prompt's guard, so it
    // shared the freeze.
    let mut stale = false;
    let stale_chord = super::input::chord_is_stale(quiet.gap());
    let Some(editor) = editing.get_mut() else {
        return;
    };

    // Same guard as the prompt's: a chord aimed at the operating system must
    // not reach the text field on the way past. Alt stays out — AltGr is how
    // European layouts type `@`, `#` and `\`, which a spell needs.
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
        // The table is `orbs-shell`'s and both frontends call it. The editor
        // decides what a key means in its current mode; branching on the mode
        // out here would be a second copy of that state machine.
        let Some(key) = super::input::pressed(event) else {
            continue;
        };
        outcome = orbs_shell::apply_to_editor(&key, editor, tower.sim()).or(outcome);
    }

    if stale {
        held.reset_all();
    }
    let scrivener = readers.as_deref().and_then(crate::sim::Readers::scrivener);
    apply(outcome, &mut editing, &mut tower, scrivener);
}

/// Act on what the editor asked for.
fn apply(
    outcome: Option<EditorOutcome>,
    editing: &mut Editing,
    tower: &mut Tower,
    scrivener: Option<&dyn orbs_sim::Scrivener>,
) {
    let Some(outcome) = outcome else {
        return;
    };
    let Some(editor) = editing.get_mut() else {
        return;
    };

    match outcome {
        EditorOutcome::Save | EditorOutcome::SaveAndClose => {
            // One call carrying the whole buffer — the editor's entire
            // contribution to a replay. `Sim::write_spell` records now and
            // queues the write for the next tick, where effects land.
            let (name, lines) = (editor.name().to_owned(), editor.lines().to_vec());
            editor.saved();
            tower.write_spell_with(&name, &lines, scrivener);
            if outcome == EditorOutcome::SaveAndClose {
                editing.close();
            } else {
                // Read again, as the autosave does: a `w` typed before the
                // pause fired leaves the settle beat nothing to save, so the
                // marks went stale.
                let domain = editor.domain().to_owned();
                editor.set_reading(tower.read_spell_with(&name, &domain, &lines, scrivener));
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
    /// Not the whole `ShellPlugin` — that wants a window and a GPU. This
    /// exercises what the `Editor` unit tests cannot reach: a `Time` that
    /// really advances, driving a save that lands in the world.
    fn app() -> App {
        let mut app = App::new();
        app.insert_resource(Time::<()>::default())
            .insert_resource(Editing::default())
            .insert_resource(Tower::new(1))
            .add_systems(Update, autosave.run_if(editing));
        app
    }

    /// The same, with a fixed spell reader installed.
    fn app_reading() -> App {
        let mut app = app();
        app.insert_resource(crate::sim::Readers::copying(Box::new(
            orbs_sim::Copyist::worked(),
        )));
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
        // Type, stop, and it is saved with no key pressed to make it happen.
        // Driven through a real `App` because the clock is the mechanism;
        // `Editor::settle`'s unit test proves the arithmetic, this the wiring.
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
    fn an_autosaved_buffer_reaches_the_scrivener_and_the_file_is_still_the_players() {
        // The seam a dump cannot reach: `ORBS_DUMP` builds no `App`, so nothing
        // in `scripts/dumps.sh` exercises the settle beat that carries a buffer
        // to the reader. The reading is what compiles, the file is byte-exact.
        let mut app = app_reading();
        app.world_mut().resource_mut::<Editing>().open(Editor::open(
            "morning.spell",
            "laboratory",
            &[],
        ));
        {
            let mut editing = app.world_mut().resource_mut::<Editing>();
            let editor = editing.get_mut().expect("the editor was just opened");
            editor.type_text("edit");
            editor.enter();
            // A line `Copyist::worked` knows and the matcher cannot read.
            editor.type_text("work the sage down");
        }
        tick(&mut app, 5.0);
        app.world_mut().resource_mut::<Tower>().step();

        let tower = app.world().resource::<Tower>();
        assert_eq!(
            tower.sim().spell("morning"),
            Some(vec!["work the sage down".to_owned()]),
            "the orb rewrote the player's file",
        );
        let node = tower
            .sim()
            .world()
            .iter_entities()
            .find(|entity| {
                entity
                    .get::<orbs_sim::tower::Name>()
                    .is_some_and(|name| name.0 == "morning.spell")
            })
            .map(|entity| entity.id())
            .expect("the spell was written");
        assert_eq!(
            orbs_sim::tower::spell::source(tower.sim().world(), node),
            vec!["grind sage".to_owned()],
            "the reading never reached the program",
        );
    }

    #[test]
    fn the_marker_follows_a_running_spell_without_a_keystroke() {
        // The half no keystroke drives: the orb moves down the file while the
        // player sits still.
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
