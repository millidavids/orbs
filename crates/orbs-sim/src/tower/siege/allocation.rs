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
/// Three against four areas, so one part of the wall gets nothing every round.
///
/// Different sizes, deliberately: a `d20` pledged to `sortie` might roll 2 and
/// waste the round, where a low roll on `succour` merely heals less. A set
/// rather than three of a kind asks *where variance is cheapest* as well as
/// where the strength goes.
pub const POOL: [Die; 3] = [Die::D6, Die::D8, Die::D20];

/// A part of the wall a die can be pledged to.
///
/// Four, and each intent makes a different one urgent — which is what turns
/// §5.1's telegraph into the thing the turn is spent on. A volley cannot be
/// answered, so [`Line`](Self::Line) is wasted against one; an onslaught is
/// everyone at +2, so [`Buckler`](Self::Buckler) is worth most.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Area {
    /// Added to every attack the garrison makes.
    ///
    /// Worth nothing against a volley — the garrison does not swing.
    Line,
    /// Added to what the enemy must beat.
    ///
    /// The only area that is never wasted — they attack on every intent.
    Buckler,
    /// Mettle put back at the end of the round.
    ///
    /// Resolves *before* the outcome is decided, so a good roll can save a
    /// siege that was otherwise lost: `Band::mend` brings troops back with the
    /// fight, and a garrison wounded to nothing stands again at one. Deliberate
    /// — it is what makes defence a real alternative to killing faster — and
    /// the manual says so too, after a review asked whether it was a bug.
    Succour,
    /// Mettle spent for damage now.
    ///
    /// The one that can lose you the siege, and why there are four areas rather
    /// than three: an option you can rarely afford.
    Sortie,
}

impl Area {
    /// Every area, in the order the board draws them.
    pub const ALL: [Self; 4] = [Self::Line, Self::Buckler, Self::Succour, Self::Sortie];

    /// The word `pledge` takes and the board prints.
    ///
    /// `buckler` and `succour`, not the obvious `shield` and `rally`: `shield`
    /// scores 667 against the live verb `wield`, `rally` 600 against the maze's
    /// `wall`. Both are in register for a game with an `athanor` in it.
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

    /// Whether a die pledged here does anything under `intent`.
    ///
    /// Only the line is ever wasted, and saying so lets the board warn before
    /// the commitment — §5.1's fairness rule applied to allocation.
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

    /// One area's strength, to add to.
    ///
    /// `pub(super)` because [`Siege::resolve`](super::Siege::resolve) is a file
    /// away now. Wider than `siege` would put a write handle on a round's
    /// arithmetic in the tower's API.
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
/// Favourable, but paid in the resource that keeps you alive. The garrison has
/// less mettle than the enemy, so a 1:1 trade would never be worth pledging
/// to; this ratio is efficient and still expensive.
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
