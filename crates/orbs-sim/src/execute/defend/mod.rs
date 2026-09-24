//! `defend`, `pledge`, `deploy`, `quaff` and `hold` — the bailey's five words
//! (§5.1). Count them from `Verb::anchor`, never from this line — `pledge`
//! arrived fifth and left six doc sites saying four.
//!
//! The shape is the sanctum's, two rooms over: `defend` lets an enemy arrive the
//! way `muster` draws a course, the two bands publish readings the way the three
//! stations do, and a spell's `if` resolves against them at cast because
//! [`tower::siege::readings`](crate::tower::siege::readings) is registered
//! unconditionally in the scene.
//!
//! None of the five takes the production slot — §19's *"a domain stands
//! alone"*, applied to the domain that most needs it: a siege exists to test
//! the automation, so a bound brewing spell keeps working while you fight.
//!
//! `hold` is the only one that advances the world. Everything else on your turn
//! is free and instant (§5.0), which is what makes the siege turn-based rather
//! than merely slow: nothing races you while you read the board.
//!
//! Split by *when*, not by what:
//!
//! | File | Lines | What it is |
//! |---|---|---|
//! | `verbs` | 292 | The five words themselves, and `wield`'s interception |
//! | `spending` | 172 | Taking an arsenal item and applying what `siege.toml` says it does |
//! | `report` | 209 | What a resolved round says — every roll to the log, one sentence to the pane, and the payout |
//! | `publish` | 141 | Every reading the bailey publishes, which is the half the scripting rests on |
//! | `shared` | 36 | The rampart, and the one-line `say` all four use |
//!
//! `verbs` is what a player types, `spending` what it costs, `report` what
//! comes back, `publish` what a spell can then ask about. One file per verb
//! would have put four copies of *find the siege, refuse if there is none* in
//! four places — the duplication `spend` exists to avoid.

mod publish;
mod report;
mod shared;
mod spending;
mod verbs;

pub(crate) use publish::publish_dice;
// `debug_siege`'s alone — see `publish::refresh` and `execute::mod`.
#[cfg(debug_assertions)]
pub(crate) use publish::refresh;
pub(super) use verbs::{defend, deploy, hold, petition, pledge, quaff, wielded};
