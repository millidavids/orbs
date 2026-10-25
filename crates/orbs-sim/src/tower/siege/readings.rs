//! The words the bailey publishes for a spell to ask about (§5.1).
//!
//! The arithmetic the language does not have: it has no ratios, so *twice the
//! defenders* is published as [`OUTNUMBERED`], the maze's pattern.
//!
//! A reading is nameable from every room, so most names here were chosen
//! against a collision, and the note says which — read it before renaming.

use super::{Intent, POOL};
use crate::tower::dice::Die;

/// The garrison is thin.
pub const FEW: &str = "few";
/// The garrison is wounded.
pub const HURT: &str = "hurt";
/// The enemy has at least twice the garrison's numbers.
pub const OUTNUMBERED: &str = "outnumbered";
/// The enemy is at full strength.
pub const MASSED: &str = "massed";
/// A side is broken.
pub const ROUTED: &str = "routed";
/// The best an area could come to once its dice are rolled — the count reading
/// too. Not `pledged`, whose prefix is the live verb `pledge`.
pub const CEILING: &str = "ceiling";
/// The coming intent will throw away anything pledged here. Not `wasted`: 667
/// against the potion `haste` and this domain's `massed`.
pub const MOOT: &str = "moot";
/// The siege has ended, either way.
///
/// What a solver's `repeat until` asks: `until the enemy has routed` looped for
/// ever on a loss. Not `over`, 600-plus against `deploy`'s synonym `order`.
pub const LIFTED: &str = "lifted";
/// How many of a side are standing. Not `troops`, which the material `troop`
/// prefix-matches, so `deploy troop` resolved to the reading.
pub const SPEARS: &str = "spears";
/// How much fight a side has left. Not `vigour`, an exact match for the secret
/// potion; `tests/naming.rs` sweeps materials now.
pub const METTLE: &str = "mettle";
/// How many of them there are.
pub const FOES: &str = "foes";
/// How many rounds have resolved.
pub const TURNS: &str = "turns";
/// What is left to spend on dice — and, on a die, what that die costs.
///
/// One word on both, so the coffer and a die compare the same reading. Not
/// §11.5's `mana` (750 against `many`) or `power` (800 against `tower`). Its cost
/// is that `qui` reaches this before `quickening-scroll`, which only `verify`
/// and `purge` can see.
pub const QUINTESSENCE: &str = "quintessence";
/// How likely a band's attacks are to tell, as a percentage.
///
/// The odds a person reads off the board, askable by a spell too (§5.1). Not
/// `peril`, whose `per` steals `peruse`'s abbreviation, nor `chance` (667
/// against `cancel`).
pub const AIM: &str = "aim";

/// Every reading the bailey publishes.
///
/// Declared by the fixture that owns it, never derived from `Role::Reading`,
/// which once had the lens teach the archive's words.
#[must_use]
pub fn readings() -> Vec<&'static str> {
    let mut out = vec![
        FEW,
        HURT,
        OUTNUMBERED,
        MASSED,
        ROUTED,
        LIFTED,
        SPEARS,
        METTLE,
        FOES,
        TURNS,
        CEILING,
        MOOT,
        // Appended, never inserted: a noun tie resolves to the first registered,
        // so an insertion re-resolves a name a solver already uses.
        QUINTESSENCE,
        AIM,
    ];
    out.extend(Intent::ALL.into_iter().map(Intent::word));
    // Each free die is published by name: `if the coffer has d20`. `POOL`, not
    // `Die::ALL`, whose other dice nothing publishes and which collide.
    out.extend(POOL.into_iter().map(Die::word));
    // The roster's: the standing garrison at its most (§19, *Supply*).
    out.push(super::FULL);
    out
}
