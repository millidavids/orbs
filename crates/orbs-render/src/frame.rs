//! The Frame — one rendered screen, and the boundary between sim and frontend.
//!
//! A `Frame` is everything a frontend needs to draw a screen and nothing about
//! how to draw it: a grid of glyphs with semantic styles, a cursor position, and
//! the linear stream that says the same thing without pixels.
//!
//! Frontends receive `&Frame` and rasterise. They may add enrichment the other
//! cannot reproduce — CRT effects, audio — provided it carries no information
//! absent from the Frame (DESIGN.md §13); otherwise the other frontend is
//! playing a worse game rather than wearing a different skin.

use crate::cell::Cell;
use crate::geometry::{GridSize, Pos, Rect};
use crate::linear::Speech;
use crate::paint::Painter;
use crate::passage::{Crossing, Kept};
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
    /// Separate from [`tints`](Self::tints), which mean *materials*: the
    /// laboratory draws an instrument panel two columns from the editor, and a
    /// verb sharing a colour with a potion beside it would be one vocabulary
    /// meaning two things.
    syntax: Vec<(Rect, Lexeme)>,
    /// Somewhere to hold a region while a crossing reads it and writes over it.
    ///
    /// [`Passage::Gather`](crate::Passage) moves glyphs, and on the arriving
    /// half the screen it moves is *this* one. Reused rather than allocated per
    /// frame; [`reset`](Self::reset) leaves it alone, being scratch not screen.
    scratch: Kept,
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
    /// Called once per rendered frame. Under Bevy the grid is
    /// [`GRID`](crate::GRID) every time, so this is a blank rather than a
    /// resize; `orbs-tui` and `ORBS_GRID` are what still change it.
    pub fn reset(&mut self, grid: GridSize) {
        self.grid = grid;
        self.cells.clear();
        self.cells.resize(grid.area(), Cell::BLANK);
        self.cursor = None;
        self.speech.clear();
        self.magnified = None;
        // Cleared, not reallocated — the panel writes the same handful of
        // regions every frame.
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
    /// A frontend uploading geometry per frame wants this: on Bevy 0.19 a
    /// re-uploaded empty buffer is worse than wasted work. Says nothing about
    /// the cursor; a caller that draws one checks separately.
    #[must_use]
    pub fn is_blank(&self) -> bool {
        self.rows()
            .all(|row| row.iter().all(|cell| cell.is_blank()))
    }

    /// Where the caret sits, if it is shown.
    ///
    /// Kept out of [`Cell`] because the terminal frontend must position the
    /// *real* terminal cursor, which is what makes the input line work with the
    /// user's line editing and with screen readers that track the caret.
    #[must_use]
    pub const fn cursor(&self) -> Option<Pos> {
        self.cursor
    }

    /// Place or hide the caret.
    pub fn set_cursor(&mut self, cursor: Option<Pos>) {
        self.cursor = cursor.filter(|&pos| self.grid.contains(pos));
    }

    /// A region whose glyphs are drawn at double size: one row of cells,
    /// occupying two rows and twice the columns on screen.
    ///
    /// Nothing in the game sets this today — the prompt did until §19 fixed the
    /// grid — and the mechanism is kept because it is one constant from coming
    /// back.
    ///
    /// On the `Frame` because it is informational, not decoration: at double
    /// width a line holds half the characters, so what fits depends on it.
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
    /// Regions, not cells: a tint belongs to *what is in an instrument*, and
    /// per cell it would spend a byte on all 7,040 cells of a 160×44 grid for a
    /// value that varies across five ([`Cell`] is pinned at 8 bytes by a test).
    ///
    /// On the `Frame` for [`Frame::magnified`]'s reason. `orbs-tui` reads it per
    /// cell while blitting — a cell can be byte-identical while the wash over it
    /// changed, which is the flask's mixture band growing.
    ///
    /// Later regions win, so a caller may paint over an earlier tint without
    /// finding and removing it.
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
    /// An empty rectangle is ignored rather than stored: the panel skips whole
    /// instruments at narrow widths and would otherwise leave zero-area entries
    /// for `tint_at` to walk.
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
    /// A side-table, for [`Wash`]'s reason: a fifth `Style` field cost +28 KiB
    /// and ~1.2 µs a frame on every screen, which `a_cell_stays_eight_bytes`
    /// rejected. A syntax run is a *region* the way an instrument's bar is.
    ///
    /// Weight still carries the reading on its own — `ORBS_DUMP` has no colour
    /// and §14 has to hold on a greyscale tube — so [`Lexeme::weight`] is
    /// applied as the run is painted and the hue is a second, finer cut.
    ///
    /// An empty rectangle is ignored, as [`Self::set_tint`] does: the editor
    /// clips runs to a scrolled window.
    pub fn lit(&mut self, area: Rect, kind: Lexeme) {
        let area = area.intersection(self.area());
        // `None` is what an unlisted region already means.
        if !area.is_empty() && kind != Lexeme::None {
            self.syntax.push((area, kind));
        }
    }

    /// What part of speech is drawn at `at`, if any.
    ///
    /// Later regions win, as [`tint_at`](Self::tint_at) resolves, so a caller
    /// may paint over an earlier run without finding and removing it.
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

    /// Keep the cells of `area`, for a crossing to depart from.
    ///
    /// Called on every *settled* frame, since [`reset`](Self::reset) has
    /// blanked last frame's screen by the time anything could ask. A 43 KiB
    /// copy at a 120×45 grid, into a buffer [`Kept`] reuses. An area outside
    /// the grid is clipped rather than refused.
    pub fn keep(&self, area: Rect, into: &mut Kept) {
        let area = area.intersection(self.area());
        if area.is_empty() {
            into.clear();
            return;
        }
        let cells = (area.row..area.bottom()).flat_map(|row| {
            (area.col..area.right())
                .map(move |col| Pos::new(col, row))
                .map(|at| self.cell(at).copied().unwrap_or(Cell::BLANK))
        });
        into.fill(area, cells);
    }

    /// Draw `crossing` over `area`, departing from `from`.
    ///
    /// Applied *after* everything else has painted, so `from` supplies the old
    /// screen and the frame itself the new one. A crossing at either endpoint is
    /// a no-op by construction — see [`crate::passage`].
    ///
    /// One call per region, not one per screen: a room change moves the gauges
    /// and the instrument panel, but not the transcript between them, which is
    /// continuous history. Each region crosses with its own
    /// [`Toward`](crate::Toward), leaving by the edge it sits against; a
    /// surface that replaces the whole pane is one call over the lot. This
    /// replaced a spared-rectangle parameter that could only give both halves
    /// one direction.
    ///
    /// It does not touch [`speech`](Self::speech) or [`cursor`](Self::cursor):
    /// the linear stream is the *settled* screen, so a reader never waits for
    /// an animation (§14, §19).
    ///
    /// It does clear every [`tint`](Self::set_tint) and [`syntax run`](Self::lit)
    /// meeting `area`, since both resolve *per cell position*; translating the
    /// rectangles alongside the cells is not worth it for half a second. A
    /// region reaching outside `area` loses the part outside too.
    pub fn cross(&mut self, area: Rect, crossing: Crossing, from: Option<&Kept>) {
        let area = area.intersection(self.area());
        if area.is_empty() {
            return;
        }
        // A wash over a moved glyph is the bar's colour with no bar in it.
        self.tints
            .retain(|(region, _)| region.intersection(area).is_empty());
        self.syntax
            .retain(|(region, _)| region.intersection(area).is_empty());

        // The two families answer different questions, and the type says which:
        // a shape with no erosion is one that moves glyphs.
        let Some(erosion) = crossing.passage.erosion() else {
            self.gather(area, crossing, from);
            return;
        };

        for row in area.row..area.bottom() {
            for col in area.col..area.right() {
                let at = Pos::new(col, row);
                // Out: the screen that is leaving. In: the one painted here.
                let source = if crossing.is_leaving() {
                    from.map_or(Cell::BLANK, |kept| kept.cell(at))
                } else {
                    self.cell(at).copied().unwrap_or(Cell::BLANK)
                };
                self.set(
                    at,
                    crate::passage::cell_at(area, crossing, erosion, at, source),
                );
            }
        }
    }

    /// Draw `crossing` over `area`, departing from what is already there.
    ///
    /// [`cross`](Self::cross) departs from a screen the shell kept across the
    /// frame boundary. The boot card is not like that: it paints itself and
    /// then leaves, so it departs from the screen in front of it. Only the
    /// *leaving* half needs this — `cross` reads the frame's own cells on the
    /// way in.
    pub fn fold(&mut self, area: Rect, crossing: Crossing) {
        let area = area.intersection(self.area());
        if area.is_empty() {
            return;
        }
        // Borrowed out and put back, so the copy allocates once rather than
        // once a frame.
        let mut scratch = std::mem::take(&mut self.scratch);
        self.keep(area, &mut scratch);
        self.cross(area, crossing, Some(&scratch));
        self.scratch = scratch;
    }

    /// The glyphs flying to the middle, or out of it.
    ///
    /// Its own pass because it reads a screen it is also writing, so a copy is
    /// taken first: [`Self::scratch`], borrowed out and put back.
    fn gather(&mut self, area: Rect, crossing: Crossing, from: Option<&Kept>) {
        let mut scratch = std::mem::take(&mut self.scratch);
        if !crossing.is_leaving() {
            self.keep(area, &mut scratch);
        }
        for row in area.row..area.bottom() {
            for col in area.col..area.right() {
                let at = Pos::new(col, row);
                let cell =
                    crate::passage::sampled(area, crossing, at).map_or(Cell::BLANK, |source| {
                        if crossing.is_leaving() {
                            from.map_or(Cell::BLANK, |kept| kept.cell(source))
                        } else {
                            scratch.cell(source)
                        }
                    });
                self.set(at, cell);
            }
        }
        self.scratch = scratch;
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
    fn a_crossing_says_nothing() {
        // §14: the linear stream is the *settled* screen. A reader waiting half
        // a second for the animation pays for a display setting in capability.
        let mut frame = Frame::new(GridSize::new(12, 4));
        let area = frame.area();
        frame
            .painter(area)
            .announce(crate::UtteranceKind::Text, crate::Role::Normal, "the forge");
        let spoken: Vec<_> = frame
            .speech()
            .utterances()
            .map(|said| said.text.to_owned())
            .collect();

        frame.cross(area, halfway(), None);

        let after: Vec<_> = frame
            .speech()
            .utterances()
            .map(|said| said.text.to_owned())
            .collect();
        assert_eq!(spoken, after);
    }

    #[test]
    fn a_crossing_stays_inside_its_region() {
        // `tween`'s "no pane leaves the span of its own endpoints", for glyphs:
        // writing past the rectangle scribbles on the untouched transcript.
        let mut frame = Frame::new(GridSize::new(6, 3));
        frame
            .painter(frame.area())
            .fill(Rect::new(0, 0, 6, 3), 'x', Style::NORMAL);
        frame.cross(Rect::new(1, 1, 2, 1), halfway(), None);

        assert_eq!(frame.to_text(), "xxxxxx\nx  xxx\nxxxxxx\n");
    }

    #[test]
    fn what_is_not_crossed_is_left_alone() {
        // The transcript is continuous history, so blanking it on a domain
        // change would say the session went away.
        let mut frame = Frame::new(GridSize::new(6, 3));
        let whole = frame.area();
        frame.painter(whole).fill(whole, 'x', Style::NORMAL);

        // The strip: the top row, leaving upward.
        frame.cross(Rect::new(0, 0, 6, 1), halfway(), None);
        // The block: the last two columns of what is left, leaving rightward.
        frame.cross(Rect::new(4, 1, 2, 2), halfway(), None);

        assert_eq!(frame.to_text(), "      \nxxxx  \nxxxx  \n");
    }

    #[test]
    fn a_crossing_drops_the_washes_it_paints_over() {
        // A wash left behind while the glyphs moved is a coloured blank cell.
        let mut frame = Frame::new(GridSize::new(8, 2));
        frame.set_tint(Rect::new(0, 0, 4, 1), Wash::plain(crate::Tint::Green));
        frame.set_tint(Rect::new(6, 1, 2, 1), Wash::plain(crate::Tint::Green));

        frame.cross(Rect::new(0, 0, 4, 1), halfway(), None);

        assert_eq!(
            frame.tint_at(Pos::new(0, 0)),
            None,
            "the crossed wash stayed"
        );
        assert_eq!(
            frame.tint_at(Pos::new(7, 1)),
            Some(Wash::plain(crate::Tint::Green)),
            "a wash outside the crossing was taken with it",
        );
    }

    #[test]
    fn keeping_clips_to_the_grid() {
        // An out-of-date layout produces a smaller picture rather than a panic,
        // as `Frame::painter` already states for its own area.
        let mut frame = Frame::new(GridSize::new(4, 2));
        frame.set(Pos::new(3, 1), Cell::new('x', Style::NORMAL));

        let mut kept = Kept::default();
        frame.keep(Rect::new(2, 0, 99, 99), &mut kept);

        assert_eq!(kept.area(), Rect::new(2, 0, 2, 2));
        assert_eq!(kept.cell(Pos::new(3, 1)), Cell::new('x', Style::NORMAL));
    }

    /// A crossing frozen at its midpoint, which is where a region is emptiest.
    fn halfway() -> Crossing {
        Crossing {
            passage: crate::Passage::Wipe,
            toward: crate::Toward::Right,
            progress: 0.5,
        }
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
