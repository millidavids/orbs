//! Numbers that roll to their new value rather than snapping (DESIGN.md §19,
//! *number go up*).
//!
//! The shell half of [`Roll`](orbs_render::Roll): this watches the panel for a
//! number changing and says how far each roll has run; `orbs_render` decides
//! what is drawn. Driven from `Time`, never the tick, for
//! [`Bench`](crate::Bench)'s reason.
//!
//! Off unless a frontend says motion is allowed. Off is the truth, so a
//! frontend with no switch a player can reach — `orbs-tui`, a dump — draws every
//! number settled without doing anything.

mod rise;
#[cfg(test)]
mod tests;
mod watching;

pub use rise::Rolled;
pub use watching::{Climb, Watch};
