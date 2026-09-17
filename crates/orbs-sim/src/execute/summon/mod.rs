//! The menagerie's circle: `summon`, `limn`, and what the circle publishes.

mod publish;
mod verbs;

pub(super) use publish::publish;
#[cfg(debug_assertions)]
pub(crate) use publish::refresh;
pub use verbs::TROOP;
pub(crate) use verbs::{limn, release, summon};
