//! The window, and how big a cell of the fixed grid is on it.
//!
//! DESIGN.md §4 and §19: the picture is **4:3** and the grid is a constant
//! ([`orbs_render::GRID`]), so a window decides the *size* of a cell and never
//! the number of them. This module feeds `orbs-render` the window's pixels and
//! holds the answer; the letterbox itself is the camera's, in
//! [`spawn_camera`].
//!
//! This used to resolve a fidelity tier — an integer scale picked so the grid
//! landed near 80×22 whatever the window — which meant every resize moved every
//! rectangle on screen. §19 records why that went.

use bevy::camera::ScalingMode;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use orbs_render::{DisplayMode, GRID, GridSize, MIN_GRID, MIN_SCALE};

/// The window, and the grid drawn on it.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Screen {
    /// Cells. Always [`GRID`] under this frontend; a field rather than a
    /// constant because `paint` and `ORBS_DUMP` read it the same way, and a dump
    /// can be handed the authoring floor instead.
    pub(crate) grid: GridSize,
    /// How the main window divides among its panes.
    ///
    /// §9 makes this a **setting, not a heuristic**: both modes grant identical
    /// capacity, panes, information and synergies, and the player may override
    /// the default at any time including mid-siege.
    pub(crate) mode: DisplayMode,
    /// The window in physical pixels — everything [`Screen::scale`] needs.
    ///
    /// **The pixels, not the scale.** A resolved `f32` would cost this `Eq`,
    /// which [`track_window`] leans on twice: once to skip a repaint when
    /// nothing moved, and once as the "has the player chosen a mode yet?"
    /// sentinel.
    pub(crate) window: (u32, u32),
}

impl Default for Screen {
    /// Before any window has been seen.
    ///
    /// **Spelled out rather than derived**, because [`DisplayMode`]'s own
    /// derived default is `Deep` and this frontend opens Wide (§9). A derive
    /// here would silently disagree with [`Screen::for_window`].
    fn default() -> Self {
        Self {
            grid: GRID,
            mode: DisplayMode::Wide,
            window: (0, 0),
        }
    }
}

impl Screen {
    /// Physical pixels per virtual pixel — how big a cell is drawn.
    ///
    /// The same number `ScalingMode::AutoMin` reaches inside the projection; see
    /// [`orbs_render::scale_for`] for why it is computed twice in two units.
    pub(crate) fn scale(self) -> f32 {
        orbs_render::scale_for(self.window)
    }

    /// Whether the window can host the game at all.
    ///
    /// `false` is a real state to render — a "window too small" screen — not a
    /// reason to panic. See `ScreenLayout::compute`.
    ///
    /// **Two ways to fail, one screen**, and both are reachable. The grid can be
    /// below the authoring floor, which only `ORBS_GRID` can now produce; or the
    /// window can be too small for the glyphs to be letters, which is the case a
    /// player reaches by dragging. Before the grid was fixed the first test was
    /// the only one there was, because a small window *was* a small grid.
    pub(crate) fn is_hostable(self) -> bool {
        self.grid.fits(MIN_GRID) && self.scale() >= MIN_SCALE
    }

    /// Resolve a window, keeping `mode` if the player has already chosen one.
    ///
    /// The grid does not enter into it. **Wide by default, always** — §9's two
    /// panes are hostable from the first frame, so `F4` works immediately rather
    /// than needing a bigger window, and there is nothing left for a heuristic
    /// to decide.
    fn for_window(pixels: (u32, u32), mode: Option<DisplayMode>) -> Self {
        Self {
            grid: GRID,
            mode: mode.unwrap_or(DisplayMode::Wide),
            window: pixels,
        }
    }

    /// The mode the player has settled on.
    ///
    /// §9: *"the player be able to override it at any time, including
    /// mid-siege."* Window size and font scale are proxies for visual acuity,
    /// not measurements of it, so the automatic choice is only ever a default —
    /// and once overridden it survives every resize.
    const fn flipped(self) -> DisplayMode {
        match self.mode {
            DisplayMode::Deep => DisplayMode::Wide,
            DisplayMode::Wide => DisplayMode::Deep,
        }
    }
}

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
        crate::crt::CrtSettings::default(),
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

