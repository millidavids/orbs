//! What the room in front of the player has made, in all, at the strip's right
//! edge (DESIGN.md §19, *number go up*).
//!
//! The room's first ledger row and a spell's share of it, climbing as work
//! lands. Silent glyphs, the rail's precedent: `status` is the spoken whole.

use orbs_render::{Painter, Pos, Rect, Style};
use orbs_sim::Prose;
use orbs_sim::tower::LedgerRow;

use super::row::cells;
use crate::climb::{Climb, Rolled, Watch};

/// Cells kept after a count for its `+N`: a space and four characters.
///
/// Kept whether or not one is showing, so a gain arriving moves nothing.
const PLUS_SLOT: u16 = 5;

/// Cells between a label and its count.
const GAP: u16 = 2;

/// One line of the column.
struct Line {
    label: String,
    rolled: Rolled,
    /// Digits the count is right-aligned in: the wider of where it started and
    /// where it ends.
    digits: usize,
}

/// The column, laid out.
pub(super) struct Column {
    lines: Vec<Line>,
    /// Cells the labels take, the share's counted whether or not it is drawn.
    labels: usize,
}

/// Lay out `row` — the count, and a spell's share when a spell has one.
pub(super) fn lay(row: &LedgerRow, climb: &Climb, prose: &Prose) -> Column {
    let line = |label: String, rolled: Rolled, truth: u32| Line {
        label,
        rolled,
        digits: rolled.widest(u64::from(truth)).to_string().len(),
    };
    let made = prose.line(&format!("ledger_{}", row.id), &[]);
    let share = prose.line("ledger_share", &[]);
    // Measured with the share's label even before a spell has one, so a
    // spell's first hand in the room widens nothing and moves no bar.
    let labels = cells(&made).max(cells(&share));
    let mut lines = vec![line(
        made,
        climb.count(Watch::Room, &row.id, u64::from(row.count)),
        row.count,
    )];
    if row.by_spell > 0 {
        let spelled = climb.count(Watch::Spells, &row.id, u64::from(row.by_spell));
        lines.push(line(share, spelled, row.by_spell));
    }
    Column { lines, labels }
}

impl Column {
    const fn label_cells(&self) -> usize {
        self.labels
    }

    fn digit_cells(&self) -> usize {
        self.lines.iter().map(|line| line.digits).max().unwrap_or(0)
    }

    /// Cells it takes, its `+N` slot included.
    pub(super) fn width(&self) -> u16 {
        let text = self.label_cells() + usize::from(GAP) + self.digit_cells();
        u16::try_from(text)
            .unwrap_or(u16::MAX)
            .saturating_add(PLUS_SLOT)
    }

    /// Draw it into `at`, one line a row.
    pub(super) fn draw(&self, painter: &mut Painter<'_>, at: Rect) {
        let (labels, digits) = (self.label_cells(), self.digit_cells());
        let label_end = at
            .col
            .saturating_add(u16::try_from(labels).unwrap_or(u16::MAX));
        for (line, row) in self.lines.iter().zip(at.row..at.bottom()) {
            let count = format!(
                "{:gap$}{:>digits$}",
                "",
                line.rolled.shown,
                gap = usize::from(GAP)
            );
            painter.glyphs(Pos::new(at.col, row), &line.label, Style::DIM);
            let written = painter.glyphs(Pos::new(label_end, row), &count, Style::default());
            if let Some((gain, weight)) = line.rolled.plus {
                let plus = format!("+{gain}");
                if cells(&plus) < usize::from(PLUS_SLOT) {
                    painter.glyphs(
                        Pos::new(label_end.saturating_add(written).saturating_add(1), row),
                        &plus,
                        Style::default().with_intensity(weight),
                    );
                }
            }
        }
    }
}
