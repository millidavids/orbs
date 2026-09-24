//! The ward's sheet, beside the transcript (DESIGN.md §10, `tower::ward`).
//!
//! The map's shape, one room over, and not gated on a word: it draws whenever a
//! reading is open, which is what makes a bound solver watchable. There is no
//! full-pane mode, because unlike a maze there is nothing to walk — a ward is
//! played from the prompt with `dial` and `probe`.
//!
//! Columns, never rows, and it splits after the instrument panel: an instrument
//! row is load-bearing where a sheet is a convenience, and whichever split runs
//! second is the one whose refusal can fire. §19 records getting that backwards.
//!
//! It refuses rather than truncating, because a row missing its pegs says a
//! press answered nothing. The maze's picture pans instead; the difference is
//! that a ward has no "where you are" to centre on.

use orbs_render::{Board, Painter, Pos, Rect, Role, Style, UtteranceKind, Wash};
use orbs_sim::content::Prose;

// The gap and the floor every board beside the transcript keeps. The sheet
// windows its rows, so its `split` is its own; the transcript it leaves is not.
use crate::beside::{GUTTER, TRANSCRIPT_FLOOR};

/// Where the sheet sits, and what is left for the transcript.
#[derive(Debug, Clone, Copy)]
pub struct Split {
    /// The sheet's own rectangle, border included. Empty when there is none.
    pub area: Rect,
    /// What the transcript gets.
    pub rest: Rect,
    /// How many presses the sheet has room for — see [`paint`].
    pub presses: usize,
}

/// The sheet's footprint, and what is left for the transcript.
pub fn split(area: Rect, board: Option<&Board>) -> Split {
    let nothing = Split {
        area: Rect::EMPTY,
        rest: area,
        presses: 0,
    };
    let Some(board) = board else {
        return nothing;
    };

    // Border on both sides, in both directions.
    let (cols, _) = board.size();
    let block = cols.saturating_add(2);

    // Columns are refused whole; rows are windowed. Clipping sideways loses a
    // press's pegs, but `Board::SHOWN` has always made this a window on the most
    // recent twelve, so a shorter pane showing ten is the same kind of view.
    //
    // It mattered: at the 80×22 authoring floor a full twelve-row sheet is one
    // row too tall, so the board vanished entirely on the twelfth press — the
    // cliff `SHOWN` exists to remove, moved rather than deleted. A blind ladder
    // averages twenty-three presses, so a bound solver lost its sheet part-way
    // through every solve with nothing said.
    let inside = area.rows.saturating_sub(2);
    let presses = board
        .showing()
        .len()
        .min(orbs_render::Board::presses_within(inside));
    let tall = orbs_render::Board::rows_for(presses).saturating_add(2);

    // `presses == 0` refuses, which the first windowing did not check. At
    // exactly eight rows the arithmetic came out `tall == 8` and the guard below
    // is `8 > 8` — false — so the sheet painted a header, a rule, an aperture
    // and two legend rows with no presses at all, while still taking 41 columns
    // off the transcript. `presses_within` names the zero case as *"where the
    // sheet does refuse"*.
    if presses == 0
        || block.saturating_add(GUTTER + TRANSCRIPT_FLOOR) > area.cols
        || tall > area.rows
    {
        return nothing;
    }

    // Anchored to the edge the instrument panel took, so the two sit together
    // and the transcript keeps one uninterrupted run of columns.
    let wanted = block.saturating_add(GUTTER);
    Split {
        area: Rect::new(area.col + area.cols - block, area.row, block, tall),
        rest: Rect::new(area.col, area.row, area.cols - wanted, area.rows),
        presses,
    }
}

/// Draw the sheet where [`split`] put it.
pub fn paint(painter: &mut Painter<'_>, at: Rect, board: &Board, presses: usize, prose: &Prose) {
    if at.is_empty() {
        return;
    }
    painter.border(at, Some(&prose.line("ward_title", &[])), Style::DIM);

    let inside = at.inset(1);
    for row in 0..inside.rows {
        // The same cap [`split`] sized the rectangle from, or the sheet draws
        // twelve presses into a box measured for ten and the foot — the legend
        // and the aperture — falls off the bottom.
        let Some(cells) = board.row_capped(usize::from(row), presses) else {
            break;
        };
        for (col, (glyph, style, tint)) in cells.into_iter().enumerate() {
            let Ok(col) = u16::try_from(col) else { break };
            if col >= inside.cols {
                break;
            }
            let at = Pos::new(inside.col + col, inside.row + row);
            // `glyphs`, so the picture is silent and the summary below says what
            // it means. A reader hearing four sigils and four pegs cell by cell
            // gets box-drawing noise, which is what §19's frame rule — structure
            // on one channel, content on the other — exists to prevent.
            painter.glyphs(at, &glyph.to_string(), style);
            if let Some(tint) = tint {
                painter.tint(Rect::new(at.col, at.row, 1, 1), Wash::plain(tint));
            }
        }
    }

    speak(painter, board, prose);
}

/// What the sheet says, for a reader.
///
/// Spoken once as a summary, never cell by cell. §14 makes the linear stream
/// architectural, and what a reader needs from a board is the last answer and
/// what it cost — not twenty glyphs transcribed. The per-press detail is already
/// in the transcript as a sentence.
fn speak(painter: &mut Painter<'_>, board: &Board, prose: &Prose) {
    let spent = board.attempts.len();
    let (aligned, astray) = board
        .attempts
        .last()
        .map_or((0, 0), |attempt| (attempt.aligned, attempt.astray));
    // No count of held sockets any more: the sheet used to say how many
    // positions were proven, which is the one thing a codemaker may not tell you
    // (§19), so the reader got a better game than the player. What is left is
    // what a sighted player reads off the rows.
    painter.announce(
        UtteranceKind::Progress,
        Role::Normal,
        &prose.line(
            "ward_spoken",
            &[
                ("quantity", &spent.to_string()),
                ("name", &aligned.to_string()),
                ("detail", &astray.to_string()),
            ],
        ),
    );
}
