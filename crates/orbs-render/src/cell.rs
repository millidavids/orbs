//! The unit of the grid.

use crate::cp437::{self, REPLACEMENT};
use crate::style::Style;

/// One cell of the grid: a glyph and what it means.
///
/// A `Cell` never carries a colour. The frontend resolves [`Style`] against the
/// active phosphor theme (Bevy) or the user's terminal palette (TUI).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cell {
    /// The glyph, guaranteed to be inside the CP437 repertoire (see
    /// [`crate::is_renderable`]).
    ///
    /// The invariant holds because [`Cell::new`] is the only constructor and it
    /// substitutes anything else.
    pub glyph: char,
    /// What the cell means.
    pub style: Style,
}

impl Cell {
    /// An empty cell in the base style. What the grid resets to.
    pub const BLANK: Self = Self {
        glyph: ' ',
        style: Style::NORMAL,
    };

    /// A cell.
    ///
    /// A glyph outside the repertoire is replaced with [`crate::REPLACEMENT`]
    /// rather than rejected: a missing glyph is an authoring bug in a prose data
    /// file, and a visibly wrong character in one cell is a better failure than
    /// a panic mid-siege or a silently blank line.
    #[must_use]
    pub fn new(glyph: char, style: Style) -> Self {
        let glyph = if cp437::is_renderable(glyph) {
            glyph
        } else {
            REPLACEMENT
        };
        Self { glyph, style }
    }

    /// Whether the cell would draw nothing but background.
    #[must_use]
    pub const fn is_blank(self) -> bool {
        self.glyph == ' '
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::BLANK
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unrenderable_glyphs_are_substituted() {
        assert_eq!(Cell::new('漢', Style::NORMAL).glyph, REPLACEMENT);
        assert_eq!(Cell::new('\n', Style::NORMAL).glyph, REPLACEMENT);
    }

    #[test]
    fn renderable_glyphs_survive_intact() {
        assert_eq!(Cell::new('░', Style::NORMAL).glyph, '░');
        assert_eq!(Cell::new('Σ', Style::NORMAL).glyph, 'Σ');
    }

    #[test]
    fn default_is_blank() {
        assert_eq!(Cell::default(), Cell::BLANK);
        assert!(Cell::BLANK.is_blank());
    }
}
