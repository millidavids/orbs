//! Registration for the window shell.

use bevy::input::common_conditions::input_just_pressed;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy::window::WindowResized;

use super::commanding::{cycle_register, export_trace, quit, quit_requested, submit};
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
    /// A set of its own so `repaint` can order after it. Both write state
    /// `repaint` then reads, so without the edge the executor could draw a
    /// burst of output whole and then set `shown` back to zero — text that
    /// flashes and rewinds. Same class of defect [`Self::Input`] exists for.
    ///
    /// And it runs after [`Self::Input`]: the input chain opens and shuts every
    /// surface — `wander`, `edit`, `weave`, `unfurl` — and these animations
    /// have to observe the result. Without the edge a `wander` drew the whole
    /// maze for one frame and only then started a crossing that departed from
    /// the maze it had just arrived at.
    ///
    /// The two sets were ordered against `repaint` and never against each other
    /// — the third time this project has paid for a set that orders against its
    /// reader but not against its writer.
    Drive,
}

/// Every resource this plugin owns, handed to `$mac` as a list of types.
///
/// A list rather than eighteen `init_resource` calls, because it is read twice:
/// `build` registers them and `reset_for_swap` puts them back when the menu
/// loads a different tower. A second hand-written copy drifts, and invisibly —
/// a resource holding the *old* game's state does not crash, it lies: `Reveal`
/// holds indices into a record stream that no longer exists.
///
/// One list, so adding a resource registers and resets it in the same edit.
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
            super::Reading,
            super::Walk,
            // Not a shell resource, and on the list anyway. `Heard` is the
            // audio watermark, derived from *this tower's* transcript exactly
            // as `Scroll` and `Panel` are. Left off, a swap kept the old mark:
            // the threshold's scratch world pushes a handful of records, `play`
            // → `2` restores a save whose sequence is in the thousands,
            // `Records::since` skipped nothing, and the carried tail rang in
            // one frame — five hundred `AudioPlayer`s, no cap. The only guard
            // was `now <= last`, which catches a rewound count and not a
            // jumped-forward one.
            crate::sound::Heard,
        );
    };
}

/// Put every shell resource back to what it is at startup, for a new tower.
///
/// Three survive, lifted out rather than left off the list:
///
/// - `Screen` holds the grid the window actually is; resetting it would tell the
///   renderer the window had changed size, which it has not.
/// - `Standing` is the menu, and the menu is what is asking. It closes itself
///   afterwards, on its own terms.
/// - `Linear` is §14's accessibility route and a property of the *player*. Its
///   `Default` reads `orbs-settings.toml`, and `settings::store` answers `None`
///   outright for a process that never called `keep()` — so with `ORBS_SAVE=off`,
///   a read-only install or a failed write, `F5` on and then loading another
///   tower turned the linear stream off with nothing said.
///
/// Taking them out and putting them back keeps [`shell_resources!`] the single
/// list: a resource added there is reset here by construction.
pub(crate) fn reset_for_swap(world: &mut World) {
    let screen = world.remove_resource::<Screen>();
    let standing = world.remove_resource::<super::Standing>();
    let linear = world.remove_resource::<Linear>();

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
    if let Some(linear) = linear {
        world.insert_resource(linear);
    }
}

/// The window, the camera, the grid, and the command line.
pub struct ShellPlugin;

