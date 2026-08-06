//! Writing into a frame.
//!
//! Every write goes through a [`Painter`] bounded to a rectangle, so painting
//! outside a pane is impossible rather than merely discouraged — a stale layout
//! draws less, never into a neighbour.
//!
//! # The split that matters
//!
//! The API divides into **content**, which speaks, and **structure**, which does
//! not:
//!
//! | Method | Cells | Speech |
//! |---|---|---|
//! | [`Painter::span`], [`Painter::paragraph`], [`Painter::progress`] | yes | yes |
//! | [`Painter::fill`], [`Painter::clear`], [`Painter::border`] | yes | no |
//! | [`Painter::announce`] | no | yes |
//!
//! Borders, rules, and padding carry no information, so speaking them would
//! drown the stream in `"┌──────┐"`. Everything else must speak, because §14
//! makes the linear stream a first-class view of the frame rather than a
//! debugging aid.
//!
//! [`Painter::border`] announces its title for exactly this reason: the title is
//! drawn *into* the structural border, so without the announcement a pane would
//! lose its identity in the linear stream.

use crate::cell::Cell;
use crate::cp437::box_drawing;
use crate::frame::Frame;
use crate::geometry::{Pos, Rect};
use crate::linear::UtteranceKind;
use crate::span::Span;
use crate::style::{Presentation, Role, Style};
use crate::wrap::Wrap;

/// A clipped writer into a region of a [`Frame`].
///
/// All positions are absolute grid coordinates, matching the rectangles
/// [`crate::ScreenLayout`] hands out.
#[derive(Debug)]
pub struct Painter<'a> {
    frame: &'a mut Frame,
    area: Rect,
}

impl<'a> Painter<'a> {
    pub(crate) fn new(frame: &'a mut Frame, area: Rect) -> Self {
        let area = area.intersection(frame.area());
        Self { frame, area }
    }

    /// The region this painter may write to.
    #[must_use]
    pub const fn area(&self) -> Rect {
        self.area
    }

