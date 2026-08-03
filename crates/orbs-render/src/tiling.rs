//! Dividing the main window among panes.
//!
//! Split out from [`crate::layout`] because it answers a different question:
//! layout decides *which regions exist* — main window, sidebar, input line —
//! and tiling decides *how the main window is carved up* between the open panes.
//! The two change for different reasons, and the tilers are where the arithmetic
//! that has to reproduce DESIGN.md §9's figures actually lives.
//!
//! Every tiler is total and covers its area exactly: no gaps, no overlap. A gap
//! would leave stale cells from the previous frame visible; an overlap would let
//! one pane write over another's content, which is exactly the failure the
//! painter's clipping exists to prevent.
//!
//! Tilers also never *place* a rectangle with no area. `ScreenLayout::main()`
//! is what a frontend zips against its open domains, so a counted-but-invisible
//! pane would bind a domain to a rectangle that cannot draw a single cell — and
//! the exact-cover tests would not notice, because a zero-area rectangle
//! contributes zero to the sum.

use crate::geometry::Rect;
use crate::layout::{MAX_MAIN_PANES, MIN_PANE_ROWS, STRIP_ROWS};

/// How many horizontal bands Deep focus needs for `panes`.
pub(crate) fn deep_bands(panes: u16) -> u16 {
    if panes == 0 {
        0
    } else {
        panes.div_ceil(deep_columns(panes))
    }
}

/// Columns per band. Two is the widest split that keeps a pane legible at the
/// 80-column floor; three would give 26 columns each.
fn deep_columns(panes: u16) -> u16 {
    if panes <= 1 { 1 } else { 2 }
}

/// The `index`-th of `count` equal slices of `total`, remainder to the earliest
/// slices. Returns `(offset, length)`.
fn slice(total: u16, index: u16, count: u16) -> (u16, u16) {
    if count == 0 {
        return (0, 0);
    }
    let base = total / count;
    let extra = total % count;
    (
        base * index + index.min(extra),
        base + u16::from(index < extra),
    )
}

/// Row-major grid. A final short band spans the full width rather than leaving a
/// hole.
pub(crate) fn deep(area: Rect, panes: u16, out: &mut [Rect; MAX_MAIN_PANES]) -> usize {
    if panes == 0 || area.is_empty() {
        return 0;
    }

    let columns = deep_columns(panes);
    let bands = deep_bands(panes);
    let mut placed = 0u16;

    for band in 0..bands {
        let (row_offset, rows) = slice(area.rows, band, bands);
        let in_band = (panes - placed).min(columns);
        for column in 0..in_band {
            let (col_offset, cols) = slice(area.cols, column, in_band);
            if rows == 0 || cols == 0 {
                continue;
            }
            let Some(slot) = out.get_mut(usize::from(placed)) else {
                return usize::from(placed);
            };
            *slot = Rect::new(area.col + col_offset, area.row + row_offset, cols, rows);
            placed += 1;
        }
    }
    usize::from(placed)
}

/// One full-size focused pane with compact strips stacked below it.
pub(crate) fn wide(area: Rect, panes: u16, out: &mut [Rect; MAX_MAIN_PANES]) -> usize {
    if panes == 0 || area.is_empty() {
        return 0;
    }

    if panes == 1 {
        // Nothing to strip; the focused pane takes the whole main window.
        return even(area, 1, out);
    }

    let strips = panes - 1;
    let strip_rows = STRIP_ROWS
        .min(area.rows.saturating_sub(MIN_PANE_ROWS) / strips)
        .max(1);
    let focus_rows = area.rows.saturating_sub(strip_rows.saturating_mul(strips));

    if focus_rows == 0 {
        // Too cramped for a focused pane plus strips; share the rows evenly so
        // every pane is still addressable.
        return even(area, panes, out);
    }

    let mut placed = 0u16;
    let mut row = area.row;
    for index in 0..panes {
        let rows = if index == 0 { focus_rows } else { strip_rows };
        if rows == 0 {
            continue;
        }
        let Some(slot) = out.get_mut(usize::from(placed)) else {
            break;
        };
        *slot = Rect::new(area.col, row, area.cols, rows);
        row += rows;
        placed += 1;
    }
    usize::from(placed)
}

