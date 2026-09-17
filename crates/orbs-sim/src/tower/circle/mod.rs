//! The menagerie's circle — a beast's temper, held by limning three glyphs (§10).

mod beast;
pub mod circuit;
mod glyph;
mod humour;
#[cfg(test)]
mod proofs;
mod shape;
pub mod temper;
mod view;

pub use beast::{Beast, Call, TROOPS, TROOPS_PAST_PAR};
pub use circuit::{Circuit, Puzzle, Turned, Wiring};
pub use glyph::{FERVOUR, GLYPHS, Glyph, HUMOURS, readings};
pub use humour::Humour;
pub use shape::{
    LESSER_PAR, LESSER_SENSES, LESSER_SHARE, LESSER_TROOPS, Shape, lesser_answer, lessers,
};
pub use temper::{OPENING, PAR, ROWS, SENSES, Temper};
pub use view::view;
