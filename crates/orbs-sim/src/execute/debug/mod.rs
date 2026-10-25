//! A door for a tester, in the builds a tester runs.
//!
//! Each word makes a See-it line's state one line away. None is a
//! [`Verb`](crate::parser::Verb), which the tutorial, completion and `recall`
//! all walk; each is matched exactly, before the parser sees the line.
//!
//! A release build has no code for any of it, only a few lines of
//! `prose.toml`, which ships whole (rule 6). So a debug session replays only
//! in a debug build.

mod door;
mod progress;
mod shortcut;
mod spawn;
mod word;

pub use door::open;
pub use spawn::run;
pub(in crate::execute) use word::argument;
pub use word::{Asking, DebugWord, Order};
