//! The augury's seam — what reads a line the orb could not read itself (§6).
//!
//! # Why the trait lives here, with no dependency to show for it
//!
//! A reader takes a line and answers with a canonical command, and both are
//! `str`. Nothing about the seam needs a tensor, a device, or a crate that
//! knows what either is — so it sits beside [`is_literal`] and
//! [`Analysis::reads_outright`], which are the two questions asked *before* it,
//! and [`Sim::submit_divined`], which is what happens after.
//!
//! Putting it in a frontend instead would have split one rule across two
//! crates and given the headless driver nothing to hold. Here, `orbs-sim` owns
//! the whole of *which lines go to a reader*, and the reader itself is somebody
//! else's problem — which is the boundary that lets this crate stay headless,
//! synchronous and testable in milliseconds.
//!
//! # A reader never sees the world
//!
//! [`Augur::read`] gets the line and nothing else, and answers with a canonical
//! command carrying **the player's own words in its slots**: `smash the sage`
//! becomes `grind sage`, not `grind /tower/laboratory/sage`. Binding a name to
//! something that is actually in the room is `parser::resolve`'s work and stays
//! deterministic, integer-scored and explainable.
//!
//! That is not a restriction, it is what makes a reader cheap: no reader needs
//! a `Scene`, none can go stale between ticks, and none can disagree with the
//! matcher about what a name means.
//!
//! # Two readers, and neither is a placeholder
//!
//! [`Fixture`] answers from a table and is how every headless instrument —
//! `ORBS_DUMP`, `scripts/dumps.sh`, `scripts/play.sh` — can reach a divined
//! line at all. [`Grammar`] matches the authored templates in
//! `content/phrasings.toml` and is the **baseline a trained reader has to
//! beat**: if it lands within a point or two, the corpus was the whole feature.
//!
//! # Nothing here is required
//!
//! A tower with no reader is the game exactly as it was: [`Sim::submit`] is
//! still the path every literal line takes, and a line the deterministic
//! pipeline cannot read still reaches §6's suggestions rather than a bare
//! error.
//!
//! [`is_literal`]: crate::parser::is_literal
//! [`Analysis::reads_outright`]: crate::parser::Analysis::reads_outright
//! [`Sim::submit_divined`]: crate::Sim::submit_divined
//! [`Sim::submit`]: crate::Sim::submit

mod copyist;
mod fixture;
mod grammar;
mod seam;

pub use copyist::Copyist;
pub use fixture::Fixture;
pub use grammar::Grammar;
pub use seam::{Augur, MAX_READINGS, Scrivener, Verbatim};
