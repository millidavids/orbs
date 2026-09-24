//! What a finished siege pays — §11.5's economy (§5.1).
//!
//! Its own file although it is forty lines: escrow is not a combat rule but
//! what the *tower's* economy pays for an episode, and the rate `orbs-balance`
//! pins. Nothing here looks at a band, a die or an area.

use super::Outcome;

/// What a whole siege is worth before the completion fraction is applied.
///
/// The escrow pool (§11.5). It scales with what arrived rather than being flat,
/// or the best play would be to abandon a hard siege and wait for an easy one.
pub const ESCROW_PER_FOE: u64 = 14;

/// The completion bonus a won siege adds, as a percentage.
pub const COMPLETION_BONUS: u64 = 50;

/// The floor a lost siege still pays, as a percentage.
///
/// §11.5: *"Escrow × completion fraction, floored at 20%."* A lost evening is
/// not a wasted one.
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
/// Derived, not picked: `ESCROW_PER_FOE / renown::RENOWN_PER` — a foe's worth
/// in experience over experience's worth in renown. So a siege's standing is
/// priced at exactly the rate a *making's* is, and the two cannot drift apart
/// without someone moving one of the two numbers.
///
/// Per foe rather than flat, for [`ESCROW_PER_FOE`]'s reason. `arrived` runs
/// [`FEWEST`](super::FEWEST)..=[`MOST`](super::MOST), so the stake is 35 to 84
/// against a first rank at 25 — which is what lets a bad night cost a title.
pub const RENOWN_PER_FOE: u64 = ESCROW_PER_FOE / crate::tower::renown::RENOWN_PER;

/// What a finished siege moves standing by, and which way.
///
/// A win earns the whole stake; a defeat pays it back scaled by how far short
/// it fell, so a collapse on the first round costs far more standing than a
/// wall carried at ninety percent — escrow's completion scaling, the other way.
///
/// `div_ceil` because the stake tops out at 84, so plain division by 100
/// truncates to nought for every completion above 88 and losing at ninety-nine
/// percent would be free. A near miss should cost *less*, never *nothing*.
///
/// A win needs no scaling: `Outcome::Held` is set only when the enemy is
/// routed, so `completion` is 100 by construction.
#[must_use]
pub fn renown_stake(arrived: u32, completion: u32, outcome: Outcome) -> u64 {
    let stake = u64::from(arrived) * RENOWN_PER_FOE;
    match outcome {
        Outcome::Held => stake,
        Outcome::Fallen => (stake * u64::from(100 - completion.min(100))).div_ceil(100),
    }
}
