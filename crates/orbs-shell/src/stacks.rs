//! The archive's map, where it sits in the session pane (DESIGN.md §10, §19).
//!
//! Columns, never rows, whichever way the panel runs. Following
//! [`super::panel::split`] would take *rows* under
//! [`Along::Top`](super::panel::Along), and at the deep-focus floor a 17-row
//! block out of 22 leaves a five-row transcript. It is also why [`split`] runs
//! *after* the panel's: whichever runs second is the one whose refusal can
//! fire, and an instrument row is load-bearing where a map is a convenience.
//!
//! Neither view refuses a pane for being short. A block too small for the whole
//! maze shows a window centred on the reading, which pans as it walks — see
//! `orbs_render::maze::viewport`. Only a keyhole is refused.
//!
//! [`paint`] puts the map beside the transcript — the watching case, where a
//! solver working for four hundred ticks is otherwise a two-cell gauge and
//! seeing it fill in is how a player finds out their rule is looping.
//!
//! [`paint_alone`] takes the whole pane, and only while the arrow keys have the
//! maze. The prompt is dead then — `input::type_into_line` discards every key —
//! so a screen that still looked like a session would offer what it cannot do.

use orbs_render::{Painter, Rect, Span, Stacks, Style, UtteranceKind};
use orbs_sim::content::Prose;

// The gap and the floor every board beside the transcript keeps. The map's own
// `split` is its own; the transcript it leaves is not.
use crate::beside::{GUTTER, TRANSCRIPT_FLOOR};

/// Where the map sits, and what is left for the transcript.
#[derive(Debug, Clone, Copy)]
pub struct Split {
    /// The map's own rectangle, border included. Empty when there is none.
    pub area: Rect,
    /// What the transcript gets.
    pub rest: Rect,
}

/// The map's footprint, and what is left for the transcript.
///
/// Takes what it can, and the picture pans inside it. Refusing unless the whole
/// maze fitted held at 15 squares across; at 33 the map vanished from every
/// small grid. `Painter::stacks` draws a window centred on the reading instead.
///
/// Still refused: a block too narrow to read anything from, or one that would
/// leave no transcript beside it.
pub fn split(area: Rect, maze: Option<&Stacks>) -> Split {
    let nothing = Split {
        area: Rect::EMPTY,
        rest: area,
    };
    let Some(maze) = maze else {
        return nothing;
    };

    // Border on both sides of the picture, in both directions.
    let (cols, rows) = maze.size();
    let spare = area.cols.saturating_sub(GUTTER + TRANSCRIPT_FLOOR);
    let block = cols.saturating_add(2).min(spare);
    let tall = rows.saturating_add(2).min(area.rows);

    if block < LEAST_BLOCK || tall < LEAST_BLOCK {
        return nothing;
    }

    // Anchored to the same edge the instrument panel took, so the two sit
    // together and the transcript keeps one uninterrupted run of columns.
    let wanted = block.saturating_add(GUTTER);
    Split {
        area: Rect::new(area.col + area.cols - block, area.row, block, tall),
        rest: Rect::new(area.col, area.row, area.cols - wanted, area.rows),
    }
}

/// The smallest block worth drawing a window into, border included.
///
/// Nine leaves seven squares — the reading and three each way, enough to see
/// the junction you are standing in. Below that the transcript is the better
/// use of the columns.
const LEAST_BLOCK: u16 = 9;

/// Draw the map where [`split`] put it, beside the transcript.
///
/// The watching case: a spell has the maze, the player has the prompt.
pub fn paint(painter: &mut Painter<'_>, at: Rect, maze: &Stacks, prose: &Prose) {
    if at.is_empty() {
        return;
    }
    painter.border(at, Some(&prose.line("maze_title", &[])), Style::DIM);
    painter.stacks(at.inset(1), maze);
}

/// Draw the maze over the whole pane, with the transcript behind it.
///
/// The walking case, and it takes the pane for the editor's reason: the prompt
/// is dead while the arrows have the keyboard, so standing in the maze is a
/// mode and should look like one. A spell's solving stays inline instead,
/// beside a live transcript.
pub fn paint_alone(painter: &mut Painter<'_>, at: Rect, maze: &Stacks, prose: &Prose) {
    if at.is_empty() {
        return;
    }
    let area = at;
    painter.border(
        area,
        Some(&prose.line("maze_title_walking", &[])),
        Style::SUCCESS,
    );

    // Centred in what the status rows leave rather than in the whole pane —
    // otherwise it drifts upward as the pane shortens and collides with them.
    let body = area.inset(1);
    let footer = FOOTER_ROWS.min(body.rows);
    let picture = Rect::new(
        body.col,
        body.row,
        body.cols,
        body.rows.saturating_sub(footer),
    );
    painter.stacks(picture, maze);

    if footer < FOOTER_ROWS {
        return;
    }
    let (walked, total) = maze.explored();
    let bottom = body.bottom().saturating_sub(FOOTER_ROWS);
    // Spoken, unlike the inline map's: the panel is not on screen here, so this
    // is the only place the count exists. §14's "completion only" rule is about
    // progress bars, not a pane's own summary.
    painter.span(
        Rect::new(body.col, bottom, body.cols, 1).origin(),
        &Span::new(&prose.line(
            "maze_walked",
            &[
                ("quantity", &walked.to_string()),
                ("detail", &total.to_string()),
            ],
        ))
        .with_style(Style::DIM),
    );
    painter.span(
        Rect::new(body.col, bottom.saturating_add(1), body.cols, 1).origin(),
        &Span::new(&prose.line("maze_keys", &[]))
            .with_style(Style::DIM)
            .with_kind(UtteranceKind::Input),
    );
}