    /// A painter for a sub-region, clipped to this one.
    pub fn sub(&mut self, area: Rect) -> Painter<'_> {
        Painter {
            area: area.intersection(self.area),
            frame: &mut *self.frame,
        }
    }

    /// Draw a span on one row, truncating at the region's right edge.
    ///
    /// Returns the number of cells written.
    ///
    /// The **full** text is recorded for the linear stream even when the visual
    /// form is truncated. A narrow pane is a visual constraint; withholding the
    /// rest of the sentence from a screen reader would make it an informational
    /// one.
    ///
    /// # Panics
    ///
    /// In debug builds, if the span is [`Presentation::Eldritch`] without an
    /// authored spoken variant. DESIGN.md §3 requires one, and the alternative
    /// to catching it here is shipping a tonal register that screen-reader
    /// players cannot hear.
    pub fn span(&mut self, at: Pos, span: &Span<'_>) -> u16 {
        debug_assert!(
            span.style().presentation != Presentation::Eldritch || span.has_spoken_variant(),
            "eldritch span with no authored spoken variant: {:?}",
            span.text()
        );
        self.frame.speech_mut().push_with(
            span.kind(),
            span.style().role,
            span.outcome(),
            span.spoken_text(),
        );
        self.put_str(at, span.text(), span.style(), u16::MAX)
    }

    /// Draw a span as word-wrapped prose filling `area`.
    ///
    /// Honours embedded newlines, including blank lines. Returns the number of
    /// rows used.
    ///
    /// # Panics
    ///
    /// As [`Painter::span`].
    pub fn paragraph(&mut self, area: Rect, span: &Span<'_>) -> u16 {
        debug_assert!(
            span.style().presentation != Presentation::Eldritch || span.has_spoken_variant(),
            "eldritch paragraph with no authored spoken variant: {:?}",
            span.text()
        );
        self.frame.speech_mut().push_with(
            span.kind(),
            span.style().role,
            span.outcome(),
            span.spoken_text(),
        );

        let area = area.intersection(self.area);
        if area.is_empty() {
            return 0;
        }

        let mut row = area.row;
        for line in span.text().split('\n') {
            if row >= area.bottom() {
                break;
            }
            if line.is_empty() {
                row = row.saturating_add(1);
                continue;
            }
            for wrapped in Wrap::new(line, area.cols) {
                if row >= area.bottom() {
                    break;
                }
                self.put_str(Pos::new(area.col, row), wrapped, span.style(), area.right());
                row = row.saturating_add(1);
            }
        }
        row.saturating_sub(area.row)
    }

    /// Draw a meter as a bar, and record `spoken` as its description.
    ///
    /// Integer-only: progress in this game is elapsed ticks against a duration
    /// (DESIGN.md §5.0), and keeping floats out of the render path keeps a
    /// deterministic sim rendering deterministically. `done` is clamped to
    /// `total`; a `total` of zero draws an empty bar.
    ///
    /// `spoken` is what a reader hears — `"east wall integrity 34 percent"`, not
    /// a row of block glyphs. §14 names progress bars specifically.
    pub fn progress(&mut self, area: Rect, done: u32, total: u32, style: Style, spoken: &str) {
        // Spoken *before* the clip test, deliberately: a meter scrolled out of
        // its pane is still a fact a listener needs, and §14's whole contract is
        // that the linear stream does not depend on what happened to fit.
        self.frame
            .speech_mut()
            .push(UtteranceKind::Progress, style.role, spoken);
        self.meter(area, done, total, style);
    }

    /// Draw a meter as a bar, silently.
    ///
    /// The same glyphs as [`Painter::progress`] with **nothing said**. For a
    /// panel of standing meters — §10.1's five instruments — where speaking each
    /// one per frame would bury the stream in furniture, and §14 asks for
    /// *"progress announcements: completion only"*.
    ///
    /// A caller using this owes the listener one summary utterance covering the
    /// panel, which is what [`Painter::announce`] is for. Drawing meters and
    /// saying nothing at all would be a screen a reader cannot see.
    ///
    /// Integer-only: progress in this game is elapsed ticks against a duration
    /// (DESIGN.md §5.0), and keeping floats out of the render path keeps a
    /// deterministic sim rendering deterministically. `done` is clamped to
    /// `total`; a `total` of zero draws an empty bar.
    pub fn meter(&mut self, area: Rect, done: u32, total: u32, style: Style) {
        let area = area.intersection(self.area);
        if area.is_empty() {
            return;
        }

        let filled = filled_of(area.cols, done, total);
        let row = Rect::new(area.col, area.row, area.cols, 1);
        self.fill(row, '░', style);
        self.fill(Rect::new(area.col, area.row, filled, 1), '█', style);
    }

    /// The same meter, drawn as a column that fills **upward**.
    ///
    /// A level reads as growing from the floor, which is what §10.1's side panel
    /// wants when the pane is taller than it is wide.
    ///
    /// It lives here rather than in a frontend because the alternative already
    /// happened: the Bevy panel re-derived the fill arithmetic and hand-drew the
    /// `█`/`░` pair, so one instrument panel drew its bar two different ways
    /// depending on which way the pane had split. Changing the glyphs or the
    /// clamping in `meter` would have left the side panel on the old ones, and
    /// a player pressing F4 would see the same five instruments in two bar
    /// vocabularies.
    pub fn meter_upward(&mut self, area: Rect, done: u32, total: u32, style: Style) {
        let area = area.intersection(self.area);
        if area.is_empty() {
            return;
        }

        let filled = filled_of(area.rows, done, total);
        for step in 0..area.rows {
            let row = area.bottom().saturating_sub(1).saturating_sub(step);
            let glyph = if step < filled { '█' } else { '░' };
            self.fill(Rect::new(area.col, row, area.cols, 1), glyph, style);
        }
    }

    /// Speak something with no visual form of its own.
    ///
    /// For information a sighted player reads from structure — a pane's title in
    /// its border, a column header, a layout grouping.
    pub fn announce(&mut self, kind: UtteranceKind, role: Role, text: &str) {
        self.frame.speech_mut().push(kind, role, text);
    }

    /// Draw text whose meaning a neighbouring span already carries. Structural:
    /// writes no speech.
    ///
    /// For a row drawn in several styles — `battlements ....... [ DEGRADED ]`,
    /// where the label is base hue, the leader is dim, and only the bracket takes
    /// the danger accent. One [`Painter::span`] announces the row as a whole;
    /// the remaining runs are drawn with this so the reader hears one sentence
    /// rather than three fragments.
    ///
    /// Returns the number of cells written.
    ///
    /// **This is not a silent [`Painter::span`].** Text drawn here is invisible
    /// to a screen reader, so use it only where a span or [`Painter::announce`]
    /// on the same row has already said what the row means.
    pub fn glyphs(&mut self, at: Pos, text: &str, style: Style) -> u16 {
        self.put_str(at, text, style, u16::MAX)
    }

    /// Fill a region with one glyph. Structural: writes no speech.
    pub fn fill(&mut self, area: Rect, glyph: char, style: Style) {
        let area = area.intersection(self.area);
        let cell = Cell::new(glyph, style);
        for row in area.row..area.bottom() {
            for col in area.col..area.right() {
                self.frame.set(Pos::new(col, row), cell);
            }
        }
    }

    /// Blank the painter's whole region.
    pub fn clear(&mut self) {
        self.fill(self.area, ' ', Style::NORMAL);
    }

    /// Draw a single-line box, optionally titled.
    ///
    /// The box itself is structural and silent; the title is announced as a
    /// [`UtteranceKind::Heading`], which is what gives the pane an identity in
    /// the linear stream.
    ///
    /// A region narrower or shorter than two cells draws nothing.
    pub fn border(&mut self, area: Rect, title: Option<&str>, style: Style) {
        let area = area.intersection(self.area);
        if area.cols < 2 || area.rows < 2 {
            return;
        }

        let left = area.col;
        let right = area.right() - 1;
        let top = area.row;
        let bottom = area.bottom() - 1;

        for col in left..=right {
            self.put_cell(Pos::new(col, top), box_drawing::HORIZONTAL, style);
            self.put_cell(Pos::new(col, bottom), box_drawing::HORIZONTAL, style);
        }
        for row in top..=bottom {
            self.put_cell(Pos::new(left, row), box_drawing::VERTICAL, style);
            self.put_cell(Pos::new(right, row), box_drawing::VERTICAL, style);
        }
        self.put_cell(Pos::new(left, top), box_drawing::TOP_LEFT, style);
        self.put_cell(Pos::new(right, top), box_drawing::TOP_RIGHT, style);
        self.put_cell(Pos::new(left, bottom), box_drawing::BOTTOM_LEFT, style);
        self.put_cell(Pos::new(right, bottom), box_drawing::BOTTOM_RIGHT, style);

        let Some(title) = title else {
            return;
        };
        self.frame
            .speech_mut()
            .push(UtteranceKind::Heading, style.role, title);

        // ` title ` inset one cell from the top-left corner, stopping short of
        // the far corner so the box never breaks.
        let mut col = left.saturating_add(1);
        for text in [" ", title, " "] {
            col = col.saturating_add(self.put_str(Pos::new(col, top), text, style, right));
        }
    }

    /// The same box, drawn only `progress` of the way round its perimeter.
    ///
    /// Silent and untitled: a border arriving a cell at a time has no identity
    /// to announce yet, and [`Painter::border`] speaks its title as a heading —
    /// which would put a pane into the linear stream before the pane exists.
    ///
    /// The walk starts at the top-left and goes clockwise, which is the reading
    /// order the shape suggests. `progress` at or above 1.0 is exactly
    /// [`border`](Self::border) with no title.
    ///
    /// Where a frontend gets `progress` from is its own business — nothing in
    /// this crate knows what a second is. The *path* is geometry, which is why
    /// it is here.
    pub fn border_revealed(&mut self, area: Rect, style: Style, progress: f32) {
        let area = area.intersection(self.area);
        if area.cols < 2 || area.rows < 2 {
            return;
        }
        // `mix` is the crate's one float-to-integer conversion, and it already
        // carries the justification for it — see `tween`.
        let mut budget = crate::tween::mix(0, perimeter(area), progress.clamp(0.0, 1.0));

        for (at, glyph) in perimeter_cells(area) {
            if budget == 0 {
                return;
            }
            self.put_cell(at, glyph, style);
            budget -= 1;
        }
    }

    fn put_cell(&mut self, at: Pos, glyph: char, style: Style) {
        if self.area.contains(at) {
            self.frame.set(at, Cell::new(glyph, style));
        }
    }

    /// Write `text` rightwards from `at`, clipped to the region and to
    /// `right_limit`. Returns cells written.
    fn put_str(&mut self, at: Pos, text: &str, style: Style, right_limit: u16) -> u16 {
        if at.row < self.area.row || at.row >= self.area.bottom() {
            return 0;
        }

        let limit = self.area.right().min(right_limit);
        let mut col = at.col.max(self.area.col);
        // Glyphs falling left of the region are consumed, not shifted right.
        let skipped = usize::from(col.saturating_sub(at.col));

        let mut written = 0u16;
        for glyph in text.chars().skip(skipped) {
            if col >= limit {
                break;
            }
            self.frame
                .set(Pos::new(col, at.row), Cell::new(glyph, style));
            col = col.saturating_add(1);
            written = written.saturating_add(1);
        }
        written
    }
}

