//! The window, and the cell grid it resolves to.
//!
//! DESIGN.md §4: cell size is an integer multiple of the 8×16 bitmap cell, and
//! the multiplier is chosen so tier 1 lands near 80×22 on any common window.
//! That arithmetic lives in `orbs-render` because layout is computed against the
//! grid; this module only feeds it the window's pixels and holds the answer.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use orbs_render::{DisplayMode, Fidelity, GridSize, MIN_GRID};

/// The grid the current window resolves to.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Screen {
    /// The active tier, or `None` when the window cannot host the 80×22 floor.
    pub(crate) fidelity: Option<Fidelity>,
    /// Cells available at that tier.
    pub(crate) grid: GridSize,
    /// How the main window divides among its panes.
    ///
    /// §9 makes this a **setting, not a heuristic**: both modes grant identical
    /// capacity, panes, information and synergies, and the player may override
    /// the default at any time including mid-siege. The window size only picks
    /// where it starts.
    pub(crate) mode: DisplayMode,
}

impl Screen {
    /// Whether the window can host the game at all.
    ///
    /// `false` is a real state to render — a "window too small" screen — not a
    /// reason to panic. See `ScreenLayout::compute`.
    pub(crate) fn is_hostable(self) -> bool {
        self.grid.fits(MIN_GRID)
    }

