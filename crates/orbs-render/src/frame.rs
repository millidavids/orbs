//! The Frame — one rendered screen, and the boundary between sim and frontend.
//!
//! A `Frame` is everything a frontend needs to draw a screen and nothing about
//! how to draw it: a grid of glyphs with semantic styles, a cursor position, and
//! the linear stream that says the same thing without pixels.
//!
//! Frontends receive `&Frame` and rasterise. They may add enrichment the other
//! frontend cannot reproduce — CRT effects, audio — provided it
//! carries **no information absent from the Frame** (DESIGN.md §13). The moment a
//! frontend conveys something the Frame does not, the other frontend is playing a
//! worse game rather than wearing a different skin.

use crate::cell::Cell;
use crate::geometry::{GridSize, Pos, Rect};
use crate::linear::Speech;
use crate::paint::Painter;
use crate::style::{Lexeme, Wash};

/// One screen's worth of cells, plus its linearisation.
///
/// Reuse a single `Frame` across ticks via [`Frame::reset`] rather than building
/// a new one; the grid is 120×45 and a siege redraws it every frame.
#[derive(Debug, Default, Clone)]
pub struct Frame {
    grid: GridSize,
    cells: Vec<Cell>,
    cursor: Option<Pos>,
    speech: Speech,
    magnified: Option<Rect>,
    /// Regions drawn in a material's colour family. See [`Frame::tint_at`].
    tints: Vec<(Rect, Wash)>,
    /// Runs of spell text, by part of speech. See [`Frame::lit`].
    ///
    /// Separate from [`tints`](Self::tints) rather than folded into it, and
    /// that is the decision: `Tint` means **materials**, and the laboratory
    /// draws an instrument panel two columns from the editor. A verb sharing a
    /// colour name with a potion in the bar beside it would be one vocabulary
    /// meaning two things on one screen.
    syntax: Vec<(Rect, Lexeme)>,
}

impl Frame {
    /// A blank frame at `grid`.
    #[must_use]
    pub fn new(grid: GridSize) -> Self {
        let mut frame = Self::default();
        frame.reset(grid);
        frame
    }

    /// Blank the frame and resize it, keeping the existing allocations.
    ///
    /// Called once per rendered frame. Under the Bevy frontend the grid is
    /// [`GRID`](crate::GRID) every time, so this is a blank rather than a
    /// resize; `orbs-tui` and `ORBS_GRID` are what still change it, and resizing
    /// is cheap because the allocations are kept.
    pub fn reset(&mut self, grid: GridSize) {
        self.grid = grid;
        self.cells.clear();
        self.cells.resize(grid.area(), Cell::BLANK);
        self.cursor = None;
        self.speech.clear();
        self.magnified = None;
        // Cleared, not reallocated — the panel writes the same handful of
        // regions every frame and this keeps the allocation across all of them.
        self.tints.clear();
        self.syntax.clear();
    }

    /// The frame's dimensions.
    #[must_use]
    pub const fn size(&self) -> GridSize {
        self.grid
    }

    /// The whole frame as a rectangle.
    #[must_use]
    pub const fn area(&self) -> Rect {
        self.grid.to_rect()
    }

    /// The cell at `pos`, or `None` if it is outside the grid.
    #[must_use]
    pub fn cell(&self, pos: Pos) -> Option<&Cell> {
        self.cells.get(self.index(pos)?)
    }

    /// One row of cells, or `None` if `row` is outside the grid.
    #[must_use]
    pub fn row(&self, row: u16) -> Option<&[Cell]> {
        if row >= self.grid.rows {
            return None;
        }
        let width = usize::from(self.grid.cols);
        let start = usize::from(row) * width;
        self.cells.get(start..start + width)
    }

    /// Every row, top to bottom. The rasterisation order for both frontends.
    pub fn rows(&self) -> impl Iterator<Item = &[Cell]> {
        self.cells.chunks_exact(usize::from(self.grid.cols).max(1))
    }

    /// Whether every cell is blank.
    ///
    /// A frontend uploading geometry per frame wants this: a screen with nothing
    /// on it produces no geometry, and re-uploading an empty buffer every frame
    /// is at best wasted work — and on Bevy 0.19 it is worse than that, which is
    /// why this exists. Says nothing about the cursor; a caller that draws one
    /// checks it separately.
    #[must_use]
    pub fn is_blank(&self) -> bool {
        self.rows()
            .all(|row| row.iter().all(|cell| cell.is_blank()))
    }

    /// Where the caret sits, if it is shown.
    ///
    /// Kept out of [`Cell`] because the terminal frontend must position the
    /// *real* terminal cursor — that is what makes the input line work with the
    /// user's own line editing and with screen readers that track the caret.
    #[must_use]
    pub const fn cursor(&self) -> Option<Pos> {
        self.cursor
    }