/// Equal full-width bands. The degenerate fallback.
fn even(area: Rect, panes: u16, out: &mut [Rect; MAX_MAIN_PANES]) -> usize {
    let mut placed = 0u16;
    for index in 0..panes {
        let (row_offset, rows) = slice(area.rows, index, panes);
        if rows == 0 {
            continue;
        }
        let Some(slot) = out.get_mut(usize::from(placed)) else {
            break;
        };
        *slot = Rect::new(area.col, area.row + row_offset, area.cols, rows);
        placed += 1;
    }
    usize::from(placed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiled(
        tiler: fn(Rect, u16, &mut [Rect; MAX_MAIN_PANES]) -> usize,
        area: Rect,
        panes: u16,
    ) -> Vec<Rect> {
        let mut out = [Rect::EMPTY; MAX_MAIN_PANES];
        let placed = tiler(area, panes, &mut out);
        out.get(..placed).unwrap_or(&[]).to_vec()
    }

    /// The invariant every tiler owes: exact cover, no overlap.
    fn assert_exact_cover(rects: &[Rect], area: Rect) {
        let covered: usize = rects.iter().map(|rect| rect.size().area()).sum();
        assert_eq!(
            covered,
            area.size().area(),
            "{rects:?} does not tile {area:?}"
        );
        for (index, a) in rects.iter().enumerate() {
            for b in &rects[index + 1..] {
                assert!(a.intersection(*b).is_empty(), "{a:?} overlaps {b:?}");
            }
        }
    }

    #[test]
    fn slices_distribute_the_remainder_to_the_earliest() {
        assert_eq!(slice(29, 0, 2), (0, 15));
        assert_eq!(slice(29, 1, 2), (15, 14));
        assert_eq!(slice(10, 0, 3), (0, 4));
        assert_eq!(slice(10, 1, 3), (4, 3));
        assert_eq!(slice(10, 2, 3), (7, 3));
    }

    #[test]
    fn slices_of_zero_count_are_empty() {
        assert_eq!(slice(10, 0, 0), (0, 0));
    }

    #[test]
    fn deep_bands_follow_the_two_column_rule() {
        assert_eq!(deep_bands(0), 0);
        assert_eq!(deep_bands(1), 1);
        assert_eq!(deep_bands(2), 1);
        assert_eq!(deep_bands(3), 2);
        assert_eq!(deep_bands(4), 2);
    }

    #[test]
    fn every_tiler_covers_its_area_exactly() {
        let areas = [
            Rect::new(0, 0, 120, 29),
            Rect::new(0, 0, 80, 21),
            Rect::new(3, 2, 41, 17),
            Rect::new(0, 0, 5, 5),
        ];
        for area in areas {
            for panes in 1..=4u16 {
                assert_exact_cover(&tiled(deep, area, panes), area);
                assert_exact_cover(&tiled(wide, area, panes), area);
                assert_exact_cover(&tiled(even, area, panes), area);
            }
        }
    }

    #[test]
    fn tilers_respect_a_non_zero_origin() {
        let area = Rect::new(7, 5, 40, 20);
        for panes in 1..=4u16 {
            for rect in tiled(deep, area, panes) {
                assert!(rect.col >= 7 && rect.row >= 5, "{rect:?} escaped {area:?}");
                assert!(rect.right() <= area.right() && rect.bottom() <= area.bottom());
            }
        }
    }

    #[test]
    fn a_pane_is_never_placed_with_no_area() {
        // `main()` is zipped against open domains. A counted-but-invisible pane
        // binds a domain to a rectangle that cannot draw a cell, and the
        // exact-cover tests cannot see it because zero area sums to zero.
        for area in [
            Rect::new(0, 0, 80, 3),
            Rect::new(0, 0, 80, 1),
            Rect::new(0, 0, 3, 4),
        ] {
            for panes in 1..=4u16 {
                for (name, tiler) in [
                    (
                        "deep",
                        deep as fn(Rect, u16, &mut [Rect; MAX_MAIN_PANES]) -> usize,
                    ),
                    ("wide", wide),
                    ("even", even),
                ] {
                    for rect in tiled(tiler, area, panes) {
                        assert!(
                            !rect.is_empty(),
                            "{name} placed an empty pane in {area:?} with {panes} panes"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn an_empty_area_places_nothing() {
        assert!(tiled(deep, Rect::EMPTY, 4).is_empty());
        assert!(tiled(wide, Rect::EMPTY, 4).is_empty());
    }

    #[test]
    fn wide_falls_back_to_even_bands_when_strips_would_starve_the_focus() {
        // Four panes into four rows: there is no room for a focused pane plus
        // three strips, so every pane gets one row rather than one getting none.
        let area = Rect::new(0, 0, 80, 4);
        let rects = tiled(wide, area, 4);
        assert_eq!(rects.len(), 4);
        assert!(rects.iter().all(|rect| rect.rows == 1));
        assert_exact_cover(&rects, area);
    }
}
