//! Registration for the window shell.

use bevy::app::AppExit;
use bevy::input::common_conditions::input_just_pressed;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy::window::WindowResized;

use super::input::{Line, SubmittedMessage, type_into_line};
use super::screen::{Screen, spawn_camera, track_window};
use crate::sim::Tower;

/// Ordering within `Update`, so the frame that draws a keystroke is the frame
/// that received it.
///
/// Without this the paint has no edge to the input systems and may run before or
/// after them, frame to frame. Mostly that is a one-frame lag nobody sees — but
/// it makes `ORBS_CAPTURE=1` screenshots non-reproducible, and reproducible
/// screenshots are a working practice here rather than a nicety.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ShellSystems {
    /// Keystrokes reach the line; finished lines reach the sim.
    Input,
}

/// The window, the camera, the grid, and the command line.
pub struct ShellPlugin;

impl Plugin for ShellPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Screen>()
            .init_resource::<Line>()
            .add_message::<SubmittedMessage>()
            .add_systems(Startup, (spawn_camera, track_window).chain())
            .add_systems(
                Update,
                (
                    type_into_line.run_if(on_message::<KeyboardInput>),
                    submit.run_if(on_message::<SubmittedMessage>),
                )
                    .chain()
                    .in_set(ShellSystems::Input),
            )
            .add_systems(
                Update,
                (
                    track_window.run_if(on_message::<WindowResized>),
                    // F10, not Escape: the moment there is a text field, Escape
                    // is "clear the line" muscle memory, and quitting the game
                    // mid-sentence is not a recoverable surprise.
                    quit.run_if(input_just_pressed(KeyCode::F10)),
                ),
            );
    }
}

/// Hand a finished line to the sim.
///
/// The sim resolves it immediately and queues any command for the next tick —
/// see `orbs_sim::session` for why those are two different clocks.
fn submit(mut lines: MessageReader<SubmittedMessage>, mut tower: ResMut<Tower>) {
    for submitted in lines.read() {
        tower.submit(&submitted.line);
    }
}

/// Leave the orb.
fn quit(mut exit: MessageWriter<AppExit>) {
    exit.write(AppExit::Success);
}

/// Typing, end to end, with no window and no GPU.
///
/// These live beside the registration they exercise because that is what is
/// being tested: whether the plugin wires a keystroke to the parser. DESIGN.md
/// §19's whole lesson is that a sixteen-command parser went six roadmap items
/// without ever receiving one, so a test that fires a real `KeyboardInput` at
/// the real plugin stack is the one this surface most needs.
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::input::ButtonState;
    use bevy::input::keyboard::Key;
    use orbs_render::{FieldName, Outcome};

    fn app() -> App {
        let mut app = App::new();
        // No renderer and no windowing backend — but `WindowPlugin` is what
        // registers `WindowResized`, which this plugin's own run conditions
        // read, so the shell genuinely depends on it.
        app.add_plugins((
            MinimalPlugins,
            bevy::input::InputPlugin,
            bevy::window::WindowPlugin::default(),
            ShellPlugin,
        ))
        .insert_resource(Tower::new(1));
        // Startup, so the systems under test see a settled world.
        app.update();
        app
    }

    fn press(app: &mut App, logical_key: Key, text: Option<&str>) {
        app.world_mut().write_message(KeyboardInput {
            // Not read by `type_into_line`; the whole point is that text comes
            // from `text`, not from a physical or logical key.
            key_code: KeyCode::KeyA,
            logical_key,
            state: ButtonState::Pressed,
            text: text.map(Into::into),
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
    }

    fn type_line(app: &mut App, line: &str) {
        for glyph in line.chars() {
            // Exactly what winit delivers: the spacebar arrives as `Key::Space`
            // with `text: Some(" ")`, never as `Key::Character(" ")`.
            let key = if glyph == ' ' {
                Key::Space
            } else {
                Key::Character(glyph.to_string().into())
            };
            press(app, key, Some(&glyph.to_string()));
        }
        press(app, Key::Enter, Some("\r"));
        app.update();
    }

    fn messages(app: &App) -> Vec<String> {
        app.world()
            .resource::<Tower>()
            .sim()
            .scrollback()
            .records()
            .iter()
            .filter_map(|record| record.field(FieldName::Message))
            .map(|value| value.with_str(str::to_owned))
            .collect()
    }

    #[test]
    fn a_typed_line_reaches_the_parser_and_comes_back() {
        let mut app = app();
        type_line(&mut app, "look around");

        // §6: the echo is canonical arcane whatever register was typed.
        assert_eq!(messages(&app), ["look around", "survey"]);
    }

    #[test]
    fn the_space_bar_works() {
        // The regression this file exists for. `Key::Space` is not
        // `Key::Character(" ")`, so a buffer built from `logical_key` loses the
        // space bar and every multi-word command in §6.1 becomes untypeable —
        // with a green test suite and a dead game.
        let mut app = app();
        type_line(&mut app, "look around");
        assert_eq!(
            messages(&app).first().map(String::as_str),
            Some("look around"),
            "the space bar was swallowed",
        );
    }

    #[test]
    fn control_characters_never_enter_the_buffer() {
        // `text` carries them — winit documents Enter as `Some("\r")` — and one
        // in the buffer occupies a cell and draws nothing, so the caret drifts
        // away from the text with no visible cause.
        let mut app = app();
        press(&mut app, Key::Tab, Some("\t"));
        press(&mut app, Key::Character("a".into()), Some("a"));
        press(&mut app, Key::Escape, Some("\u{1b}"));
        press(&mut app, Key::Enter, Some("\r"));
        app.update();

        assert_eq!(messages(&app).first().map(String::as_str), Some("a"));
    }

    #[test]
    fn backspace_deletes_and_enter_clears() {
        let mut app = app();
        for glyph in "surveyx".chars() {
            press(
                &mut app,
                Key::Character(glyph.to_string().into()),
                Some(&glyph.to_string()),
            );
        }
        press(&mut app, Key::Backspace, Some("\u{8}"));
        press(&mut app, Key::Enter, Some("\r"));
        app.update();
        assert_eq!(messages(&app).first().map(String::as_str), Some("survey"));

        // The line must not survive its own submission.
        type_line(&mut app, "look around");
        assert_eq!(messages(&app)[2], "look around");
    }

    #[test]
    fn a_bare_enter_says_nothing() {
        // Every real shell just redraws the prompt.
        let mut app = app();
        press(&mut app, Key::Enter, Some("\r"));
        app.update();
        assert!(messages(&app).is_empty());
    }

    #[test]
    fn an_unknown_word_comes_back_with_somewhere_to_go() {
        // §6 forbids a bare error, and a view must be able to tell a suggestion
        // from the command that will run.
        let mut app = app();
        type_line(&mut app, "xyzzy");

        let tower = app.world().resource::<Tower>();
        let outcomes: Vec<_> = tower
            .sim()
            .scrollback()
            .records()
            .iter()
            .filter_map(|record| record.outcome())
            .collect();
        assert_eq!(outcomes.first(), Some(&Outcome::Unresolved));
        assert!(
            outcomes.iter().skip(1).all(|o| *o == Outcome::Suggestion),
            "{outcomes:?}",
        );
    }

    #[test]
    fn a_resolved_command_waits_for_the_tick_it_will_run_on() {
        // The two clocks: the echo is instant, the effect is tick-aligned.
        let mut app = app();
        type_line(&mut app, "look around");
        assert_eq!(app.world().resource::<Tower>().sim().pending().len(), 1);
    }
}
