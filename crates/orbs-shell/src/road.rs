//! A room's mastery line, as a road under the pane's title (DESIGN.md §11.5).
//!
//! # Why every room draws it
//!
//! The weave is where progression is *acted on*; this is where it is *seen
//! while working*. A player grinding their fourth potion should not have to
//! open a screen to learn there is a fifth worth grinding — so the room's own
//! line runs under its title, in the same marks the weave uses, with the next
//! deed and how much of it is done at the far end.
//!
//! One painter for seven rooms, and geometry decided here rather than in each
//! room's board: the row is taken off the top of the pane's body before the
//! instrument panel and the boards divide the rest, so it sits under the title
//! whichever way the panel runs.
//!
//! # Rule 2
//!
//! Everything drawn arrives through [`Line`] — the stations, their walk, the
//! next deed's count — and the sentence is `prose.toml`'s. What is decided here
//! is where the cells go.

use orbs_render::{Painter, Pos, Rect, Span, Style};
use orbs_sim::{Line, Prose};

use super::stations;

/// The row the road took, and what is left for the rest of the pane.
#[derive(Debug, Clone, Copy)]
pub struct Split {
    /// One row, or empty when the road does not draw.
    pub area: Rect,
    /// The body below it.
    pub rest: Rect,
}

/// Cells the room's name takes, gap included — the loom's own reckoning:
/// `laboratory` is ten and one cell separates it from the line.
const NAME: u16 = 11;

/// Cells between stations. A station is three cells — frame, mark, frame — so
/// four keeps one run cell between neighbours.
const STRIDE: u16 = 4;

/// Rows the body must keep after the road takes one.
///
/// A transcript squeezed to nothing so a progress row could draw would be the
/// minimised half winning over the main window, which §9 forbids. Below this
/// the road yields, as the rail does.
const KEEP: u16 = 6;

/// Between the deed's sentence and its count. CP437 0xFA, like the mark for
/// a station further along.
const SEP: &str = " \u{b7} ";

/// Take the top row of `body` for the road, if there is a line to draw and
/// room to draw it.
#[must_use]
pub fn split(body: Rect, line: Option<&Line>) -> Split {
    let drawable = line.is_some_and(|line| line.open && !line.stops.is_empty());
    if !drawable || body.rows <= KEEP || body.cols < NAME + STRIDE {
        return Split {
            area: Rect::EMPTY,
            rest: body,
        };
    }
    Split {
        area: Rect::new(body.col, body.row, body.cols, 1),
        rest: Rect::new(
            body.col,
            body.row.saturating_add(1),
            body.cols,
            body.rows.saturating_sub(1),
        ),
    }
}

/// Draw the road into its row.
///
/// The name, then a run with the stations standing on it, then the next deed
/// as a sentence and its count pinned to the right edge. **The count outranks
/// the sentence** when the row is short: `3 of 5` is the fact, and the
/// sentence is what the weave's panel says at length.
pub fn paint(painter: &mut Painter<'_>, area: Rect, line: &Line, prose: &Prose) {
    if area.is_empty() {
        return;
    }
    let y = area.row;
    painter.span(
        Pos::new(area.col, y),
        &Span::new(line.domain).with_style(Style::DIM),
    );

    let start = area.col.saturating_add(NAME);
    let width = area.cols.saturating_sub(NAME);
    let stations = u16::try_from(line.stops.len()).unwrap_or(u16::MAX);
    let run_needed = STRIDE.saturating_mul(stations).saturating_add(1);

    // The label, cut down until it fits beside the stations.
    let count = line.next().map(|next| {
        prose.line(
            "weave_progress",
            &[
                ("count", &next.done.to_string()),
                ("quantity", &next.needed.to_string()),
            ],
        )
    });
    let sentence = line
        .next()
        .map(|next| prose.line(&format!("mastery_{}", next.id), &[]));
    let candidates: [Option<String>; 3] = [
        match (&sentence, &count) {
            (Some(sentence), Some(count)) => Some(format!("{sentence}{SEP}{count}")),
            (None, None) => Some(prose.line("road_done", &[])),
            _ => None,
        },
        count.clone(),
        None,
    ];
    let label = candidates.into_iter().flatten().find(|label| {
        let cells = u16::try_from(label.chars().count()).unwrap_or(u16::MAX);
        run_needed.saturating_add(cells).saturating_add(2) <= width
    });
    let label_cells = label
        .as_ref()
        .map_or(0, |label| u16::try_from(label.chars().count()).unwrap_or(0));

    let run_width = width.saturating_sub(label_cells.saturating_add(u16::from(label_cells > 0)));
    stations::run(painter, Pos::new(start, y), run_width);
    for (index, stop) in line.stops.iter().enumerate() {
        let x = start
            .saturating_add(1)
            .saturating_add(STRIDE.saturating_mul(u16::try_from(index).unwrap_or(0)));
        if x.saturating_add(1) >= start.saturating_add(run_width) {
            break;
        }
        let (mark, style) = stations::walk_mark(stop.walk);
        stations::framed(painter, x, y, mark, style, false);
    }
    if let Some(label) = label {
        painter.span(
            Pos::new(
                area.col
                    .saturating_add(area.cols)
                    .saturating_sub(label_cells),
                y,
            ),
            &Span::new(&label).with_style(Style::DIM),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use orbs_sim::{Stop, Walk};

    fn line(open: bool, stops: usize) -> Line {
        Line {
            domain: "laboratory",
            open,
            stops: (0..stops)
                .map(|index| Stop {
                    id: format!("laboratory_{}", index + 1),
                    walk: if index == 0 { Walk::Next } else { Walk::Later },
                    done: 2,
                    needed: 5,
                    opens: Vec::new(),
                })
                .collect(),
        }
    }

    #[test]
    fn the_road_takes_one_row_and_yields_when_the_body_is_short() {
        let body = Rect::new(1, 1, 60, 20);
        let split = split(body, Some(&line(true, 6)));
        assert_eq!(split.area.rows, 1);
        assert_eq!(split.rest.rows, 19);
        assert_eq!(split.rest.row, 2);

        let short = split_short();
        assert!(short.area.is_empty(), "the road squeezed a short pane");
        assert_eq!(short.rest.rows, KEEP);
    }

    fn split_short() -> Split {
        split(Rect::new(1, 1, 60, KEEP), Some(&line(true, 6)))
    }

    #[test]
    fn a_shut_room_and_a_line_with_nothing_on_it_draw_no_road() {
        let body = Rect::new(1, 1, 60, 20);
        assert!(split(body, None).area.is_empty());
        assert!(split(body, Some(&line(false, 6))).area.is_empty());
        assert!(split(body, Some(&line(true, 0))).area.is_empty());
    }
}
