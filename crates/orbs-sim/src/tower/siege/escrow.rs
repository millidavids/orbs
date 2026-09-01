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
#[must_use]
pub fn escrow(arrived: u32, completion: u32, outcome: Outcome) -> u64 {
    let pool = u64::from(arrived) * ESCROW_PER_FOE;
    match outcome {
        Outcome::Held => pool + (pool * COMPLETION_BONUS) / 100,
        Outcome::Fallen => {
            let share = u64::from(completion).max(ESCROW_FLOOR);
            (pool * share) / 100
        }
    }
}
