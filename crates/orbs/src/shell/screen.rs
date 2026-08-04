//! The window, and the cell grid it resolves to.
//!
//! DESIGN.md §4: cell size is an integer multiple of the 8×16 bitmap cell, and
//! the multiplier is chosen so tier 1 lands near 80×22 on any common window.
//! That arithmetic lives in `orbs-render` because layout is computed against the
//! grid; this module only feeds it the window's pixels and holds the answer.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use orbs_render::{Fidelity, GridSize, MIN_GRID};

/// The grid the current window resolves to.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Screen {
    /// The active tier, or `None` when the window cannot host the 80×22 floor.
    pub(crate) fidelity: Option<Fidelity>,
    /// Cells available at that tier.
    pub(crate) grid: GridSize,
}

impl Screen {
    /// Whether the window can host the game at all.
    ///
    /// `false` is a real state to render — a "window too small" screen — not a
    /// reason to panic. See `ScreenLayout::compute`.
    pub(crate) fn is_hostable(self) -> bool {
        self.grid.fits(MIN_GRID)
    }

    fn for_window(pixels: (u32, u32)) -> Self {
        let fidelity = Fidelity::tier_one(pixels);
        Self {
            fidelity,
            grid: fidelity.map_or_else(GridSize::default, |tier| tier.grid(pixels)),
        }
    }
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

    let next = Screen::for_window((window.physical_width(), window.physical_height()));
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
            let screen = Screen::for_window(window);
            assert_eq!(
                screen.fidelity.expect("hosted").scale(),
                scale,
                "{window:?} left §9's tier table",
            );
            assert!(screen.is_hostable(), "{window:?} lost the 80x22 floor");
        }
    }

    #[test]
    fn a_window_too_small_is_still_reported_rather_than_hidden() {
        let tiny = Screen::for_window((320, 200));
        assert!(!tiny.is_hostable());
    }
}
