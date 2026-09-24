//! The augury's seam — what reads a line the orb could not read itself (§6).
//!
//! The trait lives here with no dependency to show for it: a reader takes a
//! line and answers with a canonical command, and both are `str`. So it sits
//! beside [`is_literal`] and [`Analysis::reads_outright`], the questions asked
//! before it, and [`Sim::submit_divined`], which is what happens after. In a
//! frontend it would split one rule across two crates; here `orbs-sim` owns the
//! whole of *which lines go to a reader* and stays headless and synchronous.
//!
//! A reader never sees the world. [`Augur::read`] gets the line and nothing
//! else, and answers with the player's own words in the slots: `smash the sage`
//! becomes `grind sage`, not `grind /tower/laboratory/sage` — binding a name to
//! something in the room is `parser::resolve`'s work. That is what makes a
//! reader cheap: none needs a `Scene`, none goes stale between ticks, and none
//! can disagree with the matcher about what a name means.
//!
//! [`Fixture`] answers from a table and is how every headless instrument
//! reaches a divined line at all. [`Grammar`] matches the authored templates in
//! `content/phrasings.toml` and is the baseline a trained reader has to beat:
//! within a point or two of it, the corpus was the whole feature.
//!
//! None of it is required. [`Sim::submit`] is still the path every literal line
//! takes, and a line the deterministic pipeline cannot read still reaches §6's
//! suggestions rather than a bare error.
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
