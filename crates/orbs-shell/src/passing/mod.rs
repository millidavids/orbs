//! The clock a screen leaves and arrives on.
//!
//! [`orbs_render::passage`] decides what a crossing *looks* like; this decides
//! *when*, because nothing in `orbs-render` knows what a second is.
//!
//! [`PaneTransition`](crate::PaneTransition)'s twin: that animates a pane's
//! geometry, this animates its content. Two clocks because they answer to
//! different events — `F4` moves rectangles, `attend` moves screens — and one
//! doing both would have to interleave them.
//!
//! There is no `Screen` enum in this crate; which view draws is re-derived every
//! frame by [`prompt::paint`](crate::paint). So this watches a derived value,
//! [`Showing`], whose three fields each guard a specific false positive.
//!
//! §14 requires motion be disableable. The crossing rides the frontend's own
//! motion switch, passed to [`Passing::advance`] rather than queried here,
//! exactly as [`Bench`](crate::Bench) does. With motion off there is nothing
//! kept and no crossing to freeze — §19 records three animations that learned
//! *"a picture may never be able to vanish"*, and a crossing frozen at its
//! midpoint is an empty pane for the session.

mod clock;
mod pose;
mod showing;
#[cfg(test)]
mod tests;

pub use clock::Passing;
pub use showing::Showing;
