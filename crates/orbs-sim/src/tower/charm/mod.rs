//! Enchantments: temporary effects laid on the tower's tools.
//!
//! §10's Enchanting domain is *"Sequence + resource cost → persistent buffs"*,
//! and ROADMAP's exit for the phase is *"a buffed instrument visibly works
//! faster, the panel says so, and the buff decays"*. This is the model half —
//! what a charm **is** and how a read site asks about one. The forge that lays
//! them is `execute::imbue`; what each costs and how long it runs is authored in
//! `content/forge.toml` (rule 6).
//!
//! # Why this is a composition and not another `if`
//!
//! `tower::dice` had already written the objection, about the one buff that
//! existed before this: *"it works because there is exactly one source and one
//! effect, and **it does not generalise** — a second source would need the call
//! site to know about it."* The forge is that second source, five times over. So
//! the shape is `spell::budget`'s rather than `quicken::hastened`'s — the world
//! is asked a question and composes the answer, instead of each call site
//! branching on every source it happens to know about.
//!
//! | | |
//! |---|---|
//! | `kind` | what a charm does, as a closed table |
//! | `apply` | the component, the clock, and the one question a read site asks |
//! | `readings` | what a spell can ask, so the maintenance loop is writable |

mod apply;
mod kind;
mod readings;

pub use apply::{Charm, Charmed, charm_left, charmed};
pub use kind::Kind;
pub use readings::{EBBING, EBBING_AT, FORGE, GRACED, LATTICE, LIT, readings};
