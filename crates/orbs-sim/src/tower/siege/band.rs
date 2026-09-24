//! What is standing, what it intends, and what one round did to it (§5.1).
//!
//! The two sides of a siege and the shape of a turn — everything except the
//! [`Siege`](super::Siege) that holds them, which is [`battle`](super::battle).

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

/// How many troops the king assigns. Static by design.
///
/// The baseline is the same every time so the variables are legible: what the
/// enemy brought, what you brought, and how the dice fell.
pub const ASSIGNED: u32 = 6;

/// The smallest enemy that ever comes up the road.
///
/// The floor never moves, at any standing: fame lengthens the *tail* rather
/// than the whole band, so a player who has lost three fights is never handed a
/// fourth they cannot win (§11.5).
pub const FEWEST: u32 = 5;

/// The largest enemy a tower nobody has heard of will meet.
pub const BASE_MOST: u32 = 9;

/// The largest enemy anyone will ever meet.
///
/// A written ceiling rather than a drawn one: the garrison opens at
/// [`ASSIGNED`] six and `Siege::outnumbered` is `enemy >= garrison * 2`, so
/// twelve is exactly where that reading — the rung three solvers branch on —
/// becomes true at the opening.
pub const MOST: u32 = 12;

/// How many ranks of standing buy one more foe.
///
/// Ten ranks over three foes: [`BASE_MOST`] at nought, [`MOST`] at the top.
pub const RANKS_PER_FOE: usize = 3;

/// How much fight one troop has.
pub const VIGOUR: u32 = 3;

/// What a troop needs to roll to tell.
pub const AGAINST: i32 = 11;

/// Below this many troops, the garrison reads [`FEW`](super::FEW).
pub const THIN: u32 = 2;

/// At or below this share of its vigour, the garrison reads [`HURT`](super::HURT).
///
/// A half rather than a quarter: a warning that fires too late to act on is not
/// a warning, and a decision tree hangs its `quaff` rung on this.
pub const WEARY: u32 = 2;

/// How long the road stays empty after a siege, in ticks.
///
/// §11.5 fixes the siege provocation cadence at *"every 20–30 min at a normal
/// push rate"*, and at one tick a second twenty minutes is 1200. Until §5.3's
/// trace provokes them, this stands in for that gate.
///
/// The most load-bearing number in the domain. Without it `defend` is free and
/// unlimited: `orbs-balance` measured back-to-back sieges at 4.70 experience a
/// tick against clarity's 0.140, which says *ignore every other room*. The
/// escrow was not the thing to tune — a siege paying 105 for thirteen rounds is
/// right, fighting three hundred of them in two hours is not.
pub const CADENCE: u64 = 1200;

/// What a lost siege takes off the barrier.
///
/// Never ruinous, only slower (§11.5): larger than the menagerie's collapse
/// because a siege is a longer commitment, and still far short of
/// [`STANDING`](crate::tower::erosion::STANDING).
pub const DEFEAT_WEAR: u32 = 20;

/// How much a won siege puts back.
pub const VICTORY_MEND: u32 = 12;

/// What the enemy will do next round, announced a round ahead.
///
/// Telegraphing is the borrow, not determinism. Into the Breach declares exact
/// intent a turn ahead, turning the player's turn into prevention; here the
/// intent is telegraphed and the *outcome is rolled* — XCOM's bargain, under
/// its fairness rule of odds before the commitment and the roll after it.
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
            // A volley is fewer bodies, which is what makes it the intent you
            // can afford to eat while you fix something else.
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
    /// A volley is answered by nobody, which is why the three intents are a
    /// decision rather than three damage numbers: it is the cheap one to absorb
    /// and the one you can never trade against.
    #[must_use]
    pub const fn answered(self) -> bool {
        !matches!(self, Self::Volley)
    }

    /// Draw one.
    ///
    /// `pub(super)` because the split moved its two callers a file away, and not
    /// `pub` because a caller outside `siege` drawing an intent would advance
    /// the `Siege` stream and desynchronise every replay.
    pub(super) fn drawn(rngs: &mut Rngs) -> Self {
        use rand::Rng;
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
    /// Vigour is shared across the band and the count follows it, rather than
    /// each troop carrying its own pool: that keeps the board two numbers
    /// instead of a list, and §14 has to read it aloud.
    ///
    /// `pub(super)` since the split; see [`Intent::drawn`] for why not `pub`.
    pub(super) fn wound(&mut self, hits: u32) {
        self.vigour = self.vigour.saturating_sub(hits);
        // Ceiling division: a band with any vigour left has at least one
        // standing, and a band at nought vigour has nobody.
        self.count = self.count.min(self.vigour.div_ceil(VIGOUR));
    }

    /// Put `points` of fight back, never above `ceiling`, bringing troops with it.
    ///
    /// The cap has to come from outside. Capping at `self.count * VIGOUR` was
    /// the defect: [`wound`](Self::wound) derives `count` back *down* from
    /// `vigour`, so after any damage the cap is the band's current strength and
    /// a wounded line could be healed by at most two points — which made
    /// `succour` and the `mending` potion nearly inert.
    ///
    /// The count comes back with the vigour, capped at what the band was
    /// mustered at: healing that left troops down would read as thin for ever
    /// however much fight the band has.
    pub(super) fn mend(&mut self, points: u32, ceiling: u32) {
        self.vigour = (self.vigour + points).min(ceiling);
        self.count = self.vigour.div_ceil(VIGOUR).min(ceiling / VIGOUR);
    }
}

/// How a siege ended, if it has.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    /// The enemy broke.
    Held,
    /// The garrison broke.
    Fallen,
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
    /// Extra hits a sortie put on the enemy.
    ///
    /// Separate from [`dealt`](Self::dealt): a sortie is a thing the player
    /// *chose*, so the round has to say what the choice bought. Folded in, the
    /// sentence read *"take 2"* for a round that had dealt twelve.
    pub sortied: u32,
    /// What the sortie cost the garrison.
    pub spent: u32,
    /// Mettle a succour put back.
    pub mended: u32,
    /// Set if this round ended it.
    pub outcome: Option<Outcome>,
}
