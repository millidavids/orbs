//! Rectangles partway between two layouts.
//!
//! A pane that appears between one frame and the next reads as a glitch. Where
//! it appears *from* is geometry, so it is decided here rather than by whichever
//! frontend happens to be animating — the same division
//! [`ScreenLayout`](crate::ScreenLayout) already draws. A frontend supplies only
//! the progress; nothing here knows what a second is.
//!
//! # Transitional layouts are not tiled layouts
//!
//! [`tiling`](crate::tiling) promises its layouts are **total**: every tiler
//! covers its area exactly, with no gaps, no overlap, and no zero-area pane.
//! Everything here breaks all three on purpose — a pane arriving *is* a
//! zero-area rectangle for one frame, and a pane leaving *is* a gap closing.
//!
//! That exception is deliberate and bounded. It applies to the output of
//! [`ScreenLayout::transition`](crate::ScreenLayout::transition) and to nothing
//! else; `ScreenLayout::compute` keeps every guarantee it had, and the tiling
//! tests still hold it to them. What survives here is the weaker invariant that
//! actually matters: **a transitional pane never leaves the span of its own two
//! endpoints**, so it cannot escape a grid that both endpoints fitted.

use crate::geometry::Rect;
use crate::layout::DisplayMode;

/// Interpolate `from` into `to`, writing the result into `out`.
///
/// Returns how many panes were written. A pane present at only one end is born
/// from — or dies into — the [`edge`] of its own settled rectangle.
pub(crate) fn panes(
    from: &[Rect],
    to: &[Rect],
    mode: DisplayMode,
    t: f32,
    out: &mut [Rect],
) -> usize {
    let len = from.len().max(to.len()).min(out.len());
    for (index, slot) in out[..len].iter_mut().enumerate() {
        let (a, b) = match (from.get(index), to.get(index)) {
            (Some(a), Some(b)) => (*a, *b),
            (None, Some(b)) => (edge(*b, mode), *b),
            (Some(a), None) => (*a, edge(*a, mode)),
            (None, None) => continue,
        };
        *slot = lerp(a, b, t);
    }
    len
}

/// The zero-area rectangle a pane grows out of, on the side it arrives from.
///
/// **Not [`Rect::EMPTY`].** Interpolating from `EMPTY` is the obvious thing and
/// is wrong: it is `(0, 0, 0, 0)`, so a pane grows diagonally out of the
/// top-left corner at half height, straight through the pane it is supposed to
/// be appearing beside.
///
/// Per mode, because §9 puts extra panes in different places. Deep tiles them
/// side by side, so a pane slides in from the right edge of its own slot; Wide
/// stacks [`STRIP_ROWS`](crate::STRIP_ROWS) strips below the main pane, so a
/// strip unrolls downward from its own top edge instead. One rule for both would
/// send a strip sideways across the screen.
fn edge(rect: Rect, mode: DisplayMode) -> Rect {
    match mode {
        DisplayMode::Deep => Rect::new(rect.col.saturating_add(rect.cols), rect.row, 0, rect.rows),
        DisplayMode::Wide => Rect::new(rect.col, rect.row, rect.cols, 0),
    }
}

/// One rectangle partway to another.
///
/// **The far edges are interpolated, not the extents.** Rounding an origin and
/// its extent independently lets both round up at the same instant: a pane at
/// `col 147.5, cols 12.5` becomes `col 148, cols 13`, whose right edge is a cell
/// further right than *either* endpoint — off the grid, and the pane painting
/// over its neighbour on the way. Interpolating the edges and subtracting keeps
/// every transitional pane inside the span of its own endpoints, which is the
/// invariant this module keeps in place of tiling's.
fn lerp(a: Rect, b: Rect, t: f32) -> Rect {
    let col = mix(a.col, b.col, t);
    let row = mix(a.row, b.row, t);
    let right = mix(
        a.col.saturating_add(a.cols),
        b.col.saturating_add(b.cols),
        t,
    );
    let bottom = mix(
        a.row.saturating_add(a.rows),
        b.row.saturating_add(b.rows),
        t,
    );
    Rect::new(
        col,
        row,
        right.saturating_sub(col),
        bottom.saturating_sub(row),
    )
}

