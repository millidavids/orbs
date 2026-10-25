//! What a finished siege pays — §11.5's economy (§5.1).
//!
//! Not a combat rule but what the tower's economy pays for an episode, the rate
//! `orbs-balance` pins. Nothing here looks at a band, a die or an area.

use super::Outcome;

/// What a whole siege is worth before the completion fraction is applied.
///
/// Per foe, not flat, or the best play would be to abandon a hard siege.
pub const ESCROW_PER_FOE: u64 = 14;

/// The completion bonus a won siege adds, as a percentage.
pub const COMPLETION_BONUS: u64 = 50;

/// The floor a lost siege still pays, as a percentage.
///
/// §11.5: *"Escrow × completion fraction, floored at 20%."* A lost evening is
/// not a wasted one.
pub const ESCROW_FLOOR: u64 = 20;

/// What a finished siege pays: §11.5's table.
///
/// `more` is the Ley Line's `escrow` grant in percent, on the pool before
/// either branch, so a loss pays more under it too.
#[must_use]
pub fn escrow(arrived: u32, completion: u32, outcome: Outcome, more: u64) -> u64 {
    let pool = u64::from(arrived) * ESCROW_PER_FOE;
    let pool = pool + (pool * more) / 100;
    match outcome {
        Outcome::Held => pool + (pool * COMPLETION_BONUS) / 100,
        // A stilled siege keeps what the fight had earned, exactly as a loss
        // does, without the loss.
        Outcome::Fallen | Outcome::Stilled => {
            let share = u64::from(completion).max(ESCROW_FLOOR);
            (pool * share) / 100
        }
    }
}

/// What a finished siege is worth in **standing**, either way.
///
/// Derived, so a siege's standing is priced at the rate a making's is. The
/// stake runs 35 to 84 against a first rank at 25, which lets a bad night cost a
/// title.
pub const RENOWN_PER_FOE: u64 = ESCROW_PER_FOE / crate::tower::renown::RENOWN_PER;

/// What a finished siege moves standing by, and which way.
///
/// A defeat costs the stake scaled by how far short it fell. `div_ceil`, or a
/// loss at 99% would be free: a near miss costs less, never nothing.
#[must_use]
pub fn renown_stake(arrived: u32, completion: u32, outcome: Outcome) -> u64 {
    let stake = u64::from(arrived) * RENOWN_PER_FOE;
    match outcome {
        Outcome::Held => stake,
        Outcome::Fallen => (stake * u64::from(100 - completion.min(100))).div_ceil(100),
        // Neither won nor lost, so standing moves neither way.
        Outcome::Stilled => 0,
    }
}