impl Plugin for ShellPlugin {
    fn build(&self, app: &mut App) {
        // The animations run after the input: the clocks in `Drive` have to
        // observe the surface the input chain just opened or shut, or a
        // `wander` draws the whole maze for a frame and then crosses away from
        // it. See `ShellSystems`.
        app.configure_sets(Update, ShellSystems::Drive.after(ShellSystems::Input));

        // One list, read twice — see `shell_resources!`. It is also what
        // `reset_for_swap` puts back when the menu loads a different tower.
        macro_rules! register {
            ($($resource:ty,)*) => {
                $( app.init_resource::<$resource>(); )*
            };
        }
        shell_resources!(register);

        // Not on the reset list, for `Screen`'s and `Linear`'s reason: §4's
        // sticky skip is the player's property, not the tower's. Read once at
        // startup and the truth thereafter — see `setting::Skipping`.
        app.init_resource::<super::setting::Skipping>();

        app.add_message::<SubmittedMessage>()
            .add_message::<super::menuing::SwapMessage>()
            .add_message::<super::menuing::SettingMessage>()
            .add_message::<super::manualling::OpenManualMessage>()
            // Registered by the consumer, not the producer: `sim::content` only
            // exists when `ORBS_CONTENT` is set, while the `run_if` below needs
            // the message always — without this the shell panicked with
            // *"Message not initialized"* in every ordinary session.
            .add_message::<crate::sim::ManualChangedMessage>()
            // Ordered into `Input`, so `Drive` sees the world it arrives at.
            // `Panel` self-heals from the new tower only if refreshed *after*
            // the swap, and `refresh_panel` is in `Drive`; without this the
            // executor may refresh from the outgoing world and leave it a tick.
            .add_systems(
                Update,
                super::menuing::swap
                    .in_set(ShellSystems::Input)
                    .run_if(on_message::<super::menuing::SwapMessage>),
            )
            // Gated on the message alone, like the swap above it, and not on
            // `booted` or `playing`: the settings page is reachable at the
            // threshold, which is the point of putting it on the first screen.
            .add_systems(
                Update,
                super::menuing::apply_setting
                    .in_set(ShellSystems::Input)
                    .run_if(on_message::<super::menuing::SettingMessage>),
            )
            // After the input, so a function key pressed this frame is already
            // in the thing it changed. Only while the menu is up, and it
            // declines inside when nothing moved — see the system's own doc.
            .add_systems(
                Update,
                super::setting::follow_the_keys
                    .after(ShellSystems::Input)
                    // A settings page, not merely an open menu: the threshold's
                    // menu cannot be closed, so `is_open()` alone ran this
                    // every frame from launch, rebuilding rows nobody looked
                    // at.
                    .run_if(|standing: Res<super::Standing>| {
                        standing
                            .get()
                            .is_some_and(orbs_shell::Menu::showing_settings)
                    }),
            )
            .add_systems(Startup, (spawn_camera, track_window).chain())
            // Not gated on `booted`, and not in the input set: a focus loss
            // during boot strands held keys exactly as one during play does,
            // and the guard has to outlive whatever stole the window.
            .add_systems(
                Update,
                super::input::forget_held_keys.run_if(on_message::<bevy::window::WindowFocused>),
            )
            // Rule 6 reaching an already-open manual. The book is assembled at
            // open, so without this a hot-reloaded `manual.toml` changed the
            // log and nothing else. See `manualling::restock_requested`.
            .add_systems(
                Update,
                super::manualling::restock_requested
                    .in_set(ShellSystems::Drive)
                    .run_if(super::manualling::reading_the_manual)
                    .run_if(on_message::<crate::sim::ManualChangedMessage>),
            )
            // The panel only moves when the world does — see `Panel`. Not gated
            // on `booted`: `Tower` is marked changed when inserted, and that
            // frame fills the panel for the starting room; behind the gate the
            // flag has expired by the time the sequence ends.
            .add_systems(
                Update,
                super::input::refresh_panel
                    .in_set(ShellSystems::Drive)
                    .run_if(resource_changed::<crate::sim::Tower>),
            )
            .add_systems(
                Update,
                (
                    // Exactly one surface takes a keystroke. The prompt, the
                    // editor and the transcript are on screen at once, and two
                    // consuming a key is invisible until a player types `:wq`
                    // and finds it in their command history.
                    //
                    // The editor is gated here; the prompt decides inside
                    // itself, because it also has to discard what it declines.
                    // See `input::type_into_line`. First in the chain, so both
                    // text fields read the gap in front of this frame's
                    // keystroke.
                    super::input::watch_quiet,
                    super::editing::open_requested.run_if(resource_changed::<crate::sim::Tower>),
                    super::editing::type_into_editor
                        .run_if(on_message::<KeyboardInput>)
                        .run_if(super::editing::editing),
                    // After the keys, so a keystroke restarts the settle clock
                    // before it is advanced — otherwise the frame a player
                    // types on counts toward a pause they have not taken.
                    super::editing::autosave.run_if(super::editing::editing),
                    // The weave screen, on the same terms: gated by a run
                    // condition because it *consumes* keys, while the prompt
                    // below always runs because it has to discard them.
                    super::weaving::open_requested.run_if(resource_changed::<crate::sim::Tower>),
                    super::weaving::type_into_loom
                        .run_if(on_message::<KeyboardInput>)
                        .run_if(super::weaving::weaving),
                    // The keys first, then the opening — a shipped defect, not
                    // a preference. `type_into_menu` is ungated for
                    // `type_into_line`'s reason: a reader that does not run
                    // keeps its cursor, so a gated one read the word that
                    // opened the menu back into it, reached the menu's `quit`
                    // and left the orb. See `menuing`'s module doc.
                    //
                    // The manual first, because it sits over the menu — the one
                    // pair genuinely open at once, so the order is
                    // `Focus::of`'s rather than a tie-break.
                    super::manualling::type_into_manual.run_if(on_message::<KeyboardInput>),
                    super::manualling::open_requested
                        .run_if(on_message::<super::manualling::OpenManualMessage>),
                    super::menuing::type_into_menu.run_if(on_message::<KeyboardInput>),
                    super::menuing::open_requested.run_if(resource_changed::<crate::sim::Tower>),
                    // The threshold's menu, on the same terms. No word opens it
                    // — there is no prompt to type one at — so it goes up the
                    // first frame after the boot card leaves; with
                    // `ORBS_BOOT=0` that is frame one, when a stray keystroke
                    // is most likely still queued, so it sits after
                    // `type_into_menu`.
                    super::thresholding::open_at_the_threshold.run_if(super::thresholding::waiting),
                    // Unconditional while it is open, like `autosave`: the
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
                    // Ungated, and after every surface that can let go. It
                    // watches the keyboard change hands, an edge
                    // `type_into_line` cannot see: that system is gated on a
                    // keystroke arriving. See `HeldOver` — this is what stops a
                    // held arrow putting `wander` back in the prompt on the way
                    // out of the maze.
                    super::input::watch_focus,
                    // No `not_editing` here: it has to *run* to throw
                    // keystrokes away, since a reader that never runs keeps its
                    // cursor. The check moved inside; see `type_into_line`.
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
                    // Nothing typed reaches the line until the game is up:
                    // there is no input line during the sequence and §4 draws
                    // no prompt there. (The boot-skip keypress was the old
                    // reason; that skip is gone and the guard is not
                    // vestigial.)
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
                    // frame, and nothing had ever shown it. In the input set
                    // like every other surface switch: it was the one writer of
                    // `Showing`'s inputs left outside, so on the wrong
                    // interleaving the mirror painted whole for a frame and
                    // only then crossed.
                    orbs_shell::toggle_linear
                        .in_set(ShellSystems::Input)
                        .run_if(input_just_pressed(KeyCode::F5)),
                    // §6 requires the parser explain itself, and the Phase 0
                    // gate acts on failure *clustering*. Every reading is kept;
                    // this gets it out to a spreadsheet. `playing`, not
                    // `booted`: pressed at the menu it writes `orbs-parse.tsv`
                    // for a session that has not happened.
                    export_trace
                        .run_if(input_just_pressed(KeyCode::F6))
                        .run_if(crate::shell::playing),
                    // §3's tonal register, until Phase 8 drives it from threat.
                    // `playing`, because it writes to the world — and at the
                    // threshold the world is a scratch one the next swap throws
                    // away.
                    cycle_register
                        .run_if(input_just_pressed(KeyCode::F7))
                        .run_if(crate::shell::playing),
                    // `F9` is unbound — the menagerie's patient chant until the
                    // menagerie stopped having a clock (§19). Left free rather
                    // than reassigned: a key that did something a release ago
                    // is one a returning player presses expecting the first
                    // thing.
                    //
                    // F10, not Escape: with a text field on screen, Escape is
                    // "clear the line" muscle memory, and quitting mid-sentence
                    // is not a recoverable surprise.
                    quit.run_if(input_just_pressed(KeyCode::F10)),
                    // Gated on the world having moved, like the other
                    // handshakes: `submit` marks `Tower` changed, so this runs
                    // on that frame and no other. The flag is only set by a
                    // *confirmed* `quit` — the sim asks first, and `menu` is a
                    // different word now.
                    quit_requested.run_if(resource_changed::<crate::sim::Tower>),
                    // PageUp/PageDown, not the arrows: Up and Down walk the
                    // command history (§19), and a shell where Up sometimes
                    // scrolls and sometimes recalls is one you cannot type in
                    // without looking.
                    //
                    // All five of these are `playing`: paging the scratch
                    // transcript behind the threshold's menu moves `Scroll`
                    // under a screen a swap is about to replace. And not while
                    // the manual is open, the one surface answering these same
                    // keys — without `not_reading_the_manual` a single PgDn
                    // pages the chapter *and* the transcript underneath it.
                    scroll_back
                        .run_if(input_just_pressed(KeyCode::PageUp))
                        .run_if(crate::shell::playing)
                        .run_if(super::manualling::not_reading_the_manual),
                    scroll_forward
                        .run_if(input_just_pressed(KeyCode::PageDown))
                        .run_if(crate::shell::playing)
                        .run_if(super::manualling::not_reading_the_manual),
                    // The arrows scroll only while reading. At the prompt they
                    // walk the command history; inside the mode there is no
                    // history, so they are free.
                    //
                    // And they carry the manual's guard, which the first pass
                    // gave only to PgUp/PgDn. `apply_to_manual` maps the arrows
                    // to a one-row scroll and `editing::reading` is just
                    // `scroll.is_reading()`, which nothing clears when the
                    // manual opens — so after an `unfurl` one ArrowDown
                    // scrolled the chapter *and* the transcript behind it.
                    scroll_back
                        .run_if(input_just_pressed(KeyCode::ArrowUp))
                        .run_if(super::editing::reading)
                        .run_if(crate::shell::playing)
                        .run_if(super::manualling::not_reading_the_manual),
                    scroll_forward
                        .run_if(input_just_pressed(KeyCode::ArrowDown))
                        .run_if(super::editing::reading)
                        .run_if(crate::shell::playing)
                        .run_if(super::manualling::not_reading_the_manual),
                    // Escape leaves, exactly as it leaves the editor's buffer.
                    // `playing` matters most here: Escape at the threshold must
                    // reach the menu, and a `stop_reading` on the same
                    // keystroke would be a second reader of a key the menu
                    // owns. The manual's guard is the same one surface further
                    // in.
                    stop_reading
                        .run_if(input_just_pressed(KeyCode::Escape))
                        .run_if(super::editing::reading)
                        .run_if(crate::shell::playing)
                        .run_if(super::manualling::not_reading_the_manual),
                    start_reading
                        .run_if(resource_changed::<crate::sim::Tower>)
                        .run_if(crate::shell::playing),
                    // Unconditional: all three of these have to keep moving on
                    // the frames where nothing happened, which is most of them.
                    drive_panes.in_set(ShellSystems::Drive),
                    // After `refresh_panel`, explicitly, for
                    // `motion::advance`'s reason: `Showing` reads
                    // `Panel::room`, and the set orders both against `repaint`
                    // rather than each other, so the executor could start every
                    // crossing a frame late.
                    drive_passing
                        .in_set(ShellSystems::Drive)
                        .after(super::input::refresh_panel)
                        // After `motion::advance`, the other writer. Bevy
                        // serialises them, but *which order* was the executor's
                        // and they are not interchangeable: `motion` carries
                        // the tube's switch, this the clock. Turning `F3` back
                        // on, the unordered pair started a crossing a frame
                        // late; turning it off, one interleaving started a
                        // crossing the other cleared. Both invisible — and this
                        // file records three defects that were exactly a set
                        // ordered against its reader and not its writer.
                        .after(super::motion::advance),
                    drive_reveal.in_set(ShellSystems::Drive),
                    // §10.1's instruments animate on wall-clock time, not the
                    // tick — the sim must not observe it, or replay would
                    // depend on how long a frame took. See `shell::bench`.
                    //
                    // After `refresh_panel`, explicitly: it reads `Panel` to
                    // catch the edges it animates — a hearth lighting, a bowl
                    // filling — and `refresh_panel` writes it. `in_set(Drive)`
                    // alone orders them against `repaint` and not each other.
                    //
                    // Gated on `booted` with the rest: the panel is not on
                    // screen during the sequence, and `Bench` seeds `was_lit`
                    // and `was_charged` *true* so a tower opening with a fire
                    // already going does not flare on the first frame it is
                    // looked at.
                    super::motion::advance
                        .in_set(ShellSystems::Drive)
                        .after(super::input::refresh_panel),
                )
                    // Every key here is guarded: none means anything before the
                    // world runs, and `F6` would write a trace of a session
                    // that has not happened.
                    //
                    // `booted` and not `playing` for the tuple, because it is
                    // not only keys: `drive_passing` animates the boot card
                    // *leaving*, and at the threshold it leaves onto the menu.
                    // Gating on a chosen tower would freeze that crossing
                    // half-drawn. The systems that must not run before a tower
                    // exists carry `playing` individually, each next to the
                    // reason.
                    //
                    // `F4` and `F5` stay on `booted` on purpose: the display
                    // mode and the linear stream are about the *screen*, the
                    // menu is on it, and `menu_too_small` tells the player to
                    // press `F4`. So does `F10` — a way out that only works
                    // once you are in is not one.
                    .run_if(crate::boot::booted),
            );
    }
}

