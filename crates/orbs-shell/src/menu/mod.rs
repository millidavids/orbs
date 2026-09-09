//! The orb's menu: the screen `quit` opens (DESIGN.md §15, §19).
//!
//! # `quit` opens it, and that spends no verb
//!
//! `execute::quit`'s own doc already argued the shape: *"`quit` closes the spell
//! editor, `quit` closes the weave screen, and now `quit` closes the orb: the
//! word means **leave the thing you are in**, whichever thing that is."* From a
//! room, the thing you are in is the game — so `quit` steps out of it and puts
//! this in front of you, and the menu's own `quit` steps out of the orb.
//!
//! **Discoverability was already solved.** `quit` is `is_live` in every room and
//! has a manual page; a menu reached by a key nobody can find would be the exact
//! defect `execute::quit` exists to fix, arriving one level up.
//!
//! **The sim's `Quitting` handshake is unchanged.** It still means *the player
//! asked to leave*, and what leaving does was always the frontend's business —
//! that is why the sim never wrote an `AppExit` itself. So this needed no new
//! verb, no new resource and no save migration.
//!
//! # Rule 2, and why it holds no `Sim`
//!
//! Everything drawn is authored prose or something the shell handed in.
//! [`Menu`] holds what is *typed* and what was refused, exactly as [`Tapestry`]
//! does; the frontend owns the rest.
//!
//! # Three files, because a fourth page was coming
//!
//! It was one 756-line file and CLAUDE.md's floor is ~300. The split is by
//! concern rather than by type: [`state`] is what a keystroke does to the menu,
//! [`words`] is the vocabulary and the prefix rule that governs it, and
//! [`paint`] is the picture. The settings page landed on top of the split rather
//! than under it, so *"nothing moved"* was a `diff -r` rather than an argument.
//!
//! [`Tapestry`]: crate::Tapestry

mod paint;
mod state;
mod words;

pub use paint::paint;
pub use state::{Driver, Menu, Outcome};
pub use words::WORDS;
