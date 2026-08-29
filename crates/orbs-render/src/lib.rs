//! Frame / cell-buffer, layout, and semantic styling.
//!
//! `orbs-render` decides *what appears and where*. Frontends decide only *how a
//! cell is drawn* — and how big, which is why the grid is a constant here and
//! the letterbox is the frontend's. They may add enrichment the others cannot
//! reproduce (CRT effects, audio) provided that enrichment carries no
//! information absent from the Frame. See CLAUDE.md, architectural rule 2.
//!
//! # The pipeline
//!
//! ```text
//! orbs-sim ──records──▶ orbs-render ──Frame──▶ orbs      (GPU cell grid + CRT)
//!                                          └─▶ orbs-tui  (alternate screen buffer)
//! ```
//!
//! # Four things this crate exists to guarantee
//!
//! 1. **Commands emit records; presentation is a view over the record.** The
//!    [`Records`] stream is the source, and the screen, the screen reader,
//!    `sift`, and the balance harness are four views of it that can see exactly
//!    the same things. That is what lets the tube corrupt the first without the
//!    other three noticing. See [`record`] for why the model
//!    lives here rather than in `orbs-sim`.
//! 2. **No colour.** A [`Cell`] carries a [`Style`] — a [`Role`], an
//!    [`Intensity`], a [`Presentation`], a [`Depiction`] — and never a hue. Each
//!    frontend resolves those against its own palette: curated phosphor themes
//!    under Bevy, the user's terminal theme under `orbs-tui`.
//!
//!    [`Depiction`] is the odd one and the exception that proves the rule: it
//!    selects a colour ramp and says nothing, where every other channel says
//!    something and lets the frontend pick the colour. It exists so the
//!    athanor's meter can be a *picture* of a fire rather than a reading of one.
//!    See [`Style::depicted`] for the rule that keeps it from painting over
//!    anything that means something.
//! 3. **A linear stream, always.** Every frame carries a [`Speech`] alongside
//!    its cells, because a cell grid read back row by row is box-drawing
//!    characters and column fragments, not sentences. DESIGN.md §14 makes this
//!    architectural; it is the piece that genuinely cannot be retrofitted.
//! 4. **A bounded alphabet.** Glyphs are restricted to what both frontends can
//!    draw — the CP437 repertoire, expressed as Unicode. See [`cp437`].
//!
//! # Painting a screen
//!
//! ```
//! use orbs_render::{
//!     DisplayMode, Frame, GRID, INPUT_ROWS, Pos, ScreenLayout, ScreenRequest, Span,
//!     Style, UtteranceKind,
//! };
//!
//! // The grid is a constant: the window decides how big a cell is, not how
//! // many there are. Nothing in this crate consults a window at all.
//! let grid = GRID;
//!
//! let layout = ScreenLayout::compute(&ScreenRequest {
//!     grid,
//!     main_panes: 2,
//!     rail: true,
//!     mode: DisplayMode::Deep,
//!     input_rows: INPUT_ROWS,
//! });
//!
//! let mut frame = Frame::new(grid);
//! for (pane, title) in layout.main().iter().zip(["laboratory", "sanctum"]) {
//!     let mut painter = frame.painter(*pane);
//!     painter.border(*pane, Some(title), Style::DIM);
//!     painter.span(
//!         Pos::new(pane.col + 2, pane.row + 1),
//!         &Span::new("east wall breached").with_style(Style::DANGER),
//!     );
//! }
//!
//! let input = layout.input();
//! frame
//!     .painter(input)
//!     .span(input.origin(), &Span::new("orbs:~$ ").with_kind(UtteranceKind::Input));
//! frame.set_cursor(Some(Pos::new(input.col + 8, input.row)));
//!
//! // The same frame, with no pixels at all.
//! let spoken: Vec<_> = frame.speech().utterances().map(|u| u.text).collect();
//! assert_eq!(spoken[0], "laboratory");
//! assert_eq!(spoken[1], "east wall breached");
//! ```

// Public for their module docs: `cp437` documents the repertoire/encoding split,
// `record` documents the model the whole pipeline is built on. Both re-export
// their types at the crate root, which is where callers should reach for them.
pub mod cp437;
pub mod record;

