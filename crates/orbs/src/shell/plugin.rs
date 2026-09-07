//! Registration for the window shell.

use bevy::input::common_conditions::input_just_pressed;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy::window::WindowResized;

use super::commanding::{
    cycle_register, export_trace, quit, quit_requested, submit, toggle_patient,
};
use super::input::{SubmittedMessage, type_into_line};
use super::reading::{scroll_back, scroll_forward, start_reading, stop_reading};
use super::revealing::{drive_panes, drive_passing, drive_reveal, finish_reveal};
use super::window::{cycle_mode, spawn_camera, track_window};
/// Registration itself names `Tower` only through `resource_changed::<...>`,
/// which is fully qualified; the tests below drive it directly.
#[cfg(test)]
use crate::sim::Tower;
use orbs_shell::{Line, Linear, PaneTransition, Reveal, Screen};

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
    /// Animations advance: the typewriter reveal and the pane transition.
    ///
    /// A set of its own so `repaint` can order **after** it. Both write state
    /// `repaint` then reads, with no edge between them, so on the frame a burst
    /// of output lands the executor could run `repaint` first — drawing the whole
    /// burst complete — and `drive_reveal` second, setting `shown` back to zero.
    /// The next frame the same text vanishes and types itself in: output that
    /// flashes whole and then rewinds. Same class of defect [`Self::Input`]
    /// exists for.
    ///
    /// **And it runs after [`Self::Input`]**, which was missing and cost a
    /// visible defect. The input chain is what opens and shuts every surface —
    /// `wander`, `edit`, `weave`, `unfurl` — and the animations here have to
    /// observe the result, not the state in front of it. Without the edge the
    /// executor was free to drive the crossing first, so a `wander` drew the
    /// **whole maze for one frame** and only then started a transition, which
    /// then departed from the maze it had just arrived at.
    ///
    /// The two sets were ordered against `repaint` and never against each other
    /// — the same shape as the defect recorded one level down for
    /// `motion::advance` and `refresh_panel`, and the third time this project has
    /// paid for a set that orders against its reader but not against its writer.
    Drive,
}

/// Every resource this plugin owns, handed to `$mac` as a list of types.
///
/// # Why the list exists rather than eighteen `init_resource` calls
///
/// **Because the list is read twice.** `build` registers them; `reset_for_swap`
/// puts them back to their defaults when the menu loads a different tower. A
/// hand-written second copy of eighteen types is a copy that drifts, and the
/// failure it drifts into is invisible: a resource left holding the *old* game's
/// state after a swap does not crash, it just quietly lies — `Reveal` holds raw
/// indices into a record stream that no longer exists, `Passing` holds a cell
/// snapshot of a screen from another world.
///
/// So there is one list, in one place, and adding a resource to it registers and
/// resets it in the same edit.
macro_rules! shell_resources {
    ($mac:ident) => {
        $mac!(
            Screen,
            Line,
            orbs_shell::Offered,
            super::input::HeldOver,
            orbs_shell::Ghost,
            orbs_shell::Panel,
            orbs_shell::Scroll,
            super::input::Quiet,
            Linear,
            PaneTransition,
            orbs_shell::Passing,
            Reveal,
            super::Bench,
            super::Editing,
            super::Loom,
            super::Standing,
            super::Walk,
            super::Chorus,
        );
    };
}

/// Put every shell resource back to what it is at startup, for a new tower.
///
/// # Two survive, and they are lifted out rather than left off the list
///
/// - **`Screen`** holds the grid the window actually is. Resetting it would tell
///   the renderer the window had changed size, which it has not.
/// - **`Standing`** is the menu, and the menu is what is asking for this. It
///   closes itself afterwards, on its own terms.
///
/// Taking them out and putting them back — rather than writing a list of
/// sixteen — is what keeps [`shell_resources!`] the single list. A resource
/// added there is reset here by construction, and the two exceptions are named
/// once, here, with the reason.
pub(crate) fn reset_for_swap(world: &mut World) {
    let screen = world.remove_resource::<Screen>();
    let standing = world.remove_resource::<super::Standing>();

    macro_rules! blank {
        ($($resource:ty,)*) => {
            $( world.insert_resource(<$resource>::default()); )*
        };
    }
    shell_resources!(blank);

    if let Some(screen) = screen {
        world.insert_resource(screen);
    }
    if let Some(standing) = standing {
        world.insert_resource(standing);
    }
}

/// The window, the camera, the grid, and the command line.
pub struct ShellPlugin;