/// One coordinate, interpolated and rounded back to whole cells.
///
/// `f32` carries 24 bits of mantissa and a grid is bounded by `u16`, so every
/// value here converts exactly on the way in; rounding is the only lossy step,
/// and it is the one [`lerp`] is careful about.
// `f32 -> u16` has no `TryFrom`, so the cast is the only route. It is rounded
// and then clamped into `u16`'s exact range first, which is what makes both
// truncation and sign loss unreachable rather than merely unlikely — and `clamp`
// propagates NaN to neither bound, so the `is_nan` guard is what covers it.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "rounded and clamped into u16's range on the line above the cast"
)]
fn mix(a: u16, b: u16, t: f32) -> u16 {
    let value = f32::from(a) + (f32::from(b) - f32::from(a)) * t;
    if value.is_nan() {
        return a;
    }
    value.round().clamp(0.0, f32::from(u16::MAX)) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    const LEFT: Rect = Rect::new(0, 0, 30, 7);
    const RIGHT: Rect = Rect::new(30, 0, 30, 7);
    const WHOLE: Rect = Rect::new(0, 0, 60, 7);

    fn tween_at(from: &[Rect], to: &[Rect], t: f32) -> Vec<Rect> {
        let mut out = [Rect::EMPTY; 4];
        let len = panes(from, to, DisplayMode::Deep, t, &mut out);
        out[..len].to_vec()
    }

    #[test]
    fn the_endpoints_are_the_layouts_themselves() {
        // Whatever happens in between, a transition that has not started and one
        // that has finished must be indistinguishable from no transition at all.
        assert_eq!(
            tween_at(&[WHOLE], &[LEFT, RIGHT], 0.0),
            [WHOLE, edge(RIGHT, DisplayMode::Deep)]
        );
        assert_eq!(tween_at(&[WHOLE], &[LEFT, RIGHT], 1.0), [LEFT, RIGHT]);
    }

    #[test]
    fn an_arriving_pane_starts_at_the_edge_it_arrives_from() {
        let born = edge(RIGHT, DisplayMode::Deep);
        assert_eq!(born.col, RIGHT.col + RIGHT.cols, "not at the right edge");
        assert_eq!(born.cols, 0, "it should have no width yet");
        assert_eq!(born.rows, RIGHT.rows, "it is full height throughout");
        assert_eq!(born.row, RIGHT.row);
    }

    #[test]
    fn a_wide_strip_unrolls_downward_instead_of_sideways() {
        // §9 stacks Wide's extra panes as strips below the main one. Sliding one
        // in from the right would send it across the screen it is already on.
        let strip = Rect::new(0, 20, 60, crate::STRIP_ROWS);
        let born = edge(strip, DisplayMode::Wide);
        assert_eq!(born.rows, 0, "it should have no height yet");
        assert_eq!(born.cols, strip.cols, "it is full width throughout");
        assert_eq!(born.row, strip.row);
    }

    #[test]
    fn adjacent_panes_meet_exactly_at_every_step() {
        // The rounding trap. An origin and an extent rounded independently leave
        // a one-cell gap or a one-cell overlap at the join, which reads as a
        // bright seam of background or a doubled border — a glitch, not motion.
        for step in 0..=64u16 {
            let t = f32::from(step) / 64.0;
            let panes = tween_at(&[WHOLE], &[LEFT, RIGHT], t);
            let (left, right) = (panes[0], panes[1]);
            assert_eq!(
                left.col + left.cols,
                right.col,
                "t={t}: {left:?} and {right:?} do not meet",
            );
            assert_eq!(
                right.col + right.cols,
                WHOLE.cols,
                "t={t}: a gap at the edge"
            );
        }
    }

    #[test]
    fn no_transitional_pane_escapes_the_span_of_its_endpoints() {
        // The invariant that replaces tiling's. Gaps and overlaps are permitted
        // here; leaving the screen is not, because that is cells written past
        // the frame.
        for step in 0..=64u16 {
            let t = f32::from(step) / 64.0;
            for rect in tween_at(&[WHOLE], &[LEFT, RIGHT], t) {
                assert!(
                    u32::from(rect.col) + u32::from(rect.cols) <= u32::from(WHOLE.cols),
                    "t={t}: {rect:?} escaped {WHOLE:?}",
                );
            }
        }
    }

    #[test]
    fn a_departing_pane_is_an_arrival_run_backwards() {
        let arriving = tween_at(&[WHOLE], &[LEFT, RIGHT], 0.25);
        let departing = tween_at(&[LEFT, RIGHT], &[WHOLE], 0.75);
        assert_eq!(arriving, departing);
    }
}