mod bath;
mod board;
mod cell;
mod chant;
mod fire;
mod frame;
mod geometry;
mod grind;
mod layout;
mod linear;
// Public for its module docs: it is the one place that decides what motion
// *means* across the two vessels, and both `Steep` and `Stir` point at it.
pub mod liquid;
mod maze;
mod mix;
mod paint;
mod pulse;
mod pylon;
mod span;
mod style;
mod tiling;
mod tween;
mod viewport;
mod wrap;
pub use wrap::Wrap;

pub use bath::Steep;
// `SIGILS` and `TINTS` alongside the types: the glyph table and the name table
// have to be indexed together, and a caller that can reach one and not the other
// cannot check that they correspond.
pub use board::{Attempt, Board, SIGILS, TINTS};
pub use cell::Cell;
pub use cp437::{REPLACEMENT, cp437_glyph, cp437_index, is_renderable};
pub use fire::Burn;
pub use frame::Frame;
pub use geometry::{GridSize, Pos, Rect};
pub use grind::{Grind, fallen_cells};
pub use layout::{
    DEEP_FOCUS_FLOOR, DisplayMode, MAX_MAIN_PANES, MAX_PANES, MIN_PANE_ROWS, MIN_RAIL_BOX,
    RAIL_COLS, RAIL_FOOT_ROWS, STRIP_ROWS, ScreenLayout, ScreenRequest,
};
pub use linear::{Speech, Utterance, UtteranceKind};
pub use liquid::{DRIFT_EVERY, Motion, RISE_EVERY, STIR_EVERY};
pub use maze::{Square, Stacks};
pub use mix::{Band, Stir};
pub use paint::Painter;
pub use pulse::{CYCLE_SECS, FLIP_HZ};
// `TALLEST` alongside the type for the reason `SIGILS` travels with `Board`: the
// board reserves that many rows and the sim raises that many wards, and a caller
// that can reach one and not the other cannot check they agree.
// `tower::pylon::MOST` is that caller, and `GROUND`/`WARD` had no equivalent —
// they were exported beside it out of symmetry and reached by nothing, where a
// glyph is `row`'s business and a frontend is handed cells rather than chars.
pub use chant::{AHEAD, Figure};
pub use pylon::{Pylon, TALLEST};
pub use record::{
    FieldName, Outcome, Record, RecordBuilder, RecordKind, RecordView, Records, Sift, Value,
    contains_ignoring_case,
};
pub use span::Span;
pub use style::{
    Density, Depiction, Heat, Intensity, Lexeme, Presentation, Roil, Role, Style, Tint, Wash,
};
pub use viewport::{
    CELL_HEIGHT, CELL_WIDTH, GRID, INPUT_ROWS, MIN_GRID, MIN_SCALE, PICTURE, pixels, scale_for,
};

/// The first `cells` characters of `text` — what has arrived, if it is arriving.
///
/// The one place text is cut mid-reveal, so the cut is made the same way
/// everywhere: **on a character boundary**. Frame text is bounded to the CP437
/// repertoire but is still UTF-8, and slicing by byte could split a multi-byte
/// glyph into something that is not a `str` and panic.
///
/// [`RecordView::revealing`] uses it for command output; the Bevy frontend's boot
/// sequence uses it for the POST card. Nothing here knows what a second is — a
/// caller owns the clock and passes a count.
#[must_use]
pub fn arriving(text: &str, cells: u32) -> &str {
    let taken = usize::try_from(cells).unwrap_or(usize::MAX);
    &text[..char_index(text, taken)]
}

/// The byte index of character `count`, or the end of `text`.
///
/// The counterpart to [`arriving`], and here for the same reason: **count
/// characters, never bytes.** Frame text is bounded to the CP437 repertoire but
/// is still UTF-8, so a byte offset can split `░` or `é` into something that is
/// not a `str` and panic on the slice.
///
/// It lives in this crate because the two callers have to *agree*: the Bevy
/// shell uses it to place the caret and slice the viewport, and `orbs-sim`'s
/// completer uses it to compute the range a Tab replaces — then the shell splices
/// one into the other. They were two private copies, identical today, in
/// different crates; the first time either grew a nuance the caret and the
/// replaced range would have desynchronised and Tab would have overwritten the
/// wrong bytes.
#[must_use]
pub fn char_index(text: &str, count: usize) -> usize {
    text.char_indices()
        .nth(count)
        .map_or(text.len(), |(index, _)| index)
}