    /// Place or hide the caret.
    pub fn set_cursor(&mut self, cursor: Option<Pos>) {
        self.cursor = cursor.filter(|&pos| self.grid.contains(pos));
    }

    /// A region whose glyphs are drawn at **double size**.
    ///
    /// One row of cells, occupying two rows and twice the columns on screen.
    ///
    /// **Nothing in the game sets this today.** The prompt did, while
    /// [`INPUT_ROWS`](crate::INPUT_ROWS) was 2 — a second row spent to keep the
    /// line's *pixel* height when a finer fidelity tier shrank the cells. §19
    /// fixed the grid, so there is no tier to compensate for and the doubling
    /// became plain magnification: a prompt twice the transcript's size at every
    /// window, and typing into half the columns. It is one constant from coming
    /// back, and the mechanism is kept for that rather than for a caller it does
    /// not have.
    ///
    /// This lives on the `Frame` rather than in the frontend because it is
    /// **informational**, not decoration: at double width a line holds half the
    /// characters, so what fits depends on it. Rule 2 draws the line at *how a
    /// cell is drawn*, and this is what is drawn where.
    #[must_use]
    pub const fn magnified(&self) -> Option<Rect> {
        self.magnified
    }

    /// Mark a region for double-size drawing. One row only; taller is clamped.
    pub fn set_magnified(&mut self, area: Option<Rect>) {
        self.magnified = area.map(|area| Rect::new(area.col, area.row, area.cols, 1));
    }

    /// The colour family a region draws in, if one was asked for.
    ///
    /// **Regions, not cells, and that is the whole design.** A tint is a
    /// property of *what is in an instrument*, so every cell of one bar shares
    /// it — encoding it per cell would spend a byte on all 7,040 cells of a
    /// 160×44 grid to express a value that varies across five of them, and
    /// [`Cell`] is pinned at 8 bytes by a test that records what the
    /// last such byte cost.
    ///
    /// It lives on the `Frame` rather than beside it for the same reason
    /// [`Frame::magnified`] does: the moment a frontend is handed something the
    /// Frame does not carry, the other frontend is playing a worse game rather
    /// than wearing a different skin. `orbs-tui` reads this and resolves the
    /// same eight names to ANSI indices — and reads it **per cell while
    /// blitting**, because a tint is a region: a cell can be byte-identical
    /// while the wash over it changed, which is what the flask's mixture band
    /// growing looks like. A diff keyed on `Cell` alone would miss every frame
    /// of it.
    ///
    /// **Later regions win**, so a caller may paint over an earlier tint without
    /// having to find and remove it — the same last-write-wins a `Cell` has.
    #[must_use]
    pub fn tint_at(&self, at: Pos) -> Option<Wash> {
        self.tints
            .iter()
            .rev()
            .find(|(area, _)| area.contains(at))
            .map(|(_, wash)| *wash)
    }

    /// Draw a region in a material's colour family.
    ///
    /// An empty rectangle is ignored rather than stored, so a caller need not
    /// check — the panel skips whole instruments at narrow widths and would
    /// otherwise leave zero-area entries for `tint_at` to walk.
    pub fn set_tint(&mut self, area: Rect, wash: Wash) {
        let area = area.intersection(self.area());
        if !area.is_empty() {
            self.tints.push((area, wash));
        }
    }

    /// Every tinted region, for a frontend that would rather walk them than
    /// probe per cell.
    #[must_use]
    pub fn tints(&self) -> &[(Rect, Wash)] {
        &self.tints
    }

    /// Draw a run of spell text in its part of speech's colour.
    ///
    /// # A side-table, for [`Wash`]'s reason
    ///
    /// `Cell` is pinned at eight bytes and `Style` has no spare one, so a fifth
    /// field cost +28 KiB and ~1.2 µs a frame on every screen — measured, and
    /// what `a_cell_stays_eight_bytes` rejected. A syntax run is a *region* in
    /// exactly the way an instrument's bar is, so it is affordable here and
    /// would not be per cell: a handful of entries on the one frame that has an
    /// editor open, and none at all on every other screen.
    ///
    /// **Weight still carries the reading on its own.** The hue is a second,
    /// finer cut — `ORBS_DUMP` has no colour, and §14 has to hold on a greyscale
    /// tube — so [`Lexeme::weight`] is applied to the `Style` as the run is
    /// painted and this is what a frontend adds on top of it.
    ///
    /// An empty rectangle is ignored rather than stored, as [`Self::set_tint`]
    /// does:
    /// the editor clips runs to a scrolled window and would otherwise leave
    /// zero-area entries for [`lit_at`](Self::lit_at) to walk.
    pub fn lit(&mut self, area: Rect, kind: Lexeme) {
        let area = area.intersection(self.area());
        // `None` is *"nothing to say about this"*, which is what an unlisted
        // region already means — storing it would be a frontend asking twice.
        if !area.is_empty() && kind != Lexeme::None {
            self.syntax.push((area, kind));
        }
    }

