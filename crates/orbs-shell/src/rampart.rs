//! The siege, beside the transcript (DESIGN.md §5.1, `tower::siege`).
//!
//! The sanctum board's shape, one room over, and it follows the same three
//! rules. **It is not gated on a word**: it draws whenever a siege is running,
//! which is what makes a bound decision tree watchable — watching and fighting
//! are different activities and only one of them owns the keyboard. There is no
//! full-pane mode, because unlike a maze there is nothing to walk: a siege is
//! fought from the prompt with `deploy`, `quaff` and `hold`.
//!
//! **Columns, never rows**, and it splits *after* the instrument panel. §19
//! records getting that order backwards once.
//!
//! It **refuses rather than truncating**. Half a siege board is worse than none:
//! a picture missing one of its two bands shows a comparison with one side of it
//! absent, and the whole domain is that comparison.

use orbs_render::{Painter, Pos, Rampart, Rect, Role, Style, UtteranceKind, Wash};
use orbs_sim::content::Prose;

/// The gap between the board and the transcript.
const GUTTER: u16 = 1;

/// The fewest columns the transcript keeps before the board yields.
///
/// The same figure the map, the sheet and the sanctum's board use, and for the
/// same reason: below it the board has won an argument it should lose.
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
/// `const` for `pylon::split`'s reason: this board is a fixed size whatever is
/// standing on it, so the layout never has to look inside the siege it was
/// handed. That is the property `Rampart::rows` exists for.
pub const fn split(area: Rect, siege: Option<&Rampart>) -> Split {
    let nothing = Split {
        area: Rect::EMPTY,
        rest: area,
    };
    if siege.is_none() {
        return nothing;
    }

    // Border on both sides, in both directions.
    let block = Rampart::COLS.saturating_add(2);
    let tall = Rampart::rows().saturating_add(2);

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
pub fn paint(painter: &mut Painter<'_>, at: Rect, siege: &Rampart, prose: &Prose) {
    if at.is_empty() {
        return;
    }
    painter.border(at, Some(&prose.line("siege_title", &[])), Style::DIM);

    let inside = at.inset(1);
    let cols = usize::from(inside.cols);

    // What is coming, first and at the top — it is the one thing on this board
    // that is about the *future*, and it is what the turn is spent answering.
    let coming = prose.line("siege_coming", &[("state", &siege.intent)]);
    painter.glyphs(
        Pos::new(inside.col, inside.row),
        &truncate(&coming, cols),
        Style::DIM,
    );

    // **The four areas, above the bands.** They are what the turn is *for*, and
    // the bands are the consequence — so the thing being decided reads first.
    // An empty row is the most important one on the board: three dice against
    // four areas means one is always dark.
    for (offset, area) in siege.areas.iter().enumerate() {
        let Ok(offset) = u16::try_from(offset) else {
            break;
        };
        let row = Rampart::AREAS_AT + offset;
        painter.glyphs(
            Pos::new(inside.col, inside.row + row),
            &truncate(&siege.area_row(area), cols),
            // **Dim when it is moot**, which is the one place this board says
            // *do not* rather than *here is what is*. Weight rather than hue, so
            // it survives a dump and a greyscale tube (§14).
            if area.moot { Style::DIM } else { Style::NORMAL },
        );
    }

    // Theirs above the rule, yours below it — the way a wall is drawn.
    let bands = Rampart::AREAS_AT + 5;
    band(painter, inside, bands, siege, &siege.enemy);
    let rule = orbs_render::rampart::GROUND.to_string().repeat(cols);
    painter.glyphs(
        Pos::new(inside.col, inside.row + bands + 1),
        &truncate(&rule, cols),
        Style::DIM,
    );
    band(painter, inside, bands + 2, siege, &siege.garrison);

    // What is left to pledge, and then the tally.
    painter.glyphs(
        Pos::new(inside.col, inside.row + bands + 4),
        &truncate(&siege.coffer_row(), cols),
        Style::NORMAL,
    );
    painter.glyphs(
        Pos::new(inside.col, inside.row + bands + 5),
        &truncate(&siege.tally, cols),
        Style::DIM,
    );

    speak(painter, siege, prose);
}

/// Draw one side's row, and tint its bar.
fn band(
    painter: &mut Painter<'_>,
    inside: Rect,
    row: u16,
    siege: &Rampart,
    side: &orbs_render::SiegeSide,
) {
    let at = Pos::new(inside.col, inside.row + row);
    // **`glyphs`, so the picture is silent**, and the summary below says what it
    // means. A reader hearing forty cells of block and shade read out gets
    // box-drawing noise, which is what §19's frame rule exists to prevent.
    painter.glyphs(
        at,
        &truncate(&siege.row(side), usize::from(inside.cols)),
        Style::NORMAL,
    );
    // The tint is enrichment: the label already says which side this is, and
    // §14 requires the bar's *length* to carry the comparison on its own.
    let filled = Rampart::filled(side);
    if filled > 0 {
        painter.tint(
            Rect::new(at.col + Rampart::LABEL, at.row, filled, 1),
            Wash::plain(siege.tint(side)),
        );
    }
}

/// Cut a line to the box, by characters rather than bytes.
fn truncate(line: &str, cols: usize) -> String {
    line.chars().take(cols).collect()
}

/// What the board says, for a reader.
///
/// **Spoken once as a summary, never cell by cell**, which is every other
/// board's rule. What a reader needs from a siege is who is standing, what is
/// coming, and what the odds are — the per-roll detail is already in the log,
/// where it was said as a sentence.
fn speak(painter: &mut Painter<'_>, siege: &Rampart, prose: &Prose) {
    painter.announce(
        UtteranceKind::Progress,
        Role::Normal,
        &prose.line(
            "siege_spoken",
            &[
                ("quantity", &siege.garrison.troops.to_string()),
                ("name", &siege.enemy.troops.to_string()),
                ("state", &siege.intent),
                ("detail", &siege.garrison.chance.to_string()),
            ],
        ),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn siege() -> Rampart {
        Rampart {
            garrison: orbs_render::SiegeSide {
                name: "garrison",
                troops: 6,
                vigour: 18,
                full: 18,
                chance: 50,
            },
            enemy: orbs_render::SiegeSide {
                name: "enemy",
                troops: 7,
                vigour: 21,
                full: 21,
                chance: 50,
            },
            turns: 0,
            areas: ["line", "buckler", "succour", "sortie"]
                .into_iter()
                .map(|name| orbs_render::Allocation {
                    name,
                    dice: Vec::new(),
                    range: (0, 0),
                    moot: false,
                })
                .collect(),
            coffer: vec![
                ("d6".to_owned(), 1),
                ("d8".to_owned(), 2),
                ("d20".to_owned(), 5),
            ],
            quintessence: 24,
            intent: "onslaught".to_owned(),
            tally: "round 0, 7 still coming".to_owned(),
        }
    }

    fn area(cols: u16, rows: u16) -> Rect {
        Rect::new(0, 0, cols, rows)
    }

    #[test]
    fn no_siege_takes_no_columns() {
        let whole = area(120, 45);
        let split = split(whole, None);
        assert!(split.area.is_empty());
        assert_eq!(split.rest, whole);
    }

    #[test]
    fn a_siege_takes_columns_and_never_rows() {
        // **Columns, never rows** — the rule §19 records getting backwards once.
        // Taking rows under a `Top` panel leaves the deep-focus floor a five-row
        // transcript.
        let whole = area(120, 45);
        let split = split(whole, Some(&siege()));
        assert!(!split.area.is_empty());
        assert_eq!(split.area.rows, Rampart::rows() + 2);
        assert_eq!(
            split.rest.rows, whole.rows,
            "the board took rows from the transcript",
        );
        assert!(split.rest.cols < whole.cols);
    }

    #[test]
    fn it_refuses_whole_rather_than_truncating() {
        // Half a siege board is a comparison with one side missing, and the
        // domain *is* that comparison.
        let split = split(area(50, 45), Some(&siege()));
        assert!(
            split.area.is_empty(),
            "the board squeezed into a pane too narrow for it",
        );
        assert_eq!(split.rest, area(50, 45));
    }

    #[test]
    fn it_refuses_when_there_are_too_few_rows_as_well() {
        let split = split(area(120, 6), Some(&siege()));
        assert!(split.area.is_empty(), "the board drew into six rows");
    }

    #[test]
    fn the_transcript_keeps_its_floor() {
        let whole = area(Rampart::COLS + 2 + GUTTER + TRANSCRIPT_FLOOR, 45);
        let split = split(whole, Some(&siege()));
        assert!(!split.area.is_empty(), "the board yielded one column early");
        assert!(split.rest.cols >= TRANSCRIPT_FLOOR);
    }

    #[test]
    fn the_board_sits_against_the_edge_the_panel_took() {
        let whole = area(120, 45);
        let split = split(whole, Some(&siege()));
        assert_eq!(
            split.area.col + split.area.cols,
            whole.col + whole.cols,
            "the board floated off the right edge",
        );
    }
}
