//! The forge's lattice, as a picture.
//!
//! Nine glyphs in three columns, and the residue the last fall left. Pure
//! geometry and text: this crate may not depend on `orbs-sim`, so every word on
//! the board — the title, the charm's name, the tally — is handed in.
//!
//! # Shape, not colour
//!
//! §14. A lit glyph and a dark one differ in **glyph**, so the board reads in a
//! dump, in greyscale and to a screen reader. The tint is enrichment and carries
//! nothing: the whole picture is one hue.
//!
//! That is the maze's rule and the pylon's, and this domain needs it more than
//! either — the puzzle *is* which glyphs are lit, so a colour-only tell would
//! make it unplayable rather than merely uglier.

use crate::Tint;

/// A lit glyph.
pub const ALIGHT: char = '☼';

/// A dark one.
pub const DARK: char = '·';

/// The one hue the lattice draws in — enrichment, and nothing rests on it.
pub const GLOW: Tint = Tint::Gold;

/// A lattice, ready to draw.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lattice {
    /// The column names, left to right. **Handed in**, because they are content.
    pub columns: Vec<String>,
    /// Every glyph, row-major, `true` where lit.
    pub glyphs: Vec<bool>,
    /// How many columns wide.
    pub width: usize,
    /// Which columns are snapped for the attempt being built.
    ///
    /// Drawn nowhere — the grid already shows what a snap did. It is here
    /// because *what has been pressed* and *what the board looks like* are
    /// different questions, and anything reading the board to decide the next
    /// press needs the first: `orbs-balance`'s driver walks the same eight-rung
    /// table a spell does, and without this it could only count laps, which
    /// stops being true the moment a fall springs back and clears the presses.
    pub snapped: Vec<bool>,
    /// What the last fall left on the bottom row, `true` where lit.
    ///
    /// **The signal the whole puzzle turns on**, so it gets its own strip under
    /// a rule rather than being left for the player to find in the grid.
    pub residue: Vec<bool>,
    /// The charm being bound, and what has been spent — already rendered.
    pub tally: String,
    /// The board's title, already rendered.
    pub title: String,
}

impl Lattice {
    /// Cells the column labels reserve.
    const COLUMN: u16 = 9;

    /// How wide the board is, border included.
    #[must_use]
    pub fn cols(&self) -> u16 {
        let inner = Self::COLUMN * u16::try_from(self.width).unwrap_or(3);
        inner.max(u16::try_from(self.tally.chars().count()).unwrap_or(0) + 2) + 2
    }

    /// How tall it is, border included.
    ///
    /// **Fixed, never sized to what is standing.** A board that grew a row when
    /// a residue appeared would move the transcript under the player's eye at
    /// the exact moment they were reading it — the rampart's rule, and the
    /// sheet's.
    #[must_use]
    pub fn rows(&self) -> u16 {
        let high = u16::try_from(self.glyphs.len() / self.width.max(1)).unwrap_or(3);
        // border, columns, the grid, a rule, the residue, the tally, border
        high + 6
    }

    /// One row of the grid, as glyphs spaced under their columns.
    #[must_use]
    pub fn row(&self, row: usize) -> String {
        (0..self.width)
            .map(|column| {
                let lit = self
                    .glyphs
                    .get(row * self.width + column)
                    .copied()
                    .unwrap_or(false);
                Self::cell(if lit { ALIGHT } else { DARK })
            })
            .collect()
    }

    /// The residue strip, on the same stride as the grid.
    #[must_use]
    pub fn residue_row(&self) -> String {
        (0..self.width)
            .map(|column| {
                let lit = self.residue.get(column).copied().unwrap_or(false);
                Self::cell(if lit { ALIGHT } else { DARK })
            })
            .collect()
    }

    /// The column names, on the same stride.
    #[must_use]
    pub fn heading(&self) -> String {
        self.columns.iter().map(|name| Self::label(name)).collect()
    }

    /// The residue **as words**, for a reader.
    ///
    /// **Not [`residue_row`](Self::residue_row), and that was a real §14
    /// failure.** The spoken line interpolated the drawn strip, so a reader
    /// heard `☼` and `·` and twenty-four spaces where a sighted player reads
    /// three columns. This domain is the one place there is no falling back to
    /// shape: *which glyphs are lit* is the puzzle, and the residue is the whole
    /// of what a decision turns on — so it has to be sayable.
    ///
    /// Named, never positional: *"apex lit, belt dark, hem dark"* survives being
    /// heard once, where *"lit, dark, dark"* asks the listener to hold an order
    /// they were never told.
    #[must_use]
    pub fn residue_spoken(&self) -> String {
        self.columns
            .iter()
            .enumerate()
            .map(|(index, name)| {
                let lit = self.residue.get(index).copied().unwrap_or(false);
                format!("{name} {}", if lit { "lit" } else { "dark" })
            })
            .collect::<Vec<String>>()
            .join(", ")
    }

    /// One glyph, centred in its column.
    fn cell(glyph: char) -> String {
        let pad = (Self::COLUMN as usize - 1) / 2;
        format!("{:pad$}{glyph}{:pad$}", "", "", pad = pad)
    }

    /// One label, in its column.
    fn label(name: &str) -> String {
        format!("{name:<width$}", width = Self::COLUMN as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lattice() -> Lattice {
        Lattice {
            columns: vec!["apex".into(), "belt".into(), "hem".into()],
            glyphs: vec![
                true, false, true, //
                false, true, false, //
                true, false, false,
            ],
            snapped: vec![true, false, false],
            width: 3,
            residue: vec![true, false, false],
            tally: "hurried, 6 spent".into(),
            title: "lattice".into(),
        }
    }

    /// **Every row is the same width**, or the columns stop lining up and a
    /// player cannot read down one to see what a snap did.
    #[test]
    fn every_row_lines_up_under_its_column() {
        let board = lattice();
        let heading = board.heading().chars().count();
        for row in 0..3 {
            assert_eq!(
                board.row(row).chars().count(),
                heading,
                "row {row} is not the width of the heading",
            );
        }
        assert_eq!(board.residue_row().chars().count(), heading);
    }

    /// A lit glyph and a dark one differ in **shape** (§14), so the board
    /// survives greyscale, a dump and a screen reader.
    #[test]
    fn lit_and_dark_are_different_glyphs() {
        assert_ne!(ALIGHT, DARK);
        let board = lattice();
        assert!(board.row(0).contains(ALIGHT), "no lit glyph on a lit row");
        assert!(board.row(0).contains(DARK), "no dark glyph on a mixed row");
    }

    /// The board never grows or shrinks with what is on it.
    #[test]
    fn the_board_is_a_fixed_size() {
        let mut board = lattice();
        let (cols, rows) = (board.cols(), board.rows());
        board.residue = vec![false, false, false];
        board.glyphs = vec![false; 9];
        assert_eq!((board.cols(), board.rows()), (cols, rows));
    }
}
