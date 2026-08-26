//! The course of wards, beside the transcript (DESIGN.md §10, `tower::pylon`).
//!
//! The ward sheet's shape, one room over, and it follows the same three rules.
//! **It is not gated on a word**: it draws whenever a course is standing, which
//! is what makes a bound solver watchable — watching and doing are different
//! activities and only one of them owns the keyboard. There is no full-pane mode,
//! because unlike a maze there is nothing to walk: a course is played from the
//! prompt with `muster` and `haul`.
//!
//! **Columns, never rows**, and it splits *after* the instrument panel. §19
//! records getting that order backwards once.
//!
//! It **refuses rather than truncating**, and here that rule is at its
//! strongest. A clipped maze is a smaller view of the same maze and a clipped
//! sheet loses old presses, but half a Hanoi position is not a position at all —
//! a board missing its bottom row shows wards resting on nothing, which is a
//! picture of an illegal state. So this one is refused in **both** directions,
//! where the sheet windows its rows.

use orbs_render::{Painter, Pos, Pylon, Rect, Role, Style, UtteranceKind, Wash};
use orbs_sim::content::Prose;

/// The gap between the board and the transcript.
const GUTTER: u16 = 1;

/// The fewest columns the transcript keeps before the board yields.
///
/// The same figure the map and the sheet use, and for the same reason: below it
/// the board has won an argument it should lose.
const TRANSCRIPT_FLOOR: u16 = 24;

/// Where the board sits, and what is left for the transcript.
#[derive(Debug, Clone, Copy)]
pub struct Split {
    /// The board's own rectangle, border included. Empty when there is none.
    pub area: Rect,
    /// What the transcript gets.
    pub rest: Rect,
}

