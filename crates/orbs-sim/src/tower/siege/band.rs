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

/// How many troops the king assigns. **Static, and that is the design.**
///
/// The baseline is the same every time so the variables are legible: what the
/// enemy brought, what you brought, and how the dice fell. A contingent that
/// scaled would blur all three together.
pub const ASSIGNED: u32 = 6;

/// The smallest enemy that ever comes up the road.
///
/// **The floor never moves, at any standing.** A famous tower can still draw a
/// quiet night; what fame lengthens is the *tail*, not the whole band — so a
/// player who has just lost three fights is never handed a fourth they cannot
/// win, which is §11.5's *"never ruinous, only slower"* at the scale of one
/// evening.
pub const FEWEST: u32 = 5;

/// The largest enemy a tower nobody has heard of will meet.
pub const BASE_MOST: u32 = 9;

/// The largest enemy anyone will ever meet.
///
/// **Twelve, and it is a written ceiling rather than a drawn one.** The garrison
/// opens at [`ASSIGNED`] six and `Siege::outnumbered` is a *ratio* —
/// `enemy >= garrison * 2` — so twelve is exactly where that reading becomes true
/// at the opening. A curve that ran past it would put every fight above the rung
/// three shipped solvers branch on and stop the reading meaning anything, and one
/// that stopped short would leave `outnumbered` unreachable except after the
/// garrison is thinned. Twelve is the number that makes it a *decision*.
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
/// A half rather than a quarter: a warning that fires only when it is too late
/// to act on is not a warning, and this is the reading a decision tree hangs its
/// `quaff` rung on.
pub const WEARY: u32 = 2;

/// How long the road stays empty after a siege, in ticks.
///
/// §11.5 fixes the **siege provocation cadence** at *"every 20–30 min at a
/// normal push rate"*, and at one tick a second twenty minutes is 1200. Until
/// §5.3's trace actually provokes them, this is what stands in for that gate.
///
/// # It is the single most load-bearing number in the domain
///
/// Without it `defend` is free and unlimited, and `orbs-balance` measured the
/// consequence exactly: a driver fighting sieges back to back reads **4.70
/// experience a tick**, against clarity's 0.140 and scrying's 0.268. That is
/// thirty-three times the flagship, and it says *ignore every other room* —
/// which is the opposite of what a capstone should say about the six domains
/// that feed it.
///
/// **The escrow was not the thing to tune, and that was the first diagnosis.**
/// A siege paying 105 for thirteen rounds is right; fighting three hundred of
/// them in two hours is not. Cutting the reward would have made each siege feel
/// worthless *and* left the exploit — the fix belongs on how often, not on how
/// much.
pub const CADENCE: u64 = 1200;

/// What a lost siege takes off the barrier.
///
/// **Never ruinous, only slower** (§11.5). It is larger than the menagerie's
/// collapse because a siege is a longer commitment, and still far short of
/// [`STANDING`](crate::tower::erosion::STANDING).
pub const DEFEAT_WEAR: u32 = 20;

/// How much a won siege puts back.
pub const VICTORY_MEND: u32 = 12;

/// What the enemy will do next round, announced a round ahead.
///
/// # Telegraphing is the borrow, not determinism
///
/// Into the Breach declares exact enemy intent a turn ahead, which turns the
/// player's turn into *prevention* rather than reaction. Here the intent is
/// telegraphed and the **outcome is rolled** — the XCOM bargain rather than the
/// ITB one — and the rule that keeps it fair is XCOM's: **show the odds before
/// the commitment and the roll after it.** A decision under known risk is a
/// decision; a surprise is not.
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
    /// **A volley is answered by nobody**, which is the whole reason the three
    /// intents are a decision rather than three damage numbers: it is the cheap
    /// one to absorb and the one you can never trade against.
    #[must_use]
    pub const fn answered(self) -> bool {
        !matches!(self, Self::Volley)
    }

    /// Draw one.
    ///
    /// **`pub(super)`, not private, because the split moved its two callers a
    /// file away** — `Siege::begin` and `Siege::resolve`. It is not `pub`: a
    /// caller outside `siege` drawing an intent would advance the `Siege` stream
    /// and desynchronise every replay, which is precisely the thing rule 3
    /// exists to make impossible.
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
    /// **Vigour is shared across the band and the count follows it**, rather
    /// than each troop carrying its own pool. That keeps the board two numbers
    /// instead of a list, which is what makes it readable at a glance and
    /// askable in one `if` — and §14 has to read it aloud.
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
    /// # The cap has to come from outside, and this was the defect
    ///
    /// It capped at `self.count * VIGOUR` — and [`wound`](Self::wound) derives
    /// `count` back *down* from `vigour`, so after any damage the two are within
    /// `VIGOUR - 1` of each other and the cap is **the band's current strength**.
    /// A wounded line could be healed by at most two points, whatever it was
    /// given.
    ///
    /// That made `succour` nearly inert and — quietly, for longer — the
    /// `mending` potion too: `every_authored_arsenal_row_changes_the_siege`
    /// passed because the number *moved*, by one or two out of six. The dice
    /// work is what surfaced it, with `the succour rolled 4 and put back 0` on
    /// screen.
    ///
    /// **The count comes back with the vigour**, capped at what the band was
    /// mustered at. Healing that left troops down would be the same trap one
    /// field over: the band would read as thin for ever however much fight it
    /// had.
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
    /// **Separate from [`dealt`](Self::dealt) rather than folded into it.** A
    /// sortie is a thing the player *chose*, so the round has to say what the
    /// choice bought — folded in, the one visible effect of the one area that
    /// can lose you the siege was invisible, and the sentence read *"take 2"*
    /// for a round that had actually dealt twelve.
    pub sortied: u32,
    /// What the sortie cost the garrison.
    pub spent: u32,
    /// Mettle a succour put back.
    pub mended: u32,
    /// Set if this round ended it.
    pub outcome: Option<Outcome>,
}
