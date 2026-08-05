//! The glyph repertoire: code page 437.
//!
//! DESIGN.md §4 fixes the font as a CP437-style bitmap at 8×16, and §13 splits
//! the two frontends: the Bevy build draws from an embedded CP437 atlas, the
//! terminal build uses the user's font and the Unicode U+2500 box-drawing block.
//!
//! The split this crate makes is between **repertoire** and **encoding**:
//!
//! - The *repertoire* — which glyphs may appear at all — belongs here, because
//!   `orbs-render` decides what appears. It is the intersection of what both
//!   frontends can draw, and a [`crate::Frame`] must never contain anything
//!   outside it.
//! - The *encoding* belongs to the frontend. A [`crate::Cell`] holds a Unicode
//!   `char`; the Bevy build turns that into an atlas index with
//!   [`cp437_index`], and the terminal build writes the `char` out directly.
//!
//! Storing the codepage index instead would have forced the terminal frontend to
//! own a reverse table, and storing arbitrary Unicode would have let prose reach
//! the atlas with no glyph behind it.

/// Code page 437 in index order, as Unicode scalars.
///
/// Index `0x00` is the null control code rather than a glyph; it is filtered out
/// by [`cp437_index`] along with every other control character, so it can never
/// be painted. The rest of `0x00..=0x1F` are the codepage's graphic
/// interpretations of the C0 range, which are ordinary drawable glyphs here.
const TABLE: [char; 256] = [
    // 0x00
    '\u{0000}', '☺', '☻', '♥', '♦', '♣', '♠', '•', '◘', '○', '◙', '♂', '♀', '♪', '♫', '☼',
    // 0x10
    '►', '◄', '↕', '‼', '¶', '§', '▬', '↨', '↑', '↓', '→', '←', '∟', '↔', '▲', '▼',
    // 0x20
    ' ', '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/',
    // 0x30
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ':', ';', '<', '=', '>', '?',
    // 0x40
    '@', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
    // 0x50
    'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '[', '\\', ']', '^', '_',
    // 0x60
    '`', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
    // 0x70
    'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '{', '|', '}', '~', '⌂',
    // 0x80
    'Ç', 'ü', 'é', 'â', 'ä', 'à', 'å', 'ç', 'ê', 'ë', 'è', 'ï', 'î', 'ì', 'Ä', 'Å',
    // 0x90
    'É', 'æ', 'Æ', 'ô', 'ö', 'ò', 'û', 'ù', 'ÿ', 'Ö', 'Ü', '¢', '£', '¥', '₧', 'ƒ',
    // 0xA0
    'á', 'í', 'ó', 'ú', 'ñ', 'Ñ', 'ª', 'º', '¿', '⌐', '¬', '½', '¼', '¡', '«', '»',
    // 0xB0
    '░', '▒', '▓', '│', '┤', '╡', '╢', '╖', '╕', '╣', '║', '╗', '╝', '╜', '╛', '┐',
    // 0xC0
    '└', '┴', '┬', '├', '─', '┼', '╞', '╟', '╚', '╔', '╩', '╦', '╠', '═', '╬', '╧',
    // 0xD0
    '╨', '╤', '╥', '╙', '╘', '╒', '╓', '╫', '╪', '┘', '┌', '█', '▄', '▌', '▐', '▀',
    // 0xE0
    'α', 'ß', 'Γ', 'π', 'Σ', 'σ', 'µ', 'τ', 'Φ', 'Θ', 'Ω', 'δ', '∞', 'φ', 'ε', '∩',
    // 0xF0
    '≡', '±', '≥', '≤', '⌠', '⌡', '÷', '≈', '°', '∙', '·', '√', 'ⁿ', '²', '■', '\u{00A0}',
];

/// The glyph a frontend draws in place of one outside the repertoire.
///
/// Reaching this is an authoring bug — every string the game paints comes from
/// prose data files we control — so it is deliberately conspicuous rather than
/// silently blank.
pub const REPLACEMENT: char = '?';

/// The code page index for `glyph`, or `None` if it is outside the repertoire.
///
/// Control characters are always rejected, including the ones the codepage
/// assigns graphics to. `'\n'` is CP437 `0x0A`, a filled circle; painting it
/// literally would put a glyph in the grid where the caller meant a line break,
/// so line breaking stays the caller's job and this returns `None`.
#[must_use]
pub fn cp437_index(glyph: char) -> Option<u8> {
    if glyph.is_control() {
        return None;
    }

    // ASCII graphics and space are identity-mapped and are the overwhelming
    // majority of what the game draws; the scan below is the cold path.
    if glyph == ' ' || glyph.is_ascii_graphic() {
        return u8::try_from(u32::from(glyph)).ok();
    }

    let index = TABLE.iter().position(|&candidate| candidate == glyph)?;
    u8::try_from(index).ok()
}

/// Whether `glyph` can appear in a [`crate::Frame`].
#[must_use]
pub fn is_renderable(glyph: char) -> bool {
    cp437_index(glyph).is_some()
}