/// The board's footprint, and what is left for the transcript.
///
/// `const` because it can be: unlike the sheet's, this board is a fixed size
/// whatever is standing on it, so the layout never has to look inside the course
/// it was handed. That is the same property `Pylon::rows` exists for.
pub const fn split(area: Rect, course: Option<&Pylon>) -> Split {
    let nothing = Split {
        area: Rect::EMPTY,
        rest: area,
    };
    if course.is_none() {
        return nothing;
    }

    // Border on both sides, in both directions.
    let (cols, rows) = Pylon::size();
    let (block, tall) = (cols.saturating_add(2), rows.saturating_add(2));

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

/// Draw the board where [`split`] put it.
pub fn paint(painter: &mut Painter<'_>, at: Rect, course: &Pylon, prose: &Prose) {
    if at.is_empty() {
        return;
    }
    painter.border(at, Some(&prose.line("pylon_title", &[])), Style::DIM);

    let inside = at.inset(1);
    for row in 0..inside.rows {
        let Some(cells) = course.row(usize::from(row)) else {
            break;
        };
        for (col, (glyph, style, tint)) in cells.into_iter().enumerate() {
            let Ok(col) = u16::try_from(col) else { break };
            if col >= inside.cols {
                break;
            }
            let at = Pos::new(inside.col + col, inside.row + row);
            // **`glyphs`, so the picture is silent**, and the summary below says
            // what it means. A reader hearing twenty-seven cells of block and
            // rule read out gets box-drawing noise, which is what §19's frame
            // rule puts structure on one channel and content on the other to
            // prevent.
            painter.glyphs(at, &glyph.to_string(), style);
            if let Some(tint) = tint {
                painter.tint(Rect::new(at.col, at.row, 1, 1), Wash::plain(tint));
            }
        }
    }

    speak(painter, course, prose);
}

/// What the board says, for a reader.
///
/// **Spoken once as a summary, never cell by cell**, which is the sheet's rule.
/// What a reader needs from a Hanoi position is how much is home and what it has
/// cost — the per-haul detail is already in the transcript, where it was said as
/// a sentence.
///
/// **A longer line than the board's, and deliberately.** The foot of the picture
/// is 33 cells and has the rail two columns away saying how the barrier stands; a
/// reader has neither constraint nor that glance, so `pylon_spoken` says the
/// whole thing where `pylon_tally` says what fits. Both are authored, which is
/// what rule 6 asks — the painter composes prose, it does not invent it.
fn speak(painter: &mut Painter<'_>, course: &Pylon, prose: &Prose) {
    painter.announce(
        UtteranceKind::Progress,
        Role::Normal,
        &prose.line(
            "pylon_spoken",
            &[
                ("quantity", &course.height.to_string()),
                ("name", &course.hauls.to_string()),
                ("detail", &course.integrity.to_string()),
            ],
        ),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A course to lay out. The contents do not matter — the board is a fixed
    /// size whatever is standing on it, which is the point of `Pylon::rows`.
    fn course() -> Pylon {
        Pylon {
            stations: [vec![3, 2, 1], Vec::new(), Vec::new()],
            names: ["wellspring", "conduit", "barrier"],
            height: 3,
            hauls: 0,
            integrity: 100,
            tally: "3 wards, 0 hauled".to_owned(),
        }
    }

    /// The board's footprint including its border.
    fn block() -> (u16, u16) {
        let (cols, rows) = Pylon::size();
        (cols + 2, rows + 2)
    }

    #[test]
    fn no_course_means_no_board_and_the_transcript_keeps_everything() {
        let area = Rect::new(0, 0, 104, 43);
        let split = split(area, None);
        assert!(split.area.is_empty());
        assert_eq!(split.rest, area);
    }

    #[test]
    fn the_board_fits_the_grid_the_see_it_lines_use() {
        // The game's own session pane: 120 columns less the rail's 16, less two
        // for the border. If this ever stops fitting, every See-it line in
        // CLAUDE.md's sanctum block silently stops showing a board.
        let split = split(Rect::new(0, 0, 104, 43), Some(&course()));
        assert!(
            !split.area.is_empty(),
            "the board does not fit the grid the game actually runs at",
        );
        let (block, tall) = block();
        assert_eq!(split.area.cols, block);
        assert_eq!(split.area.rows, tall);
    }

    #[test]
    fn a_pane_too_narrow_is_refused_whole_and_the_transcript_keeps_the_pane() {
        // One column short of what the board and the transcript floor need
        // together. Refused, and the transcript is not narrowed by a board that
        // is not there.
        let (block, _) = block();
        let area = Rect::new(0, 0, block + GUTTER + TRANSCRIPT_FLOOR - 1, 43);
        let split = split(area, Some(&course()));
        assert!(split.area.is_empty());
        assert_eq!(split.rest, area);
    }

    #[test]
    fn a_pane_too_short_is_refused_rather_than_clipped() {
        // **The rule this board holds harder than its siblings.** A sheet windows
        // its rows and a maze pans; a Hanoi position clipped at the bottom draws
        // wards resting on nothing, which is a picture of an illegal state.
        let (_, tall) = block();
        let area = Rect::new(0, 0, 104, tall - 1);
        assert!(split(area, Some(&course())).area.is_empty());
        assert!(
            !split(Rect::new(0, 0, 104, tall), Some(&course()))
                .area
                .is_empty()
        );
    }

    #[test]
    fn the_board_still_fits_the_authoring_floor() {
        // **Unlike the maze and the sheet**, and worth pinning because it was a
        // surprise. Both of those yield at 80x22 — a maze is 33 squares across
        // and a twelve-press sheet is 41 columns and fourteen rows. A course is
        // three stations and seven storeys, which is 35 by 12 with its border,
        // and still fits beside a 24-column transcript on the narrowest grid the
        // game supports.
        //
        // So the sanctum is the one domain whose picture is never absent, and a
        // See-it line at the floor shows a board rather than a refusal.
        assert!(
            !split(Rect::new(0, 0, 78, 20), Some(&course()))
                .area
                .is_empty(),
            "the board yields at the authoring floor, where it used to fit",
        );
    }

    #[test]
    fn the_transcript_and_the_board_never_overlap() {
        let area = Rect::new(0, 0, 104, 43);
        let split = split(area, Some(&course()));
        assert!(!split.area.is_empty());
        assert_eq!(
            split.rest.col + split.rest.cols + GUTTER,
            split.area.col,
            "the gutter between them is not exactly one column",
        );
        assert_eq!(
            split.area.col + split.area.cols,
            area.col + area.cols,
            "the board is not flush with the edge the panel took",
        );
    }

    #[test]
    fn the_board_is_the_same_size_whatever_is_standing_on_it() {
        // A board that shrank with the stack would move the ground line every
        // haul, and the ground line is what a player reads position against.
        let area = Rect::new(0, 0, 104, 43);
        let short = split(area, Some(&course())).area;
        let tall = split(
            area,
            Some(&Pylon {
                stations: [(1..=7).rev().collect(), Vec::new(), Vec::new()],
                names: ["wellspring", "conduit", "barrier"],
                height: 7,
                hauls: 40,
                integrity: 12,
                tally: "7 wards, 40 hauled".to_owned(),
            }),
        )
        .area;
        assert_eq!(short, tall);
    }
}
