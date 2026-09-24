//! The menagerie's circle, as a picture: three glyphs and a beast's temper.
//!
//! Drawn beside the transcript whenever a beast waits, the rule the archive's
//! map, the lens's sheet and the sanctum's board all follow: the picture is not
//! gated on a word, because watching a bound spell hold a beast and holding one
//! yourself are different activities.
//!
//! A truth table, written sideways. Each sense is a row of eight cells — four at
//! a lesser circle — and each column is one way the senses can be lit, so the
//! temper under them reads down a column as *"when these are lit, the beast
//! answers this"*. Sideways because eight columns of two cells fit beside a
//! transcript at the 80×22 floor, where eight rows of four would not.
//!
//! It carries nothing the world lacks — rule 2's line. Every cell is a fact the
//! sim holds, and the numbers over the columns let a reader and a sighted player
//! name the same row. A lit cell and a dark one differ in glyph and a balking
//! row is marked with a shape, so the board reads in greyscale, in a dump and to
//! a screen reader (§14). The one tint is enrichment on the humours.

mod board;
#[cfg(test)]
mod tests;

pub use board::{Circle, Given, Line};
