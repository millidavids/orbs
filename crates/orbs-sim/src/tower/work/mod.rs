//! Actions that take time.
//!
//! DESIGN.md §5.0: *"Issuing an action is free; **actions take time to
//! complete.**"* Ticks are world time, not currency — there is no pool to spend
//! and a fast typist gains nothing over a slow one. The scarcity is **duration**,
//! and it is expressed as concurrency: how many things you can have in flight at
//! once.
//!
//! That is why this is the economy. §5.0 again: *"Concurrency is therefore the
//! real scarcity: how many duration-actions you can have in flight at once …
//! which means the economy and the focus system are the same system rather than
//! two bolted together."*
//!
//! # One production slot
//!
//! §11.5 starts the player at **multiplex capacity 1**, and §9's fourth
//! invariant reserves that slot for an in-flight manual action for its whole
//! duration. So brewing occupies the tower, not merely the laboratory: start a
//! brew and you are not also deciphering. That is the trade the whole focus track
//! is built on, and giving each domain its own slot would delete it while being
//! *more* code — a counter per domain where the design needs one.
//!
//! | Module | What |
//! |---|---|
//! | [`slot`] | the intervals, the pools, `begin`, `stop`, and the busy contract |
//! | [`produce`] | `transmute` and `siphon` — what an instrument makes |
//! | [`quicken`] | a domain working at double speed, for a while |
//! | [`triage`] | `purge` — §7's destruction-as-maintenance |
//! | [`land`] | the once-per-tick pass that completes both pools |

mod land;
mod produce;
mod quicken;
mod slot;
mod triage;

#[cfg(test)]
mod tests;

pub use land::finish;
pub use produce::{Product, contents};
pub use quicken::{QUICKENED_BY, QUICKENED_TICKS, Quickened, quickened};
pub use slot::{
    Bidden, Busy, CAPACITY, DIVINE_TICKS, PURGE_TICKS, Triaging, Working, begin, busy, in_flight,
    occupied, refuse_busy, stop,
};
pub use triage::purge;