    /// Resolve a window, keeping `mode` if the player has already chosen one.
    ///
    /// **Deep focus raises fidelity one step** (§9): *"fidelity rises one step;
    /// every pane is drawn at full size in a grid."* That is not a detail — it is
    /// the whole mechanism. `tier_one` returns the largest scale that still fits
    /// 80×22, so a bigger window buys a bigger glyph rather than more cells, and
    /// the grid sits near 80×22 at every window size. Multiplexing needs cells,
    /// and `Fidelity::deep` is where they come from.
    ///
    /// `deep()` was built and tested in the Frame-boundary item and never called
    /// by the game, so until now Deep focus could not actually be reached on any
    /// window — the layout had the cells for one pane and no more.
    fn for_window(pixels: (u32, u32), mode: Option<DisplayMode>) -> Self {
        let base = Fidelity::tier_one(pixels);
        let mode = mode.unwrap_or_else(|| {
            base.map_or(DisplayMode::Wide, |tier| {
                DisplayMode::default_for(tier.grid(pixels))
            })
        });

        // `deep()` is `None` at the finest scale — there is no smaller whole-pixel
        // step, so Deep focus is simply unavailable there and Wide serves instead.
        let fidelity = match mode {
            DisplayMode::Deep => base.and_then(Fidelity::deep).or(base),
            DisplayMode::Wide => base,
        };
        Self {
            fidelity,
            grid: fidelity.map_or_else(GridSize::default, |tier| tier.grid(pixels)),
            mode,
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
/// only the rendering differs. What changes on screen is the glyph size and how
/// the panes are divided, never what the game will let you do.
pub(crate) fn cycle_mode(
    window: Option<Single<&Window, With<PrimaryWindow>>>,
    mut screen: ResMut<Screen>,
) {
    let Some(window) = window else {
        return;
    };
    let wanted = screen.flipped();
    *screen = Screen::for_window(
        (window.physical_width(), window.physical_height()),
        Some(wanted),
    );
    info!(
        "focus {:?} -> tier {}x -> grid {}x{}",
        screen.mode,
        screen.fidelity.map_or(0, |tier| tier.scale()),
        screen.grid.cols,
        screen.grid.rows,
    );
}

/// The camera every frontend screen is drawn through.
pub(crate) fn spawn_camera(mut commands: Commands) {
    // The CRT is a property of the camera it curves (§4).
    //
    // MSAA is off deliberately: every edge on screen is a bitmap glyph on an
    // integer-scaled grid, so there is nothing to antialias that is not supposed
    // to be hard, and multisampling can only soften the font §4 says legibility
    // depends on.
    //
    // It is *not* why the CRT used to flash — that was the pass missing its
    // system set (see `crt::plugin`). Turning MSAA off changed the odds enough
    // to look like a fix under screenshot sampling, which is a good reminder
    // that "the symptom went away in my measurement" is not a diagnosis.
    commands.spawn((Camera2d, Msaa::Off, crate::crt::CrtSettings::default()));
}

/// Recompute the grid whenever the window changes size.
pub(crate) fn track_window(
    window: Option<Single<&Window, With<PrimaryWindow>>>,
    mut screen: ResMut<Screen>,
) {
    let Some(window) = window else {
        return;
    };

    // The mode is a setting, so a resize must not silently undo the player's
    // choice — it is carried through rather than re-derived.
    let chosen = (*screen != Screen::default()).then_some(screen.mode);
    let next = Screen::for_window((window.physical_width(), window.physical_height()), chosen);
    if next == *screen {
        return;
    }
    *screen = next;

    match next.fidelity {
        Some(tier) if next.is_hostable() => info!(
            "window {}×{} -> tier {}× -> grid {}×{}",
            window.physical_width(),
            window.physical_height(),
            tier.scale(),
            next.grid.cols,
            next.grid.rows,
        ),
        _ => warn!(
            "window {}×{} is below the {}×{} floor",
            window.physical_width(),
            window.physical_height(),
            MIN_GRID.cols,
            MIN_GRID.rows,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_grid_keeps_its_tier_at_every_common_size() {
        // A safe-area margin was tried here first, to keep the barrel warp from
        // eating the outermost column. It worked and it was wrong: `tier_one`
        // returns the *largest* scale that still fits 80×22, so the floor has
        // zero headroom and shrinking the pixel budget drops a whole tier — the
        // 1280×720 default fell from 2× to 1×, i.e. 8×16 physical-pixel glyphs.
        // Smaller text everywhere is a worse defect than a lost column, and it
        // puts the game off §9's tier table.
        //
        // The warp is normalised in `crt.wgsl` instead, which costs no cells.
        for (window, scale) in [
            ((1280u32, 720u32), 2),
            ((1920, 1080), 3),
            ((2560, 1440), 4),
            ((3840, 2160), 6),
        ] {
            let screen = Screen::for_window(window, Some(DisplayMode::Wide));
            assert_eq!(
                screen.fidelity.expect("hosted").scale(),
                scale,
                "{window:?} left §9's tier table",
            );
            assert!(screen.is_hostable(), "{window:?} lost the 80x22 floor");
        }
    }

    #[test]
    fn deep_focus_buys_the_cells_that_multiplexing_needs() {
        // §9: "fidelity rises one step; every pane is drawn at full size in a
        // grid." `tier_one` returns the largest scale that fits 80×22, so a
        // bigger window buys a bigger glyph and not more cells — Deep focus is
        // where the cells come from, and without this the layout could never
        // host more than one pane at any window size.
        for window in [(1920u32, 1080u32), (2560, 1440), (3024, 1834)] {
            let wide = Screen::for_window(window, Some(DisplayMode::Wide));
            let deep = Screen::for_window(window, Some(DisplayMode::Deep));

            assert!(
                deep.grid.cols > wide.grid.cols && deep.grid.rows > wide.grid.rows,
                "{window:?}: deep {:?} is no roomier than wide {:?}",
                deep.grid,
                wide.grid,
            );
            assert!(deep.is_hostable(), "{window:?}: deep lost the floor");
            assert!(
                deep.grid.fits(orbs_render::DEEP_FOCUS_FLOOR),
                "{window:?}: deep {:?} cannot host a second pane",
                deep.grid,
            );
        }
    }

    #[test]
    fn the_finest_scale_falls_back_to_wide_rather_than_failing() {
        // `Fidelity::deep` is `None` at scale 1: there is no smaller whole-pixel
        // step. Deep focus is simply unavailable there.
        let smallest = Screen::for_window((640, 352), Some(DisplayMode::Deep));
        assert_eq!(smallest.fidelity.map(|tier| tier.scale()), Some(1));
        assert!(smallest.is_hostable());
    }

    #[test]
    fn a_window_too_small_is_still_reported_rather_than_hidden() {
        let tiny = Screen::for_window((320, 200), None);
        assert!(!tiny.is_hostable());
    }
}
