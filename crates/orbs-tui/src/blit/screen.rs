//! The shadow buffer, and writing only what changed.
//!
//! Only differences are written, since a full repaint smears over `ssh`. The
//! key is `(glyph, ink)`, not `Cell`: a cell can be unchanged while its wash
//! changed, so resolve, then compare.

use std::io::Write;

use crossterm::style::{Attribute, Color, Print, SetAttribute, SetForegroundColor};
use crossterm::terminal::{Clear, ClearType};
use crossterm::{cursor, queue};
use orbs_render::{Frame, GridSize, Pos};

use super::narrow::narrowed;
use crate::theme::{self, Ink, Weight};

/// What the terminal is currently showing, one entry per cell.
pub(crate) struct Screen {
    /// Row-major, `grid.area()` long.
    cells: Vec<(char, Ink)>,
    grid: GridSize,
    /// Whether this terminal draws the game's symbols one column wide.
    ///
    /// See [`crate::term::symbols_are_narrow`]. `false` swaps the at-risk glyphs
    /// for ASCII the terminal cannot disagree about.
    narrow: bool,
    /// Whether the terminal must be wiped before the next draw — the half of a
    /// resize the buffer alone cannot express. See [`Screen::resize`].
    wipe: bool,
    /// Whether every hue is taken out — `sight greyscale`, or `F8`.
    ///
    /// Applied after `resolve` and before the diff key, so a toggle redraws the
    /// coloured cells and nothing else.
    greyscale: bool,
}

/// A blank cell, as the shadow buffer starts.
const BLANK: (char, Ink) = (' ', Ink::PLAIN);

impl Screen {
    /// A shadow buffer for a terminal of this size.
    pub(crate) fn new(grid: GridSize, narrow: bool) -> Self {
        Self {
            cells: vec![BLANK; grid.area()],
            grid,
            narrow,
            // The alternate screen arrives blank, and a wipe here would put a
            // visible clear between entering it and the first paint.
            wipe: false,
            greyscale: false,
        }
    }

    /// Take every hue out, or put them back.
    pub(crate) const fn set_greyscale(&mut self, greyscale: bool) {
        self.greyscale = greyscale;
    }

    /// Whether this terminal draws the game's symbols one column wide, as
    /// measured at startup — handed on to the next session's buffer.
    pub(crate) const fn narrow(&self) -> bool {
        self.narrow
    }

    /// Forget everything, because the terminal is a different size or has been
    /// scribbled on.
    ///
    /// On every resize. The terminal is wiped too, or blank cells compare equal
    /// over old glyphs; in the redraw's flush, so no frame is empty.
    pub(crate) fn resize(&mut self, grid: GridSize) {
        self.grid = grid;
        self.cells.clear();
        self.cells.resize(grid.area(), BLANK);
        self.wipe = true;
    }

    /// Write the cells that changed, and put the cursor where the Frame says.
    ///
    /// # Errors
    ///
    /// If the terminal cannot be written to.
    pub(crate) fn draw(&mut self, frame: &Frame, out: &mut impl Write) -> std::io::Result<()> {
        // A resize between poll and paint is normal; the Frame is the authority.
        if frame.size() != self.grid {
            self.resize(frame.size());
        }
        // In the same flush as the cells that follow: a wipe on its own write
        // would put one empty frame on screen.
        if std::mem::take(&mut self.wipe) {
            queue!(out, Clear(ClearType::All))?;
        }

        // Track what the terminal is already set to, so a run of cells in one
        // colour costs one escape sequence rather than one per cell.
        let mut pen: Option<Ink> = None;
        let mut at: Option<(u16, u16)> = None;

        for (row, cells) in frame.rows().enumerate() {
            let Ok(row) = u16::try_from(row) else {
                break;
            };
            for (col, cell) in cells.iter().enumerate() {
                let Ok(col) = u16::try_from(col) else {
                    break;
                };
                // Both side-tables, read per cell: a cell can be byte-identical
                // while the region over it changed.
                let here = Pos::new(col, row);
                let ink = theme::resolve(cell.style, frame.tint_at(here), frame.lit_at(here));
                let ink = if self.greyscale {
                    ink.without_hue()
                } else {
                    ink
                };
                let glyph = self.glyph(cell.glyph);
                let index = self.index(col, row);
                if self.cells[index] == (glyph, ink) {
                    continue;
                }
                self.cells[index] = (glyph, ink);

                // Only move when the last write did not leave us here.
                if at != Some((col, row)) {
                    queue!(out, cursor::MoveTo(col, row))?;
                }
                if pen != Some(ink) {
                    set_pen(out, ink)?;
                    pen = Some(ink);
                }
                queue!(out, Print(glyph))?;
                at = Some((col.saturating_add(1), row));
            }
        }

        // The real terminal cursor, which `orbs-render` keeps off `Cell` so the
        // input line works with line editing and a screen reader can track it.
        if let Some(pos) = frame.cursor() {
            queue!(out, cursor::MoveTo(pos.col, pos.row), cursor::Show)?;
        } else {
            queue!(out, cursor::Hide)?;
        }
        out.flush()
    }

    const fn index(&self, col: u16, row: u16) -> usize {
        row as usize * self.grid.cols as usize + col as usize
    }

    /// The glyph this terminal can actually draw in one column.
    const fn glyph(&self, glyph: char) -> char {
        if self.narrow { glyph } else { narrowed(glyph) }
    }
}

/// Set the terminal's pen to `ink`.
///
/// `Attribute::Reset` first, because dim and bold are not opposites: SGR 22
/// clears both and there is no "not dim" that leaves bold alone. Colour before
/// weight, because under `NO_COLOR` crossterm writes a colour as `ESC[m` — SGR
/// 0 — which would wipe a weight written before it, and weight is the channel
/// built to survive a terminal with no colour.
fn set_pen(out: &mut impl Write, ink: Ink) -> std::io::Result<()> {
    queue!(
        out,
        SetAttribute(Attribute::Reset),
        SetForegroundColor(ink.colour.unwrap_or(Color::Reset))
    )?;
    match ink.weight {
        Weight::Dim => queue!(out, SetAttribute(Attribute::Dim)),
        Weight::Plain => Ok(()),
        Weight::Bold => queue!(out, SetAttribute(Attribute::Bold)),
    }
}
