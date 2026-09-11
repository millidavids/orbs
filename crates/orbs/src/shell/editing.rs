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
    // `Option` for `commanding::submit`'s reason: half the tests here build the
    // shell alone, and a bare `Res` fails parameter validation there. Absent is
    // the same as empty — the reading is the text.
    readers: Option<Res<crate::sim::Readers>>,
) {
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
    let mut editor = Editor::open(&request.name, &request.domain, &request.lines);
    // **Read before the first keystroke**, so a spell opened with a fault in it
    // says so on the way in rather than after the first pause in the typing.
    let scrivener = readers.as_deref().and_then(crate::sim::Readers::scrivener);
    editor.set_reading(tower.read_spell_with(
        &request.name,
        &request.domain,
        &request.lines,
        scrivener,
    ));
    // **And the guide, for the same reason**: it opens on the vocabulary, and a
    // pane that filled in only after the first keystroke would look broken to
    // exactly the player it is there for.
    editor.refresh(tower.sim());
    editing.open(editor);
}

/// Write the buffer out once the player has stopped typing, and keep the
/// running-line marker current.
///
/// **Unconditional while the editor is open**, because both halves are clocks
/// rather than reactions: the settle timer has to run on the frames where
/// nothing was typed — those are the only frames it can *finish* on — and a
/// spell's marker moves on ticks the player is not touching the keyboard for.
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
        // **On the same beat as the save, not every frame.** Reading a buffer
        // means parsing it and resolving every name in it against the room; at
        // 60 Hz that is sixty parses a second to answer a question that can only
        // change when a key is pressed. The pause the save waits for is exactly
        // the moment the answer might have changed.
        //
        // **The write first, then the reading**, so a line that changed is read
        // once on this beat: the write queues its reading, and the editor's
        // finds it there rather than asking the reader a second time.
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
        // **The table is `orbs-shell`'s**, and both frontends call it. Both of
        // these keys mean something in both of the editor's states and **the
        // editor decides which** — branching on the mode out here would be a
        // second copy of that state machine. The table was a second copy of a
        // different kind, and the terminal build had already been written from
        // it without the `or` correction that lives in it now.
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
            // One call, carrying the whole buffer — the editor's entire
            // contribution to a replay. `Sim::write_spell` records the
            // submission now and queues the write for the next tick, because
            // effects land on tick boundaries.
            let (name, lines) = (editor.name().to_owned(), editor.lines().to_vec());
            editor.saved();
            tower.write_spell_with(&name, &lines, scrivener);
            if outcome == EditorOutcome::SaveAndClose {
                editing.close();
            } else {
                // **And read again, as the autosave does.** A `w` typed before
                // the pause fired leaves the settle beat nothing to save, so the
                // marks stayed on the buffer from a few keystrokes earlier.
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
    fn an_autosaved_buffer_reaches_the_scrivener_and_the_file_is_still_the_players() {
        // **The seam a dump cannot reach.** `ORBS_DUMP` builds no `App`, so
        // everything `scripts/dumps.sh` proves about the spell reader it proves
        // about `Sim::write_spell_reading` — never about the settle beat that
        // carries a buffer to it. `commanding` makes the same argument about the
        // message that carries a typed line to the prompt's reader.
        //
        // Both halves are asserted, because they are the whole shape of this
        // feature: the reading is what compiles, and the file is byte-exact.
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
