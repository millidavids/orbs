//! The five puzzle instruments' pictures (§19, *every instrument has a tempo*).
//!
//! One module rather than five, because the five share every rule that makes a
//! picture safe — the join, the at-rest stillness, the flip ceiling — and those
//! are tested once over all of them.

mod circle;
mod lattice;
mod lectern;
mod prism;
mod pylon;
mod rite;
#[cfg(test)]
mod tests;

pub(crate) use rite::cell;
pub use rite::{Figure, Rite};
