//! What a siege's dice cost (§11.5, §5.1).
//!
//! **The pool itself lives at [`tower::quintessence`](crate::tower::quintessence)
//! now, and that is a return rather than a reversal.** §11.5's resource table has
//! always read *"Mana — produced by passive regeneration, Ley Line steps"*; what
//! shipped at `0.8.15` was the siege's narrowing of it, a fixed pool granted on
//! `defend` and gone when the fight ended. Enchanting spends the same resource,
//! so the pool came up to the tower where the design had it and the siege became
//! one of two rooms drawing on it. §19 records the move.
//!
//! What is left here is the part that is genuinely about **dice**: what a face
//! count is worth in quintessence.
//!
//! # Nothing here draws
//!
//! Rule 3, stated where it would be easiest to break: this is arithmetic over a
//! `const` table. There is **no `RngStream` on any path in this file**.

use crate::tower::dice::Die;

/// How many faces one point of quintessence buys.
///
/// **The cost is derived from the die rather than authored per die**, so the
/// price is fair by construction at every die, including the four the arsenal
/// has not issued yet. Three hand-authored numbers could drift into an ordering
/// where the `d20` is cheaper than the `d6`; this cannot. It is one constant to
/// sweep rather than three to keep consistent.
///
/// Rule 6 is about **prose**, and CLAUDE.md exempts parser tables by name —
/// `Die::faces` is already a `const` table in Rust, and this is arithmetic over
/// it rather than authored content.
pub const FACES_PER_POINT: u32 = 4;

/// What pledging `die` costs.
///
/// **Never free.** A die that cost nothing would be one you always pledge, which
/// is the decision switched off for that row — so the floor is one point even for
/// a `d4`.
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

    /// **The tuning claim, as arithmetic rather than a comment.**
    ///
    /// A full allocation of the three shipped dice is eight a round, so a whole
    /// tower's ceiling is three unrestrained rounds of a siege that runs six to
    /// thirteen. That is the shape the decision wants: enough to matter every
    /// round, never enough to stop choosing.
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
