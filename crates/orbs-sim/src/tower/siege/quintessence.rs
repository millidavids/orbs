//! What a siege's dice cost (§11.5, §5.1).
//!
//! The pool is the tower's, at
//! [`tower::quintessence`](crate::tower::quintessence), since enchanting spends
//! it too (§19). What is left here is what a face count is worth. Nothing here
//! draws.

use crate::tower::dice::Die;

/// How many faces one point of quintessence buys.
///
/// One constant over `Die::faces`, so no die can come to cost less than a
/// smaller one.
pub const FACES_PER_POINT: u32 = 4;

/// What pledging `die` costs — never free, or that row stops being a decision.
#[must_use]
pub const fn cost_of(die: Die) -> u32 {
    let cost = die.faces() / FACES_PER_POINT;
    if cost == 0 { 1 } else { cost }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tower::erosion::STANDING;
    use crate::tower::quintessence::{QUINTESSENCE_BASE, ceiling_for};
    use crate::tower::siege::POOL;

    #[test]
    fn a_bigger_die_costs_more_and_none_of_them_are_free() {
        let mut last = 0;
        for die in Die::ALL {
            let cost = cost_of(die);
            assert!(cost > 0, "{} was free to pledge", die.word());
            assert!(
                cost >= last,
                "{} costs less than the die below it",
                die.word(),
            );
            last = cost;
        }
    }

    /// The tuning claim, as arithmetic rather than a comment. A full allocation
    /// of the three shipped dice is eight a round, so a whole tower's ceiling is
    /// three unrestrained rounds of a siege that runs six to thirteen — enough
    /// to matter every round, never enough to stop choosing.
    #[test]
    fn the_ceiling_buys_about_three_unrestrained_rounds() {
        let round: u32 = POOL.into_iter().map(cost_of).sum();
        assert_eq!(round, 8, "a full allocation is not eight a round");
        assert_eq!(
            ceiling_for(STANDING, 0) / round,
            3,
            "a whole tower no longer holds three rounds' worth",
        );
        assert_eq!(QUINTESSENCE_BASE, 24, "the base moved without this moving");
    }

    /// A siege can always afford *something*, however worn the tower.
    ///
    /// The anti-spiral, at the one place it would actually bite: a floor that
    /// let the ceiling fall below the cheapest die would make a defeat able to
    /// leave the next fight unplayable rather than merely harder.
    #[test]
    fn even_a_ruined_tower_can_pledge_something() {
        let cheapest = POOL.into_iter().map(cost_of).min().expect("dice");
        assert!(
            ceiling_for(0, 0) >= cheapest,
            "a ruined tower cannot afford a single die",
        );
    }
}
