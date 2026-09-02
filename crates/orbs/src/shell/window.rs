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
/// **It no longer changes the glyph size**, and that is the one thing this
/// change cost. Deep focus used to drop a fidelity tier as well as re-dividing
/// the panes, roughly doubling the cells; with one grid there is no tier to
/// drop, so `F4` is purely `tiling::deep` against `tiling::wide`. §19 records
/// what that owes back — §9 sold Wide focus as the large-text mode, and the
/// affordance that replaces it is a font-scale setting, not this key.
pub(crate) fn cycle_mode(mut screen: ResMut<Screen>) {
    screen.mode = screen.flipped();
    info!(
        "focus {:?} -> grid {}x{}",
        screen.mode, GRID.cols, GRID.rows
    );
}

/// The camera every frontend screen is drawn through — **and the letterbox**.
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
        // **This one line is the 4:3 letterbox.** `AutoMin` shows at least this
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
        // **Seeded from the environment**, so the tube's state is reachable
        // without a keypress. `F3` still cycles it; this is what lets a See-it
        // line capture the one property §14's accommodation turns on — that
        // turning the tube *off* does not turn the accommodation off with it.
        // `ORBS_CAPTURE` presses no keys, so without this that property could
        // only ever be checked by a person, and it is the property most worth
        // checking automatically. Phase 13's settings screen replaces it.
        crate::crt::seeded(),
    ));
}

/// Note the window's size whenever it changes.
///
/// **It no longer recomputes anything.** The grid is fixed and the projection
/// letterboxes itself, so all this does is keep [`Screen::window`] current for
/// the two things that still read physical pixels: the CRT's cell size, which
/// keeps the scanlines landing on real pixels, and [`Screen::is_hostable`].
pub(crate) fn track_window(
    window: Option<Single<&Window, With<PrimaryWindow>>>,
    mut screen: ResMut<Screen>,
) {
    let Some(window) = window else {
        return;
    };

    // The mode is a setting, so a resize must not silently undo the player's
    // choice — it is carried through rather than re-derived.
    let chosen = (screen.window != (0, 0)).then_some(screen.mode);
    let next = Screen::for_window((window.physical_width(), window.physical_height()), chosen);
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
