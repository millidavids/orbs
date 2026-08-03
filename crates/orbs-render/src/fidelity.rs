//! Fidelity tiers — how many cells a window is worth.
//!
//! DESIGN.md §4 and §9: cell size is an **integer** multiple of the 8×16 bitmap
//! cell, because bitmap fonts must scale by whole pixels to stay crisp.
//! Tier 1 is the largest scale that still meets the 80×22 floor; engaging
//! multiplexing drops one step, roughly doubling the available cells so four
//! panes fit at full function.
//!
//! This lives in `orbs-render` rather than in the Bevy frontend because the
//! *grid* is what layout is computed against, and layout is this crate's job.
//! The frontend supplies window pixels and receives a [`GridSize`]; what it does
//! with the leftover pixels — letterboxing, centring — is its own business.
//!
//! The terminal frontend has no use for any of this: a terminal is whatever size
//! the user made it, so `orbs-tui` reports its grid directly and never consults
//! a tier.

use core::num::NonZeroU8;

use crate::geometry::GridSize;

/// Width of one bitmap cell at scale 1, in pixels.
pub const CELL_WIDTH: u16 = 8;

/// Height of one bitmap cell at scale 1, in pixels.
pub const CELL_HEIGHT: u16 = 16;

/// The declared minimum grid. All layout is authored against this floor.
pub const MIN_GRID: GridSize = GridSize::new(80, 22);

/// An integer scale factor on the 8×16 cell.
///
/// Larger scale means bigger glyphs and fewer cells. Non-zero by construction —
/// a scale of zero would be a cell of no pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Fidelity(NonZeroU8);

impl Fidelity {
    /// Scale 1 — one screen pixel per font pixel. The finest tier there is.
    pub const FINEST: Self = Self(NonZeroU8::MIN);

    /// A fidelity at an explicit scale.
    #[must_use]
    pub const fn new(scale: NonZeroU8) -> Self {
        Self(scale)
    }

    /// The scale factor.
    #[must_use]
    pub const fn scale(self) -> u8 {
        self.0.get()
    }

    /// The size of one cell in pixels at this scale.
    #[must_use]
    pub fn cell_pixels(self) -> (u16, u16) {
        let scale = u16::from(self.scale());
        (
            CELL_WIDTH.saturating_mul(scale),
            CELL_HEIGHT.saturating_mul(scale),
        )
    }

    /// The grid a window of `window` pixels yields at this scale.
    ///
    /// Leftover pixels are discarded; the frontend decides whether to letterbox
    /// or centre what remains.
    #[must_use]
    pub fn grid(self, window: (u32, u32)) -> GridSize {
        let (cell_width, cell_height) = self.cell_pixels();
        let cols = window.0 / u32::from(cell_width);
        let rows = window.1 / u32::from(cell_height);
        GridSize::new(
            u16::try_from(cols).unwrap_or(u16::MAX),
            u16::try_from(rows).unwrap_or(u16::MAX),
        )
    }

    /// The default tier for a window: the largest scale whose grid still meets
    /// [`MIN_GRID`].
    ///
    /// `None` means the window is too small to host the game at all — below
    /// roughly 640×352 — which is a real answer the frontend must handle rather
    /// than a case to clamp away.
    #[must_use]
    pub fn tier_one(window: (u32, u32)) -> Option<Self> {
        // The grid shrinks monotonically as scale rises, so the first failure is
        // one past the last candidate.
        let mut best = None;
        let mut scale = NonZeroU8::MIN;
        loop {
            let candidate = Self(scale);
            if !candidate.grid(window).fits(MIN_GRID) {
                return best;
            }
            best = Some(candidate);
            let Some(next) = scale.checked_add(1) else {
                return best;
            };
            scale = next;
        }
    }

    /// One step finer — the tier Deep-focus multiplexing engages (§9).
    ///
    /// `None` at scale 1: there is no smaller whole-pixel step, so Deep focus is
    /// simply unavailable on that window and Wide focus serves instead.
    #[must_use]
    pub fn deep(self) -> Option<Self> {
        NonZeroU8::new(self.scale() - 1).map(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One row of the tier table in DESIGN.md §9.
    fn assert_tiers(window: (u32, u32), one: (u8, GridSize), two: (u8, GridSize)) {
        let tier_one = Fidelity::tier_one(window).expect("window supports the floor");
        assert_eq!(tier_one.scale(), one.0, "tier 1 scale at {window:?}");
        assert_eq!(tier_one.grid(window), one.1, "tier 1 grid at {window:?}");

        let tier_two = tier_one.deep().expect("a finer step exists");
        assert_eq!(tier_two.scale(), two.0, "tier 2 scale at {window:?}");
        assert_eq!(tier_two.grid(window), two.1, "tier 2 grid at {window:?}");
    }

    /// The table in DESIGN.md §9, verbatim. If this fails, either the formula
    /// drifted or the design's own numbers are wrong — both are worth stopping
    /// for.
    #[test]
    fn the_design_tier_table_reproduces_exactly() {
        let floor = GridSize::new(80, 22);
        assert_tiers((1920, 1080), (3, floor), (2, GridSize::new(120, 33)));
        assert_tiers((2560, 1440), (4, floor), (3, GridSize::new(106, 30)));
        assert_tiers((1280, 720), (2, floor), (1, GridSize::new(160, 45)));
    }

    #[test]
    fn tier_one_always_meets_the_floor() {
        for width in (640..3840).step_by(97) {
            for height in (352..2160).step_by(89) {
                let window = (width, height);
                if let Some(tier) = Fidelity::tier_one(window) {
                    assert!(
                        tier.grid(window).fits(MIN_GRID),
                        "tier 1 at {window:?} fell below the floor"
                    );
                }
            }
        }
    }

    #[test]
    fn tier_one_is_the_largest_scale_that_fits() {
        let window = (1920, 1080);
        let tier = Fidelity::tier_one(window).expect("supported");
        let coarser = Fidelity::new(NonZeroU8::new(tier.scale() + 1).expect("non-zero"));
        assert!(!coarser.grid(window).fits(MIN_GRID));
    }

    #[test]
    fn a_window_below_the_floor_has_no_tier() {
        // 80x22 at scale 1 needs 640x352.
        assert_eq!(Fidelity::tier_one((639, 352)), None);
        assert_eq!(Fidelity::tier_one((640, 351)), None);
        assert!(Fidelity::tier_one((640, 352)).is_some());
    }

    #[test]
    fn deep_focus_is_unavailable_at_the_finest_scale() {
        assert_eq!(Fidelity::FINEST.deep(), None);
    }

    #[test]
    fn deep_focus_yields_strictly_more_cells() {
        let window = (1920, 1080);
        let one = Fidelity::tier_one(window).expect("supported");
        let two = one.deep().expect("a finer step exists");
        assert!(two.grid(window).area() > one.grid(window).area());
    }
}
