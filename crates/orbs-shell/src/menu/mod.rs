//! The orb's menu: the screen `quit` opens (DESIGN.md §15, §19).
//!
//! `quit` opens it, spending no verb: the word means *leave the thing you are
//! in*, and from a room that thing is the game. The menu's own `quit` steps out
//! of the orb. `quit` is `is_live` everywhere and has a manual page, so
//! discoverability was already solved.
//!
//! The sim's `Quitting` handshake is unchanged — it still means *the player
//! asked to leave*, and what leaving does was always the frontend's — so this
//! needed no new verb, resource or save migration.
//!
//! Rule 2: everything drawn is authored prose or something the shell handed in.
//! [`Menu`] holds what is typed and what was refused, as [`Tapestry`] does.
//!
//! Split by concern from one 756-line file: [`state`] is the menu's shape,
//! [`pages`] routes a finished line to the page that is up, [`playing`] and
//! [`setting`] are those pages, [`words`] is the vocabulary and its prefix
//! rule, [`stance`] is where the menu is standing, [`paint`] is the picture.
//! Each split but the last came *before* its feature, so *"nothing moved"* was
//! a `diff -r` rather than an argument.
//!
//! [`Tapestry`]: crate::Tapestry

mod pages;
mod paint;
mod playing;
mod setting;
mod stance;
mod state;
mod words;

pub use paint::{SETTINGS_ROWS, paint};
pub use stance::Stance;
pub use state::{Driver, Menu, Outcome};
pub use words::WORDS;
