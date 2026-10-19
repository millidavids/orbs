//! The bailey's siege — a turn-based defence resolved on the dice (§10, §5.1).
//!
//! An unnamed king assigns the wizard a fixed contingent to hold his tower. An
//! enemy arrives; you spend what the arsenal has, and you `hold` to let a round
//! resolve. The enemy tells you what it will do next before it does it.
//!
//! There is no clock in it, which is the sanctum's position. §10 wants outcome
//! to follow *what the player chooses given readable state, never how fast or
//! precisely they act*, and §19 called the sanctum's clockless puzzle *"a better
//! position than a real-time mechanic with a reflex escape hatch bolted on"*.
//! Here too, the world advances when you end your turn. §10.1's one exception,
//! the menagerie, is not extended here.
//!
//! Turn-based serves both halves, which is the argument the first plan missed by
//! resting on accessibility alone: by hand it makes the board readable, and
//! scripted it makes the domain automatable at all. A spell gets a bounded
//! number of instructions per tick, and a real-time siege would outrun the
//! decision tree — evaluating a branch against a board that had already moved.
//! That is `PACE = 1` in the menagerie (§19).
//!
//! §5.0's *"no per-command tick cost"* is preserved: commands inside your turn
//! are free and instant, and the clock advances on [`Siege::resolve`], whose
//! only caller is [`hold`](crate::execute).
//!
//! It takes no production slot, and that is load-bearing. §19: *"a domain stands
//! alone; the tower-wide systems only enhance it."* A siege holding `CAPACITY`
//! would freeze the automation the enemy exists to attack, so the premise would
//! have nothing to attack. A bound brewing spell keeps running while you fight.
//!
//! The readings carry the arithmetic, not the language. *"If the enemy count is
//! twice that of defenders"* is not expressible — the spell language has
//! at-least / at-most / exactly and place-against-place, and no ratios — and
//! growing arithmetic would be the wrong fix. This is the maze's pattern
//! instead: the world publishes a derived word and the spell asks for it, the
//! way `spoil`, `exit` and `back` do. So the siege publishes [`OUTNUMBERED`],
//! [`FEW`] and [`HURT`].
//!
//! This was one 1,429-line file, kept that way on the grounds that every other
//! domain model is the same size. That argument was about `ward.rs` and
//! `maze.rs`, which hold one model each where this held five separable things.
//!
//! The submodules are private and everything below is re-exported, so these are
//! code spans rather than doc links — a `[`link`]` to a private module is a
//! rustdoc error under `-D warnings`.
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
//! `battle` is over the guideline and left that way deliberately: what is left
//! is one type's state, its readings and `resolve`, and cutting between them
//! would put `hurt`, `completion` and `coffer` a file away from the round that
//! changes all three. The line and the board were separable, and went.
//!
//! The re-exports below are the whole public surface and it did not change.
//! Every caller says `siege::SPEARS`, `siege::Area`, `siege::escrow` as before,
//! which is what made this safe at the end of a phase and is the property to
//! preserve if anything is added.
//!
//! What splitting cost is privacy: `Band::wound`, `Band::mend` and
//! `Intent::drawn` were private to one file and are `pub(super)` now. The
//! alternative was fully `pub`, putting a band's damage model in the tower's API.

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
