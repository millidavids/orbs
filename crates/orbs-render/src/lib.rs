//! Frame / cell-buffer, layout, and semantic styling.
//!
//! `orbs-render` decides *what appears and where*. Frontends decide only *how a
//! cell is drawn*, and may add enrichment the others cannot reproduce (CRT
//! effects, audio, fidelity tiers) provided that enrichment carries no
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
//!    [`Intensity`], a [`Presentation`] — and never a hue. Each frontend
//!    resolves those against its own palette: curated phosphor themes under
//!    Bevy, the user's terminal theme under `orbs-tui`.
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
//!     DisplayMode, Fidelity, Frame, Pos, ScreenLayout, ScreenRequest, Span, Style,
//!     UtteranceKind,
//! };
//!
//! let window = (1920, 1080);
//! let tier = Fidelity::tier_one(window).expect("1080p hosts the floor");
//! let grid = tier.grid(window);
//!
//! let layout = ScreenLayout::compute(&ScreenRequest {
//!     grid,
//!     main_panes: 2,
//!     sidebar_panes: 1,
//!     mode: DisplayMode::Deep,
//!     input_rows: tier.input_rows(),
//! });
//!
//! let mut frame = Frame::new(grid);
//! for (pane, title) in layout.main().iter().zip(["laboratory", "battlements"]) {
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

mod cell;
mod fidelity;
mod frame;
mod geometry;
mod layout;
mod linear;
mod paint;
mod span;
mod style;
mod tiling;
mod tween;
mod wrap;

pub use cell::Cell;
pub use cp437::{REPLACEMENT, cp437_glyph, cp437_index, is_renderable};
pub use fidelity::{CELL_HEIGHT, CELL_WIDTH, Fidelity, MIN_GRID};
pub use frame::Frame;
pub use geometry::{GridSize, Pos, Rect};
pub use layout::{
    DEEP_FOCUS_FLOOR, DisplayMode, MAX_MAIN_PANES, MAX_PANES, MIN_PANE_ROWS, STRIP_ROWS,
    ScreenLayout, ScreenRequest,
};
pub use linear::{Speech, Utterance, UtteranceKind};
pub use paint::Painter;
pub use record::{
    FieldName, Outcome, Record, RecordBuilder, RecordKind, RecordView, Records, Sift, Value,
    contains_ignoring_case,
};
pub use span::Span;
pub use style::{Intensity, Presentation, Role, Style};

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
