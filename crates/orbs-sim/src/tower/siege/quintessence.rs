//! What a siege has to spend on its dice (§11.5, §5.1).
//!
//! **§11.5's mana, built at last, under a name that survives the sweep.** The
//! design has always said *"a fixed pool granted on entry, with no
//! regeneration"*; what it lacked was anything to spend it on. Pledging is that
//! thing.
//!
//! # No regeneration, and the reason is §14 rather than balance
//!
//! §11.5 fixes this and the argument is not about difficulty: §14's
//! screen-reader mode advances siege ticks on **player input**, so a per-tick
//! regeneration would mean *more typing produces more quintessence* — inverting
//! the economy for exactly the players that mode exists to serve. A fixed pool
//! gives identical quintessence-per-decision whether ticks come from a clock or
//! a keystroke.
//!
//! So the pool is decided once, at [`defend`](crate::execute), and only ever
//! falls. **Declining to pledge costs nothing**, which is what makes *not*
//! spending a move rather than an omission.
//!
//! # Nothing here draws
//!
//! Rule 3, stated where it would be easiest to break: this module is integer
//! arithmetic over two values that are themselves replayed state — the tower's
//! integrity and how far up the ley line the player has come. There is **no
//! `RngStream` on any path in this file**, and adding one would make the size of
//! a siege's pool depend on the stream position rather than on the world.
//!
//! # Why the pool is not simply flat
//!
//! §19 deferred *"the integrity → nuisance-rate coupling"* to Phase 8 and it was
//! never taken, so a worn tower has cost the player nothing but a taller course.
//! This is that coupling, arrived at from the other side: **integrity buys you
//! dice.** Keeping the sanctum solved is what lets you gamble on the wall.
//!
//! [`FLOOR_PCT`] is what stops it spiralling. §11.5 requires a loss to be *"never
//! ruinous, only slower"*, and a pool that fell to nothing would make the siege
//! after a bad one unwinnable by arithmetic rather than by play.

use super::super::dice::Die;
use super::super::erosion::STANDING;

/// Faces a die must have per point of quintessence it costs.
///
/// # Derived from the die rather than authored per die, and that is a change
///
/// The plan called for a `[dice]` table in `siege.toml`. That file is
/// `#[serde(transparent)]` over a flat name → `Spendable` map
/// (`content/siege.rs:48`), so a nested table would parse as an arsenal item
/// missing its `verb` and fail the load — and a whole new content file for three
/// numbers is a heavier answer than the question deserves.
///
/// **A divisor is also the better shape.** A die's expected contribution is
/// `(faces + 1) / 2`, so a cost proportional to *faces* is a cost proportional to
/// what the die is worth — the price is fair by construction at every die,
/// including the four the arsenal has not issued yet. Three hand-authored numbers
/// could drift into an ordering where the `d20` is cheaper than the `d6`; this
/// cannot. It is one constant to sweep rather than three to keep consistent.
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

/// The pool a tower at full [`STANDING`] grants, before the ley line.
///
/// **A first-pass number and nothing more.** Against the costs in `siege.toml`
/// (`d6` 1, `d8` 2, `d20` 5) a full allocation is eight a round, so twenty-four
/// buys three unrestrained rounds of a siege that runs six to thirteen — which
/// is the shape the decision wants: enough to matter every round, never enough
/// to stop choosing. `orbs-balance` is the instrument that settles it, and until
/// it has been run this is arithmetic rather than a measurement.
pub const QUINTESSENCE_BASE: u32 = 24;

/// What one ley-line step granting quintessence adds to the base.
pub const PER_LEY_STEP: u32 = 6;

/// The share of the base a tower worn to nothing still grants, as a percentage.
///
/// **The anti-spiral, and the reason this is a floor rather than a fraction.** A
/// lost siege takes [`DEFEAT_WEAR`](super::DEFEAT_WEAR) off the barrier, so an
/// unfloored pool would make each defeat cheapen the next fight until it could
/// not be won at all — punishment compounding into a dead end, which §11.5
/// forbids: *"never ruinous, only slower"*.
///
/// At a half, neglect is expensive and never fatal.
pub const FLOOR_PCT: u32 = 50;

