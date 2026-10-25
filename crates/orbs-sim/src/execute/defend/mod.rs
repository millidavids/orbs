//! The bailey's words (§5.1) — count them from `Verb::anchor`, not here.
//!
//! The sanctum's shape: `defend` as `muster`, the bands' readings as the
//! stations', [`tower::siege::readings`](crate::tower::siege::readings)
//! registered unconditionally. No production slot, so a bound brewing spell
//! keeps working while you fight (§19). Only `hold` advances the world (§5.0).
//!
//! Split by *when*, not by verb, so *find the siege* is written once:
//!
//! | File | What it is |
//! |---|---|
//! | `verbs` | The words, and `wield`'s interception |
//! | `spending` | Opening or raising a supply, `stop`, and stillness (§19, *Supply*) |
//! | `drawing` | A supply's draws: at opening, each round, and what a still hands back |
//! | `report` | What a resolved round says, and the payout |
//! | `publish` | Every reading the bailey publishes |
//! | `shared` | The rampart, and the one-line `say` |

mod drawing;
mod publish;
mod report;
mod shared;
mod spending;
mod verbs;

pub(crate) use publish::publish_dice;
// `debug_siege`'s alone — see `publish::refresh` and `execute::mod`.
#[cfg(debug_assertions)]
pub(crate) use publish::refresh;
pub(super) use spending::stop as stop_supply;
pub(super) use verbs::{defend, deploy, hold, petition, pledge, quaff, wielded};
