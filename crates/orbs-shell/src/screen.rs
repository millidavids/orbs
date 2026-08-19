//! The window, and how big a cell of the fixed grid is on it.
//!
//! DESIGN.md §4 and §19: the picture is **4:3** and the grid is a constant
//! ([`orbs_render::GRID`]), so a window decides the *size* of a cell and never
//! the number of them. This module holds the answer; feeding it a window and
//! drawing the letterbox are the Bevy frontend's, in [`super::window`].
//!
//! This used to resolve a fidelity tier — an integer scale picked so the grid
//! landed near 80×22 whatever the window — which meant every resize moved every
//! rectangle on screen. §19 records why that went.
//!
//! # Nothing here is Bevy's, and nothing here is pixels-only
//!
//! Every painter that takes a `&Screen` reads `grid`, `mode` and — at the rail's
//! foot and in the session title — `scale`. A frontend with no window still has
//! to answer all three, and [`mod@super::dump`] has always shown how: hand it
//! [`orbs_render::PICTURE`] and the scale is `1.00`, which for a surface drawing
//! one cell per cell is as true as it is for a 1280×720 window.
//!
//! **`window` stays pixels rather than a resolved `f32`** so this keeps `Eq`,
//! which [`super::window::track_window`] leans on twice: once to skip a repaint
//! when nothing moved, and once as the "has the player chosen a mode yet?"
//! sentinel.

// The one engine item this file uses, and `bevy_ecs::Resource` is the same trait
// `bevy::prelude::Resource` is — `bevy` re-exports this crate, and the lockfile
// holds exactly one copy of it.
use bevy_ecs::prelude::Resource;
use orbs_render::{DisplayMode, GRID, GridSize, MIN_GRID, MIN_SCALE};

/// The window, and the grid drawn on it.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Screen {
    /// Cells. Always [`GRID`] under the Bevy frontend; a field rather than a
    /// constant because `paint` and `ORBS_DUMP` read it the same way, and a dump
    /// can be handed the authoring floor instead — as can a terminal, which is
    /// whatever size the user made it.
    pub grid: GridSize,
    /// How the main window divides among its panes.
    ///
    /// §9 makes this a **setting, not a heuristic**: both modes grant identical
    /// capacity, panes, information and synergies, and the player may override
    /// the default at any time including mid-siege.
    pub mode: DisplayMode,
    /// The window in physical pixels — everything [`Screen::scale`] needs.
    ///
    /// A frontend without one passes [`orbs_render::PICTURE`], which is the
    /// 1:1 answer. See the module note.
    pub window: (u32, u32),
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
    #[must_use]
    pub fn scale(self) -> f32 {
        orbs_render::scale_for(self.window)
    }

    /// Whether the window can host the game at all.
    ///
    /// `false` is a real state to render — a "window too small" screen — not a
    /// reason to panic. See `ScreenLayout::compute`.
    ///
    /// **Two ways to fail, one screen**, and both are reachable. The grid can be
    /// below the authoring floor, which `ORBS_GRID` and a shrunk terminal both
    /// produce; or the window can be too small for the glyphs to be letters,
    /// which is the case a player reaches by dragging. Before the grid was fixed
    /// the first test was the only one there was, because a small window *was* a
    /// small grid.
    #[must_use]
    pub fn is_hostable(self) -> bool {
        self.grid.fits(MIN_GRID) && self.scale() >= MIN_SCALE
    }

    /// Resolve a window, keeping `mode` if the player has already chosen one.
    ///
    /// The grid does not enter into it. **Wide by default, always** — §9's two
    /// panes are hostable from the first frame, so `F4` works immediately rather
    /// than needing a bigger window, and there is nothing left for a heuristic
    /// to decide.
    #[must_use]
    pub fn for_window(pixels: (u32, u32), mode: Option<DisplayMode>) -> Self {
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
    #[must_use]
    pub const fn flipped(self) -> DisplayMode {
        match self.mode {
            DisplayMode::Deep => DisplayMode::Wide,
            DisplayMode::Wide => DisplayMode::Deep,
        }
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
    fn a_frontend_with_no_window_is_still_hostable() {
        // What `ORBS_DUMP` passes, and what `orbs-tui` does: the picture itself,
        // which is scale 1.00 — the floor, exactly. A terminal has no pixels to
        // report and must not therefore be told its screen is too small.
        let picture = orbs_render::PICTURE;
        let windowless = Screen::for_window((u32::from(picture.0), u32::from(picture.1)), None);
        assert_eq!(windowless.scale(), MIN_SCALE);
        assert!(
            windowless.is_hostable(),
            "a windowless frontend was refused"
        );
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

        // ...and a grid below the authoring floor, which `ORBS_GRID` and a
        // shrunk terminal both produce. Before the grid was fixed these were the
        // same test.
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
