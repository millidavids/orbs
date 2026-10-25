//! What is standing, what it intends, and what one round did to it (§5.1).
//!
//! The two sides and the shape of a turn; the [`Siege`](super::Siege) holding
//! them is in [`battle`](super::battle).

use serde::{Deserialize, Serialize};

use super::Strengths;
use crate::rng::{RngStream, Rngs};
use crate::tower::dice::Landed;

/// The room.
pub const BAILEY: &str = "bailey";
/// The fixture you watch from, and what the verbs anchor to.
pub const RAMPART: &str = "rampart";
/// Your side.
pub const GARRISON: &str = "garrison";
/// Theirs.
pub const ENEMY: &str = "enemy";

/// How many troops the king assigns: the standing garrison's floor (§19,
/// *Supply*). Above it the tower keeps what the menagerie brings.
pub const ASSIGNED: u32 = 6;

/// The smallest enemy that ever comes up the road.
///
/// Fixed at any standing: fame lengthens the tail, so a player who has lost
/// three fights is never handed a fourth they cannot win (§11.5).
pub const FEWEST: u32 = 5;

/// The largest enemy a tower nobody has heard of will meet.
pub const BASE_MOST: u32 = 9;

/// The largest enemy anyone will ever meet.
///
/// Twelve because `outnumbered` is `enemy >= garrison * 2` against an opening
/// garrison of six: exactly where the rung three solvers branch on comes true.
pub const MOST: u32 = 12;

/// How many ranks of standing buy one more foe: ten ranks over three foes,
/// [`BASE_MOST`] at nought and [`MOST`] at the top.
pub const RANKS_PER_FOE: usize = 3;

/// How much fight one troop has.
pub const VIGOUR: u32 = 3;

/// What a troop needs to roll to tell.
pub const AGAINST: i32 = 11;

/// Below this many troops, the garrison reads [`FEW`](super::FEW).
pub const THIN: u32 = 2;

/// At or below this share of its vigour, the garrison reads [`HURT`](super::HURT).
///
/// A half, because a warning that fires too late to act on is none, and a
/// decision tree hangs its `quaff` rung on it.
pub const WEARY: u32 = 2;

/// How long the road stays empty after a siege, in ticks.
///
/// §11.5's *every 20–30 min*, at one tick a second. The most load-bearing number
/// here: without it back-to-back sieges paid 4.70 experience a tick against
/// clarity's 0.140, which says *ignore every other room*.
pub const CADENCE: u64 = 1200;

/// What a lost siege takes off the barrier.
///
/// Never ruinous, only slower (§11.5): more than the menagerie's collapse, far
/// short of [`STANDING`](crate::tower::erosion::STANDING).
pub const DEFEAT_WEAR: u32 = 20;

/// How much a won siege puts back.
pub const VICTORY_MEND: u32 = 12;

/// What the enemy will do next round, announced a round ahead.
///
/// The intent is telegraphed and the outcome rolled: odds before the commitment,
/// the roll after it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Intent {
    /// Close on the wall. Ordinary damage, ordinary numbers.
    Advance,
    /// Everything at once. More attackers roll, and they roll harder.
    Onslaught,
    /// Shoot from range. Fewer attackers, but the garrison cannot answer.
    Volley,
}

impl Intent {
    /// Every intent.
    pub const ALL: [Self; 3] = [Self::Advance, Self::Onslaught, Self::Volley];

    /// The word a spell asks for and the board draws.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Advance => "advance",
            Self::Onslaught => "onslaught",
            Self::Volley => "volley",
        }
    }

    /// How many of the enemy roll under this intent, out of `count`.
    #[must_use]
    pub const fn attackers(self, count: u32) -> u32 {
        match self {
            Self::Advance => count,
            Self::Onslaught => count,
            // Fewer bodies: the intent you can afford to eat while fixing
            // something else.
            Self::Volley => count.div_ceil(2),
        }
    }

    /// What the enemy adds to each roll under this intent.
    #[must_use]
    pub const fn edge(self) -> i32 {
        match self {
            Self::Advance => 0,
            Self::Onslaught => 2,
            Self::Volley => 0,
        }
    }

    /// Whether the garrison strikes back this round.
    ///
    /// Nobody answers a volley, which makes the three intents a decision rather
    /// than three damage numbers.
    #[must_use]
    pub const fn answered(self) -> bool {
        !matches!(self, Self::Volley)
    }

    /// Draw one.
    ///
    /// Not `pub`: a caller outside `siege` drawing an intent would advance the
    /// `Siege` stream and desynchronise every replay.
    pub(super) fn drawn(rngs: &mut Rngs) -> Self {
        use rand::RngExt;
        let roll = rngs
            .stream(RngStream::Siege)
            .random_range(0..Self::ALL.len());
        Self::ALL[roll]
    }
}

/// One side of a siege.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Band {
    /// How many are standing.
    pub count: u32,
    /// How much fight the standing ones have between them.
    pub vigour: u32,
}

impl Band {
    /// A band at full strength.
    #[must_use]
    pub const fn new(count: u32) -> Self {
        Self {
            count,
            vigour: count * VIGOUR,
        }
    }

    /// Whether it is broken.
    #[must_use]
    pub const fn routed(self) -> bool {
        self.count == 0
    }

    /// Take `hits` off, dropping troops as vigour runs out.
    ///
    /// Vigour is shared and the count follows it, so the board is two numbers
    /// §14 can read aloud rather than a list.
    pub(super) fn wound(&mut self, hits: u32) {
        self.vigour = self.vigour.saturating_sub(hits);
        // Any vigour left is at least one standing.
        self.count = self.count.min(self.vigour.div_ceil(VIGOUR));
    }

    /// Put `points` of fight back, never above `ceiling`, bringing troops with it.
    ///
    /// The cap comes from outside: [`wound`](Self::wound) derives `count` down
    /// from `vigour`, so a cap of `count * VIGOUR` let a wounded line heal by two
    /// points at most. Troops return with the vigour, never past `bodies` (§19,
    /// *Supply*) and never fewer than it had — an older save's line can stand
    /// above today's most, and a heal must not kill.
    pub(super) fn mend(&mut self, points: u32, ceiling: u32, bodies: u32) {
        self.vigour = (self.vigour + points).min(ceiling);
        let mended = self
            .vigour
            .div_ceil(VIGOUR)
            .min(ceiling / VIGOUR)
            .min(bodies);
        self.count = self.count.max(mended);
    }
}

/// How a siege ended, if it has.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    /// The enemy broke.
    Held,
    /// The garrison broke.
    Fallen,
    /// Stillness ended it with both sides standing (§19).
    Stilled,
}

/// What one resolved round did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Round {
    /// What the enemy did.
    pub intent: Intent,
    /// Every roll the enemy made.
    pub struck: Vec<Landed>,
    /// Every roll the garrison made. Empty on a volley.
    pub answered: Vec<Landed>,
    /// Hits the garrison took.
    pub taken: u32,
    /// Hits the enemy took from the garrison swinging.
    pub dealt: u32,
    /// What each area came to once its dice were rolled.
    pub strengths: Strengths,
    /// Extra hits a sortie put on the enemy — apart from `dealt`, so the round
    /// says what the player's choice bought.
    pub sortied: u32,
    /// What the sortie cost the garrison.
    pub spent: u32,
    /// Mettle a succour put back.
    pub mended: u32,
    /// Set if this round ended it.
    pub outcome: Option<Outcome>,
}