/// What a siege opens with, given the tower and the ley line.
///
/// Integrity scales the whole pool between [`FLOOR_PCT`] and full; each ley step
/// raises what is being scaled. **Integer throughout** — a percentage of a small
/// number is exact here and a float would be one more thing replay has to trust.
#[must_use]
pub fn pool_for(integrity: u32, steps: usize) -> u32 {
    let steps = u32::try_from(steps).unwrap_or(u32::MAX);
    let base = QUINTESSENCE_BASE.saturating_add(steps.saturating_mul(PER_LEY_STEP));
    // Clamped, because `Integrity::get` is capped at `STANDING` and a value past
    // it would scale the pool *above* the base rather than up to it.
    let standing = integrity.min(STANDING);
    let scale = FLOOR_PCT + (100 - FLOOR_PCT) * standing / STANDING;
    base.saturating_mul(scale) / 100
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The two ends and the floor**, which is the whole of what the curve
    /// promises: a kept tower gets everything, a ruined one gets half, and the
    /// half is a floor rather than a limit approached.
    #[test]
    fn a_kept_tower_grants_the_whole_pool_and_a_ruined_one_grants_the_floor() {
        assert_eq!(pool_for(STANDING, 0), QUINTESSENCE_BASE);
        assert_eq!(
            pool_for(0, 0),
            QUINTESSENCE_BASE * FLOOR_PCT / 100,
            "a worn tower fell past the floor",
        );
    }

    /// **Never nought, at any integrity.** A siege you cannot pledge in at all is
    /// not a harder siege, it is a siege with the mechanic switched off — and it
    /// would arrive exactly when the player is already losing.
    #[test]
    fn a_siege_can_always_pledge_something() {
        for integrity in 0..=STANDING {
            assert!(
                pool_for(integrity, 0) > 0,
                "integrity {integrity} left nothing to pledge",
            );
        }
    }

    /// Monotonic: repairing the tower never lowers the pool.
    #[test]
    fn a_better_kept_tower_is_never_worse_off() {
        let mut last = 0;
        for integrity in 0..=STANDING {
            let pool = pool_for(integrity, 0);
            assert!(
                pool >= last,
                "integrity {integrity} granted less than {}",
                integrity - 1,
            );
            last = pool;
        }
        assert!(last > pool_for(0, 0), "integrity bought nothing at all");
    }

    /// A ley step is worth something at every integrity, including nought —
    /// otherwise the weave would sell a node that a worn tower cannot use.
    #[test]
    fn a_ley_step_is_worth_having_however_worn_the_tower_is() {
        for integrity in [0, 1, STANDING / 2, STANDING] {
            assert!(
                pool_for(integrity, 1) > pool_for(integrity, 0),
                "a ley step bought nothing at integrity {integrity}",
            );
        }
    }

    /// **Saturating rather than panicking.** `steps` comes from content and
    /// `integrity` from a save; neither should be able to make this arithmetic
    /// overflow in a debug build.
    #[test]
    fn absurd_inputs_do_not_overflow() {
        let _ = pool_for(u32::MAX, usize::MAX);
        let _ = pool_for(0, usize::MAX);
    }

    /// **A bigger die always costs more, and nothing is free.** Both halves are
    /// the decision: a free die is a row you never think about, and a
    /// non-monotonic table would make the `d20` the thrifty choice.
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

    /// The three the wizard actually holds, pinned — this is the table a player
    /// reads off the board, and the numbers the balance sweep will move.
    #[test]
    fn the_pool_buys_about_three_unrestrained_rounds() {
        let round: u32 = super::super::POOL.into_iter().map(cost_of).sum();
        assert_eq!(round, 8, "a full allocation is not eight a round");
        let pool = pool_for(STANDING, 0);
        assert_eq!(
            pool / round,
            3,
            "a kept tower does not buy three full rounds — {pool} against {round}",
        );
    }
}
