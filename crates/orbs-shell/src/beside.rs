//! A fixed-size board beside the transcript — the one layout four rooms share.
//!
//! The pylon, the rampart, the lattice and the menagerie's board each carried
//! this function, character for character, with only the board's size between
//! them. **Four copies of one rule is how two of them come to disagree**, which
//! §19 records more often than anything else, so the rule lives here and each
//! board says only how big it is.
//!
//! **Columns, never rows**, and every caller splits *after* the instrument
//! panel — an instrument row is load-bearing where a board is a convenience, and
//! whichever split runs second is the one whose refusal can fire. §19 records
//! getting that order backwards once.
//!
//! **Refused whole rather than truncated**, in both directions. A board that is
//! a fixed size whatever is on it has no smaller view of itself to fall back to:
//! half a Hanoi position shows wards resting on nothing, and half a truth table
//! is a question with rows missing. The sheet and the map window their rows
//! instead, and keep their own `split` for that reason — but they share the two
//! figures below, because the transcript they leave is the same transcript.

use orbs_render::Rect;

/// The gap between a board and the transcript.
pub(crate) const GUTTER: u16 = 1;

/// The fewest columns the transcript keeps before a board yields.
///
/// Below it the board has won an argument it should lose: a transcript squeezed
/// to a couple of words a line is not a transcript, and the board is the newer
/// thing and therefore the one that yields.
pub(crate) const TRANSCRIPT_FLOOR: u16 = 24;

/// Where a board sits, and what is left for the transcript.
///
/// **`pub(crate)`, as everything in this module is** — CLAUDE.md's visibility
/// rule: it is shared by the four boards inside this crate and by nothing
/// outside it.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Split {
    /// The board's own rectangle, border included. Empty when there is none.
    pub(crate) area: Rect,
    /// What the transcript gets.
    pub(crate) rest: Rect,
}

/// Take a board's footprint off the right of `area`, if there is a board and room.
///
/// `footprint` is the board's size **with its border**, or `None` when nothing
/// is open — which leaves the transcript the whole of `area`.
///
/// `const` because it can be: the board is a fixed size whatever is standing on
/// it, so the layout never has to look inside what it was handed.
pub(crate) const fn split(area: Rect, footprint: Option<(u16, u16)>) -> Split {
    let nothing = Split {
        area: Rect::EMPTY,
        rest: area,
    };
    let Some((block, tall)) = footprint else {
        return nothing;
    };

    if block.saturating_add(GUTTER + TRANSCRIPT_FLOOR) > area.cols || tall > area.rows {
        return nothing;
    }

    // Anchored to the edge the instrument panel took, so the two sit together
    // and the transcript keeps one uninterrupted run of columns.
    let wanted = block.saturating_add(GUTTER);
    Split {
        area: Rect::new(area.col + area.cols - block, area.row, block, tall),
        rest: Rect::new(area.col, area.row, area.cols - wanted, area.rows),
    }
}

/// A board's inner size with its border added, both ways.
#[must_use]
pub(crate) const fn bordered((cols, rows): (u16, u16)) -> (u16, u16) {
    (cols.saturating_add(2), rows.saturating_add(2))
}
