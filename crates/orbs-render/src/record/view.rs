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
use crate::record::kind::RecordKind;
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
    Prompt {
        prompt: &'a str,
    },
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
            Mode::Lines | Mode::Prompt { .. } => self,
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

    /// The command-line surface: a marker, then the line.
    ///
    /// [`RecordView::lines`] is not sufficient here. Every record the parser
    /// emits carries one canonical command form, so through a line view an
    /// unresolved input draws as a column of bare words and `meditate` draws as
    /// `meditate count` — neither of which is the prompt DESIGN.md §6 describes.
    ///
    /// This view draws two extra channels, both derived from the record and
    /// neither of them prose:
    ///
    /// - the [`Record::marker`] glyph, so a suggestion cannot be mistaken for
    ///   the command that will run;
    /// - the player's own [`RecordKind::Input`] lines with the shell prompt in
    ///   front of them, so the transcript reads as a session.
    ///
    /// Intensity comes from [`Record::style`], which derives it the same way.
    ///
    /// `prompt` is what the player's own lines are drawn behind — the wizard's
    /// name, which is world state rather than a rendering choice, so it arrives
    /// as an argument instead of living here as a constant.
    #[must_use]
    pub const fn prompt(prompt: &str) -> RecordView<'_> {
        RecordView {
            mode: Mode::Prompt { prompt },
        }
    }

    /// Rows `records` would need at `cols` wide, without drawing anything.
    ///
    /// A line view is no longer one row per record — a listing packs across the
    /// pane — so a caller that wants the *newest* records has to ask rather than
    /// count. Before this existed the transcript assumed one row each, and every
    /// packed listing left that many blank rows at the bottom of the pane while
    /// dropping the same number of records off the top.
    ///
    /// Unbounded in the record count, so a caller holding a long stream should
    /// pass a window rather than the whole of it.
    #[must_use]
    pub fn height<'r>(&self, cols: u16, records: impl Iterator<Item = Record<'r>> + Clone) -> u16 {
        let prompt = match self.mode {
            // A table is one row per record plus the header, always.
            Mode::Table { header, .. } => {
                let rows = to_cols(records.count());
                return if header { rows.saturating_add(1) } else { rows };
            }
            Mode::Lines => None,
            Mode::Prompt { prompt } => Some(prompt),
        };

        let indent = indent_for(prompt);
        let (mut rows, mut rest) = (0u16, records);
        let mut at_run_start = true;
        loop {
            let run = rest.clone();
            let Some(record) = rest.next() else { break };
            // Only ever planned at a run's first record. A run that declines
            // declines for its whole length, and re-asking at each of its records
            // would walk the remainder every time — quadratic in the run, on a
            // path that runs per frame.
            if at_run_start
                && record.kind().tiles()
                && let Some(plan) = Tiling::plan(cols, indent, run)
            {
                for _ in 1..plan.count {
                    rest.next();
                }
                rows = rows.saturating_add(plan.rows());
                continue;
            }
            at_run_start = !record.kind().tiles();
            rows = rows.saturating_add(1);
        }
        rows
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
            Mode::Lines => draw_lines(&mut painter, area, records, None),
            Mode::Prompt { prompt } => draw_lines(&mut painter, area, records, Some(prompt)),
        }
    }
}

/// Cells reserved in front of a line for its marker.
const MARKER_WIDTH: u16 = 2;

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
    records: impl Iterator<Item = Record<'r>> + Clone,
    prompt: Option<&str>,
) -> u16 {
    let (mut drawn, mut speech) = (String::new(), String::new());
    let mut row = area.row;
    let mut rest = records;
    // See `RecordView::height`: a declined run must not be re-planned at each of
    // its records, or the measuring is quadratic in the run's length.
    let mut at_run_start = true;
    loop {
        if row >= area.bottom() {
            break;
        }
        // Cloned before the record is taken, so a tiling run can be measured
        // from its own first row rather than needing a second pass over the
        // stream or a `Vec` in the render path.
        let run = rest.clone();
        let Some(record) = rest.next() else { break };

        // The sub-area is what is *left* of the pane, not the pane moved down.
        // `Rect { row, ..area }` keeps the full row count, so `bottom()` slides
        // with the run and a listing starting partway down draws past the pane —
        // where `Painter` clips the cells but not the speech, and a reader hears
        // rows nobody can see.
        let remaining = Rect {
            row,
            rows: area.bottom().saturating_sub(row),
            ..area
        };
        if at_run_start
            && record.kind().tiles()
            && let Some((rows, packed)) = draw_tiled(painter, remaining, prompt, run)
        {
            // `record` was the first of them.
            for _ in 1..packed {
                rest.next();
            }
            row = row.saturating_add(rows);
            continue;
        }
        at_run_start = !record.kind().tiles();

        drawn.clear();
        // Content only: an annotation drawn as text would put an internal token
        // — `"resolved survey"` — on screen for a player to read.
        record.write_line(&mut drawn);
        speech.clear();
        record.speak(&mut speech);

        let style = record.style();
        let mut col = area.col;
        if let Some(prompt) = prompt {
            // The marker and the prompt are drawn silently. Both restate what
            // the linear stream already carries — an utterance's kind says it
            // is `Input`, and a reader filtering by outcome reads the
            // annotation — so speaking them would be saying it twice.
            if record.kind() == RecordKind::Input {
                col = col.saturating_add(painter.glyphs(Pos::new(col, row), prompt, style));
            } else {
                if let Some(marker) = record.marker() {
                    painter.glyphs(Pos::new(col, row), marker.encode_utf8(&mut [0; 4]), style);
                }
                col = col.saturating_add(MARKER_WIDTH);
            }
        }

        let mut span = crate::span::Span::new(&drawn)
            .with_style(style)
            .with_kind(record.kind().utterance())
            .with_spoken(&speech);
        // The marker and the intensity are both silent channels. Tagging the
        // utterance is what stops `xyzzy` from linearising as three identical
        // lines, with a listener unable to tell the error from the offers.
        if let Some(outcome) = record.outcome() {
            span = span.with_outcome(outcome);
        }
        painter.span(Pos::new(col, row), &span);
        row = row.saturating_add(1);
    }
    row.saturating_sub(area.row)
}