/// The first character of `line` that cannot be drawn, with its byte offset.
///
/// For linting authored prose. DESIGN.md §13 puts every string the game shows
/// into hot-reloadable data files, and a writer reaching for the typographic
/// characters a word processor produces — `—`, `’`, `“` — will silently get
/// [`REPLACEMENT`] on screen. This is what a content pipeline calls to turn that
/// into an error at load time instead of a `?` mid-siege.
///
/// Pass **one line at a time**: `'\n'` is not a drawable glyph and is reported
/// like any other, because line breaking belongs to the caller.
#[must_use]
pub fn first_unrenderable(line: &str) -> Option<(usize, char)> {
    line.char_indices()
        .find(|&(_, glyph)| !is_renderable(glyph))
}

/// The glyph at a code page index.
///
/// The inverse of [`cp437_index`], for a frontend reading a legacy asset.
#[must_use]
pub fn cp437_glyph(index: u8) -> char {
    TABLE[usize::from(index)]
}

/// Box-drawing glyphs, named so callers never hand-type a codepoint.
///
/// These are the U+2500 block members that CP437 also carries, which is what
/// makes a single set serve both frontends.
pub mod box_drawing {
    /// `┌`
    pub const TOP_LEFT: char = '┌';
    /// `┐`
    pub const TOP_RIGHT: char = '┐';
    /// `└`
    pub const BOTTOM_LEFT: char = '└';
    /// `┘`
    pub const BOTTOM_RIGHT: char = '┘';
    /// `─`
    pub const HORIZONTAL: char = '─';
    /// `│`
    pub const VERTICAL: char = '│';
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_codepage_slot_is_distinct() {
        // The table is hand-entered; a duplicate would mean a wrong glyph and a
        // reverse lookup that silently resolves to the wrong atlas index.
        let mut sorted = TABLE;
        sorted.sort_unstable();
        for pair in sorted.windows(2) {
            assert_ne!(pair[0], pair[1], "duplicate glyph in the CP437 table");
        }
    }

    #[test]
    fn ascii_is_identity_mapped() {
        for byte in 0x20..=0x7Eu8 {
            let glyph = char::from(byte);
            assert_eq!(cp437_index(glyph), Some(byte), "ASCII {glyph:?} moved");
            assert_eq!(cp437_glyph(byte), glyph);
        }
    }

    #[test]
    fn the_fast_path_agrees_with_the_table() {
        for (index, &glyph) in TABLE.iter().enumerate() {
            if glyph.is_control() {
                continue;
            }
            let expected = u8::try_from(index).ok();
            assert_eq!(cp437_index(glyph), expected, "index {index:#04x} disagrees");
        }
    }

    #[test]
    fn control_characters_are_not_renderable() {
        // '\n' is a filled circle in CP437; painting it would be silently wrong.
        for glyph in ['\0', '\n', '\r', '\t', '\u{7f}'] {
            assert!(!is_renderable(glyph), "{glyph:?} should be rejected");
        }
    }

    #[test]
    fn box_drawing_is_inside_the_repertoire() {
        use box_drawing::{BOTTOM_LEFT, BOTTOM_RIGHT, HORIZONTAL, TOP_LEFT, TOP_RIGHT, VERTICAL};
        for glyph in [
            TOP_LEFT,
            TOP_RIGHT,
            BOTTOM_LEFT,
            BOTTOM_RIGHT,
            HORIZONTAL,
            VERTICAL,
        ] {
            assert!(is_renderable(glyph), "{glyph:?} is not in the codepage");
        }
    }

    #[test]
    fn characters_outside_the_repertoire_are_rejected() {
        // No glyph behind these in an 8x16 CP437 atlas, and the wide ones would
        // break the one-char-one-cell assumption the whole grid rests on.
        for glyph in ['漢', '😀', 'Ж', '€'] {
            assert!(!is_renderable(glyph), "{glyph:?} should be rejected");
        }
    }

    #[test]
    fn the_replacement_glyph_is_itself_renderable() {
        assert!(is_renderable(REPLACEMENT));
    }

    #[test]
    fn the_prose_lint_catches_typographic_punctuation() {
        // The exact failure this exists to prevent: DESIGN.md §4's own boot
        // header uses an em-dash, which has no CP437 glyph and would render as
        // `?` on the title screen.
        assert_eq!(
            first_unrenderable("O.R.B.S. v0.9.3  —  cold start"),
            Some((17, '—'))
        );
        // Smart quotes and ellipses are the same trap from a word processor.
        assert!(first_unrenderable("the wizard’s orb").is_some());
        assert!(first_unrenderable("waiting…").is_some());
    }

    #[test]
    fn the_prose_lint_passes_text_the_font_can_draw() {
        for line in [
            "O.R.B.S. v0.9.3  --  cold start",
            "east_wall integrity 34% [ DEGRADED ]",
            "┌ laboratory ─────┐",
            "",
        ] {
            assert_eq!(first_unrenderable(line), None, "{line:?} was rejected");
        }
    }
}
