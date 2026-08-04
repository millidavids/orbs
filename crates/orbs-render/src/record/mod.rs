//! Structured records — what commands emit, and what everything else reads.
//!
//! Architectural rule 4: *commands emit records; presentation is a view over the
//! record.* DESIGN.md §7 makes it binding on §13, and lists what depends on it:
//! pipes, `sift`, the eldritch renderer, screen-reader linearisation, and the
//! test harness.
//!
//! ```text
//! command ──▶ Records ──┬──▶ RecordView ──▶ Frame     what a player sees
//!                       ├──▶ Record::speak            what a reader hears
//!                       ├──▶ Records::sift            what a pipe stage passes on
//!                       └──▶ fields                   what the harness measures
//! ```
//!
//! Every one of those four is a *view*. None of them is the source, and none can
//! see anything the others cannot — which is the property that lets the tube
//! corrupt the first without the second, third, or fourth noticing.
//!
//! # Why this lives in `orbs-render`
//!
//! A record is the interface between `orbs-sim` and `orbs-render`, and an
//! interface belongs to whichever side both can depend on. This crate has no
//! dependencies at all, so `orbs-sim` can take it on for the cost of a
//! millisecond of compile time; the reverse would drag the world model, parser,
//! and script engine into the crate whose tests must stay pure presentation.
//!
//! It also already owns the vocabulary a record has to carry —
//! [`Role`](crate::Role), [`Presentation`](crate::Presentation),
//! [`UtteranceKind`](crate::UtteranceKind) — so putting records anywhere else
//! would mean either duplicating that vocabulary or depending on this crate
//! anyway.
//!
//! The direction costs one thing: `orbs-sim` can now *name* [`Painter`](crate::Painter).
//! Architectural rule 2 forbids it from using one, and `crates/orbs-sim/tests/boundaries.rs`
//! is what holds it to that.

mod field;
mod kind;
mod sift;
mod stream;
mod view;

pub use field::{FieldName, Value};
pub use kind::RecordKind;
pub use sift::Sift;
pub use stream::{Record, RecordBuilder, Records};
pub use view::RecordView;
