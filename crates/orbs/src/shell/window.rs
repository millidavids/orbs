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
/// Both modes grant identical capacity (§9). Only the split changes, not the
/// glyph size (§19).
pub(crate) fn cycle_mode(mut screen: ResMut<Screen>) {
    screen.mode = screen.flipped();
    info!(
        "focus {:?} -> grid {}x{}",
        screen.mode, GRID.cols, GRID.rows
    );
    // One setting seen twice — see `crt::tube::cycle`. §9 makes the mode a
    // *setting* rather than a heuristic in as many words, and a setting that
    // does not survive a relaunch is a heuristic with extra steps.
    orbs_shell::settings::set(orbs_shell::settings::FOCUS, screen.mode.word());
}

/// The camera every frontend screen is drawn through, and the letterbox.
pub(crate) fn spawn_camera(mut commands: Commands) {
    // The CRT is a property of the camera it curves (§4).
    //
    // MSAA off: every edge is a bitmap glyph's, meant to be hard (§4).
    commands.spawn((
        Camera2d,
        // The 4:3 letterbox: scaled by `min(W/960, H/720)` and centred. Set
        // once, since `camera_system` re-derives it on every resize.
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: f32::from(orbs_render::PICTURE.0),
                min_height: f32::from(orbs_render::PICTURE.1),
            },
            ..OrthographicProjection::default_2d()
        }),
        Msaa::Off,
        // The palette is solved in sRGB and drawn as solved. Bevy 0.20's 2D
        // camera tonemaps `Linear` by default; 0.19's did not.
        bevy::core_pipeline::tonemapping::Tonemapping::None,
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
/// Keeps [`Screen::window`] current for the CRT's cell size and
/// [`Screen::is_hostable`].
pub(crate) fn track_window(
    window: Option<Single<&Window, With<PrimaryWindow>>>,
    mut screen: ResMut<Screen>,
) {
    let Some(window) = window else {
        return;
    };

    // The mode is carried, always, never re-derived from the window, or the
    // persisted `focus` is lost on the first frame.
    let next = Screen::for_window(
        (window.physical_width(), window.physical_height()),
        Some(screen.mode),
    )
    .with_fit(screen.fit);
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

/// Point the camera at the picture [`Screen`] describes.
///
/// `AutoMin` fills the window and re-derives itself on a resize, so a filled
/// picture needs nothing from here. A whole fit cannot be said that way: the
/// camera is told how much of the world to show, so a cell is exactly
/// `Screen::scale` pixels — the number the tube's cell size is read from.
pub(crate) fn fit_projection(
    screen: Res<Screen>,
    mut cameras: Query<&mut Projection, With<Camera2d>>,
) {
    let scaling = scaling_for(*screen);
    let origin = origin_for(*screen);
    for mut projection in &mut cameras {
        if let Projection::Orthographic(orthographic) = projection.as_mut() {
            orthographic.scaling_mode = scaling;
            orthographic.viewport_origin = origin;
        }
    }
}

/// Where the camera puts the picture's middle: [`Screen::middle`] as a share of
/// the window, counted from the bottom left as Bevy counts it.
fn origin_for(screen: Screen) -> Vec2 {
    let window = (
        orbs_render::pixels(screen.window.0),
        orbs_render::pixels(screen.window.1),
    );
    if window.0 < 1.0 || window.1 < 1.0 {
        return Vec2::splat(0.5);
    }
    let (x, y) = screen.middle();
    Vec2::new(x / window.0, 1.0 - y / window.1)
}

/// How the camera scales the picture for `screen`.
fn scaling_for(screen: Screen) -> ScalingMode {
    let window = (
        orbs_render::pixels(screen.window.0),
        orbs_render::pixels(screen.window.1),
    );
    if screen.fit == orbs_shell::Fit::Fill || window.0 < 1.0 || window.1 < 1.0 {
        return ScalingMode::AutoMin {
            min_width: f32::from(orbs_render::PICTURE.0),
            min_height: f32::from(orbs_render::PICTURE.1),
        };
    }
    let scale = screen.scale();
    ScalingMode::Fixed {
        width: window.0 / scale,
        height: window.1 / scale,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_whole_camera_draws_a_cell_at_the_screens_own_scale() {
        // The tube reads `Screen::scale` for its cell size; the camera must put
        // a cell at exactly that many pixels, or the scanlines miss the rows.
        for window in [(1920u32, 1080u32), (2560, 1440), (2880, 1620), (3840, 2160)] {
            let screen = Screen::for_window(window, None).with_fit(orbs_shell::Fit::Whole);
            let ScalingMode::Fixed { width, height } = scaling_for(screen) else {
                panic!("a whole fit on {window:?} was not a fixed camera");
            };
            let pixels_per_unit = orbs_render::pixels(window.0) / width;
            assert!(
                (pixels_per_unit - screen.scale()).abs() < 1e-4,
                "{window:?}: the camera draws a unit at {pixels_per_unit} and the tube reads {}",
                screen.scale(),
            );
            assert!(
                (orbs_render::pixels(window.1) / height - screen.scale()).abs() < 1e-4,
                "{window:?}: the two axes disagree",
            );
        }
    }

    #[test]
    fn the_camera_puts_the_picture_where_the_screen_says() {
        // The tube is told `Screen::middle` too, so the two must agree.
        for window in [(1921u32, 1081u32), (1920, 1080)] {
            for fit in orbs_shell::Fit::ALL {
                let screen = Screen::for_window(window, None).with_fit(fit);
                let origin = origin_for(screen);
                let (x, y) = screen.middle();
                assert!((origin.x * orbs_render::pixels(window.0) - x).abs() < 1e-3);
                assert!((orbs_render::pixels(window.1) * (1.0 - origin.y) - y).abs() < 1e-3);
            }
        }
    }

    #[test]
    fn a_window_arriving_does_not_replace_the_mode_the_screen_already_holds() {
        // A mode in hand is carried, never re-derived, against
        // `Screen::for_window`, since the system wants a `Window`.
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