impl Plugin for ShellPlugin {
    fn build(&self, app: &mut App) {
        // **The animations run after the input, and that edge was missing.** The
        // input chain is what opens and shuts every surface — `wander`, `edit`,
        // `weave`, `unfurl` — and the clocks in `Drive` have to observe the
        // result rather than the state in front of it. Without it a `wander` drew
        // the whole maze for one frame and only then began a crossing, which then
        // departed from the maze it had just arrived at. See `ShellSystems`.
        app.configure_sets(Update, ShellSystems::Drive.after(ShellSystems::Input));

        // **One list, read twice** — see `shell_resources!`. It is also what
        // `reset_for_swap` puts back when the menu loads a different tower.
        macro_rules! register {
            ($($resource:ty,)*) => {
                $( app.init_resource::<$resource>(); )*
            };
        }
        shell_resources!(register);

        app.add_message::<SubmittedMessage>()
            .add_message::<super::menuing::SwapMessage>()
            // **Ordered into `Input`, so `Drive` sees the world it arrives at.**
            // `Panel` self-heals from the new tower, but only if it is refreshed
            // *after* the swap — and `refresh_panel` is in `Drive`, which is
            // configured `.after(Input)`. Without this edge the executor is free
            // to refresh the panel from the outgoing world and leave it there
            // until the next tick.
            .add_systems(
                Update,
                super::menuing::swap
                    .in_set(ShellSystems::Input)
                    .run_if(on_message::<super::menuing::SwapMessage>),
            )
            .add_systems(Startup, (spawn_camera, track_window).chain())
            // **Not** gated on `booted`, and not in the input set. A focus loss
            // during the boot sequence strands held keys exactly as one during
            // play does, and the guard has to outlive whatever stole the window.
            .add_systems(
                Update,
                super::input::forget_held_keys.run_if(on_message::<bevy::window::WindowFocused>),
            )
            // The panel only moves when the world does — see `Panel`. **Not**
            // gated on `booted`: `Tower` is marked changed when it is inserted,
            // and that is the frame that fills the panel for the starting room.
            // Behind the boot gate the flag has long expired by the time the
            // sequence ends, leaving the panel blank until the next tick.
            .add_systems(
                Update,
                super::input::refresh_panel
                    .in_set(ShellSystems::Drive)
                    .run_if(resource_changed::<crate::sim::Tower>),
            )
            .add_systems(
                Update,
                (
                    // **Exactly one surface takes a keystroke.** The prompt, the
                    // editor and — since `unfurl` — the transcript are all on
                    // screen at once, and the failure where two consume a key is
                    // invisible until a player types `:wq` and finds it in their
                    // command history.
                    //
                    // The editor is gated here; the prompt decides *inside*
                    // itself, because it also has to discard what it declines.
                    // See `input::type_into_line`.
                    // **First in the chain**, so both text fields read the gap
                    // in front of this frame's keystroke rather than zero.
                    super::input::watch_quiet,
                    super::editing::open_requested.run_if(resource_changed::<crate::sim::Tower>),
                    super::editing::type_into_editor
                        .run_if(on_message::<KeyboardInput>)
                        .run_if(super::editing::editing),
                    // **After** the keys, so a keystroke restarts the settle
                    // clock before it is advanced rather than after — otherwise
                    // the frame a player types on counts toward the pause they
                    // have not taken yet.
                    super::editing::autosave.run_if(super::editing::editing),
                    // The weave screen, on the same terms: gated by a run
                    // condition because it *consumes* keys, while the prompt
                    // below always runs because it has to discard them.
                    super::weaving::open_requested.run_if(resource_changed::<crate::sim::Tower>),
                    super::weaving::type_into_loom
                        .run_if(on_message::<KeyboardInput>)
                        .run_if(super::weaving::weaving),
                    // **The keys first, then the opening** — and this order is a
                    // shipped defect, not a preference. `type_into_menu` is
                    // ungated for the reason `type_into_line` is: a reader that
                    // does not run keeps its cursor, so a gated one read the
                    // word that opened the menu straight back into it, reached
                    // the menu's own `quit`, and left the orb. Running it here,
                    // before `open_requested`, means the opening frame is one it
                    // has already emptied. See `menuing`'s module doc.
                    super::menuing::type_into_menu.run_if(on_message::<KeyboardInput>),
                    super::menuing::open_requested.run_if(resource_changed::<crate::sim::Tower>),
                    // **Unconditional while it is open**, like `autosave`: the
                    // world ticks behind the screen, so a threshold crossed
                    // while a player is looking should land while they look.
                    super::weaving::refresh.run_if(super::weaving::weaving),
                    // The stacks' arrows, on the same terms as the two
                    // above. It owns no pane — the map draws whether or not
                    // anybody said the word — so what is gated here is only the
                    // keyboard.
                    super::wandering::open_requested.run_if(resource_changed::<crate::sim::Tower>),
                    super::wandering::type_into_maze
                        .run_if(on_message::<KeyboardInput>)
                        .run_if(super::wandering::walking),
                    // Unconditional while it is open, like `autosave` and the
                    // loom's `refresh`: a spell can close the maze from under
                    // the player, and holding the keyboard over a pane with no
                    // map on it is the worst of the three ways that ends.
                    super::wandering::close_when_gone.run_if(super::wandering::walking),
                    // The menagerie's, on exactly the same three terms. It owns
                    // no pane either — the figure draws whether or not anybody
                    // said `chorus` — and it closes itself for a sharper reason
                    // than the maze does: a chant ends *on its own*, so a player
                    // left holding the arrows over nothing would have a dead
                    // prompt and no way to discover why.
                    super::chorusing::open_requested.run_if(resource_changed::<crate::sim::Tower>),
                    super::chorusing::type_into_chant
                        .run_if(on_message::<KeyboardInput>)
                        .run_if(super::chorusing::chorusing),
                    super::chorusing::close_when_gone.run_if(super::chorusing::chorusing),
                    // **Ungated, and after every surface that can let go.** It
                    // watches for the keyboard changing hands, which is an edge
                    // `type_into_line` cannot see for itself — that system is
                    // gated on a keystroke arriving, so the frames where a surface
                    // held the keyboard in silence are invisible to it. See
                    // `HeldOver`: this is what stops a held arrow putting `wander`
                    // back in the prompt on the way out of the maze.
                    super::input::watch_focus,
                    // **No `not_editing` here.** It has to *run* to throw the
                    // keystrokes away — a reader that never runs keeps its
                    // cursor, and everything typed while another surface had the
                    // keyboard arrived the instant this did. The check moved
                    // inside; see `type_into_line`.
                    type_into_line.run_if(on_message::<KeyboardInput>),
                    // Before `submit`, so a keystroke completes the output that
                    // is already on screen rather than the output its own line
                    // is about to produce.
                    finish_reveal.run_if(on_message::<KeyboardInput>),
                    submit.run_if(on_message::<SubmittedMessage>),
                    // After `submit`, so the frame that clears the line clears
                    // the suggestion with it. Gated on what it actually depends
                    // on — see `Ghost`.
                    super::input::suggest.run_if(
                        resource_changed::<Line>.or_else(resource_changed::<crate::sim::Tower>),
                    ),
                )
                    .chain()
                    .in_set(ShellSystems::Input)
                    // Nothing typed reaches the line until the game is up: there
                    // is no input line during the sequence, and §4 draws no
                    // prompt there. (This used to be justified by the keypress
                    // that skipped boot needing not to be the first letter of a
                    // command. That skip is gone; the guard is not vestigial.)
                    .run_if(crate::boot::booted),
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
                    // **In the input set, like every other surface switch.** It
                    // was the one writer of `Showing`'s inputs left outside it,
                    // so `Drive.after(Input)` did not reach it: on the wrong
                    // interleaving the mirror painted whole for a frame and only
                    // then crossed — the same defect that edge was added for,
                    // one surface over.
                    orbs_shell::toggle_linear
                        .in_set(ShellSystems::Input)
                        .run_if(input_just_pressed(KeyCode::F5)),
                    // §6 requires the parser explain itself, and the Phase 0
                    // gate acts on failure *clustering*. Every reading is kept
                    // as it happens; this is what gets it out to a spreadsheet.
                    export_trace.run_if(input_just_pressed(KeyCode::F6)),
                    // §3's tonal register, until Phase 8 drives it from threat.
                    cycle_register.run_if(input_just_pressed(KeyCode::F7)),
                    // **`F9`, not `F8`** — `F8` is the greyscale accommodation
                    // and these two are neighbours in what they are for, which
                    // is exactly why they must not be neighbours a finger can
                    // slip between. Both join the settings screen in Phase 13.
                    toggle_patient.run_if(input_just_pressed(KeyCode::F9)),
                    // F10, not Escape: the moment there is a text field, Escape
                    // is "clear the line" muscle memory, and quitting the game
                    // mid-sentence is not a recoverable surprise.
                    quit.run_if(input_just_pressed(KeyCode::F10)),
                    // **Gated on the world having moved**, like the other
                    // handshakes: `submit` resolves the verb and marks `Tower`
                    // changed, so this runs on that frame and no other. The flag
                    // it takes is only ever set by a *confirmed* `quit` — the
                    // sim asks first, and `menu` is a different word now.
                    quit_requested.run_if(resource_changed::<crate::sim::Tower>),
                    // **PageUp/PageDown, not the arrows.** Up and Down walk the
                    // command history (§19) and must keep doing so — a shell
                    // where Up sometimes scrolls and sometimes recalls is a shell
                    // you cannot type in without looking.
                    scroll_back.run_if(input_just_pressed(KeyCode::PageUp)),
                    scroll_forward.run_if(input_just_pressed(KeyCode::PageDown)),
                    // **The arrows scroll only while reading.** At the prompt
                    // they walk the command history and must keep doing so — a
                    // shell where Up sometimes scrolls and sometimes recalls is
                    // a shell you cannot type in without looking. Inside the
                    // mode there is no history to walk, so they are free.
                    scroll_back
                        .run_if(input_just_pressed(KeyCode::ArrowUp))
                        .run_if(super::editing::reading),
                    scroll_forward
                        .run_if(input_just_pressed(KeyCode::ArrowDown))
                        .run_if(super::editing::reading),
                    // Escape leaves, exactly as it leaves the editor's buffer.
                    stop_reading
                        .run_if(input_just_pressed(KeyCode::Escape))
                        .run_if(super::editing::reading),
                    start_reading.run_if(resource_changed::<crate::sim::Tower>),
                    // Unconditional: all three of these have to keep moving on
                    // the frames where nothing happened, which is most of them.
                    drive_panes.in_set(ShellSystems::Drive),
                    // **After `refresh_panel`, explicitly**, for the reason
                    // `motion::advance` below is: `Showing` reads `Panel::room`,
                    // and a set orders both against `repaint` rather than
                    // against each other. Left to the executor this would see
                    // the previous frame's panel and start every crossing a
                    // frame late.
                    drive_passing
                        .in_set(ShellSystems::Drive)
                        .after(super::input::refresh_panel)
                        // **After `motion::advance`, which is the other writer.**
                        // Both hold `ResMut<Passing>`, so Bevy already serialises
                        // them — but *which order* was left to the executor, and
                        // they are not interchangeable: `motion` carries the
                        // tube's switch and this carries the clock. Turning `F3`
                        // back on, the unordered pair started a crossing a frame
                        // late; turning it off, one interleaving started a
                        // crossing that the other immediately cleared.
                        //
                        // Both outcomes are invisible, and the edge is here
                        // anyway — this file records three defects that were
                        // exactly "a set ordered against its reader and not its
                        // writer", and a fourth left in on the grounds that it
                        // does not show yet is how the fifth arrives.
                        .after(super::motion::advance),
                    drive_reveal.in_set(ShellSystems::Drive),
                    // §10.1's instruments animate on wall-clock time, not on the
                    // tick — the sim must not be able to observe it, or replay
                    // would depend on how long a frame took. See `shell::bench`.
                    //
                    // **After `refresh_panel`, explicitly.** It reads `Panel` to
                    // catch the two edges it animates — a hearth lighting, a
                    // bowl filling — and `refresh_panel` is what writes it. Both
                    // were merely `in_set(Drive)`, which orders them against
                    // `repaint` and not against each other, so the executor was
                    // free to run this first and see the *previous* frame's
                    // panel. That is the same class of defect `Drive` itself
                    // exists for, one level down.
                    //
                    // **Gated on `booted` with the rest, deliberately.** The
                    // panel is not on screen during the sequence, and the clock
                    // starting at zero when the game appears is what anyone
                    // would want. The edges cost nothing either: `Bench` seeds
                    // `was_lit` and `was_charged` *true* precisely so a tower
                    // that opens with a fire already going does not flare on the
                    // first frame it is looked at.
                    super::motion::advance
                        .in_set(ShellSystems::Drive)
                        .after(super::input::refresh_panel),
                )
                    // Every key here is guarded: none of them means anything
                    // before the world runs, and `F6` would write a trace of a
                    // session that has not happened. (`F10` was singled out when
                    // a keypress skipped boot — quitting and skipping in one
                    // keystroke. That skip is gone; the guard still earns its
                    // place.)
                    .run_if(crate::boot::booted),
            );
    }
}

