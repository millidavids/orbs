//! What a siege's dice cost (§11.5, §5.1).
//!
//! The pool lives at [`tower::quintessence`](crate::tower::quintessence) now, a
//! return rather than a reversal: §11.5's table always read *"Mana — produced by
//! passive regeneration, Ley Line steps"*, and what shipped at `0.8.15` was the
//! siege narrowing it to a fixed pool granted on `defend`. Enchanting spends the
//! same resource, so the pool came up to the tower (§19).
//!
//! What is left here is the part about dice: what a face count is worth.
//!
//! Nothing here draws — rule 3, stated where it would be easiest to break. This
//! is arithmetic over a `const` table, with no `RngStream` on any path.

use crate::tower::dice::Die;

/// How many faces one point of quintessence buys.
///
/// Derived from the die rather than authored per die, so the price is fair by
/// construction at every die including the four not yet issued. Three
/// hand-authored numbers could drift into an ordering where the `d20` is cheaper
/// than the `d6`; this cannot, and it is one constant to sweep rather than three
/// to keep consistent.
///
/// Rule 6 is about prose, and CLAUDE.md exempts parser tables by name:
/// `Die::faces` is already a `const` table, and this is arithmetic over it.
pub const FACES_PER_POINT: u32 = 4;

/// What pledging `die` costs.
///
/// Never free: a die that cost nothing is one you always pledge, which switches
/// the decision off for that row. So the floor is one point, even for a `d4`.
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