/// Cells a marked line's text is inset by, which a listing matches so a pane
/// does not appear to change its left edge partway down.
const fn indent_for(prompt: Option<&str>) -> u16 {
    if prompt.is_some() { MARKER_WIDTH } else { 0 }
}

/// How a run of [tiling](RecordKind::tiles) records packs across a pane.
///
/// Split out from the drawing so [`RecordView::height`] and [`draw_tiled`]
/// cannot disagree about it. They did not have to: a pane that measures one way
/// and draws another leaves blank rows at the bottom while dropping history off
/// the top, which is what the transcript did the first time this was looked at.
#[derive(Debug, Clone, Copy)]
struct Tiling {
    indent: u16,
    /// Cells from the start of one column to the start of the next.
    stride: u16,
    per_row: u16,
    count: usize,
}

impl Tiling {
    /// Plan the run beginning at `run`'s first record, or `None` to stack.
    ///
    /// The run ends at the first record that does not tile; `run` may continue
    /// past it.
    ///
    /// # Why it can decline
    ///
    /// A tiled row has no room for a per-record [marker](Record::marker), and a
    /// marker is information — the difference between a candidate and the
    /// command that will run. Rather than drop it, a run carrying any marker
    /// declines to tile and falls back to one record per line, where the marker
    /// column exists. Nothing that reaches here today carries one; this is what
    /// keeps that true by construction instead of by convention.
    fn plan<'r>(
        cols: u16,
        indent: u16,
        run: impl Iterator<Item = Record<'r>> + Clone,
    ) -> Option<Self> {
        let available = cols.saturating_sub(indent);
        if available == 0 {
            return None;
        }

        let (mut widest, mut count) = (0usize, 0usize);
        let mut drawn = String::new();
        for record in run.take_while(|record| record.kind().tiles()) {
            if record.marker().is_some() {
                return None;
            }
            drawn.clear();
            record.write_line(&mut drawn);
            widest = widest.max(drawn.chars().count());
            count += 1;
        }
        // One name is not a listing, and packing it would only move it right.
        if count < 2 || widest == 0 {
            return None;
        }

        let stride = to_cols(widest.saturating_add(COLUMN_GAP));
        let per_row = available / stride;
        // Nothing gained, and stacking keeps the fallback in one place.
        (per_row >= 2).then_some(Self {
            indent,
            stride,
            per_row,
            count,
        })
    }

    /// Rows the whole run needs.
    fn rows(self) -> u16 {
        to_cols(self.count.div_ceil(usize::from(self.per_row)))
    }
}
fn draw_tiled<'r>(
    painter: &mut Painter<'_>,
    area: Rect,
    prompt: Option<&str>,
    run: impl Iterator<Item = Record<'r>> + Clone,
) -> Option<(u16, usize)> {
    let plan = Tiling::plan(area.cols, indent_for(prompt), run.clone())?;
    let (indent, stride, per_row) = (plan.indent, plan.stride, plan.per_row);
    let tiling = run.take_while(|record| record.kind().tiles());

    let (mut drawn, mut speech, mut placed) = (String::new(), String::new(), 0usize);
    for record in tiling {
        let row = area
            .row
            .saturating_add(to_cols(placed / usize::from(per_row)));
        if row >= area.bottom() {
            break;
        }
        let col = area
            .col
            .saturating_add(indent)
            .saturating_add(to_cols(placed % usize::from(per_row)) * stride);

        drawn.clear();
        record.write_line(&mut drawn);
        speech.clear();
        record.speak(&mut speech);
        // Spoken in stream order, one utterance per record, exactly as the
        // stacked path does — so §14's linear form is unchanged by the wrap.
        painter.span(
            Pos::new(col, row),
            &crate::span::Span::new(&drawn)
                .with_style(record.style())
                .with_kind(record.kind().utterance())
                .with_spoken(&speech),
        );
        placed += 1;
    }

    let rows = placed.div_ceil(usize::from(per_row));
    Some((to_cols(rows), placed))
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
