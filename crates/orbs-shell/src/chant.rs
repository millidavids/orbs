//! The figure travelling toward the circle, beside the transcript
//! (DESIGN.md §10, `tower::chant`).
//!
//! The sanctum's board, one room over, and it follows the same three rules.
//! **It is not gated on a word**: it draws whenever a chant is running, which is
//! what makes a bound solver watchable — watching and doing are different
//! activities and only one of them owns the keyboard.
//!
//! **Columns, never rows**, and it splits *after* the instrument panel. §19
//! records getting that order backwards once.
//!
//! It **refuses rather than truncating**, and here the rule is at its strongest
//! of the three boards. A clipped maze is a smaller view of the same maze and a
//! clipped sheet loses old presses; a figure missing its lower rows is a figure
//! whose *next* syllable is off screen, which is worse than no board at all —
//! the player would be answering a question they cannot see while believing they
//! can. So it is refused in both directions.

use orbs_render::{Figure, Painter, Pos, Rect, Role, Style, UtteranceKind, Wash};
use orbs_sim::content::Prose;

/// The gap between the board and the transcript.
const GUTTER: u16 = 1;

/// The fewest columns the transcript keeps before the board yields.
///
/// The same figure the map, the sheet and the sanctum's board use.
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
/// `const` because it can be: the board is a fixed size whatever is travelling
/// on it, so the layout never has to look inside the figure it was handed. That
/// is the property `Figure::rows` exists for.
pub const fn split(area: Rect, figure: Option<&Figure>) -> Split {
    let nothing = Split {
        area: Rect::EMPTY,
        rest: area,
    };
    if figure.is_none() {
        return nothing;
    }

    let (cols, rows) = Figure::size();
    let (block, tall) = (cols.saturating_add(2), rows.saturating_add(2));

    if block.saturating_add(GUTTER + TRANSCRIPT_FLOOR) > area.cols || tall > area.rows {
        return nothing;
    }

    let wanted = block.saturating_add(GUTTER);
    Split {
        area: Rect::new(area.col + area.cols - block, area.row, block, tall),
        rest: Rect::new(area.col, area.row, area.cols - wanted, area.rows),
    }
}

/// Draw the board where [`split`] put it.
pub fn paint(painter: &mut Painter<'_>, at: Rect, figure: &Figure, prose: &Prose) {
    if at.is_empty() {
        return;
    }
    painter.border(at, Some(&prose.line("chant_title", &[])), Style::DIM);

    let inside = at.inset(1);
    for row in 0..inside.rows {
        let Some(cells) = figure.row(usize::from(row)) else {
            break;
        };
        for (col, (glyph, style, tint)) in cells.into_iter().enumerate() {
            let Ok(col) = u16::try_from(col) else { break };
            if col >= inside.cols {
                break;
            }
            let at = Pos::new(inside.col + col, inside.row + row);
            // **`glyphs`, so the picture is silent**, and the summary below says
            // what it means. A reader hearing forty cells of arrow and rule read
            // out gets noise, which is what §19's frame rule exists to prevent.
            painter.glyphs(at, &glyph.to_string(), style);
            if let Some(tint) = tint {
                painter.tint(Rect::new(at.col, at.row, 1, 1), Wash::plain(tint));
            }
        }
    }

    speak(painter, figure, prose);
}

/// What the board says, for a reader.
///
/// **Spoken once as a summary, never cell by cell**, which is every board's
/// rule. What a reader needs from a figure is *what is next* — the one thing
/// they must act on — and how it is going; the rest is on the transcript,
/// already said as sentences.
///
/// **`next` is the whole point of this line.** A sighted player reads the row
/// resting on the rule; a reader has no rows, so if this said only a tally the
/// domain would be unplayable by ear. That is why it names the syllable first.
fn speak(painter: &mut Painter<'_>, figure: &Figure, prose: &Prose) {
    let next = figure
        .coming
        .first()
        .and_then(|lane| figure.lanes.get(*lane))
        .map_or("", |(_, name)| *name);
    painter.announce(
        UtteranceKind::Progress,
        Role::Normal,
        &prose.line(
            "chant_spoken",
            &[
                ("name", next),
                // **The tally's number, not `coming.len()`.** `coming` is capped
                // at `AHEAD` for the picture, so a reader was told *"8 to come"*
                // where the board drew *"12 to come"* — the two disagreed for
                // the first five syllables of every chant, and §14's route got
                // the false one. The sentence under the rule is already written
                // from `remaining()`, so this reads it rather than counting a
                // second time.
                ("quantity", &figure.remaining.to_string()),
                (
                    "detail",
                    &figure
                        .sung
                        .iter()
                        .filter(|struck| !**struck)
                        .count()
                        .to_string(),
                ),
            ],
        ),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn figure() -> Figure {
        Figure {
            coming: vec![1, 3, 0],
            lanes: vec![
                ('\u{25C4}', "leftward"),
                ('\u{25B2}', "skyward"),
                ('\u{25BC}', "earthward"),
                ('\u{25BA}', "rightward"),
            ],
            sung: vec![true, false],
            until: 0,
            remaining: 3,
            tally: "3 to come".to_owned(),
        }
    }

    #[test]
    fn no_figure_leaves_the_whole_pane_to_the_transcript() {
        let area = Rect::new(0, 0, 120, 40);
        let split = split(area, None);
        assert!(split.area.is_empty());
        assert_eq!(split.rest, area);
    }

    #[test]
    fn a_board_takes_columns_and_leaves_the_transcript_a_run_of_them() {
        let area = Rect::new(0, 0, 120, 40);
        let split = split(area, Some(&figure()));
        assert!(!split.area.is_empty());
        assert_eq!(split.area.rows, Figure::rows() + 2);
        assert_eq!(split.rest.rows, area.rows, "it took rows");
        assert!(split.rest.cols >= TRANSCRIPT_FLOOR);
        assert_eq!(
            split.area.col + split.area.cols,
            area.col + area.cols,
            "the board is not against the edge the panel took",
        );
    }

    /// **Refused whole, in both directions.** A figure whose next syllable is
    /// off screen is worse than no board: the player answers a question they
    /// cannot see while believing they can.
    #[test]
    fn a_board_that_will_not_fit_is_not_drawn_at_all() {
        let figure = figure();
        for area in [
            Rect::new(0, 0, 60, 40), // too narrow for board, gutter and floor
            Rect::new(0, 0, 120, 8), // too short for the lanes
        ] {
            let split = split(area, Some(&figure));
            assert!(
                split.area.is_empty(),
                "{area:?} drew a board it could not hold",
            );
            assert_eq!(split.rest, area, "{area:?} yielded columns for nothing");
        }
    }

    /// **It fits the 80×22 authoring floor**, which the maze and the ward sheet
    /// do not — the sanctum's board is the other one that does.
    ///
    /// This test asserted the opposite for one commit, on an assumption rather
    /// than an arithmetic: 42 columns of board plus a gutter and the 24-column
    /// transcript floor is 67, and eleven rows plus a border is thirteen. Both
    /// fit twice over. **The domain is unplayable by hand without a board**, so
    /// a figure that yielded at the floor would make the floor unplayable — which
    /// is a much better reason to fit than tidiness.
    #[test]
    fn the_board_survives_the_authoring_floor() {
        let split = split(Rect::new(0, 0, 80, 22), Some(&figure()));
        assert!(!split.area.is_empty(), "the floor lost its board");
        assert!(split.rest.cols >= TRANSCRIPT_FLOOR);
    }
}