/// How many of `steps` a meter fills.
///
/// Widened so a long duration cannot overflow the multiply, and clamped so a
/// finished meter never overruns its own track.
fn filled_of(steps: u16, done: u32, total: u32) -> u16 {
    if total == 0 {
        return 0;
    }
    let scaled = u64::from(steps) * u64::from(done.min(total)) / u64::from(total);
    u16::try_from(scaled).unwrap_or(steps)
}

/// How many cells a box's outline occupies.
///
/// `2 * (w + h) - 4`: the four corners would otherwise be counted twice. Only
/// meaningful for a region at least two cells each way, which is the same floor
/// [`Painter::border`] draws nothing below.
const fn perimeter(area: Rect) -> u16 {
    let doubled = area.cols.saturating_add(area.rows).saturating_mul(2);
    doubled.saturating_sub(4)
}

/// The outline's cells, clockwise from the top-left corner.
///
/// Clockwise from the top left because that is the order the shape reads in, and
/// because starting anywhere else makes the corner a player watches for arrive
/// last.
fn perimeter_cells(area: Rect) -> impl Iterator<Item = (Pos, char)> {
    let left = area.col;
    let right = area.right().saturating_sub(1);
    let top = area.row;
    let bottom = area.bottom().saturating_sub(1);

    let across_top = (left..=right).map(move |col| {
        let glyph = if col == left {
            box_drawing::TOP_LEFT
        } else if col == right {
            box_drawing::TOP_RIGHT
        } else {
            box_drawing::HORIZONTAL
        };
        (Pos::new(col, top), glyph)
    });
    let down_right = (top.saturating_add(1)..bottom)
        .map(move |row| (Pos::new(right, row), box_drawing::VERTICAL));
    let back_along_bottom = (left..=right).rev().map(move |col| {
        let glyph = if col == left {
            box_drawing::BOTTOM_LEFT
        } else if col == right {
            box_drawing::BOTTOM_RIGHT
        } else {
            box_drawing::HORIZONTAL
        };
        (Pos::new(col, bottom), glyph)
    });
    let up_left = (top.saturating_add(1)..bottom)
        .rev()
        .map(move |row| (Pos::new(left, row), box_drawing::VERTICAL));

    across_top
        .chain(down_right)
        .chain(back_along_bottom)
        .chain(up_left)
}
