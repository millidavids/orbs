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
    commands.spawn((Camera2d, crate::crt::CrtSettings::default()));
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
