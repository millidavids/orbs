//! Standing supplies: no produced item does anything alone (§19, *Supply*).
//!
//! An item's verb opens it, the verb again raises it one rate, and `stop`
//! closes it. While open it draws from the arsenal, running at the fastest
//! rate the held count still meets and freshness allows.

mod open;
mod rate;
mod readings;
mod word;

pub use open::{Open, Scope, Supplies, held_in_arsenal};
pub use rate::{Raise, Refusal, Slowed, may_raise, ran_low, running, slowed};
pub use readings::{DRAINED, DRAW, gauge, readings, restocked};
pub use word::{Ration, Word, applied, barred, ration, shelved, word};
