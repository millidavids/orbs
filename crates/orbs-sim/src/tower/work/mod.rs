//! Actions that take time.
//!
//! §5.0: issuing is free and actions take time, so the scarcity is concurrency —
//! the economy and the focus system are one. One production slot for the whole
//! tower (§11.5, §9): start a brew and you are not also deciphering, the trade
//! the focus track is built on.
//!
//! | Module | What |
//! |---|---|
//! | [`slot`] | the intervals, the pools, `begin`, `stop`, and the busy contract |
//! | [`produce`] | `transmute` — what an instrument makes |
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
pub use quicken::{QUICKENED_BY, Quickened, hurried_from, quickened, quickened_by};
pub use slot::{
    Bidden, Busy, CAPACITY, DIVINE_TICKS, PURGE_TICKS, Triaging, Working, begin, busy, in_flight,
    occupied, refuse_busy, stop,
};
pub use triage::purge;
