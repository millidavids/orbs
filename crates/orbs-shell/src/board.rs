//! The ward's sheet, beside the transcript (DESIGN.md §10, `tower::ward`).
//!
//! The map's shape, one room over. **It is not gated on a word**: it draws
//! whenever a reading is open, which is what makes a bound solver watchable —
//! watching and doing are different activities, and only one of them owns the
//! keyboard. There is no full-pane mode here at all, because unlike a maze there
//! is nothing to walk: a ward is played from the prompt with `dial` and `probe`.
//!
//! **Columns, never rows**, and it splits *after* the instrument panel — an
//! instrument row is load-bearing where a sheet is a convenience, and whichever
//! split runs second is the one whose refusal can fire. §19 records getting that
//! backwards.
//!
//! It **refuses rather than truncating**: a row missing its pegs says a press
//! answered nothing, which is worse than no sheet at all. That is the maze's
//! rule where the maze's own picture pans instead, and the difference is that a
//! ward has no "where you are" to centre on — every row matters equally.

use orbs_render::{Board, Painter, Pos, Rect, Role, Style, UtteranceKind, Wash};
use orbs_sim::content::Prose;

/// The gap between the sheet and the transcript.
const GUTTER: u16 = 1;

/// The fewest columns the transcript keeps before the sheet yields.
///
/// The same figure the map uses, and for the same reason: below it the sheet has
/// won an argument it should lose.
const TRANSCRIPT_FLOOR: u16 = 24;

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

    // **Columns are refused whole; rows are windowed.** Clipping a sheet
    // sideways loses a press's pegs, which says a press answered nothing — but
    // `Board::SHOWN` has always made this a window on the most recent twelve, so
    // a shorter pane showing ten of them is the same kind of view rather than a
    // lost comparison.
    //
    // It mattered: at the 80×22 authoring floor a full twelve-row sheet is one
    // row too tall, so the board **vanished entirely on the twelfth press** —
    // the cliff `SHOWN` exists to remove, moved rather than deleted. A blind
    // ladder averages twenty-three presses, so a bound solver lost its sheet
    // part-way through every solve, with no partial view and nothing said.
    let inside = area.rows.saturating_sub(2);
    let presses = board
        .showing()
        .len()
        .min(orbs_render::Board::presses_within(inside));
    let tall = orbs_render::Board::rows_for(presses).saturating_add(2);

    // **`presses == 0` refuses**, which the first version of this windowing did
    // not check. At exactly eight rows the arithmetic came out `tall == 8` and
    // the guard below is `8 > 8` — false — so the sheet was laid out and painted
    // with a header, a rule, an aperture and two legend rows and **no presses at
    // all**, while still taking 41 columns off the transcript. That is the exact
    // inverse of this module's rule, and `presses_within` names the zero case as
    // *"where the sheet does refuse"*.
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
        // **The same cap [`split`] sized the rectangle from**, or the sheet
        // would draw twelve presses into a box measured for ten and the foot —
        // the legend and the aperture — would fall off the bottom.
        let Some(cells) = board.row_capped(usize::from(row), presses) else {
            break;
        };
        for (col, (glyph, style, tint)) in cells.into_iter().enumerate() {
            let Ok(col) = u16::try_from(col) else { break };
            if col >= inside.cols {
                break;
            }
            let at = Pos::new(inside.col + col, inside.row + row);
            // **`glyphs`, so the picture is silent**, and the summary below says
            // what it means. A reader hearing four sigils and four pegs read out
            // cell by cell would get box-drawing noise, which is exactly what
            // §19's frame rule puts structure on one channel and content on the
            // other to prevent.
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
/// **Spoken once as a summary, never cell by cell.** §14 makes the linear stream
/// architectural rather than a nicety, and the thing a reader needs from a board
/// is the last answer and how many presses it has cost — not a transcription of
/// twenty glyphs. The per-press detail is already in the transcript, where it was
/// said as a sentence.
fn speak(painter: &mut Painter<'_>, board: &Board, prose: &Prose) {
    let spent = board.attempts.len();
    let (aligned, astray) = board
        .attempts
        .last()
        .map_or((0, 0), |attempt| (attempt.aligned, attempt.astray));
    // **No count of held sockets any more.** The sheet used to say how many
    // positions were proven, which is the one thing a codemaker may not tell you
    // (§19) — the reader got a better game than the player. What is left is the
    // last answer and what it cost, which is what a sighted player reads off the
    // rows.
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
