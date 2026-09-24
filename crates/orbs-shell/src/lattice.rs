//! The forge's board, beside the transcript.
//!
//! It draws whenever a lattice is open, not when a word is typed — the map's
//! rule, the sheet's, the pylon's and the rampart's. That is what makes a bound
//! solver watchable.
//!
//! Columns, never rows: taking rows under a `Top` panel leaves the deep-focus
//! floor a five-row transcript (§19). And it refuses whole rather than
//! truncating — a lattice half off the edge is a puzzle a player would misread.

use orbs_render::lattice::{GLOW, Lattice};
use orbs_render::{Painter, Pos, Rect, Role, Style, UtteranceKind, Wash};

use orbs_sim::content::Prose;

use crate::beside::{self, Split};

/// Take the board's columns off `area`, if there is a lattice and room for it.
///
/// `Lattice::cols` and `rows` already count the border, so nothing is added.
#[must_use]
pub fn split(area: Rect, lattice: Option<&Lattice>) -> Split {
    beside::split(area, lattice.map(|open| (open.cols(), open.rows())))
}

/// Draw it.
pub fn paint(painter: &mut Painter<'_>, at: Rect, lattice: &Lattice, prose: &Prose) {
    if at.is_empty() {
        return;
    }
    painter.border(at, Some(&lattice.title), Style::DIM);

    let inside = at.inset(1);
    let mut row = inside.row;

    painter.glyphs(Pos::new(inside.col, row), &lattice.heading(), Style::DIM);
    row += 1;

    let high = lattice.glyphs.len() / lattice.width.max(1);
    for line in 0..high {
        painter.glyphs(Pos::new(inside.col, row), &lattice.row(line), Style::NORMAL);
        row += 1;
    }
    // The tint is enrichment and the glyph carries the identity (§14). One hue
    // over the whole grid, so taking colour away leaves the puzzle exactly as
    // readable — which it has to be, since *which glyphs are lit* is the puzzle.
    painter.tint(
        Rect::new(
            inside.col,
            inside.row + 1,
            inside.cols,
            u16::try_from(high).unwrap_or(0),
        ),
        Wash::plain(GLOW),
    );

    // The rule, then what the last fall left. Under a rule of its own, because
    // the residue is what the grid *did* rather than part of it — read as one
    // picture it would be nine glyphs where there are twelve.
    painter.glyphs(
        Pos::new(inside.col, row),
        &"─".repeat(usize::from(inside.cols)),
        Style::DIM,
    );
    row += 1;
    painter.glyphs(
        Pos::new(inside.col, row),
        &lattice.residue_row(),
        Style::NORMAL,
    );
    row += 1;
    painter.glyphs(Pos::new(inside.col, row), &lattice.tally, Style::DIM);

    // One utterance, not twelve — §14's *"progress announcements: completion
    // only"*. The residue is the sentence worth saying, because it is the whole
    // of what a decision here turns on.
    let spoken = prose.line(
        "lattice_spoken",
        // Words, not the drawn strip: `residue_row()` is padded glyph art, and a
        // reader would hear `☼`, `·` and twenty-four spaces.
        &[
            ("name", &lattice.tally),
            ("detail", &lattice.residue_spoken()),
        ],
    );
    painter.announce(UtteranceKind::Progress, Role::Normal, &spoken);
}
