//! Registration for the window shell.

use bevy::input::common_conditions::input_just_pressed;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy::window::WindowResized;

use super::commanding::{cycle_register, export_trace, quit, quit_requested, submit};
use super::input::{SubmittedMessage, type_into_line};
use super::modal::is_open;
use super::reading::{read_keys, reading_key, scroll_back, scroll_forward, start_reading};
use super::revealing::{drive_panes, drive_passing, drive_reveal, finish_reveal};
use super::window::{cycle_mode, fit_projection, spawn_camera, track_window};
/// Registration itself names `Tower` only through `resource_changed::<...>`,
/// which is fully qualified; the tests drive it directly.
#[cfg(test)]
use crate::sim::Tower;
use orbs_shell::{Editor, Line, Linear, PaneTransition, Reveal, Screen, Tapestry};

/// Ordering within `Update`, so the frame that draws a keystroke is the frame
/// that received it.
///
/// Without the edge, `ORBS_CAPTURE=1` screenshots stop reproducing.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ShellSystems {
    /// Keystrokes reach the line; finished lines reach the sim.
    Input,
    /// Animations advance: the typewriter reveal and the pane transition.
    ///
    /// Before `repaint`, which reads what these write, or text flashes and
    /// rewinds. After [`Self::Input`], which opens and shuts the surfaces these
    /// animate — a set ordered against its writer as well as its reader.
    Drive,
}

/// Every resource this plugin owns, handed to `$mac` as a list of types.
///
/// One list for `build` and `reset_for_swap`, so a resource added is reset by
/// the same edit; a stale one does not crash, it lies.
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
            super::Climb,
            super::Editing,
            super::Loom,
            super::Standing,
            super::Reading,
            super::Walk,
            // Every swap ends an installation: the wake, a load from the menu,
            // and another `new`, which sets a fresh one after the reset.
            super::Installing,
            // The audio watermark, derived from this tower's transcript, so
            // reset too, or a restored tail rings at once.
            crate::sound::Heard,
            // The glass's answer to this tower, or a swap carries a siege over.
            crate::crt::Strain,
            // Whether this tower's threshold menu is up yet.
            super::thresholding::Opened,
        );
    };
}

