//! The tower's two standings, as bars at the top of the pane (DESIGN.md §11.5),
//! and what the room in front of the player has made in all.
//!
//! Two bars and no more: experience and renown are the only numbers about the
//! tower over its whole life rather than about a room, a run or a fight. The
//! column at the strip's right edge is the one exception, and not a bar — a
//! room's whole-life count, drawn only where it costs the bars nothing they
//! need (§19, *number go up*).
//!
//! Both fill from the tier *behind* to the tier *ahead*. The weave's bar
//! measures the whole line to ten thousand, which is right for a screen about
//! the whole line and wrong for a glance — at 8 experience it is a sliver that
//! does not move for an hour. Tier to tier it fills visibly and empties once.
//!
//! Two rows, and expensive ones: §9 wants the main window fully functional and
//! the road already took one. These yield before the transcript does, so
//! [`split`] hands the body back untouched when a pane is short.
//!
//! A number rolls to its new value rather than snapping — see `crate::climb`.

mod column;
mod row;
mod strip;
#[cfg(test)]
mod tests;

pub(crate) use strip::{paint, split};
