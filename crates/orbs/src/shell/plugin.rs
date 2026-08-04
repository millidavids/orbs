//! Registration for the window shell.

use bevy::app::AppExit;
use bevy::input::common_conditions::input_just_pressed;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy::window::WindowResized;

use orbs_render::DEEP_FOCUS_FLOOR;

use super::input::{Line, SubmittedMessage, type_into_line};
use super::linear::Linear;
use super::reveal::Reveal;
use super::screen::{Screen, cycle_mode, spawn_camera, track_window};
use super::transition::PaneTransition;
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
            .init_resource::<Linear>()
            .init_resource::<PaneTransition>()
            .init_resource::<Reveal>()
            .add_message::<SubmittedMessage>()
            .add_systems(Startup, (spawn_camera, track_window).chain())
            .add_systems(
                Update,
                (
                    type_into_line.run_if(on_message::<KeyboardInput>),
                    // Before `submit`, so a keystroke completes the output that
                    // is already on screen rather than the output its own line
                    // is about to produce.
                    finish_reveal.run_if(on_message::<KeyboardInput>),
                    submit.run_if(on_message::<SubmittedMessage>),
                )
                    .chain()
                    .in_set(ShellSystems::Input),
            )
            .add_systems(
                Update,
                (
                    track_window.run_if(on_message::<WindowResized>),
                    // §9 requires the focus mode be overridable at any time,
                    // including mid-siege, so it is a key rather than a
                    // heuristic the player has to fight.
                    cycle_mode.run_if(input_just_pressed(KeyCode::F4)),
                    // §14 makes the linear stream a first-class view of the
                    // frame. Nothing had ever shown it, which is how a stream
                    // that is subtly wrong stays that way.
                    super::linear::toggle.run_if(input_just_pressed(KeyCode::F5)),
                    // §6 requires the parser explain itself, and the Phase 0
                    // gate acts on failure *clustering*. Every reading is kept
                    // as it happens; this is what gets it out to a spreadsheet.
                    export_trace.run_if(input_just_pressed(KeyCode::F6)),
                    // §3's tonal register, until Phase 2 drives it from threat.
                    cycle_register.run_if(input_just_pressed(KeyCode::F7)),
                    // F10, not Escape: the moment there is a text field, Escape
                    // is "clear the line" muscle memory, and quitting the game
                    // mid-sentence is not a recoverable surprise.
                    quit.run_if(input_just_pressed(KeyCode::F10)),
                    // Unconditional: both of these have to keep moving on the
                    // frames where nothing happened, which is most of them.
                    drive_panes,
                    drive_reveal,
                ),
            );
    }
}

/// Let the newest output arrive, a character at a time.
///
/// Reads the scrollback rather than listening for a message: output is produced
/// by the sim on a tick, and the frontend is a *caller* of the sim rather than
/// something it can notify (rule 3). The record count growing is the signal.
fn drive_reveal(tower: Res<Tower>, time: Res<Time>, mut reveal: ResMut<Reveal>) {
    let records = tower.sim().scrollback().records();
    let cells = tail_cells(records, reveal.settled_len());
    reveal.observe(records.len(), cells);
    reveal.advance(time.delta_secs());
}

/// Characters in the records added since `from`.
///
/// Measured through `to_line`, which is what the view draws, so the budget and
/// the drawing agree about what a character is.
fn tail_cells(records: &orbs_render::Records, from: usize) -> u16 {
    let counted: usize = records
        .iter()
        .skip(from)
        .map(|record| record.to_line().chars().count())
        .sum();
    u16::try_from(counted).unwrap_or(u16::MAX)
}

/// Any keystroke puts the whole of the output on screen at once.
///
/// §9's parity rule in practice: waiting must never be something a player is
/// made to do, or the animation stops being flavour and starts being a cost that
/// an accessibility setting could buy its way out of.
fn finish_reveal(mut reveal: ResMut<Reveal>) {
    reveal.finish();
}

