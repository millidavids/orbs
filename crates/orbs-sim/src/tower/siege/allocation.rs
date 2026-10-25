//! Where a die goes — the coffer, the four areas, and what a round's dice came
//! to (§5.1).
//!
//! Three dice against four areas, so the board can never be covered: choosing
//! what to leave dark *is* the turn.

use serde::{Deserialize, Serialize};

use super::Intent;
use crate::tower::dice::Die;

/// Where the wizard's dice wait between rounds.
pub const COFFER: &str = "coffer";

/// The dice a wizard starts a siege holding.
///
/// Different sizes, so the turn asks where variance is cheapest as well as
/// where strength goes: a `d20` on `sortie` may waste the round, a low roll on
/// `succour` only heals less.
pub const POOL: [Die; 3] = [Die::D6, Die::D8, Die::D20];

/// A part of the wall a die can be pledged to.
///
/// Each intent makes a different one urgent, which turns §5.1's telegraph into
/// the turn: a volley wastes the [`Line`](Self::Line), an onslaught makes the
/// [`Buckler`](Self::Buckler) worth most.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Area {
    /// Added to every attack the garrison makes; nothing against a volley.
    Line,
    /// Added to what the enemy must beat. Never wasted.
    Buckler,
    /// Mettle put back at the end of the round.
    ///
    /// Before the outcome is decided, so a good roll can stand a garrison
    /// wounded to nothing back up at one. Deliberate: it makes defence a real
    /// alternative to killing faster.
    Succour,
    /// Mettle spent for damage now — the one that can lose you the siege.
    Sortie,
}

impl Area {
    /// Every area, in the order the board draws them.
    pub const ALL: [Self; 4] = [Self::Line, Self::Buckler, Self::Succour, Self::Sortie];

    /// The word `pledge` takes and the board prints.
    ///
    /// Not `shield` (667 against `wield`) or `rally` (600 against the maze's
    /// `wall`).
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Line => "line",
            Self::Buckler => "buckler",
            Self::Succour => "succour",
            Self::Sortie => "sortie",
        }
    }

    /// Read one back from its word.
    #[must_use]
    pub fn named(word: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|area| area.word() == word)
    }

    /// Whether a die pledged here does anything under `intent` — so the board
    /// can warn before the commitment (§5.1).
    #[must_use]
    pub const fn answers(self, intent: Intent) -> bool {
        !matches!(self, Self::Line) || intent.answered()
    }
}

/// What each area came to once its dice were rolled.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Strengths {
    /// Added to every garrison attack.
    pub line: u32,
    /// Added to what the enemy must beat.
    pub buckler: u32,
    /// Mettle put back at the end of the round.
    pub succour: u32,
    /// Mettle spent for damage now.
    pub sortie: u32,
}

impl Strengths {
    /// One area's strength.
    #[must_use]
    pub const fn of(&self, area: Area) -> u32 {
        match area {
            Area::Line => self.line,
            Area::Buckler => self.buckler,
            Area::Succour => self.succour,
            Area::Sortie => self.sortie,
        }
    }

    /// One area's strength, to add to — for
    /// [`Siege::resolve`](super::Siege::resolve) and nothing outside `siege`.
    pub(super) const fn of_mut(&mut self, area: Area) -> &mut u32 {
        match area {
            Area::Line => &mut self.line,
            Area::Buckler => &mut self.buckler,
            Area::Succour => &mut self.succour,
            Area::Sortie => &mut self.sortie,
        }
    }
}

/// How much of a sortie's strength lands on the enemy.
///
/// Better than 1:1, which the smaller garrison could never afford, and still
/// paid in the mettle that keeps you alive.
pub const SORTIE_DEALT: u32 = 2;

/// How much of a sortie's strength the garrison pays.
pub const SORTIE_COST: u32 = 3;

/// One die, pledged to one area for the coming round.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pledge {
    /// Which die.
    pub die: Die,
    /// Where it is pledged.
    pub area: Area,
}
