//! The window, and how big a cell of the fixed grid is on it.
//!
//! The picture is 4:3 and the grid is a constant ([`orbs_render::GRID`]), so a
//! window decides the *size* of a cell and never the number of them (§4, §19).
//! Feeding it a window and drawing the letterbox are the Bevy frontend's job.
//!
//! A frontend with no window still answers `grid`, `mode` and `scale`: hand it
//! [`orbs_render::PICTURE`] and the scale is `1.00`, as [`mod@super::dump`]
//! does.
//!
//! `window` stays pixels rather than a resolved `f32` to keep `Eq`, which
//! `shell::window::track_window` uses to skip a repaint and as its "has the
//! player chosen a mode yet?" sentinel.

// `bevy_ecs::Resource` is the same trait `bevy::prelude::Resource` is — `bevy`
// re-exports this crate.
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
    /// A setting, not a heuristic: both modes grant identical capacity and the
    /// player may override the default at any time, mid-siege included (§9).
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
    /// Spelled out rather than derived: [`DisplayMode`]'s derived default is
    /// `Deep` and this frontend opens Wide (§9), so a derive would silently
    /// disagree with [`Screen::for_window`].
    fn default() -> Self {
        Self {
            grid: GRID,
            // What the player last chose. A setting that did not survive a
            // relaunch is a heuristic with extra steps; `F4` writes it back.
            mode: crate::settings::get(crate::settings::FOCUS)
                .and_then(|word| DisplayMode::named(&word))
                .unwrap_or(DisplayMode::Wide),
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
    /// Two ways to fail, both reachable: a grid below the authoring floor
    /// (`ORBS_GRID`, a shrunk terminal), or a window too small for the glyphs
    /// to be letters (dragging). Before the grid was fixed a small window *was*
    /// a small grid, so the first test was the only one.
    #[must_use]
    pub fn is_hostable(self) -> bool {
        self.grid.fits(MIN_GRID) && self.scale() >= MIN_SCALE
    }

    /// Resolve a window, keeping `mode` if the player has already chosen one.
    ///
    /// The grid does not enter into it. Wide by default: §9's two panes are
    /// hostable from the first frame, so `F4` works without a bigger window.
    #[must_use]
    pub fn for_window(pixels: (u32, u32), mode: Option<DisplayMode>) -> Self {
        Self {
            grid: GRID,
            mode: mode.unwrap_or(DisplayMode::Wide),
            window: pixels,
        }
    }

    /// A screen with no window behind it — a terminal, or a dump.
    ///
    /// The constructor three callers were open-coding, and had drifted apart
    /// on the mode. [`orbs_render::PICTURE`] is the honest stand-in: [`scale`]
    /// comes out at exactly `MIN_SCALE`, so hostability turns on the *grid*,
    /// which `for_window` hard-codes and this overrides.
    ///
    /// [`scale`]: Self::scale
    #[must_use]
    pub fn windowless(grid: GridSize, mode: Option<DisplayMode>) -> Self {
        Self {
            grid,
            ..Self::for_window(
                (
                    u32::from(orbs_render::PICTURE.0),
                    u32::from(orbs_render::PICTURE.1),
                ),
                mode,
            )
        }
    }

    /// The mode the player has settled on.
    ///
    /// Window size and font scale are proxies for visual acuity, not
    /// measurements, so the automatic choice is only a default — and once
    /// overridden it survives every resize (§9).
    #[must_use]
    pub const fn flipped(self) -> DisplayMode {
        // The rule is `DisplayMode`'s — a frontend holding only a mode should
        // not have to build a screen to ask it.
        self.mode.flipped()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use orbs_render::{ScreenLayout, ScreenRequest};

    #[test]
    fn every_window_gets_the_same_grid() {
        // The window buys a bigger glyph and never more cells, replacing the
        // tier table §9 used to carry.
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
        // Panes, borders and the prompt are laid out once: at three very
        // different windows the layout is identical, so no sentence that
        // fitted stops fitting.
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
        // Why 720 is the picture's height: common display heights are whole
        // multiples of it, so three of these four are pixel-exact.
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
        // The picture itself is scale 1.00, exactly the floor. A terminal has
        // no pixels to report and must not be told its screen is too small.
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
        // `F4` used to drop a fidelity tier too, so the modes had different
        // grids; now only the tiling differs.
        let window = (1920u32, 1080);
        let wide = Screen::for_window(window, Some(DisplayMode::Wide));
        let deep = Screen::for_window(window, Some(DisplayMode::Deep));

        assert_eq!(wide.grid, deep.grid);
        assert_eq!(wide.scale(), deep.scale());
        assert_ne!(wide.mode, deep.mode);
    }

    #[test]
    fn both_halves_of_the_hostable_test_are_reachable() {
        // A window too small for letters, reached by dragging...
        let squinting = Screen::for_window((640, 480), None);
        assert!(squinting.grid.fits(MIN_GRID), "the grid is fixed and fits");
        assert!(
            !squinting.is_hostable(),
            "scale {} passed",
            squinting.scale()
        );

        // ...and a grid below the authoring floor. Before the grid was fixed
        // these were the same test.
        let cramped = Screen {
            grid: GridSize::new(40, 10),
            ..Screen::for_window((1280, 720), None)
        };
        assert!(cramped.scale() >= MIN_SCALE, "the window is fine");
        assert!(!cramped.is_hostable(), "a 40x10 grid passed");
    }

    #[test]
    fn the_default_opens_wide_rather_than_deep() {
        // `DisplayMode`'s derived default is `Deep`, so `Screen`'s `Default` is
        // written out. That trap, as a test.
        assert_eq!(Screen::default().mode, DisplayMode::Wide);
        assert_eq!(Screen::default().grid, GRID);
    }
}
