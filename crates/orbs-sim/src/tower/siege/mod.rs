//! The bailey's siege — a turn-based defence resolved on the dice (§10, §5.1).
//!
//! An unnamed king assigns the wizard a fixed contingent to hold his tower. An
//! enemy arrives; you spend what the arsenal has, and you `hold` to let a round
//! resolve. The enemy tells you what it will do next before it does it.
//!
//! # There is no clock in it, and that is the same position the sanctum takes
//!
//! §10 wants outcome to follow *what the player chooses given readable state,
//! **never how fast or precisely they act***. The sanctum reached that by
//! choosing a puzzle with no clock at all, and §19 called that *"a better
//! position than a real-time mechanic with a reflex escape hatch bolted on"*.
//! The siege takes it too: **the world advances when you end your turn**, never
//! while you are deciding. §10.1's one exception — the menagerie — is not
//! extended here.
//!
//! Turn-based serves both halves, which is the argument the first plan missed by
//! resting it on accessibility alone:
//!
//! - **By hand**, it is what makes the board readable. You weigh the arsenal
//!   against the enemy with nothing racing you.
//! - **Scripted**, it is what makes the domain automatable at all. A spell gets
//!   a bounded number of instructions per tick, and a real-time siege would
//!   outrun the decision tree — evaluating a branch against a board that had
//!   already moved. That is `PACE = 1` in the menagerie (§19), one domain over.
//!
//! **§5.0's *"no per-command tick cost"* is preserved.** Commands inside your
//! turn are free and instant; the clock advances on [`Siege::resolve`], which
//! [`hold`](crate::execute) is the only caller of. Typist fairness is untouched.
//!
//! # It takes no production slot, and that is load-bearing
//!
//! §19: *"a domain stands alone; the tower-wide systems only enhance it."* A
//! siege that held the tower-wide `CAPACITY` would freeze the automation the
//! enemy exists to attack, and the premise — *"the enemy attacks the
//! automation"* — would have nothing to attack. So a siege costs no slot, the
//! way the lens, the sanctum, the archive and the menagerie cost none, and a
//! bound brewing spell keeps running while you fight.
//!
//! # The readings carry the arithmetic, not the language
//!
//! *"If the enemy count is twice that of defenders"* is not expressible: the
//! spell language has at-least / at-most / exactly and place-against-place, and
//! no ratios. Growing arithmetic would be the wrong fix. This is the maze's
//! pattern instead — the world publishes a derived word and the spell asks for
//! it, the way `spoil`, `exit` and `back` already work. So the siege publishes
//! [`OUTNUMBERED`], [`FEW`], [`HURT`], and a spell reads those.
//!
//! # How it is split, and why the seams are where they are
//!
//! This was one 1,429-line file, and the module doc argued for keeping it that
//! way on the grounds that every other domain model is the same size. That
//! argument was about `ward.rs` and `maze.rs` rather than about this file: those
//! hold **one** model each, and this held five separable things.
//!
//! The submodules are private and everything below is re-exported, so these are
//! code spans rather than doc links — a `[`link`]` to a private module is a
//! rustdoc error under `-D warnings`.
//!
//! | File | Lines | What it is |
//! |---|---|---|
//! | `allocation` | 164 | Where a die goes. The dice, the four areas, what a round's dice came to |
//! | `band` | 262 | What is standing, what it intends, and what one round did to it |
//! | `battle` | 473 | [`Siege`] itself: the state, the readings' arithmetic, and `resolve` |
//! | `escrow` | 43 | §11.5's economy. **Not the siege's combat rules**, which is the seam |
//! | `quintessence` | 145 | What a siege has to spend on its dice, and what the tower's repair buys it |
//! | `readings` | 109 | The words the domain publishes for a spell to ask about |
//!
//! **`battle` is still over the guideline and is left that way deliberately.**
//! It is one type's inherent `impl`, and the only cut available is *arithmetic
//! here, `resolve` there* — which would put `hurt`, `completion` and `coffer` a
//! file away from the round that changes all three. Splitting a single type's
//! methods across files to reach a line count makes a reader open two files to
//! answer one question, which is the cost the rule exists to avoid rather than
//! to impose.
//!
//! **The re-exports below are the whole public surface and it did not change.**
//! Every caller says `siege::SPEARS`, `siege::Area`, `siege::escrow` exactly as
//! before — which is what made this a safe move to make at the end of a phase,
//! and is the property to preserve if anything is ever added here.
//!
//! **What splitting actually cost is privacy.** `Band::wound`, `Band::mend` and
//! `Intent::drawn` were private to one file and are `pub(super)` now, reachable
//! anywhere inside `siege`. That is the trade: the alternative was making them
//! fully `pub`, which would put a band's damage model in the tower's API.

mod allocation;
mod band;
mod battle;
mod escrow;
mod quintessence;
mod readings;

#[cfg(test)]
mod tests;

pub use allocation::{Area, COFFER, POOL, Pledge, SORTIE_COST, SORTIE_DEALT, Strengths};
pub use band::{
    AGAINST, ASSIGNED, BAILEY, Band, CADENCE, DEFEAT_WEAR, ENEMY, GARRISON, Intent, Outcome,
    RAMPART, Round, THIN, VICTORY_MEND, VIGOUR, WEARY,
};
pub use battle::{Pledged, Siege};
pub use escrow::{COMPLETION_BONUS, ESCROW_FLOOR, ESCROW_PER_FOE, escrow};
pub use quintessence::{FACES_PER_POINT, cost_of};
pub use readings::{
    AIM, CEILING, FEW, FOES, HURT, LIFTED, MASSED, METTLE, MOOT, OUTNUMBERED, QUINTESSENCE, ROUTED,
    SPEARS, TURNS, readings,
};
