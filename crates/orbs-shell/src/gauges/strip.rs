//! The split, and the strip: two rows and, when there is room, the room's
//! column at its right edge.

use orbs_render::{Painter, Rect};
use orbs_sim::tower::LedgerRow;
use orbs_sim::{Prose, Toward};

use super::row::{Laid, lay};
use crate::climb::{Climb, Watch};

/// The rows the gauges took, and what is left for the rest of the pane.
#[derive(Debug, Clone, Copy)]
pub struct Split {
    /// Up to two rows, or empty when the gauges do not draw.
    pub area: Rect,
    /// The body below them.
    pub rest: Rect,
}

/// Rows the gauges want.
pub(super) const ROWS: u16 = 2;

/// Rows the body must keep after they take theirs.
///
/// Higher than the road's, because these cost twice as much: a transcript
/// squeezed to nothing so two standing bars could draw is the minimised half
/// beating the main window, which §9 forbids.
///
/// The guard subtracts [`ROWS`] where `road::split`'s does not — the road takes
/// one row, so a body of `KEEP + 1` leaves exactly `KEEP`, while these take two
/// and the same test let a body of nine keep seven.
pub(super) const KEEP: u16 = 8;

/// The narrowest pane worth drawing a bracketed gauge in.
///
/// The label, a gauge with room to fill, and a reading wide enough for two
/// five-figure numbers and a slash. Below this the row would be brackets and a
/// truncated number, which says less than nothing.
pub(super) const LEAST: u16 = 34;

/// The shortest bar a gauge keeps beside the room's column.
///
/// The column is a count and the bar is the standing, so the bar outranks it:
/// below this the column drops whole rather than the bar shrinking to a stub.
pub(super) const BESIDE: u16 = 24;

/// Cells between the gauges and the column.
const GAP: u16 = 2;

/// Take the top rows for the gauges, if there is room.
#[must_use]
pub const fn split(body: Rect) -> Split {
    if body.rows.saturating_sub(ROWS) < KEEP || body.cols < LEAST {
        return Split {
            area: Rect::EMPTY,
            rest: body,
        };
    }
    Split {
        area: Rect::new(body.col, body.row, body.cols, ROWS),
        rest: Rect::new(
            body.col,
            body.row.saturating_add(ROWS),
            body.cols,
            body.rows.saturating_sub(ROWS),
        ),
    }
}

/// Draw both gauges into their rows, and the room's column beside them.
///
/// `rank` is what the tower is called, drawn after renown's reading when there
/// is room — a title is the point of the second bar and the first thing to drop
/// when there is not. `Laid::draw` is where that dropping happens, and it is
/// measured against whether a bar survives rather than guessed at from the
/// pane's width.
///
/// The column reserves its width first, so a gain's `+N` yields to it and never
/// the reverse. It draws only if every row keeps a bar of [`BESIDE`] and its
/// title; otherwise it drops whole and the gauges are exactly as they were.
pub fn paint(
    painter: &mut Painter<'_>,
    area: Rect,
    station: Toward,
    rankward: Toward,
    rank: Option<&str>,
    room: Option<&LedgerRow>,
    climb: &Climb,
    prose: &Prose,
) {
    if area.is_empty() {
        return;
    }
    let ley = lay(
        &prose.line("gauge_ley", &[]),
        station,
        climb.toward(Watch::Ley, station),
        None,
        prose,
    );
    let renown = (area.rows >= 2)
        .then(|| {
            lay(
                &prose.line("gauge_renown", &[]),
                rankward,
                climb.toward(Watch::Renown, rankward),
                rank,
                prose,
            )
        })
        .flatten();

    let column = room
        .map(|row| super::column::lay(row, climb, prose))
        .filter(|column| {
            let left = area.cols.saturating_sub(column.width().saturating_add(GAP));
            left >= LEAST
                && [&ley, &renown]
                    .into_iter()
                    .flatten()
                    .all(|laid: &Laid| laid.needs(BESIDE) <= left)
        });
    let taken = column
        .as_ref()
        .map_or(0, |column| column.width().saturating_add(GAP));
    let gauges = area.cols.saturating_sub(taken);

    if let Some(laid) = &ley {
        laid.draw(painter, Rect::new(area.col, area.row, gauges, 1));
    }
    if let Some(laid) = &renown {
        laid.draw(
            painter,
            Rect::new(area.col, area.row.saturating_add(1), gauges, 1),
        );
    }
    if let Some(column) = column {
        let width = column.width();
        column.draw(
            painter,
            Rect::new(
                area.right().saturating_sub(width),
                area.row,
                width,
                area.rows,
            ),
        );
    }
}