/// Rows the full-pane view keeps under the picture: the count, and the keys.
const FOOTER_ROWS: u16 = 2;

#[cfg(test)]
mod tests {
    use super::*;
    use orbs_render::Square;

    /// The real grid: 33 squares across and 23 down, one character each.
    ///
    /// Not square: a square fixture would let a bug that swapped the axes pass.
    fn maze() -> Stacks {
        Stacks {
            squares: vec![Square::default(); 33 * 23],
            width: 33,
            at: 0,
            exit: Some(33 * 23 - 1),
            spoils: Vec::new(),
        }
    }

    /// The session-pane body at a grid, given the panel's width.
    ///
    /// Transcribed from the traced geometry rather than recomputed, since what
    /// is under test is this module's arithmetic and not the layout's.
    /// [`the_games_body_is_still_the_one_transcribed_here`] keeps it honest.
    const BODIES: [(&str, u16, u16); 2] = [
        // The running game: 120x45 Wide, two panes, no panel in the archive.
        ("the game", 118, 38),
        // `ORBS_GRID=160x45` as a dump: two panes, `Top` panel.
        ("a wide dump", 78, 39),
    ];

    /// The transcription above, checked against the layout it was traced from.
    ///
    /// `BODIES[0]` went stale silently when the grid stopped following the
    /// window, and every test over it passed — a wrong rectangle is still a
    /// rectangle. The grid is a constant now, so the real number is reachable.
    #[test]
    fn the_games_body_is_still_the_one_transcribed_here() {
        // The chain `prompt::paint` walks: the grid, two panes, Wide, the input
        // line's rows, then the first pane inset by its border. The archive
        // carries no instruments, so `panel::split` hands the body straight on.
        let layout = orbs_render::ScreenLayout::compute(&orbs_render::ScreenRequest {
            main_panes: 2,
            mode: orbs_render::DisplayMode::Wide,
            ..orbs_render::ScreenRequest::single(orbs_render::GRID)
        });
        let body = layout.main()[0].inset(1);

        assert_eq!(
            (body.cols, body.rows),
            (BODIES[0].1, BODIES[0].2),
            "the game's session pane moved; re-trace BODIES[0]",
        );
    }

    /// Panes the whole 33×23 picture does not fit in, which get a window on it.
    ///
    /// It fitted every grid at 7×7. At 16×16 the block wants 35 rows and these
    /// have 18 and 22, so they show the part the reading is standing in.
    const TOO_SMALL: [(&str, u16, u16); 2] = [
        // `ORBS_DUMP`'s own default, 80x22, `Side` panel.
        ("the dump floor", 74, 18),
        // `ORBS_GRID=100x28`, the deep-focus floor: `Top` panel.
        ("deep focus", 48, 22),
    ];

    #[test]
    fn the_map_fits_every_grid_the_see_it_lines_use() {
        // Taking rows under a `Top` panel would leave deep focus with a
        // five-row transcript, which this list exists to keep failing loudly.
        for (name, cols, rows) in BODIES {
            let split = split(Rect::new(0, 0, cols, rows), Some(&maze()));
            assert!(!split.area.is_empty(), "{name}: no room for the map");
            assert_eq!(split.area.cols, 35, "{name}");
            assert_eq!(split.area.rows, 25, "{name}");
            assert!(
                split.rest.cols >= TRANSCRIPT_FLOOR,
                "{name}: the transcript was squeezed to {}",
                split.rest.cols,
            );
            assert_eq!(split.rest.rows, rows, "{name}: the map took rows");
        }
    }

    #[test]
    fn a_pane_too_short_for_the_whole_maze_still_gets_a_window() {
        // Refusing outright meant the map vanished from every small grid once
        // the maze went from 7×7 to 16×16. Only a keyhole is still refused.
        for (name, cols, rows) in TOO_SMALL {
            let split = split(Rect::new(0, 0, cols, rows), Some(&maze()));
            assert!(!split.area.is_empty(), "{name}: no window at all");
            assert!(split.area.rows <= rows, "{name}: the block overflowed");
            assert!(
                split.rest.cols >= TRANSCRIPT_FLOOR,
                "{name}: the transcript was squeezed to {}",
                split.rest.cols,
            );
        }
    }

    #[test]
    fn a_short_pane_shrinks_the_block_rather_than_overflowing() {
        let short = split(Rect::new(0, 0, 118, 20), Some(&maze()));
        assert_eq!(short.area.rows, 20, "the block did not shrink to the pane");
        assert!(short.rest.cols >= TRANSCRIPT_FLOOR);
    }

    #[test]
    fn a_keyhole_is_refused_and_the_transcript_keeps_the_pane() {
        // Below `LEAST_BLOCK` there is no junction to read in it.
        let narrow = split(Rect::new(0, 0, 30, 20), Some(&maze()));
        assert!(narrow.area.is_empty(), "a keyhole was drawn");
        assert_eq!(narrow.rest, Rect::new(0, 0, 30, 20));
    }

    #[test]
    fn no_maze_takes_nothing() {
        let pane = Rect::new(0, 0, 118, 38);
        assert_eq!(split(pane, None).rest, pane);
    }

    #[test]
    fn the_transcript_and_the_map_never_overlap() {
        for (name, cols, rows) in BODIES {
            let split = split(Rect::new(0, 0, cols, rows), Some(&maze()));
            assert!(
                split.rest.right() <= split.area.col,
                "{name}: the transcript ran into the map",
            );
        }
    }
}
