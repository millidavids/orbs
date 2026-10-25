//! The bailey's siege — a turn-based defence resolved on the dice (§10, §5.1).
//!
//! A king assigns the wizard a contingent; an enemy arrives and says what it
//! will do next; you spend what the arsenal has and `hold` to resolve a round.
//!
//! Turn-based, with no clock: by hand the board is readable, and scripted the
//! decision tree is not outrun by a board that has moved. The clock advances
//! only in [`Siege::resolve`], whose one caller is [`hold`](crate::execute).
//!
//! It takes no production slot, and that is load-bearing: a siege holding the
//! slot would freeze the automation the enemy exists to attack (§19, *a domain
//! stands alone*). The readings carry the arithmetic the language lacks —
//! [`OUTNUMBERED`], [`FEW`], [`HURT`].
//!
//! The submodules are private and re-exported below, so they are code spans
//! here: a link to a private module fails rustdoc under `-D warnings`.
//!
//! | File | What it is |
//! |---|---|
//! | `allocation` | Where a die goes. The dice, the four areas, what a round's dice came to |
//! | `band` | What is standing, what it intends, and what one round did to it |
//! | `battle` | [`Siege`] itself: the state, the readings' arithmetic, and `resolve` |
//! | `line` | The line on the wall: the levy, and what a supply does to its room between rounds |
//! | `board` | What the rampart's board draws |
//! | `garrison` | The tower's standing garrison, at home and in reserve (§19, *Supply*) |
//! | `escrow` | §11.5's economy. Not the siege's combat rules, which is the seam |
//! | `quintessence` | What a siege has to spend on its dice, and what the tower's repair buys it |
//! | `readings` | The words the domain publishes for a spell to ask about |
//! | `standing` | How big a siege arrives, and what `petition` buying it down costs |
//!
//! `battle` is over the length guideline on purpose: cutting it would put
//! `hurt`, `completion` and `coffer` a file away from the round that changes
//! them. The split's cost is that `Band::wound`, `Band::mend` and
//! `Intent::drawn` are `pub(super)` rather than private.

mod allocation;
mod band;
mod battle;
mod board;
mod escrow;
mod garrison;
mod line;
mod quintessence;
mod readings;
mod standing;

#[cfg(test)]
mod tests;

pub use allocation::{Area, COFFER, POOL, Pledge, SORTIE_COST, SORTIE_DEALT, Strengths};
pub use band::{
    AGAINST, ASSIGNED, BAILEY, BASE_MOST, Band, CADENCE, DEFEAT_WEAR, ENEMY, FEWEST, GARRISON,
    Intent, MOST, Outcome, RAMPART, RANKS_PER_FOE, Round, THIN, VICTORY_MEND, VIGOUR, WEARY,
};
pub use battle::{Pledged, Siege, besieged};
pub use escrow::{
    COMPLETION_BONUS, ESCROW_FLOOR, ESCROW_PER_FOE, RENOWN_PER_FOE, escrow, renown_stake,
};
pub use garrison::{
    DESERT_EVERY, FULL, GARRISON_FLOOR, GARRISON_MOST, Garrison, ROSTER, desert, disband, enlist,
    held, levy, most_held, roll_call,
};
pub use quintessence::{FACES_PER_POINT, cost_of};
pub use readings::{
    AIM, CEILING, FEW, FOES, HURT, LIFTED, MASSED, METTLE, MOOT, OUTNUMBERED, QUINTESSENCE, ROUTED,
    SPEARS, TURNS, readings,
};
pub use standing::{PETITION_PER_FOE, Petitioned, most_at};