#[cfg(test)]
mod tests {
    use super::*;

    use orbs_render::{ScreenLayout, ScreenRequest};

    #[test]
    fn every_window_gets_the_same_grid() {
        // The request, stated as an assertion: the window buys a bigger glyph
        // and never more cells. This replaces the tier table §9 used to carry,
        // which said the opposite in four rows.
        for window in [
            (1280u32, 720u32),
            (1920, 1080),
            (2560, 1440),
            (3840, 2160),
            (3440, 1440),
            (800, 1200),
            (320, 200),
        ] {
            let screen = Screen::for_window(window, Some(DisplayMode::Wide));
            assert_eq!(screen.grid, GRID, "{window:?} moved the grid");
        }
    }

    #[test]
    fn a_resize_moves_no_rectangle() {
        // **This is the whole point of the change.** Panes, borders and the
        // prompt are laid out once and never again: at three very different
        // windows the layout is identical, so nothing reflows and no sentence
        // that fitted stops fitting.
        let layout = |window| {
            let screen = Screen::for_window(window, Some(DisplayMode::Wide));
            ScreenLayout::compute(&ScreenRequest {
                main_panes: 2,
                ..ScreenRequest::single(screen.grid)
            })
        };

        let opening = layout((1280u32, 720u32));
        assert_eq!(opening, layout((3840, 2160)), "4K reflowed the screen");
        assert_eq!(opening, layout((800, 1200)), "a tall window reflowed it");
    }

    #[test]
    fn the_glyph_grows_with_the_window_even_though_the_grid_does_not() {
        // The other half of the same bargain, and the reason 720 was chosen for
        // the picture's height: the common display heights are whole multiples
        // of it, so three of these four are pixel-exact.
        for (window, scale) in [
            ((1280u32, 720u32), 1.0),
            ((1920, 1080), 1.5),
            ((2560, 1440), 2.0),
            ((3840, 2160), 3.0),
        ] {
            let screen = Screen::for_window(window, Some(DisplayMode::Wide));
            assert_eq!(screen.scale(), scale, "{window:?} left the scale table");
            assert!(screen.is_hostable(), "{window:?} is below the floor");
        }
    }

    #[test]
    fn focus_changes_the_split_and_not_the_grid() {
        // What `F4` cost. It used to drop a fidelity tier as well, so the two
        // modes had different grids; now only the tiling differs.
        let window = (1920u32, 1080);
        let wide = Screen::for_window(window, Some(DisplayMode::Wide));
        let deep = Screen::for_window(window, Some(DisplayMode::Deep));

        assert_eq!(wide.grid, deep.grid);
        assert_eq!(wide.scale(), deep.scale());
        assert_ne!(wide.mode, deep.mode);
    }

    #[test]
    fn both_halves_of_the_hostable_test_are_reachable() {
        // A window too small for letters, which is what a player reaches by
        // dragging...
        let squinting = Screen::for_window((640, 480), None);
        assert!(squinting.grid.fits(MIN_GRID), "the grid is fixed and fits");
        assert!(
            !squinting.is_hostable(),
            "scale {} passed",
            squinting.scale()
        );

        // ...and a grid below the authoring floor, which only `ORBS_GRID` can
        // now produce. Before the grid was fixed these were the same test.
        let cramped = Screen {
            grid: GridSize::new(40, 10),
            ..Screen::for_window((1280, 720), None)
        };
        assert!(cramped.scale() >= MIN_SCALE, "the window is fine");
        assert!(!cramped.is_hostable(), "a 40x10 grid passed");
    }

    #[test]
    fn the_default_opens_wide_rather_than_deep() {
        // `DisplayMode`'s own derived default is `Deep`, so `Screen`'s `Default`
        // is written out. This is that trap, as a test.
        assert_eq!(Screen::default().mode, DisplayMode::Wide);
        assert_eq!(Screen::default().grid, GRID);
    }
}