// `page_rows` and `page_step` moved to `orbs_shell::page_step`: how many
// records a screenful holds is not a keyboard question but what the transcript
// would fit, and a second measure of the same stream would page by a different
// amount than it showed. This frontend's wrapper is in `reading.rs`.

/// Typing, end to end, with no window and no GPU.
///
/// Beside the registration they exercise, because that is what is under test:
/// whether the plugin wires a keystroke to the parser. §19's lesson is that a
/// sixteen-command parser went six roadmap items without ever receiving one.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::{Reading, Standing};
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
        // And released, which `tap` next door already knew: a key left down
        // means the *next* press is not a fresh one. Since `press` stamps every
        // keystroke `KeyCode::KeyA`, one unreleased press left `A` held for the
        // rest of the test — a false failure the moment `watch_focus` read it.
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
    /// The silence is the evidence. The first fix read the keystroke's `text`
    /// instead, believing macOS hands back none under a chord — winit 0.30.13
    /// sets `text` from `logical_key.to_text()` with no modifier check, so
    /// `Cmd+A` carries `Some("a")` and that fix typed a letter on every paste.
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
        // The word exists because the key could not be discovered: the border
        // advertised `PgDn newest` only once you were *already* scrolled back,
        // which in a game with no mouse is no affordance.
        //
        // The half that must not regress is the exit: Escape means the same
        // thing here as in the editor, or the player is stuck with nothing to
        // type into.
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
        // Three surfaces can own the keyboard, and two consuming a key is
        // invisible until a player types `:wq` and finds it in their history.
        //
        // Asserted on the line, not the run conditions: a correct predicate
        // wired to nothing reads exactly like one that works, and the run
        // conditions were where this could go wrong.
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
        // Recovery *without* a `WindowFocused`, because the reported failure —
        // `Cmd+Shift+Ctrl+4` — leaves no focus event to hang a fix on. Every
        // recovery path in Bevy 0.19 hangs off that event, and Bevy drops
        // winit's `ModifiersChanged`, so a silently swallowed key-up leaves the
        // field dead for the session.
        //
        // The evidence that breaks the deadlock is the keystroke itself: the OS
        // gave us text, so it is not treating this as a command.
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
        // The surface it was reported on. The prompt and the editor share the
        // guard, so they shared the freeze, and a fix tested only on the prompt
        // would have been half a fix.
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
        // The counterweight, and the regression the first fix shipped: it
        // treated "the key came with text" as proof no chord was held, but
        // winit sets `text` from `logical_key.to_text()` with no modifier
        // check, so `Cmd+A` typed a letter into the prompt.
        //
        // A held chord is used within moments of being pressed, so no silence
        // has elapsed and the guard stands.
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
        // `Cmd+Shift+Ctrl+4` on macOS hands the window to the screenshot
        // overlay mid-chord, so Cmd and Ctrl's *release* goes there and never
        // to us. Held state then says they are down for ever and the prompt is
        // dead with nothing on screen to say why.
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
        // in the buffer takes a cell and draws nothing, so the caret drifts
        // with no visible cause. `Escape` clears the line now, and Tab
        // *completes*, so what it must not do is leave its own `\t` behind; an
        // exact line here would be asserting the completer's answer instead.
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
        // §9 requires the focus mode be overridable at any time, and this is
        // the end-to-end proof the key reaches `cycle_mode`.
        //
        // It used to assert that Deep focus bought cells, back when the grid
        // was derived from the window. §19 fixed the grid, so what survives is
        // the half that was always the point — the split changes — plus the
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
        // for. Without this the test passes on a `cycle_mode` that only sets a
        // field nobody reads.
        //
        // Two panes explicitly, not `PANES`, which is 1 since the tower rail
        // replaced the telemetry pane — one pane tiles identically in both
        // modes, so the live count made this assert something `F4` currently
        // cannot do. What it holds is that the two modes ask the tiler for
        // different shapes.
        //
        // The half genuinely lost — that `F4` changes what is on screen *right
        // now* — is recorded on `PANES`, because a test cannot hold a claim the
        // game has stopped making.
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
    /// `press` stamps every keystroke `KeyCode::KeyA`, since `type_into_line`
    /// does not read the field. `watch_focus` does — it tracks which *physical*
    /// key is still down — so a test using `press` would put every key in one
    /// bucket and pass against a fix that swallowed the keyboard for ever.
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

        // Walking: the arrow goes down and stays down, which is the state the
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

        // The held arrow outliving the Escape is how the word came back: key
        // repeat keeps delivering, the maze has let go by this frame, and at
        // the prompt an arrow means *recall history* — newest entry, the
        // `wander`.
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

        // And the other half. Let go and press again: that is a real keystroke
        // and recall must still work. A version that swallowed the keyboard
        // from the moment a surface closed would pass everything above and
        // break `Up`.
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
    /// The regression test for `orbs_shell::Focus`, and for the defect its
    /// module comment describes: a forgotten surface *"types into an invisible
    /// prompt while the player looks at something else."* Nothing asserted that
    /// before — `leaving_the_maze_leaves_nothing_in_the_prompt` covers one
    /// surface.
    ///
    /// A loop over the four on purpose: a fifth added without its arm is a
    /// missing row in a table rather than a test nobody remembered to write.
    #[test]
    fn no_surface_lets_a_keystroke_reach_the_prompt() {
        for (surface, opening) in [
            // A spell is written *for* a domain, so `scribe` from the tower
            // landing opens nothing — which the "asserts nothing" guard below
            // caught on the first run.
            ("editor", &["attend laboratory", "scribe drill"][..]),
            ("weave", &["weave"][..]),
            ("maze", &["attend archive", "research", "wander"][..]),
            ("reading", &["unfurl"][..]),
            // There was a fifth row, `chant`. The menagerie is typed now (§19),
            // so the row went with the surface rather than staying to assert
            // something that cannot open.
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

    /// The word, through the real plugin stack, to the real menu.
    ///
    /// Not the dump: `ORBS_DUMP` goes through `orbs_shell::dump`, which builds
    /// no `App` and takes the handshake itself, so it drew a menu while the
    /// live route to one was never exercised. `menuing::open_requested` is a
    /// *system* with a run condition in a schedule; a still photograph reaches
    /// none of it.
    #[test]
    fn the_word_menu_opens_the_menu() {
        let mut app = app();
        type_line(&mut app, "menu");
        // The effect lands on the tick, not at `submit`: `submit` echoes and
        // queues, and every verb's effect runs at the next `step`, so a check
        // before this finds the flag unset — the mistake `quit` shipped once.
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
    /// Superseded (§19): for one iteration `quit` opened the menu, so stopping
    /// was two steps through a screen nobody asked for. Now `quit` leaves and
    /// the two steps are a *question*, like every other refusal in the game.
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
        // The surprise the question exists to prevent: a `quit` typed and
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
    /// The timing the other two tests do not have. `type_line` updates per
    /// keystroke, so the `KeyboardInput` messages are long dropped by the time
    /// the menu opens — but when `quit` lands on a tick boundary they are the
    /// same frame, and `type_into_menu`'s reader has never run, so its cursor
    /// is at the start of whatever `Messages` retains.
    ///
    /// `weaving.rs`'s recorded defect one surface over: *"a system that does
    /// not run keeps its message cursor"*, and a menu re-reading the keystrokes
    /// that opened it spells `quit` into itself and leaves the orb.
    #[test]
    fn escape_out_of_the_manual_leaves_the_menu_standing() {
        // Two surfaces, two `MessageReader` cursors, one keystroke.
        // `MenuOutcome::OpenManual` leaves the menu open underneath, and
        // `type_into_manual` runs first — so the Escape that shut the manual
        // was gone from `Reading::is_open` before `type_into_menu` looked, and
        // the menu read the same event and closed too. `ORBS_DUMP` builds no
        // `App`, so `dumps.sh`'s capture is green either way; this layer can
        // see it.
        let mut app = app();
        app.world_mut().resource_mut::<Standing>().open(
            orbs_shell::Stance::InTower,
            orbs_shell::Driver::default(),
            crate::shell::setting::defaults(),
        );
        let book = orbs_shell::manual_book(app.world().resource::<Tower>().sim());
        app.world_mut().resource_mut::<Reading>().open(book);
        app.update();
        assert!(app.world().resource::<Reading>().is_open());
        assert!(app.world().resource::<Standing>().is_open());

        // One Escape: out of the manual, and no further.
        app.world_mut().write_message(KeyboardInput {
            key_code: KeyCode::Escape,
            logical_key: Key::Escape,
            state: ButtonState::Pressed,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
        app.update();

        assert!(
            !app.world().resource::<Reading>().is_open(),
            "Escape did not leave the manual",
        );
        assert!(
            app.world().resource::<Standing>().is_open(),
            "the same Escape closed the menu behind it",
        );
    }

    #[test]
    fn a_word_typed_at_the_manual_does_not_also_drive_the_menu() {
        // The other half, and the one that could leave the game: `q` is not a
        // chapter, so the menu underneath prefix-matched its own `quit` and
        // wrote an `AppExit` from inside the manual.
        let mut app = app();
        app.world_mut().resource_mut::<Standing>().open(
            orbs_shell::Stance::InTower,
            orbs_shell::Driver::default(),
            crate::shell::setting::defaults(),
        );
        let book = orbs_shell::manual_book(app.world().resource::<Tower>().sim());
        app.world_mut().resource_mut::<Reading>().open(book);
        app.update();

        type_only(&mut app, "q");
        app.world_mut().write_message(KeyboardInput {
            key_code: KeyCode::Enter,
            logical_key: Key::Enter,
            state: ButtonState::Pressed,
            text: Some("\r".into()),
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
        app.update();

        assert!(
            app.world()
                .resource::<Messages<bevy::app::AppExit>>()
                .is_empty(),
            "typing at the manual left the orb",
        );
        assert!(
            app.world().resource::<Standing>().is_open(),
            "typing at the manual closed the menu behind it",
        );
    }

    #[test]
    fn opening_the_menu_does_not_eat_the_word_that_opened_it() {
        let mut app = app();

        // The keystrokes, with no update between them and the tick.
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

    /// The shell standing at the threshold: no tower chosen, the menu up.
    ///
    /// `Threshold::Waiting` is inserted directly rather than by `SimPlugin`,
    /// which this harness does not install — what is under test here is the
    /// *keyboard*, and the plugin's own half has tests beside it in `sim`.
    fn at_the_threshold() -> App {
        let mut app = app();
        app.insert_resource(orbs_shell::Threshold::Waiting);
        // One frame for `open_at_the_threshold` to put the menu up.
        app.update();
        app
    }

    /// The keyboard belongs to the menu, whatever is pressed at it.
    ///
    /// A real keyboard and not a dump: `ORBS_DUMP` builds no `App` and presses
    /// no key, so it draws a correct threshold whatever the routing does — §19
    /// records the orb's menu shipping a keyboard defect through four green
    /// dump latches.
    ///
    /// It holds the routing: keys reach the menu, none leaks to the prompt
    /// behind it, none ends the session. It does *not* hold "the menu is never
    /// closed" — `open_at_the_threshold` puts the menu back on the same frame
    /// anything closes it, so this passed with `Stance::may_close` deliberately
    /// broken. That rule is held where it can fail: `orbs-shell`'s
    /// `the_threshold_has_no_way_to_close_the_menu`.
    #[test]
    fn the_threshold_gives_no_keystroke_to_the_prompt_behind_it() {
        let mut app = at_the_threshold();
        assert!(
            app.world().resource::<crate::shell::Standing>().is_open(),
            "the threshold did not put its menu up",
        );

        // Escape, the way out of every other surface in the game; `resume`,
        // whole and by the prefix that reaches it over a tower; and an ordinary
        // sentence. After all of it the menu still has the keyboard.
        for _ in 0..3 {
            tap(&mut app, KeyCode::Escape, Key::Escape);
        }
        type_line(&mut app, "resume");
        type_line(&mut app, "r");
        type_line(&mut app, "look around");
        assert!(
            app.world().resource::<crate::shell::Standing>().is_open(),
            "the menu did not have the keyboard after all that",
        );

        // And nothing above leaked into a prompt nobody can see.
        assert!(
            app.world()
                .resource::<Messages<bevy::app::AppExit>>()
                .is_empty(),
            "a keystroke at the threshold left the orb",
        );
        assert_eq!(
            app.world().resource::<Line>().text(),
            "",
            "the menu's keystrokes reached the prompt behind it",
        );
    }

    /// ...and `quit` still leaves, which is what makes the above a rule about
    /// *closing* rather than about the keyboard being dead.
    #[test]
    fn quit_leaves_the_orb_from_the_threshold() {
        let mut app = at_the_threshold();
        type_line(&mut app, "quit");
        assert!(
            !app.world()
                .resource::<Messages<bevy::app::AppExit>>()
                .is_empty(),
            "the threshold had no way out of the orb at all",
        );
    }

    /// The menu is up before the first keystroke can land anywhere else.
    ///
    /// With `ORBS_BOOT=0` the frame the sequence finishes is frame one, which is
    /// exactly when a stray keystroke is most likely to still be in the queue —
    /// and `menuing.rs` records what a surface opening on such a frame costs.
    #[test]
    fn the_threshold_takes_the_keyboard_before_the_prompt_does() {
        let mut app = app();
        app.insert_resource(orbs_shell::Threshold::Waiting);
        // Typed on the very frame the menu goes up, with no update in between.
        type_only(&mut app, "look around");
        assert!(
            app.world().resource::<crate::shell::Standing>().is_open(),
            "the threshold never opened its menu",
        );
        assert_eq!(
            app.world().resource::<Line>().text(),
            "",
            "a keystroke reached the prompt behind the threshold's menu",
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
            reading: app.world().resource::<orbs_shell::Scroll>().is_reading(),
            menuing: app.world().resource::<crate::shell::Standing>().is_open(),
            reading_manual: app.world().resource::<crate::shell::Reading>().is_open(),
        })
    }
}