/// Aim the pane transition at what the current grid asks for, and let it move.
///
/// The pane count was derived inside `paint` every frame, which is why a pane
/// used to appear between one frame and the next. It is decided here instead so
/// that a *change* in it is something with a beginning.
fn drive_panes(screen: Res<Screen>, time: Res<Time>, mut panes: ResMut<PaneTransition>) {
    // A second pane only where there is room for one. At the 80×22 floor a
    // secondary pane is a four-row strip (§9) — a border, a header and one row —
    // which is why §9 sets `DEEP_FOCUS_FLOOR` in the first place.
    panes.retarget(if screen.grid.fits(DEEP_FOCUS_FLOOR) {
        2
    } else {
        1
    });
    panes.advance(time.delta_secs());
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

/// Where the parse trace is written.
///
/// TSV beside the binary, per §19: no dependency, survives `grep`, pastes into a
/// spreadsheet. One row per *candidate*, not per input, because the gate needs
/// to know whether a miss was the verb or the argument.
const TRACE_PATH: &str = "orbs-parse.tsv";

/// Write the parse trace to disk.
fn export_trace(tower: Res<Tower>) {
    let log = tower.sim().parse_log();
    match std::fs::write(TRACE_PATH, log.to_tsv()) {
        Ok(()) => info!(
            "parse trace -> {TRACE_PATH}: {} inputs, {} resolved, {} forced, {} ambiguous, \
             {} incomplete, {} unresolved",
            log.records().len(),
            log.resolved(),
            log.forced(),
            log.ambiguous(),
            log.incomplete(),
            log.unresolved(),
        ),
        // A failed export must not take the session down with it — the tester
        // whose run it was recording is still playing.
        Err(error) => warn!("parse trace -> {TRACE_PATH} failed: {error}"),
    }
}

/// Step the orb's tonal register.
///
/// Type a command with the register on and the echo comes back in a different
/// face; then `peruse orb.log` and the log lines come back **plain**, because §3
/// exempts the diagnostic surfaces from the eldritch treatment and only from
/// that one. A sabotage tell is not exempt anywhere, which is the asymmetry the
/// whole disjointness rule buys.
fn cycle_register(mut tower: ResMut<Tower>) {
    info!("register: {:?}", tower.cycle_register());
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

    /// Press a key the way winit does.
    ///
    /// Setting `ButtonInput` directly does not work: `keyboard_input_system`
    /// clears it at the start of `PreUpdate` and rebuilds it from the message
    /// stream, so a hand-set press is gone before any `input_just_pressed`
    /// condition runs. Writing the message is the only path that behaves like
    /// the real thing.
    fn press_key(app: &mut App, key_code: KeyCode, logical_key: Key) {
        app.world_mut().write_message(KeyboardInput {
            key_code,
            logical_key,
            state: ButtonState::Pressed,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
        app.update();
    }

    #[test]
    fn f4_switches_focus_and_buys_the_cells_a_second_pane_needs() {
        // §9 requires the focus mode be overridable at any time. The switch is
        // also the only way to reach a second pane at all — `tier_one` sizes the
        // grid to ~80×22 at every window size, so Deep focus stepping fidelity
        // finer is where the cells come from.
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            bevy::input::InputPlugin,
            bevy::window::WindowPlugin {
                primary_window: Some(Window {
                    resolution: bevy::window::WindowResolution::new(2560, 1440),
                    ..default()
                }),
                ..default()
            },
            ShellPlugin,
        ))
        .insert_resource(Tower::new(1));
        app.update();

        let before = *app.world().resource::<Screen>();
        assert!(before.is_hostable(), "the test window must host the floor");

        press_key(&mut app, KeyCode::F4, Key::F4);

        let after = *app.world().resource::<Screen>();
        assert_ne!(after.mode, before.mode, "F4 did not reach `cycle_mode`");
        assert!(
            after.grid.cols > before.grid.cols,
            "focus switched without buying cells: {:?} -> {:?}",
            before.grid,
            after.grid,
        );
        assert!(
            after.grid.fits(orbs_render::DEEP_FOCUS_FLOOR),
            "deep focus still cannot host a second pane: {:?}",
            after.grid,
        );
    }

    #[test]
    fn typing_never_reaches_the_focus_key() {
        // F4 must not also land in the line buffer. `text` is `None` for a
        // function key, which is what keeps the two apart.
        let mut app = app();
        press(&mut app, Key::F4, None);
        press(&mut app, Key::Enter, Some("\r"));
        app.update();
        assert!(messages(&app).is_empty(), "a function key was typed");
    }
}
