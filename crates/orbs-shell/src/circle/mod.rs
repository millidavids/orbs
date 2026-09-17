//! The circle and the beast waiting at it, beside the transcript (DESIGN.md §10,
//! `tower::circle`).

mod board;
mod speech;
#[cfg(test)]
mod tests;

pub(crate) use board::{paint, split};
