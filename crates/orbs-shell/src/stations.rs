//! One vocabulary for a station on a line, wherever a line is drawn.
//!
//! The weave draws the Ley Line and the seven mastery lines; a room's pane
//! draws its own line as a road under the title. Three surfaces, one set of
//! glyphs — `[•]` for a thing had, `[○]` for the thing in reach, `[·]` for
//! what lies beyond — because a player who has learned the marks on one screen
//! must find the same marks meaning the same things on the next.
//!
//! **All of these are in CP437 and were checked**, which is not a formality:
//! `●` (U+25CF) is *not* in the table, and the renderer skips what it cannot
//! draw — so "taken" would have rendered as nothing at all, collapsing the one
//! distinction §14 says must not be carried by colour alone. `•` is 0x07, `○`
//! is 0x09, `·` is 0xFA, and `─` is 0xC4.

use orbs_render::{Intensity, Painter, Pos, Role, Style};
use orbs_sim::{Standing, Walk};

/// Had: a step passed, a node chosen, a station reached.
pub const HAD: char = '\u{2022}';
/// In reach: an open fork node, or the station a room is working toward.
pub const NEXT: char = '\u{25cb}';
/// Beyond: locked, or further along the line.
pub const LATER: char = '\u{b7}';
/// The line itself.
pub const RUN: char = '\u{2500}';

/// The frame around a station sitting on a line, and the frame around the
/// aimed one.
///
/// **Two pairs, because brightness could not do it.** An aimed `○` drawn Bright
/// is identical to the sibling beside it — `NEXT` is already Bright — and §14
/// forbids the difference being colour. So the aimed station changes its
/// *cells*. `»` is CP437 0xAF, which the editor already uses to mark the line an
/// invocation has reached; `«` is 0xAE beside it.
pub const FRAME: (char, char) = ('[', ']');
pub const AIMED: (char, char) = ('\u{ab}', '\u{bb}');

/// The mark and style for a fork node or a step.
#[must_use]
pub fn standing_mark(standing: Standing) -> (char, Style) {
    match standing {
        Standing::Taken => (HAD, Style::default().with_role(Role::Success)),
        Standing::Open => (NEXT, Style::default().with_intensity(Intensity::Bright)),
        Standing::Locked => (LATER, Style::DIM),
    }
}

/// The mark and style for a station on a room's line.
#[must_use]
pub fn walk_mark(walk: Walk) -> (char, Style) {
    match walk {
        Walk::Reached => (HAD, Style::default().with_role(Role::Success)),
        Walk::Next => (NEXT, Style::default().with_intensity(Intensity::Bright)),
        Walk::Later => (LATER, Style::DIM),
    }
}

/// A mark in its frame — `[○]`, or `«○»` and Bright when aimed at.
///
/// **A frame either side, always** — a station standing on a line needs to
/// read as a station rather than as a break in it. The frame overwrites one
/// cell of the run on each side, which is why a run is drawn before its
/// stations.
///
/// **Bright as well as framed, when aimed.** The frame is what survives
/// greyscale; the brightness is what the eye finds first. Two carriers for one
/// fact is what §14 asks for — neither is doing it alone.
///
/// **Silent.** `Painter::span` would push the glyph's own text into the speech
/// stream, so a reader would hear "`○`" and be told nothing; the caller owes the
/// listener an utterance in words, which is the division `Painter::meter`
/// makes.
pub fn framed(painter: &mut Painter<'_>, x: u16, y: u16, mark: char, style: Style, aimed: bool) {
    let (open, close) = if aimed { AIMED } else { FRAME };
    let frame = if aimed {
        Style::default().with_intensity(Intensity::Bright)
    } else {
        Style::DIM
    };
    painter.glyphs(Pos::new(x.saturating_sub(1), y), &open.to_string(), frame);
    painter.glyphs(Pos::new(x.saturating_add(1), y), &close.to_string(), frame);
    painter.glyphs(
        Pos::new(x, y),
        &mark.to_string(),
        if aimed {
            style.with_intensity(Intensity::Bright)
        } else {
            style
        },
    );
}

/// A mark with no frame of its own — for a line with more stations on it than
/// the pane can frame.
///
/// **The aimed one keeps its frame.** The frame is what carries "aimed" without
/// colour (§14), so dropping it would leave brightness doing the job alone; and
/// at the tight gap the two frame cells land on the run between marks rather
/// than on a neighbour. Everything else about it is [`framed`]'s: silent, and a
/// run drawn first.
pub fn bare(painter: &mut Painter<'_>, x: u16, y: u16, mark: char, style: Style, aimed: bool) {
    if aimed {
        framed(painter, x, y, mark, style, true);
        return;
    }
    painter.glyphs(Pos::new(x, y), &mark.to_string(), style);
}

/// A run of the line, `width` cells long. Silent: the road is a line, not a
/// fact, and every fact on it is a station.
pub fn run(painter: &mut Painter<'_>, at: Pos, width: u16) {
    painter.glyphs(at, &RUN.to_string().repeat(usize::from(width)), Style::DIM);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_glyph_is_one_cp437_can_draw() {
        // The check the module header describes, as a test: a mark the renderer
        // skips is a state that draws as nothing.
        for glyph in [HAD, NEXT, LATER, RUN, FRAME.0, FRAME.1, AIMED.0, AIMED.1] {
            assert!(
                orbs_render::cp437::is_renderable(glyph),
                "{glyph:?} is not in CP437",
            );
        }
    }

    #[test]
    fn had_next_and_later_are_three_different_marks() {
        // §14: the state is carried by the glyph for anyone who can see it, so
        // two states sharing a glyph would be one state to a colour-blind eye.
        let marks: std::collections::BTreeSet<char> = [
            walk_mark(Walk::Reached).0,
            walk_mark(Walk::Next).0,
            walk_mark(Walk::Later).0,
        ]
        .into_iter()
        .collect();
        assert_eq!(marks.len(), 3);
        assert_eq!(standing_mark(Standing::Taken).0, walk_mark(Walk::Reached).0);
        assert_eq!(standing_mark(Standing::Open).0, walk_mark(Walk::Next).0);
        assert_eq!(standing_mark(Standing::Locked).0, walk_mark(Walk::Later).0);
    }
}
