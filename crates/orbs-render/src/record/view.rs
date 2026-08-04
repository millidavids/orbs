//! Records become cells here — and nowhere else.
//!
//! This module is the whole claim of architectural rule 4 made concrete:
//! *presentation is a view over the record*. Nothing here adds information. A
//! table decides widths, alignment, and which columns to show; it cannot invent
//! a value, and it cannot suppress one from the screen-reader stream, because
//! the linear form comes from [`Record::speak`] rather than from anything drawn.
//!
//! Three consequences fall out rather than being implemented:
//!
//! - **Linearised tables are correct by construction** (§14). Every row speaks
//!   `label: value` because the labels live on the record, so a view physically
//!   cannot forget them or reorder them away from their values.
//! - **A truncated column costs a sighted player and nobody else.** The row's
//!   speech is built before any width is applied, so a narrow pane is a visual
//!   constraint and never an informational one — the same rule
//!   [`Painter::span`] follows.
//! - **A missing field draws as a gap.** §8.1 names malformed record boundaries
//!   as a structural sabotage signature; a record short a field leaves a hole in
//!   its row without any special case being written for it.

use crate::geometry::{Pos, Rect};
use crate::paint::Painter;
use crate::record::field::FieldName;
use crate::record::stream::Record;
use crate::style::Style;

/// Cells between adjacent columns.
const COLUMN_GAP: usize = 2;

/// How a run of records becomes cells.
#[derive(Debug, Clone, Copy)]
pub struct RecordView<'a> {
    mode: Mode<'a>,
}

#[derive(Debug, Clone, Copy)]
enum Mode<'a> {
    Table {
        columns: &'a [FieldName],
        header: bool,
    },
    Lines,
}

impl<'a> RecordView<'a> {
    /// A column-aligned table over the named fields, with a header row.
    ///
    /// Fields a record does not carry are left blank; fields not named here are
    /// still spoken, because the row's speech is the whole record.
    #[must_use]
    pub const fn table(columns: &'a [FieldName]) -> Self {
        Self {
            mode: Mode::Table {
                columns,
                header: true,
            },
        }
    }

    /// The same table with no header row.
    #[must_use]
    pub const fn without_header(self) -> Self {
        match self.mode {
            Mode::Table { columns, .. } => Self {
                mode: Mode::Table {
                    columns,
                    header: false,
                },
            },
            Mode::Lines => self,
        }
    }

    /// One line per record, every field in emit order, space separated.
    ///
    /// What a log pane and the orb's own speech want: prose and log lines carry
    /// their structure in their wording, not in their columns.
    #[must_use]
    pub const fn lines() -> RecordView<'static> {
        RecordView { mode: Mode::Lines }
    }

    /// Draw `records` into `area`, returning the number of rows used.
    ///
    /// The iterator must be `Clone` because a table measures its columns in one
    /// pass and draws them in a second; that is what keeps a per-frame `Vec` of
    /// rows out of the render path.
    pub fn draw<'r>(
        &self,
        painter: &mut Painter<'_>,
        area: Rect,
        records: impl Iterator<Item = Record<'r>> + Clone,
    ) -> u16 {
        let mut painter = painter.sub(area);
        let area = painter.area();
        if area.is_empty() {
            return 0;
        }
        match self.mode {
            Mode::Table { columns, header } => {
                draw_table(&mut painter, area, columns, header, records)
            }
            Mode::Lines => draw_lines(&mut painter, area, records),
        }
    }
}

fn draw_table<'r>(
    painter: &mut Painter<'_>,
    area: Rect,
    columns: &[FieldName],
    header: bool,
    records: impl Iterator<Item = Record<'r>> + Clone,
) -> u16 {
    if columns.is_empty() {
        return 0;
    }

    let mut widths: Vec<usize> = columns
        .iter()
        .map(|column| {
            if header {
                column.label().chars().count()
            } else {
                0
            }
        })
        .collect();
    for record in records.clone() {
        for (width, column) in widths.iter_mut().zip(columns) {
            if let Some(value) = record.field(*column) {
                *width = (*width).max(value.width());
            }
        }
    }

    let mut row = area.row;
    if header {
        // Drawn silently. Every row below speaks its own `label: value` pairs
        // (§14), so the header carries no information a reader is missing — and
        // announcing it would make every table open with a redundant recital of
        // words the listener is about to hear on each line anyway.
        for (index, column) in columns.iter().enumerate() {
            if let Some(at) = column_start(area, &widths, index) {
                painter.glyphs(at, column.label(), Style::DIM);
            }
        }
        row = row.saturating_add(1);
    }

    let mut speech = String::new();
    for record in records {
        if row >= area.bottom() {
            break;
        }
        speech.clear();
        record.speak(&mut speech);
        painter.announce(record.kind().utterance(), record.role(), &speech);

        let style = record.style();
        for (index, column) in columns.iter().enumerate() {
            let Some(value) = record.field(*column) else {
                continue;
            };
            let Some(at) = column_start(Rect { row, ..area }, &widths, index) else {
                continue;
            };
            // Numbers align right within their column; text aligns left. The
            // view can make that choice only because the record kept the number
            // a number instead of emitting it pre-rendered.
            let at = if value.is_numeric() {
                let pad = widths[index].saturating_sub(value.width());
                Pos::new(at.col.saturating_add(to_cols(pad)), at.row)
            } else {
                at
            };
            value.with_str(|text| painter.glyphs(at, text, style));
        }
        row = row.saturating_add(1);
    }
    row.saturating_sub(area.row)
}

fn draw_lines<'r>(
    painter: &mut Painter<'_>,
    area: Rect,
    records: impl Iterator<Item = Record<'r>>,
) -> u16 {
    let (mut drawn, mut speech) = (String::new(), String::new());
    let mut row = area.row;
    for record in records {
        if row >= area.bottom() {
            break;
        }
        drawn.clear();
        // Content only. A line view names no columns, so an annotation would
        // otherwise be drawn as text — `"resolved survey"` — putting an internal
        // token on screen for a player to read.
        for (index, (_, value)) in record.content().enumerate() {
            if index > 0 {
                drawn.push(' ');
            }
            value.write(&mut drawn);
        }
        speech.clear();
        record.speak(&mut speech);

        painter.span(
            Pos::new(area.col, row),
            &crate::span::Span::new(&drawn)
                .with_style(record.style())
                .with_kind(record.kind().utterance())
                .with_spoken(&speech),
        );
        row = row.saturating_add(1);
    }
    row.saturating_sub(area.row)
}

/// Where column `index` begins, or `None` if it starts past the right edge.
fn column_start(area: Rect, widths: &[usize], index: usize) -> Option<Pos> {
    let offset: usize = widths[..index].iter().sum::<usize>() + COLUMN_GAP * index;
    (offset < usize::from(area.cols))
        .then(|| Pos::new(area.col.saturating_add(to_cols(offset)), area.row))
}

/// A measured width as a column count, saturating rather than wrapping.
///
/// Anything that does not fit in a `u16` is off the far edge of any grid this
/// game will ever host, so clamping is the correct answer and not a fudge.
fn to_cols(width: usize) -> u16 {
    u16::try_from(width).unwrap_or(u16::MAX)
}
