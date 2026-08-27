//! The picture — a fixed grid, and how big a cell of it is on a given window.
//!
//! DESIGN.md §4 and §9: cell size is a multiple of the 8×16 bitmap cell, and the
//! picture is **4:3**. Those two facts together decide the grid rather than
//! leaving it to the window, because a cell is twice as tall as it is wide:
//!
//! ```text
//! cols·CELL_WIDTH / (rows·CELL_HEIGHT) = 4/3   ⇔   cols : rows = 8 : 3
//! ```
//!
//! So [`GRID`] is a constant and the window decides only [`scale_for`] — how
//! many physical pixels one virtual pixel is worth. Resizing changes the size of
//! the text and nothing else: no pane moves, no border is redrawn at a new
//! width, and no sentence that fitted stops fitting.
//!
//! # What this replaced, and why
//!
//! This was `Fidelity`, an integer scale chosen *from* the window, with the grid
//! falling out of the division — 1280×720 gave 160×45 and 1920×1080 gave 120×33.
//! Every layout constant then existed to manage a grid moving under it, and `F4`
//! changed the grid as a side effect of changing the split. §19 records the
//! decision to fix the grid instead; the cost is that a window can no longer buy
//! more text by being larger, only bigger text.
//!
//! # Whose arithmetic is whose
//!
//! The frontend does not compute the letterbox — `ScalingMode::AutoMin` in the
//! camera projection does, and it works in *logical* pixels. [`scale_for`] is
//! the same number in the open, in **physical** pixels, for the two callers that
//! need it outside the projection: the CRT's cell size and the legibility floor.
//! The two agree because aspect ratio is scale-factor invariant — on a 2×
//! display the projection sees half the pixels and maps each to twice the area.
//! Do not "fix" one to match the units of the other.
//!
//! The terminal frontend has no use for any of this: a terminal is whatever size
//! the user made it, so `orbs-tui` reports its grid directly.

use crate::geometry::GridSize;

/// Width of one bitmap cell at native size, in pixels.
pub const CELL_WIDTH: u16 = 8;

/// Height of one bitmap cell at native size, in pixels.
pub const CELL_HEIGHT: u16 = 16;

/// The grid, fixed. 4:3 at native cell size, and the same 45 rows the game drew
/// before the grid stopped following the window.
///
/// 120 columns is 40 fewer than the old default window yielded, which is the
/// price of the aspect: 8:3 is the only ratio of columns to rows that is 4:3 in
/// pixels, and the next step up the family — 160×60 — puts every common display
/// on a fractional scale and costs 1080p 44% of its glyph height. See §19.
pub const GRID: GridSize = GridSize::new(120, 45);

/// [`GRID`] in pixels at native cell size — the picture the window letterboxes.
pub const PICTURE: (u16, u16) = (GRID.cols * CELL_WIDTH, GRID.rows * CELL_HEIGHT);

// The aspect, asserted rather than trusted. A grid off the 8:3 line is a 4:3
// picture that is not 4:3, which is invisible in a dump and obvious only on a
// window nobody has resized yet — so it fails to compile instead.
const _: () = assert!(
    PICTURE.0 as u32 * 3 == PICTURE.1 as u32 * 4,
    "GRID must be 8 columns to 3 rows, or the picture is not 4:3",
);

/// The declared minimum grid. All layout is authored against this floor.
///
/// [`GRID`] clears it comfortably, so nothing the game draws is ever below it.
/// It survives as the *authoring* floor: `ORBS_GRID=80x22` is how a surface is
/// checked against the tightest screen the design admits, and it is still half
/// of the frontend's "can this host the game" test, because a dump can be handed
/// a grid smaller than any window would produce.
pub const MIN_GRID: GridSize = GridSize::new(80, 22);

/// Below this the bitmap font stops getting smaller and starts losing strokes.
///
/// Minification drops whole source columns — an 8-pixel glyph drawn into 6
/// pixels is missing two of them, and which two depends on where the letter
/// sits. At [`GRID`] this floor is a 960×720 window.
pub const MIN_SCALE: f32 = 1.0;

/// Rows the input line occupies.
///
/// **One, at the same size as everything else.** Two would mean double-size
/// glyphs drawn into half the columns, and that is what this was for one
/// version — because it used to be *derived*, spending a second row at the
/// finest tier to buy back the pixel height a fine cell took away.
///
/// With one grid there is no tier to compensate for, so the doubling stopped
/// being compensation and became magnification: a prompt drawn twice the size of
/// the transcript at every window, and three times the size again at 4K. §19
/// already recorded size as *"a blunt instrument for distinguishability"*; on a
/// fixed grid it is not even measuring the right thing.
///
/// The halving mattered for more than looks: at 2× the line was written into
/// **half** the columns, so a 120-column grid gave the player 60 cells to type
/// into. It now gives 118.
pub const INPUT_ROWS: u16 = 1;

