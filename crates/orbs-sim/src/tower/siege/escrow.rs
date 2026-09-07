//! What a finished siege pays — §11.5's economy (§5.1).
//!
//! **This is the seam the old single-file module argued was not worth taking,
//! and it was the one real one.** Escrow is not a combat rule: it is what the
//! *tower's* economy pays for an episode, it is the thing `orbs-balance` pins a
//! rate against, and none of it looks at a band, a die or an area. Its being
//! forty lines is an argument about size rather than about cohesion, and
//! cohesion is what CLAUDE.md's rule actually tests.

use super::Outcome;

/// What a whole siege is worth before the completion fraction is applied.
///
/// **The escrow pool** (§11.5). It scales with what arrived rather than being
/// flat, so a bigger enemy is worth more — otherwise the best play would be to
/// abandon a hard siege and wait for an easy one.
pub const ESCROW_PER_FOE: u64 = 14;

/// The completion bonus a won siege adds, as a percentage.
pub const COMPLETION_BONUS: u64 = 50;

/// The floor a lost siege still pays, as a percentage.
///
/// §11.5: *"Escrow × completion fraction, floored at 20%."* This is what stops a
/// lost evening being a wasted one — *"effort is never wasted; only cynicism
/// is"*.
pub const ESCROW_FLOOR: u64 = 20;

/// What a finished siege pays.
///
/// §11.5's table, exactly: a win is the full pool plus half again; a loss is the
/// pool scaled by how far you got, and never below [`ESCROW_FLOOR`] of it.
/// `more` is the Ley Line's `escrow` grant in percent, applied to the pool
/// before either branch, so a loss under it pays more too.
#[must_use]
pub fn escrow(arrived: u32, completion: u32, outcome: Outcome, more: u64) -> u64 {
    let pool = u64::from(arrived) * ESCROW_PER_FOE;
    let pool = pool + (pool * more) / 100;
    match outcome {
        Outcome::Held => pool + (pool * COMPLETION_BONUS) / 100,
        Outcome::Fallen => {
            let share = u64::from(completion).max(ESCROW_FLOOR);
            (pool * share) / 100
        }
    }
}

/// What a finished siege is worth in **standing**, either way.
///
/// # The rate is derived, not picked
///
/// `ESCROW_PER_FOE / renown::RENOWN_PER` — what a foe is worth in experience,
/// divided by what experience is worth in renown. A siege's standing is
/// therefore priced at exactly the rate a *making's* standing is, and the two
/// cannot drift apart without someone moving one of the two numbers it is built
/// from.
///
/// **It is per foe rather than flat** for [`ESCROW_PER_FOE`]'s reason: a bigger
/// enemy is worth more, or the best play is to duck the hard siege and wait for
/// an easy one. `arrived` runs [`FEWEST`](super::FEWEST)..=[`MOST`](super::MOST)
/// — 5 to 12, since standing lengthens the tail — so this is a stake of 35 to
/// **84**, against a first rank at 25. That is what makes a bad night able to
/// cost a title; at one per foe the most a defeat could ever take is twelve, and
/// the box this implements would have built a number that cannot fall.
pub const RENOWN_PER_FOE: u64 = ESCROW_PER_FOE / crate::tower::renown::RENOWN_PER;

/// What a finished siege moves standing by, and which way.
///
/// A win earns the whole stake; a defeat pays it back scaled by **how far short
/// it fell**, so a collapse on the first round costs far more standing than a
/// wall carried at ninety percent. That is escrow's own completion scaling,
/// pointed the other way.
///
/// # `div_ceil`, and it is not a rounding nicety
///
/// `arrived` is at most [`MOST`](super::MOST), so the stake tops out at 84 and
/// plain division by 100 truncates **to nought** for every completion above 88 —
/// losing at ninety-nine percent would cost nothing at all, which says the
/// near-miss was free. A loss that close should cost *less*, never *nothing*, so
/// the fraction rounds up and any defeat short of the enemy breaking costs at
/// least one.
///
/// A win needs no scaling: `completion` is `felled * 100 / arrived` and
/// `Outcome::Held` is set only when the enemy is routed, so it is 100 by
/// construction.
#[must_use]
pub fn renown_stake(arrived: u32, completion: u32, outcome: Outcome) -> u64 {
    let stake = u64::from(arrived) * RENOWN_PER_FOE;
    match outcome {
        Outcome::Held => stake,
        Outcome::Fallen => (stake * u64::from(100 - completion.min(100))).div_ceil(100),
    }
}
