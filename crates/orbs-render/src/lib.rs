//! Frame / cell-buffer, layout, and semantic styling.
//!
//! What appears and where; frontends decide how a cell is drawn, and how big
//! (rule 2).
//!
//! ```text
//! orbs-sim ──records──▶ orbs-render ──Frame──▶ orbs      (GPU cell grid + CRT)
//!                                          └─▶ orbs-tui  (alternate screen buffer)
//! ```
//!
//! Four things this crate exists to guarantee:
//!
//! 1. Presentation is a view over [`Records`] (rule 4); see [`record`].
//! 2. No colour: a [`Style`] means, a frontend resolves. [`Depiction`] picks a
//!    ramp and says nothing, kept off meaning by [`Style::depicted`].
//! 3. A [`Speech`] beside every frame's cells (§14).
//! 4. The CP437 repertoire as Unicode ([`cp437`]).
//!
//! Painting a screen:
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

// Public for their module docs: the repertoire/encoding split, and the model
// the whole pipeline is built on. Both re-export their types at the crate root,
// which is where callers should reach for them.
pub mod cp437;
pub mod record;

mod bath;
mod board;
mod cell;
mod circle;
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
// Public for its module docs: it decides what a screen *leaving* looks like,
// and the safety argument for a crossing is written there, not in `pulse`.
pub mod passage;
mod pulse;
mod pylon;
// `pub`, unlike the other painters' modules: `orbs-shell`'s siege painter
// draws the rule between the two bands and needs the glyph by name.
pub mod lattice;
pub mod rampart;
mod roll;
mod span;
mod style;
mod tiling;
mod tween;
mod viewport;
mod wrap;
pub use wrap::Wrap;

pub use bath::Steep;
// `SIGILS` and `TINTS` alongside the types: they are indexed together, and a
// caller that can reach one and not the other cannot check they correspond.
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
pub use passage::{Crossing, Kept, Passage, Toward};
pub use pulse::{CYCLE_SECS, FLIP_HZ};
// `TALLEST` alongside the type for the reason `SIGILS` travels with `Board`:
// `tower::pylon::MOST` checks the rows reserved against the wards raised.
// `GROUND`/`WARD` are not exported — a glyph is `row`'s business, and a
// frontend is handed cells rather than chars.
// `Line` as `CircleLine`: `line` already means a row of text in this crate.
pub use circle::{Circle, Given as CircleGiven, Line as CircleLine};
pub use pylon::{Pylon, TALLEST};
// `Side`, not `Band` — `mix::Band` already means a stripe of one reagent in the
// flask, and two `Band`s in one namespace is a rename waiting to happen.
pub use lattice::Lattice as LatticeBoard;
pub use rampart::{Allocation, Rampart, Side as SiegeSide};
pub use record::{
    FieldName, Outcome, Record, RecordBuilder, RecordKind, RecordView, Records, Sift, Value,
    contains_ignoring_case,
};
pub use roll::{PLUS_SECS, ROLL_SECS, ROLL_STEPS, Roll, fading};
pub use span::Span;
pub use style::{
    Density, Depiction, Fill, Heat, Intensity, Lexeme, Presentation, Roil, Role, Style, Tint, Wash,
};
pub use viewport::{
    CELL_HEIGHT, CELL_WIDTH, GRID, INPUT_ROWS, MIN_GRID, MIN_SCALE, PICTURE, pixels, scale_for,
};

/// The first `cells` characters of `text` — what has arrived, if it is arriving.
///
/// The one place text is cut mid-reveal, so the cut is always on a character
/// boundary: Frame text is bounded to CP437 but is still UTF-8, and slicing by
/// byte could split a multi-byte glyph and panic. Nothing here knows what a
/// second is — a caller owns the clock and passes a count.
#[must_use]
pub fn arriving(text: &str, cells: u32) -> &str {
    let taken = usize::try_from(cells).unwrap_or(usize::MAX);
    &text[..char_index(text, taken)]
}

/// The byte index of character `count`, or the end of `text`.
///
/// [`arriving`]'s counterpart, counting characters, never bytes. Here so the
/// shell's caret and the sim's completer agree.
#[must_use]
pub fn char_index(text: &str, count: usize) -> usize {
    text.char_indices()
        .nth(count)
        .map_or(text.len(), |(index, _)| index)
}
