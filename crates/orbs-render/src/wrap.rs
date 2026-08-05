//! Greedy word wrapping, by cells rather than by bytes.
//!
//! Prose is the bulk of this game's content — a ~88k-word budget (DESIGN.md
//! §12) — and every pane is narrow, so wrapping runs on almost everything the
//! game draws. It yields borrowed slices and allocates nothing.
//!
//! Widths count **characters**, not bytes: the grid is one glyph per cell, and
//! the repertoire ([`crate::cp437`]) admits multi-byte glyphs like `░` and `Σ`
//! that occupy exactly one cell.

/// Lines of `text` greedily fitted to `width` cells.
///
/// A word longer than the line is broken mid-word rather than allowed to
/// overflow the pane, because overflowing would corrupt a neighbouring pane's
/// column of the grid.
///
/// Embedded newlines are **not** handled here — the caller splits on them first,
/// which is what lets an empty line consume a row instead of vanishing.
pub(crate) struct Wrap<'a> {
    rest: &'a str,
    width: u16,
}

impl<'a> Wrap<'a> {
    pub(crate) fn new(text: &'a str, width: u16) -> Self {
        Self {
            rest: text.trim_start_matches(' '),
            width,
        }
    }
}

impl<'a> Iterator for Wrap<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let width = usize::from(self.width);
        if width == 0 || self.rest.is_empty() {
            return None;
        }

        // Walk one line's worth of characters, remembering the last space we
        // could break at.
        let mut last_space = None;
        let mut overflow = None;
        for (count, (byte, glyph)) in self.rest.char_indices().enumerate() {
            if count == width {
                overflow = Some(byte);
                break;
            }
            if glyph == ' ' {
                last_space = Some(byte);
            }
        }

        let Some(overflow) = overflow else {
            // What remains fits on one line.
            let line = self.rest.trim_end_matches(' ');
            self.rest = "";
            return Some(line);
        };

        // Break at the last space that fits; failing that the word is wider than
        // the pane, so break it flush at the edge.
        let (line_end, next_start) = match last_space {
            Some(space) => (space, space + 1),
            None => (overflow, overflow),
        };

        // `next_start` is always past zero — `width` is non-zero and the input is
        // left-trimmed — so `rest` strictly shrinks and this terminates.
        let line = self.rest[..line_end].trim_end_matches(' ');
        self.rest = self.rest[next_start..].trim_start_matches(' ');
        Some(line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wrap(text: &str, width: u16) -> Vec<&str> {
        Wrap::new(text, width).collect()
    }

    #[test]
    fn text_that_fits_stays_on_one_line() {
        assert_eq!(wrap("the orb warms", 20), ["the orb warms"]);
    }

    #[test]
    fn exact_width_does_not_wrap() {
        assert_eq!(wrap("abcde", 5), ["abcde"]);
    }

    #[test]
    fn one_char_over_wraps() {
        assert_eq!(wrap("abc de", 5), ["abc", "de"]);
    }

    #[test]
    fn breaks_at_spaces() {
        assert_eq!(
            wrap("the athanor seethes and will not settle", 12),
            ["the athanor", "seethes and", "will not", "settle"]
        );
    }

    #[test]
    fn a_word_wider_than_the_pane_is_broken_rather_than_overflowed() {
        assert_eq!(
            wrap("supercalifragilistic", 6),
            ["superc", "alifra", "gilist", "ic"]
        );
    }

    #[test]
    fn runs_of_spaces_do_not_produce_blank_lines() {
        assert_eq!(wrap("  a    b  ", 3), ["a", "b"]);
    }

    #[test]
    fn multibyte_glyphs_count_as_one_cell_each() {
        // Each of these is one grid cell but two or three UTF-8 bytes; counting
        // bytes would wrap after two glyphs instead of five.
        assert_eq!(wrap("░▒▓░▒▓", 5), ["░▒▓░▒", "▓"]);
    }

    #[test]
    fn zero_width_yields_nothing() {
        assert!(wrap("anything", 0).is_empty());
    }

    #[test]
    fn empty_input_yields_nothing() {
        assert!(wrap("", 10).is_empty());
        assert!(wrap("   ", 10).is_empty());
    }
}
