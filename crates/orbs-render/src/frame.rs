//! The Frame — one rendered screen, and the boundary between sim and frontend.
//!
//! A `Frame` is everything a frontend needs to draw a screen and nothing about
//! how to draw it: a grid of glyphs with semantic styles, a cursor position, and
//! the linear stream that says the same thing without pixels.
//!
//! Frontends receive `&Frame` and rasterise. They may add enrichment the other
//! frontend cannot reproduce — CRT effects, audio, fidelity tiers — provided it
//! carries **no information absent from the Frame** (DESIGN.md §13). The moment a
//! frontend conveys something the Frame does not, the other frontend is playing a
//! worse game rather than wearing a different skin.

use crate::cell::Cell;
use crate::geometry::{GridSize, Pos, Rect};
use crate::linear::Speech;
use crate::paint::Painter;

/// One screen's worth of cells, plus its linearisation.
///
/// Reuse a single `Frame` across ticks via [`Frame::reset`] rather than building
/// a new one; the grid can reach 160×45 and a siege redraws it every frame.
#[derive(Debug, Default, Clone)]
pub struct Frame {
    grid: GridSize,
    cells: Vec<Cell>,
    cursor: Option<Pos>,
    speech: Speech,
    magnified: Option<Rect>,
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
    /// Called once per rendered frame. Resizing is cheap and normal: the grid
    /// changes whenever the window resizes or multiplexing shifts the fidelity
    /// tier (§9).
    pub fn reset(&mut self, grid: GridSize) {
        self.grid = grid;
        self.cells.clear();
        self.cells.resize(grid.area(), Cell::BLANK);
        self.cursor = None;
        self.speech.clear();
        self.magnified = None;
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
    /// One row of cells, occupying two rows and twice the columns on screen. The
    /// prompt uses it at fine fidelity: a 32-pixel line is the thing you are
    /// typing into rendered at the size of the transcript around it, and giving
    /// it a blank row for company makes it no easier to read.
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
