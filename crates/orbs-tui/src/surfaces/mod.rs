//! Which of the five things on screen has the keyboard, and what it does with it.
//!
//! One enum and a `match`, since a terminal hands one keystroke to whoever asks
//! and has no queue. The surfaces are `orbs-shell`'s; this picks which.

mod dispatch;
mod state;
#[cfg(test)]
mod tests;

pub(crate) use state::{Owner, Surfaces};
