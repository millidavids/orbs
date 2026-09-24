//! The manual: the game explaining itself, from inside.
//!
//! A surface and not a `recall` page. `recall`/`help`/`man` has answered *what
//! does this word do* since Phase 1 and is not going anywhere — it prints
//! records into the transcript, so it wraps, tiles, sifts, speaks and pages for
//! free — but it answers about the room you are standing in.
//!
//! That is why it cannot be this: every one of its eight branches needs a `Sim`
//! with a player standing somewhere, so there was no way to read *how do I play
//! this* before playing it. The threshold put a screen in front of a player with
//! no tower, and the manual is what that screen owes them.
//!
//! It borrows everything `recall` already wrote. [`assemble`] builds the book
//! from the authored chapters in `manual.toml` and the 578
//! `man_`/`recall_`/`using_` keys accumulated since Phase 1; §12 budgets ~88k
//! words and the cheapest are the ones already written.
//!
//! Three files, for `menu`'s reasons: [`state`] is what a keystroke does,
//! [`assemble`] is where the book comes from, [`paint`] is the picture.

mod assemble;
mod paint;
mod state;

pub use assemble::book;
pub use paint::paint;
pub use state::{Outcome, Reader, Showing};