/// Put every shell resource back to what it is at startup, for a new tower.
///
/// Three survive, lifted out rather than left off the list:
///
/// - `Screen`, the window's real grid.
/// - `Standing`, the menu that is asking, which closes itself.
/// - `Linear`, the player's (§14); its `Default` may not read the setting back.
///
/// Lifted out and put back, so [`shell_resources!`] stays the single list.
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
        // ...and whether a new tower is installed first, for the same reason.
        app.init_resource::<super::Installs>();
        // ...and whether the player asked for stillness, and how fast they read.
        app.init_resource::<super::Stillness>();
        app.init_resource::<super::revealing::Paced>();
        // ...and whether a repeated echo is drawn.
        app.init_resource::<super::echoing::Echoing>();
        // ...and whether the glass clouds while the window is in the background.
        app.init_resource::<super::Away>();
        app.init_resource::<super::Absence>();
        // ...and the name a new tower is raised with.
        app.init_resource::<super::setting::Naming>();

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
            // On the tower moving, not only on a line: `attend` is judged by
            // where the player stands, which changes on the tick after it.
            .add_systems(
                Update,
                super::installing::follow_install
                    .in_set(ShellSystems::Drive)
                    .run_if(is_open::<orbs_shell::Install>)
                    .run_if(resource_changed::<crate::sim::Tower>),
            )
            .add_systems(
                Update,
                (
                    // Exactly one surface takes a keystroke. The editor is
                    // gated here; the prompt decides inside itself, since it
                    // must discard what it declines (`input::type_into_line`).
                    // The silence first, so both read this frame's gap.
                    super::input::watch_quiet,
                    super::editing::open_requested.run_if(resource_changed::<crate::sim::Tower>),
                    super::editing::type_into_editor
                        .run_if(on_message::<KeyboardInput>)
                        .run_if(is_open::<Editor>),
                    // After the keys, so a keystroke restarts the settle clock
                    // before it is advanced — otherwise the frame a player
                    // types on counts toward a pause they have not taken.
                    super::editing::autosave.run_if(is_open::<Editor>),
                    // The weave screen, on the same terms: gated by a run
                    // condition because it *consumes* keys, while the prompt
                    // below always runs because it has to discard them.
                    super::weaving::open_requested.run_if(resource_changed::<crate::sim::Tower>),
                    super::weaving::type_into_loom
                        .run_if(on_message::<KeyboardInput>)
                        .run_if(is_open::<Tapestry>),
                    // The keys before the opening, ungated, or the opening word
                    // is read back into the menu (`menuing`'s module doc). The
                    // manual first, since it sits over the menu (`Focus::of`).
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
                    super::weaving::refresh.run_if(is_open::<Tapestry>),
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
                    // Before the paint, so the frame a setting lands is the
                    // frame it shows.
                    super::echoing::keep_echo
                        .before(ShellSystems::Drive)
                        .run_if(resource_exists::<crate::sim::Tower>),
                    // Every frame, for the ramp; it writes only on a change.
                    super::away::notice.run_if(resource_exists::<super::Away>),
                    // After the window is known, so a whole fit has a size to
                    // divide. Only when the screen moved: a resize, a mode, a fit.
                    fit_projection
                        .after(track_window)
                        .run_if(resource_changed::<Screen>),
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
                    // `F9` stays unbound since the chant went (§19). F10, not
                    // Escape, which clears a line by muscle memory.
                    quit.run_if(input_just_pressed(KeyCode::F10)),
                    // Gated on the world having moved, like the other
                    // handshakes: `submit` marks `Tower` changed, so this runs
                    // on that frame and no other. The flag is only set by a
                    // *confirmed* `quit` — the sim asks first, and `menu` is a
                    // different word now.
                    quit_requested.run_if(resource_changed::<crate::sim::Tower>),
                    // PageUp/PageDown, since the arrows recall history (§19).
                    // `playing`, and not while the manual answers these keys.
                    scroll_back
                        .run_if(input_just_pressed(KeyCode::PageUp))
                        .run_if(crate::shell::playing)
                        .run_if(super::manualling::not_reading_the_manual),
                    scroll_forward
                        .run_if(input_just_pressed(KeyCode::PageDown))
                        .run_if(crate::shell::playing)
                        .run_if(super::manualling::not_reading_the_manual),
                    // The arrows, Home and End move the transcript only while
                    // reading, and carry the manual's guard too. `playing` for
                    // Escape, which at the threshold is the menu's.
                    read_keys
                        .run_if(reading_key)
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
                        // After `motion::advance`, the other writer: it carries
                        // the tube's switch, this the clock.
                        .after(super::motion::advance),
                    drive_reveal.in_set(ShellSystems::Drive),
                    // Wall-clock, never the sim's (`shell::bench`). After
                    // `refresh_panel`, which writes the `Panel` this reads for
                    // its edges. `booted`, and `Bench` seeds its edges true, so
                    // a lit fire does not flare on the first frame.
                    super::motion::advance
                        .in_set(ShellSystems::Drive)
                        .after(super::input::refresh_panel),
                )
                    // `booted`, not `playing`, since `drive_passing` animates
                    // the card leaving onto the menu; what needs a tower carries
                    // `playing` itself. `F4`, `F5` and `F10` are the screen's,
                    // so they work at the menu too.
                    .run_if(crate::boot::booted),
            );
    }
}

/// Typing, end to end, with no window and no GPU.
///
/// Beside the registration they exercise, because that is what is under test:
/// whether the plugin wires a keystroke to the parser. §19's lesson is that a
/// sixteen-command parser went six roadmap items without ever receiving one.
#[cfg(test)]
mod tests;
