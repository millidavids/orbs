//! Enchantments: temporary effects laid on the tower's tools.
//!
//! The model half of §10's Enchanting: what a charm is and how a read site asks
//! about one. `execute::imbue` lays them; `content/forge.toml` prices them.
//! `spell::budget`'s shape — the world is asked and composes the answer — rather
//! than every call site branching on each source it knows.
//!
//! | | |
//! |---|---|
//! | `kind` | what a charm does, as a closed table |
//! | `apply` | the component, the clock, and the one question a read site asks |
//! | `readings` | what a spell can ask, so the maintenance loop is writable |

mod apply;
mod kind;
mod readings;

pub use apply::{Charm, Charmed, charm_left, charmed, longest_of_each, soonest};
pub use kind::Kind;
pub use readings::{EBBING, EBBING_AT, FORGE, GRACED, LATTICE, LIT, readings};
