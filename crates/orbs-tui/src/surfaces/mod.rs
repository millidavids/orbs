//! Which of the five things on screen has the keyboard, and what it does with it.
//!
//! The Bevy build spends four resources, four run conditions and a four-term
//! predicate on this, and its own comment predicts the ceiling: *"a fifth surface
//! refactors this first."* Here it is one enum and a `match`, because a terminal
//! delivers one keystroke at a time to whoever is asking — there is no message
//! queue to leave keys waiting in, and so no need to *run* in order to throw them
//! away.
//!
//! The state machines themselves are not reimplemented: `Editor` and `Tapestry`
//! are `orbs-shell`'s, and this decides only which a key reaches.

mod dispatch;
mod state;
#[cfg(test)]
mod tests;

pub(crate) use state::{Owner, Surfaces, Swap};