// `page_rows` and `page_step` — how many *records* a screenful of transcript
// holds — moved to `orbs_shell::page_step`. It is not a keyboard question: it is
// what the transcript would fit, measured with the same `RecordView` the
// transcript is drawn with, and a second measure of the same stream would page
// by a different amount than it showed. This frontend's wrapper is in
// `reading.rs` with the keys that use it.

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
            logical_key: logical_key.clone(),
            state: ButtonState::Pressed,
            text: text.map(Into::into),
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
        // **And released, which is what `tap` next door already knew.** Its doc
        // says it outright: a key left down means the *next* press of it is not a
        // fresh one. Because `press` stamps every keystroke `KeyCode::KeyA`, one
        // unreleased press left `A` held for the rest of the test — invisible while
        // nothing read `key_code`, and a false failure the moment `watch_focus`
        // did. A keystroke is a press and a release; modelling half of one is what
        // made a correct fix look broken.
        app.world_mut().write_message(KeyboardInput {
            key_code: KeyCode::KeyA,
            logical_key,
            state: ButtonState::Released,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
    }

    /// Press a **physical** key, the way `input_just_pressed` sees one.
    ///
    /// [`press`] hardcodes `key_code` because the text field reads `text` and
    /// `logical_key` and nothing else — but the F-keys, Escape and the paging
    /// keys are all bound through `ButtonInput`, which `keyboard_input_system`
    /// fills from `key_code` alone. A test using `press` for those presses `A`
    /// forever and the binding never fires.
    fn tap(app: &mut App, key_code: KeyCode, logical_key: Key) {
        app.world_mut().write_message(KeyboardInput {
            key_code,
            logical_key,
            state: ButtonState::Pressed,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
        app.update();
        // Released on the way out, or `input_just_pressed` sees it held and the
        // *next* tap of the same key is not a fresh press.
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(key_code);
    }

    /// Hold a modifier down and let the keyboard fall silent long enough that
    /// the chord can only be a ghost.
    ///
    /// **The silence is the evidence.** The first version of this fix read the
    /// keystroke's `text` instead, on the belief that macOS hands back none
    /// under a chord — winit 0.30.13 `platform_impl/macos/event.rs:154` sets
    /// `text` from `logical_key.to_text()` with no modifier check at all, so
    /// `Cmd+A` carries `Some("a")` and that fix typed a letter into the prompt
    /// on every copy, paste and select-all.
    fn ghost(app: &mut App, key: KeyCode) {
        hold(app, key);
        app.world_mut()
            .resource_mut::<super::super::input::Quiet>()
            .silent_for(30.0);
    }

    /// Hold a modifier down, the way winit reports one.
    fn hold(app: &mut App, key: KeyCode) {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(key);
    }

    #[test]
    fn unfurl_hands_the_transcript_the_keyboard_and_escape_hands_it_back() {
        // **The word exists because the key could not be discovered.** `PageUp`
        // has scrolled the transcript since the transcript existed, and the
        // border advertised `PgDn newest` only once you were *already* scrolled
        // back — an affordance that announced itself exclusively to players who
        // had found it. In a game with no mouse, that is no affordance.
        //
        // The half that must not regress is the exit: Escape means the same
        // thing here as in the editor, or the player is stuck in a mode with no
        // way out and nothing on screen to type into.
        let mut app = app();
        assert!(
            !app.world().resource::<orbs_shell::Scroll>().is_reading(),
            "the transcript had the keyboard before anyone asked",
        );

        type_only(&mut app, "unfurl");
        press(&mut app, Key::Enter, Some("\r"));
        app.update();
        // The write lands on a tick, and the mode is entered from the request.
        app.world_mut().resource_mut::<Tower>().step();
        app.update();
        assert!(
            app.world().resource::<orbs_shell::Scroll>().is_reading(),
            "`unfurl` did not hand over the keyboard",
        );

        tap(&mut app, KeyCode::Escape, Key::Escape);
        assert!(
            !app.world().resource::<orbs_shell::Scroll>().is_reading(),
            "escape did not return to the prompt",
        );
    }

    #[test]
    fn the_prompt_is_deaf_while_the_transcript_is_being_read() {
        // Three surfaces can own the keyboard now — the prompt, the editor and
        // the transcript — and the failure where two consume a key is invisible
        // until a player types `:wq` and finds it in their command history.
        //
        // Asserted on the **line**, not on the run conditions: a predicate that
        // is correct and not wired to anything reads exactly like one that
        // works, and the run conditions were where this could go wrong.
        let mut app = app();
        app.world_mut().resource_mut::<Line>().clear();
        app.world_mut().resource_mut::<orbs_shell::Scroll>().read();

        type_only(&mut app, "survey");
        assert_eq!(
            app.world().resource::<Line>().viewport(80).0,
            "",
            "typing reached the prompt while the transcript had the keyboard",
        );

        // ...and it comes back the moment reading ends.
        app.world_mut()
            .resource_mut::<orbs_shell::Scroll>()
            .stop_reading();
        type_only(&mut app, "survey");
        assert_eq!(
            app.world().resource::<Line>().viewport(80).0,
            "survey",
            "the prompt did not get the keyboard back",
        );
    }

    #[test]
    fn a_modifier_stuck_with_no_focus_event_still_lets_you_type() {
        // **The fix above was not enough, and this is the test that says why.**
        // It asserts recovery *without* a `WindowFocused`, because the reported
        // failure — `Cmd+Shift+Ctrl+4` — leaves no focus event to hang a fix on.
        //
        // Every recovery path in Bevy 0.19 hangs off exactly that event:
        // `WindowFocused(false)` → `check_keyboard_focus_lost` →
        // `KeyboardFocusLost` → `release_all`. And Bevy drops winit's
        // `ModifiersChanged`, which is the OS saying what is *really* down. So
        // when a key-up is swallowed silently there is no mechanism anywhere to
        // notice, and the field is dead for the rest of the session.
        //
        // The evidence that breaks the deadlock is the keystroke itself: the OS
        // gave us text, so the OS is not treating this as a command.
        let mut app = app();
        ghost(&mut app, KeyCode::SuperLeft);
        ghost(&mut app, KeyCode::ControlLeft);

        type_only(&mut app, "survey");
        assert_eq!(
            app.world().resource::<Line>().viewport(80).0,
            "survey",
            "a ghost modifier ate every keystroke, with no focus event to clear it",
        );
        assert!(
            !app.world()
                .resource::<ButtonInput<KeyCode>>()
                .any_pressed([KeyCode::SuperLeft, KeyCode::ControlLeft]),
            "the ghost survived the keystroke that disproved it",
        );
    }

    #[test]
    fn the_editor_recovers_from_a_ghost_modifier_too() {
        // **The surface it was reported on.** The prompt and the editor share
        // the guard, which is why they shared the freeze — and a fix tested only
        // on the prompt would have been half a fix, exactly as the focus hook
        // was.
        let mut app = app();
        app.world_mut()
            .resource_mut::<crate::shell::Editing>()
            .open(crate::shell::Editor::open("x.spell", "laboratory", &[]));
        ghost(&mut app, KeyCode::SuperLeft);

        // `edit` drops into the buffer, then a word into the spell. Both are
        // ordinary typing, and both were being eaten.
        type_only(&mut app, "edit");
        press(&mut app, Key::Enter, Some("\r"));
        app.update();
        type_only(&mut app, "survey");

        let mut editing = app.world_mut().resource_mut::<crate::shell::Editing>();
        let editor = editing.get_mut().expect("the editor closed");
        assert_eq!(
            editor.lines(),
            ["survey"],
            "a ghost modifier froze the editor",
        );
    }

    #[test]
    fn a_chord_in_use_is_still_a_chord() {
        // **The counterweight, and the regression the first fix shipped.** That
        // version treated "the key came with text" as proof no chord was held —
        // but winit 0.30.13 sets `text` from `logical_key.to_text()` with no
        // modifier check (`platform_impl/macos/event.rs:154`), so `Cmd+A` and
        // `Cmd+V` carry `Some("a")`/`Some("v")` and every copy, paste and
        // select-all typed a letter into the prompt.
        //
        // A chord a person is holding is used within moments of being pressed,
        // so no silence has elapsed and the guard stands.
        let mut app = app();
        hold(&mut app, KeyCode::SuperLeft);

        type_only(&mut app, "a");
        assert_eq!(
            app.world().resource::<Line>().viewport(80).0,
            "",
            "`Cmd+A` typed its letter into the prompt",
        );

        press(&mut app, Key::Enter, Some("\r"));
        app.update();
        assert!(
            app.world()
                .resource::<ButtonInput<KeyCode>>()
                .pressed(KeyCode::SuperLeft),
            "a chord in use was mistaken for a ghost",
        );
        assert_eq!(
            app.world().resource::<Line>().viewport(80).0,
            "",
            "`Cmd+Enter` reached the prompt",
        );
    }

    #[test]
    fn losing_the_window_forgets_held_modifiers() {
        // **The bug this test exists for.** `Cmd+Shift+Ctrl+4` on macOS hands
        // the window to the screenshot overlay mid-chord, so the *release* for
        // Cmd and Ctrl is delivered to that overlay and never to us. Held state
        // then says they are down forever, every keystroke after it hits the
        // chord guard, and the prompt is dead with nothing on screen to say why.
        let mut app = app();
        hold(&mut app, KeyCode::SuperLeft);
        hold(&mut app, KeyCode::ControlLeft);

        app.world_mut().write_message(bevy::window::WindowFocused {
            window: Entity::PLACEHOLDER,
            focused: false,
        });
        app.update();

        assert!(
            !app.world()
                .resource::<ButtonInput<KeyCode>>()
                .any_pressed([KeyCode::SuperLeft, KeyCode::ControlLeft]),
            "a stolen window left its modifiers held"
        );

        // And typing works again, which is the thing the player noticed.
        type_only(&mut app, "survey");
        assert_eq!(app.world().resource::<Line>().viewport(80).0, "survey");
    }

    #[test]
    fn a_chord_does_not_reach_enter_or_backspace() {
        // Both bypassed the chord guard, so `Cmd+Enter` submitted the line on
        // its way to the operating system.
        let mut app = app();
        type_only(&mut app, "survey");

        hold(&mut app, KeyCode::SuperLeft);
        press(&mut app, Key::Enter, Some("\r"));
        press(&mut app, Key::Backspace, Some("\u{8}"));
        app.update();

        assert_eq!(
            app.world().resource::<Line>().viewport(80).0,
            "survey",
            "a chord edited or submitted the line"
        );
    }

    /// Type `line` without submitting it, for tests that inspect the buffer.
    fn type_only(app: &mut App, line: &str) {
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
        app.update();
    }

    /// Type `line` and press Enter.
    fn type_line(app: &mut App, line: &str) {
        type_only(app, line);
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
        // `Escape` is no longer among them — it clears the line now, which is
        // the muscle memory `plugin.rs` moved quit off `Esc` to make room for.
        // Tab now *completes* rather than doing nothing, so what it must not do
        // is leave its own `\t` behind — asserting an exact line here would be
        // asserting the completer's answer instead.
        let mut tabbed = app();
        press(&mut tabbed, Key::Tab, Some("\t"));
        tabbed.update();
        assert!(
            !tabbed.world().resource::<Line>().text().contains('\t'),
            "a tab reached the buffer"
        );

        let mut app = app();
        press(&mut app, Key::Character("a".into()), Some("a"));
        press(&mut app, Key::Enter, Some("\r"));
        app.update();
        assert_eq!(messages(&app).first().map(String::as_str), Some("a"));
    }

    #[test]
    fn tab_extends_as_far_as_the_candidates_agree() {
        let mut app = app();
        type_line(&mut app, "attend laboratory");
        type_only(&mut app, "wield mo");
        press(&mut app, Key::Tab, Some("\t"));
        app.update();

        assert_eq!(
            app.world().resource::<Line>().viewport(80).0,
            "wield mortar_and_pestle ",
            "a lone candidate should complete and leave a space"
        );
    }

    #[test]
    fn tab_lists_when_extending_would_add_nothing() {
        // readline's `show-all-if-ambiguous`. The candidates are **transient
        // Frame content**, never a record: a Tab press is not a submission, and
        // a frontend writing to the log breaks replay by construction.
        let mut app = app();
        type_line(&mut app, "attend laboratory");
        type_only(&mut app, "wield ");
        press(&mut app, Key::Tab, Some("\t"));
        app.update();

        let offered = &app.world().resource::<orbs_shell::Offered>().options;
        assert!(offered.len() > 1, "expected a list, got {offered:?}");
        assert!(offered.iter().any(|name| name == "alembic"), "{offered:?}");
    }

    #[test]
    fn repeated_tab_cycles_through_the_candidates() {
        // readline's `menu-complete`. The first press says there is more than one
        // answer; every press after that has to *choose* one, or the list is a
        // dead end that leaves the player typing the name out by hand.
        let mut app = app();
        type_line(&mut app, "attend laboratory");
        type_only(&mut app, "wield ");

        // The first press lists and leaves the line alone (bash's default).
        press(&mut app, Key::Tab, Some("\t"));
        app.update();
        assert_eq!(
            app.world().resource::<Line>().text(),
            "wield ",
            "the first Tab changed the line instead of listing"
        );

        // Every press after it chooses.
        let mut seen = Vec::new();
        for _ in 0..3 {
            press(&mut app, Key::Tab, Some("\t"));
            app.update();
            seen.push(app.world().resource::<Line>().text().to_owned());
        }
        assert!(
            seen[0] != seen[1] && seen[1] != seen[2],
            "Tab did not move through the options: {seen:?}"
        );
        for line in &seen {
            assert!(line.starts_with("wield "), "{line:?}");
        }
        // ...and the list says which one the line is holding.
        assert_eq!(
            app.world().resource::<orbs_shell::Offered>().current,
            Some(2),
            "the listing does not mark where the cycle has reached"
        );
    }

    #[test]
    fn typing_ends_the_tab_cycle() {
        // Otherwise the next Tab resumes a completion the player has typed past,
        // and splices a candidate into the middle of a word.
        let mut app = app();
        type_line(&mut app, "attend laboratory");
        type_only(&mut app, "wield ");
        press(&mut app, Key::Tab, Some("\t"));
        app.update();

        type_only(&mut app, "a");
        assert_eq!(
            app.world().resource::<orbs_shell::Offered>().current,
            None,
            "the cycle survived a keystroke"
        );
    }

    #[test]
    fn the_ghost_prefers_history_over_completion() {
        // fish's order and zsh's default: history is the cheap half and usually
        // the right one, because a brew loop is a player repeating themselves.
        let mut app = app();
        type_line(&mut app, "attend laboratory");
        type_line(&mut app, "wield athanor");

        type_only(&mut app, "wield a");
        let scene_open = {
            let tower = app.world().resource::<Tower>();
            (
                tower.sim().scene().clone(),
                !tower.sim().choices().is_empty(),
            )
        };
        let ghost = app
            .world()
            .resource::<Line>()
            .ghost(&scene_open.0, scene_open.1);
        assert_eq!(
            ghost, "thanor",
            "history should win: completion alone would offer `alembic` too"
        );
    }

    #[test]
    fn escape_clears_the_line() {
        let mut app = app();
        type_only(&mut app, "survey");
        press(&mut app, Key::Escape, Some("\u{1b}"));
        app.update();

        assert_eq!(app.world().resource::<Line>().viewport(80).0, "");
    }

    #[test]
    fn the_caret_moves_and_edits_land_where_it_is() {
        let mut app = app();
        type_only(&mut app, "srvey");

        // Walk back to just after the `s` and put the missing `u` in.
        for _ in 0..4 {
            press(&mut app, Key::ArrowLeft, None);
        }
        app.update();
        type_only(&mut app, "u");

        assert_eq!(app.world().resource::<Line>().viewport(80).0, "survey");
    }

    #[test]
    fn up_walks_history_filtered_by_what_is_typed() {
        // fish's default and zsh's `history-beginning-search-backward`: with a
        // brew loop repeating itself, typing `wield ` and pressing Up should
        // walk the wields, not the whole log.
        let mut app = app();
        for line in [
            "survey",
            "wield mortar_and_pestle",
            "status",
            "wield alembic",
        ] {
            type_line(&mut app, line);
        }

        type_only(&mut app, "wield ");
        press(&mut app, Key::ArrowUp, None);
        app.update();
        assert_eq!(
            app.world().resource::<Line>().viewport(80).0,
            "wield alembic"
        );

        press(&mut app, Key::ArrowUp, None);
        app.update();
        assert_eq!(
            app.world().resource::<Line>().viewport(80).0,
            "wield mortar_and_pestle",
            "the second Up searched the recalled line instead of the anchor"
        );

        // Walking forward past the newest restores what was being typed.
        press(&mut app, Key::ArrowDown, None);
        press(&mut app, Key::ArrowDown, None);
        app.update();
        assert_eq!(app.world().resource::<Line>().viewport(80).0, "wield ");
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
    fn f4_switches_focus_without_moving_the_grid() {
        // §9 requires the focus mode be overridable at any time, so it is a key
        // rather than a heuristic, and this is the end-to-end proof the key
        // reaches `cycle_mode`.
        //
        // **It used to assert that Deep focus bought cells**, because the grid
        // was derived from the window and Deep dropped a fidelity tier to make
        // room for a second pane. §19 fixed the grid; there is one grid now, and
        // both modes have room for every pane §9 allows. What survives is the
        // half that was always the point — the split changes — plus the new
        // guarantee that nothing else does.
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

        assert_eq!(
            after.grid, before.grid,
            "F4 moved the grid, which is what a fixed grid exists to prevent",
        );
        assert_eq!(after.scale(), before.scale(), "F4 resized the glyphs");
        assert!(
            after.grid.fits(orbs_render::DEEP_FOCUS_FLOOR),
            "the fixed grid cannot host a second pane: {:?}",
            after.grid,
        );
        // ...and what actually changed: the split the two modes ask the tiler
        // for. Without this the test would pass on a `cycle_mode` that only set
        // a field nobody reads.
        //
        // **Two panes explicitly, not `PANES`.** `PANES` is 1 since the tower
        // rail replaced the telemetry pane, and one pane tiles identically in
        // both modes — so asking about the live count made this assert that `F4`
        // does something it currently cannot. What it is really holding is that
        // **the two modes ask the tiler for different shapes**, which is a
        // property of `cycle_mode` and the tiler and is as true today as it will
        // be when multiplexing puts the second pane back (Phase 11a).
        //
        // The half that is genuinely lost — that `F4` changes what is on screen
        // *right now* — is recorded on `PANES` rather than asserted here,
        // because a test cannot hold a claim the game has stopped making.
        let request = |screen: Screen| orbs_render::ScreenRequest {
            main_panes: 2,
            mode: screen.mode,
            ..orbs_render::ScreenRequest::single(screen.grid)
        };
        assert_ne!(
            orbs_render::ScreenLayout::compute(&request(before)).main(),
            orbs_render::ScreenLayout::compute(&request(after)).main(),
            "the two focus modes tile identically, so F4 could never do anything",
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

    /// Press or release a key carrying its **real** `key_code`.
    ///
    /// `press` above stamps every keystroke `KeyCode::KeyA` on the stated grounds
    /// that `type_into_line` does not read the field. `watch_focus` does — it has
    /// to, because what it tracks is which *physical* key is still down — so a test
    /// about it that used `press` would put every key in one bucket and pass
    /// against a fix that swallowed the keyboard for ever.
    fn physical(app: &mut App, key_code: KeyCode, logical_key: Key, state: ButtonState) {
        app.world_mut().write_message(KeyboardInput {
            key_code,
            logical_key,
            state,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
    }

    #[test]
    fn leaving_the_maze_leaves_nothing_in_the_prompt() {
        let mut app = app();
        type_line(&mut app, "attend archive");
        app.world_mut().resource_mut::<Tower>().step();
        app.update();
        type_line(&mut app, "research");
        app.world_mut().resource_mut::<Tower>().step();
        app.update();
        type_line(&mut app, "wander");
        app.world_mut().resource_mut::<Tower>().step();
        app.update();
        assert!(
            app.world().resource::<crate::shell::Walk>().is_open(),
            "the arrows never took the maze, so this asserts nothing",
        );

        // Walking: the arrow goes down and **stays** down, which is the state the
        // bug needs. Nothing releases it before the Escape.
        physical(
            &mut app,
            KeyCode::ArrowUp,
            Key::ArrowUp,
            ButtonState::Pressed,
        );
        app.update();

        physical(&mut app, KeyCode::Escape, Key::Escape, ButtonState::Pressed);
        app.update();
        assert!(!app.world().resource::<crate::shell::Walk>().is_open());

        // **The held arrow outliving the Escape is how the word came back.** Key
        // repeat keeps delivering while a finger is on the key, and by this frame
        // the maze has let go — so the arrow reaches the prompt, where an arrow
        // means *recall history*, and the newest entry is the `wander` that opened
        // the maze.
        physical(
            &mut app,
            KeyCode::ArrowUp,
            Key::ArrowUp,
            ButtonState::Pressed,
        );
        app.update();

        assert_eq!(
            app.world().resource::<Line>().text(),
            "",
            "the prompt kept the word that opened the maze",
        );
        assert!(
            app.world().resource::<crate::shell::Ghost>().0.is_empty(),
            "a suggestion outlived the line it trailed",
        );
        assert!(
            app.world().resource::<crate::shell::Offered>().is_empty(),
            "a Tab listing outlived the maze",
        );

        // **And the other half, which is what makes the fix a fix.** Let go of the
        // arrow and press it again: that is a real keystroke and history recall must
        // still work. A version that swallowed the keyboard from the moment a
        // surface closed would pass everything above and break `Up` for ever.
        physical(
            &mut app,
            KeyCode::ArrowUp,
            Key::ArrowUp,
            ButtonState::Released,
        );
        app.update();
        physical(
            &mut app,
            KeyCode::ArrowUp,
            Key::ArrowUp,
            ButtonState::Pressed,
        );
        app.update();
        assert_eq!(
            app.world().resource::<Line>().text(),
            "wander",
            "a fresh press of Up no longer recalls history",
        );
    }

    /// Every surface swallows the keyboard, and the prompt gets nothing.
    ///
    /// **The regression test for `orbs_shell::Focus` itself**, and for the defect
    /// its module comment describes: a surface whose term was forgotten does not
    /// fail loudly, it *"types into an invisible prompt while the player looks at
    /// something else, and the characters arrive later."* Nothing asserted that
    /// before — `leaving_the_maze_leaves_nothing_in_the_prompt` covers one
    /// surface and only the handoff *out* of it.
    ///
    /// It is written as a loop over the four on purpose: a fifth added without
    /// its arm is then a missing row in a table rather than a test nobody
    /// remembered to write.
    #[test]
    fn no_surface_lets_a_keystroke_reach_the_prompt() {
        for (surface, opening) in [
            // **A spell is written *for* a domain**, so `scribe` from the tower
            // landing opens nothing. Getting that wrong is what the "asserts
            // nothing" guard below is for, and it caught it on the first run.
            ("editor", &["attend laboratory", "scribe drill"][..]),
            ("weave", &["weave"][..]),
            ("maze", &["attend archive", "research", "wander"][..]),
            ("reading", &["unfurl"][..]),
            // **The fifth, which this table's own doc promised would be a row.**
            // It was added as a surface and not as a row, so the regression this
            // test exists to prevent went unasserted for the only surface the
            // phase introduced — which is the failure the doc describes, made by
            // the person who wrote the doc.
            ("chant", &["attend menagerie", "summon", "chorus"][..]),
        ] {
            let mut app = app();
            for line in opening {
                type_line(&mut app, line);
                app.world_mut().resource_mut::<Tower>().step();
                app.update();
            }
            assert!(
                focus_of(&app).is_elsewhere(),
                "{surface} never took the keyboard, so this asserts nothing",
            );

            type_only(&mut app, "zzz");
            app.update();
            assert_eq!(
                app.world().resource::<Line>().text(),
                "",
                "{surface} let typing through to the prompt",
            );
        }
    }

    /// What the app's surfaces answer, as they stand.
    ///
    /// **Built from the resources rather than run through the `SystemParam`**,
    /// The word, through the real plugin stack, to the real menu.
    ///
    /// # Why this test and not the dump
    ///
    /// **`ORBS_DUMP` proved the wrong thing.** It goes through
    /// `orbs_shell::dump`, which builds no `App` and takes the handshake itself
    /// — so it drew a menu while the live build's route to one was never
    /// exercised. The whole point of `menuing::open_requested` is that it is a
    /// *system*, with a run condition, in a schedule; none of that is reachable
    /// from a still photograph.
    #[test]
    fn the_word_menu_opens_the_menu() {
        let mut app = app();
        type_line(&mut app, "menu");
        // **The effect lands on the tick, not at `submit`.** `submit` echoes and
        // queues; every verb's effect runs at the next `step`, which is what
        // keeps effects tick-aligned — so a check before this one finds the flag
        // unset, which is the mistake `quit` shipped with once already.
        app.world_mut().resource_mut::<Tower>().step();
        app.update();

        assert!(
            app.world().resource::<crate::shell::Standing>().is_open(),
            "`menu` did not open the menu",
        );
        assert_eq!(focus_of(&app), orbs_shell::Focus::Menu);
        assert!(
            app.world()
                .resource::<Messages<bevy::app::AppExit>>()
                .is_empty(),
            "opening the menu left the orb",
        );
    }

    /// `quit` asks once, and the second one goes.
    ///
    /// # The behaviour this replaces
    ///
    /// For one iteration `quit` opened the menu and leaving was a choice made
    /// there — so a player who wanted to stop had to learn that stopping was two
    /// steps through a screen they had not asked for. **Superseded** (§19):
    /// `quit` leaves, and the two steps are a *question* instead, which is what
    /// every other refusal in the game already looks like.
    #[test]
    fn quit_asks_once_and_the_second_one_leaves() {
        let mut app = app();
        type_line(&mut app, "quit");
        app.world_mut().resource_mut::<Tower>().step();
        app.update();
        assert!(
            app.world()
                .resource::<Messages<bevy::app::AppExit>>()
                .is_empty(),
            "the first `quit` left the orb without asking",
        );
        assert!(
            app.world().resource::<Tower>().sim().is_asking_to_quit(),
            "the first `quit` did not ask",
        );
        assert!(
            !app.world().resource::<crate::shell::Standing>().is_open(),
            "`quit` opened the menu, which is the behaviour that was superseded",
        );

        type_line(&mut app, "quit");
        app.world_mut().resource_mut::<Tower>().step();
        app.update();
        assert!(
            !app.world()
                .resource::<Messages<bevy::app::AppExit>>()
                .is_empty(),
            "the second `quit` did not leave",
        );
    }

    /// ...and anything else answers *no*.
    #[test]
    fn any_other_word_calls_off_a_pending_quit() {
        // **The surprise the question exists to prevent**: a `quit` typed and
        // thought better of, ending a session three commands later.
        let mut app = app();
        type_line(&mut app, "quit");
        app.world_mut().resource_mut::<Tower>().step();
        app.update();

        type_line(&mut app, "look around");
        app.world_mut().resource_mut::<Tower>().step();
        app.update();
        assert!(
            !app.world().resource::<Tower>().sim().is_asking_to_quit(),
            "the question outlived the next line",
        );

        type_line(&mut app, "quit");
        app.world_mut().resource_mut::<Tower>().step();
        app.update();
        assert!(
            app.world()
                .resource::<Messages<bevy::app::AppExit>>()
                .is_empty(),
            "a `quit` after a cancelled one left without asking again",
        );
    }

    /// The menu must not eat the word that opened it.
    ///
    /// # The timing the other two tests do not have
    ///
    /// `type_line` runs an `app.update()` per keystroke, so by the time the menu
    /// opens the `KeyboardInput` messages are long dropped. In the real game
    /// `advance` is `FixedUpdate` at 1 Hz, so usually sixty frames pass between
    /// the Enter and `Quitting` — but **when `quit` lands on a tick boundary
    /// they are the same frame**, and `type_into_menu`'s `MessageReader` has
    /// never run, so its cursor is at the start of whatever `Messages` still
    /// retains.
    ///
    /// That is `weaving.rs`'s recorded defect one surface over: *"a system that
    /// does not run keeps its message cursor"*, and a menu that re-reads the
    /// keystrokes that opened it spells `quit` into itself and leaves the orb —
    /// which looks exactly like `quit` having never stopped ending the session.
    #[test]
    fn opening_the_menu_does_not_eat_the_word_that_opened_it() {
        let mut app = app();

        // The keystrokes, with **no update between them and the tick**.
        type_only(&mut app, "menu");
        app.world_mut().write_message(KeyboardInput {
            key_code: KeyCode::Enter,
            logical_key: Key::Enter,
            state: ButtonState::Pressed,
            text: Some("\r".into()),
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
        {
            let mut tower = app.world_mut().resource_mut::<Tower>();
            tower.submit("menu");
            tower.step();
        }
        app.update();

        assert!(
            app.world().resource::<crate::shell::Standing>().is_open(),
            "the menu ate the word that opened it and closed again",
        );
        assert!(
            app.world()
                .resource::<Messages<bevy::app::AppExit>>()
                .is_empty(),
            "the menu read back the word that opened it and left the orb",
        );
        assert_eq!(
            app.world()
                .resource::<crate::shell::Standing>()
                .get()
                .map(orbs_shell::Menu::command),
            Some(""),
            "the keystrokes that opened the menu were typed into it",
        );
    }

    /// because a `SystemParam` needs a system to live in and this is a helper
    /// inside an assertion. The ordering it asks about is `orbs_shell::Focus`'s
    /// either way, which is the whole point of that type.
    fn focus_of(app: &App) -> orbs_shell::Focus {
        orbs_shell::Focus::of(orbs_shell::Open {
            editing: app
                .world()
                .resource::<crate::shell::editing::Editing>()
                .is_open(),
            weaving: app.world().resource::<crate::shell::Loom>().is_open(),
            walking: app.world().resource::<crate::shell::Walk>().is_open(),
            chorusing: app.world().resource::<crate::shell::Chorus>().is_open(),
            reading: app.world().resource::<orbs_shell::Scroll>().is_reading(),
            menuing: app.world().resource::<crate::shell::Standing>().is_open(),
        })
    }
}
