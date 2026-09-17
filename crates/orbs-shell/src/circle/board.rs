//! The circle's board: where it sits and how it is drawn.
//!
//! The sanctum's board, one room over, and it follows the same three rules.
//! **It is not gated on a word**: it draws whenever a beast waits, which is what
//! makes a bound solver watchable. There is no full-pane mode, because there is
//! nothing to walk: a circle is limned from the prompt with `limn` and `summon`.
//!
//! **Columns, never rows**, and it splits *after* the instrument panel.
//!
//! It **refuses rather than truncating** — a truth table missing a row is a
//! question with part of it torn off, and a player would answer it believing it
//! whole. See [`beside`](crate::beside).

use orbs_render::{Circle, Painter, Pos, Rect, Style, Wash};
use orbs_sim::content::Prose;

use crate::beside::{self, Split};

/// The board's footprint, and what is left for the transcript.
pub(crate) const fn split(area: Rect, circle: Option<&Circle>) -> Split {
    beside::split(
        area,
        match circle {
            Some(_) => Some(beside::bordered(Circle::size())),
            None => None,
        },
    )
}

/// Draw the board where [`split`] put it.
pub(crate) fn paint(painter: &mut Painter<'_>, at: Rect, circle: &Circle, prose: &Prose) {
    if at.is_empty() {
        return;
    }
    painter.border(at, Some(&prose.line("circle_title", &[])), Style::DIM);

    let inside = at.inset(1);
    for row in 0..inside.rows {
        let Some(cells) = circle.row(usize::from(row)) else {
            break;
        };
        for (col, (glyph, style, tint)) in cells.into_iter().enumerate() {
            let Ok(col) = u16::try_from(col) else { break };
            if col >= inside.cols {
                break;
            }
            let at = Pos::new(inside.col + col, inside.row + row);
            // **`glyphs`, so the picture is silent**, and the summary says what
            // it means. A reader hearing forty-four cells a row of suns and dots
            // gets noise, which is the pylon's rule and the lattice's.
            painter.glyphs(at, &glyph.to_string(), style);
            if let Some(tint) = tint {
                painter.tint(Rect::new(at.col, at.row, 1, 1), Wash::plain(tint));
            }
        }
    }

    super::speech::speak(painter, circle, prose);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::beside::TRANSCRIPT_FLOOR;

    fn circle() -> Circle {
        Circle {
            senses: vec!["blood".into(), "bone".into(), "breath".into()],
            temper: vec![false, true, true, false, false, true, true, true],
            labels: ["temper".into(), "answer".into()],
            tally: "not yet called in".into(),
            ..Circle::default()
        }
    }

    #[test]
    fn no_beast_leaves_the_whole_pane_to_the_transcript() {
        let area = Rect::new(0, 0, 104, 43);
        let split = split(area, None);
        assert!(split.area.is_empty());
        assert_eq!(split.rest, area);
    }

    /// **It fits the 80×22 authoring floor**, as the pylon's board does and the
    /// maze and the sheet do not. The domain is played by reading the table, so a
    /// board that yielded at the floor would make the floor unplayable.
    #[test]
    fn the_board_still_fits_the_authoring_floor() {
        let split = split(Rect::new(0, 0, 78, 20), Some(&circle()));
        assert!(!split.area.is_empty(), "the floor lost its board");
        assert!(split.rest.cols >= TRANSCRIPT_FLOOR);
        let (cols, rows) = beside::bordered(Circle::size());
        assert_eq!((split.area.cols, split.area.rows), (cols, rows));
    }

    #[test]
    fn a_pane_too_short_is_refused_rather_than_clipped() {
        let (_, tall) = beside::bordered(Circle::size());
        assert!(
            split(Rect::new(0, 0, 104, tall - 1), Some(&circle()))
                .area
                .is_empty()
        );
    }
}
