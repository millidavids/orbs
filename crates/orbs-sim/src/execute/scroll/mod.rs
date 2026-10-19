//! Scrolls, and what spending them does (§10, §19 *Supply*).
//!
//! Four walks of the stacks assemble into scrolls at the lectern; this is the
//! other half. Outside a siege a scroll is a calm supply: `wield` opens it,
//! it draws every few hundred ticks while open, and `stop` closes it.

mod calm;
mod glean;
mod quicken;
mod spend;
mod verdant;

pub(crate) use calm::drain;
pub(in crate::execute) use glean::{gather, gathering};
pub use spend::Scroll;
pub(in crate::execute) use spend::spend;
pub(crate) use spend::spending;