    /// What part of speech is drawn at `at`, if any.
    ///
    /// **Later regions win**, exactly as [`tint_at`](Self::tint_at) resolves, so
    /// a caller may paint over an earlier run without finding and removing it.
    #[must_use]
    pub fn lit_at(&self, at: Pos) -> Option<Lexeme> {
        self.syntax
            .iter()
            .rev()
            .find(|(area, _)| area.contains(at))
            .map(|(_, kind)| *kind)
    }

    /// Every lit region, for a frontend that would rather walk them than probe
    /// per cell — and for the dump, which is the only instrument that can show
    /// this at all.
    #[must_use]
    pub fn syntax(&self) -> &[(Rect, Lexeme)] {
        &self.syntax
    }

    /// The linear stream for this frame.
    #[must_use]
    pub const fn speech(&self) -> &Speech {
        &self.speech
    }

    /// A painter bounded to `area`.
    ///
    /// The returned painter cannot write outside `area`, and `area` is itself
    /// clipped to the frame, so an out-of-date layout produces a smaller drawing
    /// rather than a panic.
    pub fn painter(&mut self, area: Rect) -> Painter<'_> {
        Painter::new(self, area)
    }

    /// The frame's glyphs as newline-separated rows.
    ///
    /// A diagnostic and test view — it discards every style, which is most of
    /// the frame's meaning. Not a rendering path.
    #[must_use]
    pub fn to_text(&self) -> String {
        let mut out = String::with_capacity(self.cells.len() + usize::from(self.grid.rows));
        for row in self.rows() {
            out.extend(row.iter().map(|cell| cell.glyph));
            out.push('\n');
        }
        out
    }

    pub(crate) fn set(&mut self, pos: Pos, cell: Cell) {
        if let Some(index) = self.index(pos) {
            self.cells[index] = cell;
        }
    }

    pub(crate) const fn speech_mut(&mut self) -> &mut Speech {
        &mut self.speech
    }

    fn index(&self, pos: Pos) -> Option<usize> {
        if !self.grid.contains(pos) {
            return None;
        }
        Some(usize::from(pos.row) * usize::from(self.grid.cols) + usize::from(pos.col))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::Style;

    #[test]
    fn a_new_frame_is_blank() {
        let frame = Frame::new(GridSize::new(4, 2));
        assert_eq!(frame.to_text(), "    \n    \n");
        assert!(frame.speech().is_empty());
        assert_eq!(frame.cursor(), None);
    }

    #[test]
    fn cells_address_row_major() {
        let mut frame = Frame::new(GridSize::new(3, 2));
        frame.set(Pos::new(2, 1), Cell::new('x', Style::NORMAL));
        assert_eq!(frame.to_text(), "   \n  x\n");
    }

    #[test]
    fn writes_outside_the_grid_are_dropped() {
        let mut frame = Frame::new(GridSize::new(2, 2));
        frame.set(Pos::new(9, 9), Cell::new('x', Style::NORMAL));
        assert_eq!(frame.to_text(), "  \n  \n");
    }

    #[test]
    fn reset_reuses_the_allocation() {
        let mut frame = Frame::new(GridSize::new(80, 22));
        let capacity = frame.cells.capacity();

        frame.set(Pos::ORIGIN, Cell::new('x', Style::NORMAL));
        frame.reset(GridSize::new(80, 22));

        assert_eq!(frame.cell(Pos::ORIGIN), Some(&Cell::BLANK));
        assert_eq!(frame.cells.capacity(), capacity);
    }

    #[test]
    fn reset_resizes() {
        let mut frame = Frame::new(GridSize::new(80, 22));
        frame.reset(GridSize::new(120, 33));
        assert_eq!(frame.size(), GridSize::new(120, 33));
        assert_eq!(frame.cells.len(), 120 * 33);
    }

    #[test]
    fn a_cursor_outside_the_grid_is_refused() {
        let mut frame = Frame::new(GridSize::new(4, 4));
        frame.set_cursor(Some(Pos::new(9, 9)));
        assert_eq!(frame.cursor(), None);

        frame.set_cursor(Some(Pos::new(3, 3)));
        assert_eq!(frame.cursor(), Some(Pos::new(3, 3)));
    }

    #[test]
    fn rows_cover_the_whole_grid() {
        let frame = Frame::new(GridSize::new(7, 5));
        assert_eq!(frame.rows().count(), 5);
        assert!(frame.rows().all(|row| row.len() == 7));
        assert_eq!(frame.row(4).map(<[Cell]>::len), Some(7));
        assert_eq!(frame.row(5), None);
    }
}