/// Physical pixels per virtual pixel: how much of the window one cell is worth.
///
/// The picture fits *inside* the window on both axes, so the smaller ratio wins
/// and the other axis gets bars. This is the number `ScalingMode::AutoMin`
/// arrives at inside the projection; see the module header for why it is
/// restated here in different units.
///
/// ```
/// use orbs_render::{PICTURE, scale_for};
///
/// // Native cell size: an 8x16 glyph, drawn as the pixels it was designed as.
/// assert_eq!(scale_for((1280, 720)), 1.0);
/// // 720 divides the common display heights, so they land on whole steps.
/// assert_eq!(scale_for((2560, 1440)), 2.0);
/// assert_eq!(scale_for((3840, 2160)), 3.0);
///
/// // **The default window.** A half step rather than a whole one, and it is
/// // only safe because the cell is even on both axes: 8x16 becomes 12x24, so
/// // every cell boundary is still a whole pixel.
/// assert_eq!(scale_for((1920, 1080)), 1.5);
///
/// // ...and the picture never exceeds the window on either axis.
/// let scale = scale_for((1920, 1080));
/// assert!(f32::from(PICTURE.0) * scale <= 1920.0);
/// assert!(f32::from(PICTURE.1) * scale <= 1080.0);
/// ```
#[must_use]
pub fn scale_for(window: (u32, u32)) -> f32 {
    let across = pixels(window.0) / f32::from(PICTURE.0);
    let down = pixels(window.1) / f32::from(PICTURE.1);
    across.min(down)
}

/// A window measurement as a float, without a lossy cast.
///
/// A window past 65535 physical pixels on one axis is not a case worth carrying
/// arithmetic for, and clamping there keeps the conversion **exact** — `u32` has
/// nine more bits than an `f32` mantissa, so the direct cast is a rounding step
/// clippy is right to refuse.
#[must_use]
pub fn pixels(measure: u32) -> f32 {
    f32::from(u16::try_from(measure).unwrap_or(u16::MAX))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_picture_is_four_by_three() {
        // The const assertion above says this at compile time; this says it in a
        // failure message, and pins the numbers §19 records.
        assert_eq!(PICTURE, (960, 720));
        assert_eq!(u32::from(PICTURE.0) * 3, u32::from(PICTURE.1) * 4);
    }

    #[test]
    fn the_grid_clears_the_authoring_floor() {
        assert!(GRID.fits(MIN_GRID), "the fixed grid is below MIN_GRID");
    }

    #[test]
    fn the_picture_fits_inside_every_window_and_touches_one_axis() {
        // The letterbox property, stated directly: scaled up by `scale_for`, the
        // picture never exceeds the window, and meets it on at least one axis —
        // which is what makes the bars appear on one axis rather than both.
        for window in [
            (1280u32, 720u32),
            (1920, 1080),
            (2560, 1440),
            (3840, 2160),
            (1366, 768),
            (800, 1200),
            (960, 720),
            (3440, 1440),
        ] {
            let scale = scale_for(window);
            let across = f32::from(PICTURE.0) * scale;
            let down = f32::from(PICTURE.1) * scale;
            let (wide, tall) = (pixels(window.0), pixels(window.1));

            assert!(
                across <= wide + 0.001 && down <= tall + 0.001,
                "{window:?} overflows: the picture is {across}×{down}",
            );
            assert!(
                (across - wide).abs() < 0.001 || (down - tall).abs() < 0.001,
                "{window:?} touches neither axis: the picture is {across}×{down}",
            );
        }
    }

    #[test]
    fn the_common_display_heights_land_on_whole_steps() {
        // Why 720 rather than 960 (§19). Every common desktop height is a whole
        // multiple of the picture's, so the glyphs are pixel-exact on three of
        // the four and a regular 3:2 on the other.
        assert_eq!(scale_for((1280, 720)), 1.0);
        assert_eq!(scale_for((1920, 1080)), 1.5);
        assert_eq!(scale_for((2560, 1440)), 2.0);
        assert_eq!(scale_for((3840, 2160)), 3.0);
    }

    #[test]
    fn a_window_narrower_than_it_is_tall_is_limited_by_its_width() {
        // Both axes constrain, which is the half of the request that a
        // height-only fit would miss.
        assert_eq!(scale_for((960, 4000)), 1.0);
        assert_eq!(scale_for((480, 720)), 0.5);
    }

    #[test]
    fn native_size_is_where_the_font_is_its_own_pixels() {
        // 1280×720 is scale 1 — the sharpest the game ever is. It **was** the
        // default window and is not; `main.rs` opens at 1920×1080 now, which is
        // scale 1.5. This test is about `scale_for`, and its name used to claim
        // it was about the default, which stopped being true without it failing.
        assert_eq!(scale_for((1280, 720)), MIN_SCALE);
    }

    /// The default window, and the property that makes 1.5 safe.
    ///
    /// A half step rather than a whole one is only survivable because the cell
    /// is even on both axes: 8×16 becomes 12×24, so every cell boundary is still
    /// a whole pixel. **That is the thing to check if the default moves again**
    /// — not whether the scale is an integer, but whether `8s` and `16s` are.
    #[test]
    fn the_default_window_lands_the_cell_on_whole_pixels() {
        let scale = scale_for((1920, 1080));
        assert!(
            (scale - 1.5).abs() < f32::EPSILON,
            "the default is not 1.5x"
        );
        for cell in [8.0_f32, 16.0] {
            let scaled = cell * scale;
            assert!(
                (scaled - scaled.round()).abs() < f32::EPSILON,
                "a cell edge of {cell} lands on {scaled}, which is half a pixel",
            );
        }
        // ...and the bars survive, which is the half of the original rationale
        // that had to be preserved.
        assert!(
            f32::from(PICTURE.0) * scale < 1920.0,
            "the letterbox is gone, so nothing exercises the 4:3 fit"
        );
    }
}
