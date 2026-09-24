//! The camera, the letterbox, and feeding [`Screen`] a window.
//!
//! Everything here needs a GPU, a winit window, or both, which is exactly what
//! separates it from [`orbs_shell::Screen`]: that answers *how big is a cell
//! and how many are there*, and a terminal answers it too. This one puts a
//! 4:3 picture in the middle of a resizable window, and nothing else can.

use bevy::camera::ScalingMode;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use orbs_render::{GRID, MIN_SCALE};

use orbs_shell::Screen;

/// Flip the focus mode.
///
/// Both modes grant identical capacity, panes, information and synergies (§9);
/// only the rendering differs.
///
/// It no longer changes the glyph size, which is the one thing that cost. Deep
/// focus used to drop a fidelity tier as well as re-dividing the panes; with one
/// grid there is no tier to drop, so `F4` is purely `tiling::deep` against
/// `tiling::wide`. §19 records the debt — §9 sold Wide focus as the large-text
/// mode, and what replaces it is a font-scale setting, not this key.
pub(crate) fn cycle_mode(mut screen: ResMut<Screen>) {
    screen.mode = screen.flipped();
    info!(
        "focus {:?} -> grid {}x{}",
        screen.mode, GRID.cols, GRID.rows
    );
    // One setting seen twice — see `crt::plugin::cycle`. §9 makes the mode a
    // *setting* rather than a heuristic in as many words, and a setting that
    // does not survive a relaunch is a heuristic with extra steps.
    super::remember_setting(super::FOCUS, screen.mode.word());
}

/// The camera every frontend screen is drawn through, and the letterbox.
pub(crate) fn spawn_camera(mut commands: Commands) {
    // The CRT is a property of the camera it curves (§4).
    //
    // MSAA is off deliberately: every edge on screen is the border of a bitmap
    // glyph, and adjacent cells abut exactly, so there is nothing to antialias
    // that is not supposed to be hard. Multisampling can only soften the font §4
    // says legibility depends on.
    //
    // It is *not* why the CRT used to flash — that was the pass missing its
    // system set (see `crt::plugin`). Turning MSAA off changed the odds enough
    // to look like a fix under screenshot sampling, which is a good reminder
    // that "the symptom went away in my measurement" is not a diagnosis.
    commands.spawn((
        Camera2d,
        // This one line is the 4:3 letterbox. `AutoMin` shows at least this
        // much world in the window's own aspect and centres it on
        // `viewport_origin`, which defaults to the middle — so the picture is
        // scaled by `min(W/960, H/720)`, centred, with bars on whichever axis
        // has room to spare. `grid::build` emits its mesh in virtual pixels
        // around the origin, so nothing else has to know.
        //
        // It re-derives itself: `bevy_render`'s `camera_system` re-runs
        // `update` on `WindowResized` and `WindowScaleFactorChanged`. That is
        // why it is set once here rather than by a system with a resize guard —
        // a projection left unset draws *correctly* at the opening window and
        // wrong everywhere else, which is the worst way for this to fail.
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: f32::from(orbs_render::PICTURE.0),
                min_height: f32::from(orbs_render::PICTURE.1),
            },
            ..OrthographicProjection::default_2d()
        }),
        Msaa::Off,
        // Seeded from the environment, so the tube's state is reachable without
        // a keypress. `F3` still cycles it; this lets a See-it line capture the
        // property §14 turns on — that turning the tube off does not turn the
        // accommodation off with it. `ORBS_CAPTURE` presses no keys, so without
        // this only a person could ever check it.
        crate::crt::seeded(),
    ));
}

/// Note the window's size whenever it changes.
///
/// It no longer recomputes anything. The grid is fixed and the projection
/// letterboxes itself, so all this does is keep [`Screen::window`] current for
/// the two things that still read physical pixels: the CRT's cell size, which
/// keeps the scanlines on real pixels, and [`Screen::is_hostable`].
pub(crate) fn track_window(
    window: Option<Single<&Window, With<PrimaryWindow>>>,
    mut screen: ResMut<Screen>,
) {
    let Some(window) = window else {
        return;
    };

    // The mode is a setting, so a resize must not silently undo the player's
    // choice — it is carried through rather than re-derived.
    //
    // Unconditionally, and that is the fix. This asked
    // `(screen.window != (0, 0)).then_some(screen.mode)`, using the absent
    // window as a stand-in for *"has a mode been chosen?"*. The two stopped
    // meaning the same thing when `Screen::default` learned to read the
    // persisted `focus`: it sets a real mode and leaves `window` at `(0, 0)`, so
    // this system, running in `Startup`, threw the setting away on frame one.
    // Set `focus deep`, quit, relaunch: Wide. A `Screen` always has a mode now.
    //
    // `screen.rs`'s own test cannot reach this: a test binary never calls
    // `settings::keep()`, so the read is always empty there.
    let next = Screen::for_window(
        (window.physical_width(), window.physical_height()),
        Some(screen.mode),
    );
    if next == *screen {
        return;
    }
    *screen = next;

    if next.is_hostable() {
        info!(
            "window {}×{} -> scale {:.3}× -> grid {}×{}",
            next.window.0,
            next.window.1,
            next.scale(),
            next.grid.cols,
            next.grid.rows,
        );
    } else {
        warn!(
            "window {}×{} is scale {:.3}×, below the {MIN_SCALE}× floor — \
             the picture wants {}×{}",
            next.window.0,
            next.window.1,
            next.scale(),
            orbs_render::PICTURE.0,
            orbs_render::PICTURE.1,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_window_arriving_does_not_replace_the_mode_the_screen_already_holds() {
        // The shipped defect, as its own rule: `track_window` derived *"has a
        // mode been chosen?"* from `window != (0, 0)`, and the two stopped
        // meaning the same thing when `Screen::default` learned to read the
        // persisted `focus`. Running in `Startup`, the first window threw the
        // setting away.
        //
        // Stated against `Screen::for_window` rather than the system, which
        // wants a `Window` entity, and because the rule is the whole of what was
        // wrong: a mode in hand is carried, never re-derived.
        for mode in orbs_render::DisplayMode::ALL {
            let held = Screen {
                mode,
                ..Screen::default()
            };
            let next = Screen::for_window((2880, 1620), Some(held.mode));
            assert_eq!(
                next.mode, mode,
                "a window arriving replaced {mode:?} with {:?}",
                next.mode,
            );
        }
    }

    #[test]
    fn a_screen_with_no_window_yet_still_carries_a_mode() {
        // The other half: the sentinel is gone, so `Screen::default` has to be
        // a complete answer on its own. It is — `mode` falls back to `Wide`
        // when nothing is remembered, which is what §9 says this frontend opens
        // in.
        let fresh = Screen::default();
        assert_eq!(fresh.window, (0, 0), "a fresh screen claims a window");
        assert_eq!(
            fresh.mode,
            orbs_render::DisplayMode::Wide,
            "with no setting kept, the default is not what §9 says",
        );
    }
}
