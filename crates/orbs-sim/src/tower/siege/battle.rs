//! [`Siege`] itself — the state, the readings' arithmetic, and `resolve` (§5.1).
//!
//! **The one thing here that advances the world.** Everything else in the domain
//! is instant; [`Siege::resolve`] is what `hold` calls, and §5.0's *"no
//! per-command tick cost"* rests on it being the only one.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use super::{
    AGAINST, ASSIGNED, Area, Band, ENEMY, GARRISON, Intent, Outcome, POOL, Pledge, Round,
    SORTIE_COST, SORTIE_DEALT, Strengths, THIN, VIGOUR,
};
use crate::rng::{RngStream, Rngs};
use crate::tower::dice::{Die, Effect, Landed, Modifier, Roll};

/// What happened when a die was pledged.
///
/// **Three answers rather than a `bool`**, because the two failures need
/// different sentences: a die already behind something is a mistake the player
/// can see on the board, and a die they cannot pay for is the decision the
/// domain is about. Collapsing them would make the refusal say the wrong thing
/// half the time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pledged {
    /// It went down, at this cost.
    Made {
        /// What it took out of the pool.
        cost: u32,
    },
    /// That die is already pledged somewhere.
    Spent,
    /// There is not quintessence enough for it.
    Short {
        /// What it would have cost.
        cost: u32,
    },
}

/// A siege in progress.
///
/// **A component on the rampart, not a resource**, which is the archive's and
/// the sanctum's shape: it saves through the path every node saves through, and
/// `survey rampart` can read it.
#[derive(Component, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Siege {
    /// How many rounds have resolved.
    pub turns: u32,
    /// Yours.
    pub garrison: Band,
    /// Theirs.
    pub enemy: Band,
    /// What the enemy does next, announced now.
    pub intent: Intent,
    /// What the enemy arrived with — for the completion fraction.
    pub arrived: u32,
    /// What the garrison has ever had, counting reinforcements.
    ///
    /// **The denominator, and without it `hurt` was unreachable.** `Band::wound`
    /// derives `count` from `vigour` (`count = ceil(vigour / VIGOUR)`), so the
    /// two can never diverge: `vigour >= 3·count − 2` always holds, and a `hurt`
    /// test of `vigour <= 2·count` therefore implies `count < 3`. At one or two
    /// troops `few` is *also* true and fires first in an `else if` ladder, so the
    /// `quaff mending` rung of the shipped solver was dead code and `siege.toml`'s
    /// healing entries were unreachable.
    ///
    /// The unit test passed because it hand-built a band the game cannot produce.
    /// A reading about *how worn* a side is has to measure against what it once
    /// had, exactly as the enemy's `completion` measures against `arrived`.
    #[serde(default)]
    pub mustered: u32,
    /// Modifiers the player has staged for the coming round.
    ///
    /// **Cleared when the round resolves**, because an advantage bought with a
    /// potion is spent on the round it was bought for. A modifier that persisted
    /// would make the first turn the only one that mattered.
    pub staged: Vec<Modifier>,
    /// Dice pledged to areas for the coming round.
    ///
    /// **Cleared when the round resolves**, exactly as [`staged`](Self::staged)
    /// is and for the same reason: a pledge is bought for the round it was made
    /// for, and one that persisted would make the first turn the only one that
    /// mattered. The dice themselves come straight back — what is scarce is the
    /// *round*, not the die.
    #[serde(default)]
    pub pledges: Vec<Pledge>,
    /// The tower's standing bonus on every answering roll — the Ley Line's
    /// `edge` grant, read once when the siege begins.
    ///
    /// **On the siege rather than read per round**, so the board's odds and
    /// the round's rolls read one number, and a node taken mid-fight lands on
    /// the next siege rather than half way through this one.
    #[serde(default)]
    pub edge: i32,
    /// What a format-8 save recorded as this siege's private pool.
    ///
    /// **A migration shim, and it has to be a field rather than a lookup.**
    /// `document::migrate` runs on the already-deserialised save and reads
    /// `node.siege`, which is *this type* — and `Siege` has no
    /// `deny_unknown_fields`, so deleting the field outright would have serde
    /// discard the format-8 value **silently, before the migration could ever
    /// see it**. Deleting and migrating are not two orderings of one change;
    /// only one of them works.
    ///
    /// Written by nothing and read only by the 8 → 9 migration, which lifts it
    /// into [`tower::Quintessence`](crate::tower::Quintessence) and leaves
    /// `None` behind. It costs one `Option<u32>` to keep until 1.0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quintessence: Option<u32>,
    /// Set once the siege is over.
    pub outcome: Option<Outcome>,
    /// The tick the road is clear again, once this one has ended.
    ///
    /// **On the siege rather than in a resource**, so it travels in the save
    /// with everything else about the fight and a player cannot clear the
    /// cadence by quitting — the rule §19 records for the `verify` cooldown, one
    /// surface over.
    #[serde(default)]
    pub clear_at: Option<u64>,
}

impl Siege {
    /// Begin one. The garrison is always [`ASSIGNED`]; the enemy is drawn.
    ///
    /// **A siege grants nothing now.** The pool is the tower's and the player
    /// brings whatever they have — which is the whole of what makes enchanting
    /// during a siege cost something, and what stops a fight being a fresh
    /// allowance that expires unspent.
    ///
    /// It took a pool as an argument for one phase, so that the number arrived
    /// *before* the two draws below and could not reorder them. That constraint
    /// is gone with the parameter, and the draws are untouched.
    #[must_use]
    pub fn begin(rngs: &mut Rngs) -> Self {
        use rand::Rng;
        // **Drawn before the intent, and unconditionally.** Order is part of the
        // replay contract: swapping these two would change every existing seed.
        let enemy = rngs.stream(RngStream::Siege).random_range(5..=9);
        let intent = Intent::drawn(rngs);
        Self {
            turns: 0,
            garrison: Band::new(ASSIGNED),
            enemy: Band::new(enemy),
            intent,
            arrived: enemy,
            mustered: ASSIGNED,
            staged: Vec::new(),
            pledges: Vec::new(),
            edge: 0,
            quintessence: None,
            outcome: None,
            clear_at: None,
        }
    }

    /// Whether it is still being fought.
    #[must_use]
    pub const fn running(&self) -> bool {
        self.outcome.is_none()
    }

    /// The garrison is thin.
    #[must_use]
    pub const fn few(&self) -> bool {
        self.garrison.count <= THIN && self.garrison.count > 0
    }

    /// The garrison is wounded — at or below half of what it ever had.
    ///
    /// **Against [`mustered`](Self::mustered), never against the current count.**
    /// See that field: measured against `count`, this was unreachable above two
    /// troops and shadowed by `few` below them.
    #[must_use]
    pub const fn hurt(&self) -> bool {
        self.garrison.count > 0 && self.garrison.vigour * 2 <= self.mustered * VIGOUR
    }

    /// The enemy has at least twice the garrison's numbers.
    ///
    /// **The ratio the language cannot express**, published as a word. This is
    /// the whole reason the readings exist rather than an arithmetic grammar.
    #[must_use]
    pub const fn outnumbered(&self) -> bool {
        self.enemy.count >= self.garrison.count * 2 && self.garrison.count > 0
    }

    /// The enemy has taken nothing yet.
    #[must_use]
    pub const fn massed(&self) -> bool {
        self.enemy.count == self.arrived
    }

    /// Which dice are still in the coffer — pledged to nothing.
    ///
    /// **Held, not affordable.** This is what the board draws and what `free`
    /// asks; [`affordable`](Self::affordable) is the narrower question a spell
    /// gets, and keeping them apart is what lets the refusal say *"you have it,
    /// you cannot pay for it"* rather than pretending the die is gone.
    #[must_use]
    pub fn coffer(&self) -> Vec<Die> {
        POOL.into_iter()
            .filter(|die| !self.pledges.iter().any(|pledge| pledge.die == *die))
            .collect()
    }

    /// Which dice are in the coffer *and* within what is left to spend.
    ///
    /// **What the coffer publishes, and the reason is a boundary defect.** The
    /// natural positive sentence — `if the coffer has more quintessence than the
    /// d20` — is *wrong*: `watch.rs`'s comparison against another place is
    /// **strict**, so it excludes the die you can exactly afford, and there is no
    /// *at least as many* comparative to reach for. Publishing affordability as a
    /// derived word makes `if the coffer has d20` mean *"I hold it and can pay for
    /// it"* — the maze's `spoil`/`exit` pattern, and a natural positive with no
    /// grammar at all.
    ///
    /// **`pool` is handed in**, which is what keeps this module `World`-free —
    /// the property that lets `siege/tests.rs` prove the whole model with no
    /// `World` at all.
    #[must_use]
    pub fn affordable(&self, pool: u32) -> Vec<Die> {
        self.coffer()
            .into_iter()
            .filter(|die| super::cost_of(*die) <= pool)
            .collect()
    }

    /// Whether `die` is unpledged — says nothing about paying for it.
    #[must_use]
    pub fn free(&self, die: Die) -> bool {
        self.coffer().contains(&die)
    }

    /// Whether `pool` is quintessence enough for `die`.
    #[must_use]
    pub const fn affords(die: Die, pool: u32) -> bool {
        super::cost_of(die) <= pool
    }

    /// The dice pledged to `area`, in the order they were pledged.
    #[must_use]
    pub fn pledged(&self, area: Area) -> Vec<Die> {
        self.pledges
            .iter()
            .filter(|pledge| pledge.area == area)
            .map(|pledge| pledge.die)
            .collect()
    }

    /// The best and worst `area` could come to, before anything is rolled.
    ///
    /// **The range is shown before the commitment**, which is §5.1's fairness
    /// rule carried from the dice to the allocation: a player choosing where to
    /// gamble has to be able to see what they are gambling. `(0, 0)` where
    /// nothing is pledged.
    #[must_use]
    pub fn range(&self, area: Area) -> (u32, u32) {
        self.pledged(area)
            .into_iter()
            .fold((0, 0), |(low, high), die| (low + 1, high + die.faces()))
    }

    /// Pledge `die` to `area`, spending its cost.
    ///
    /// **Two refusals, and the caller must be able to tell them apart** — a die
    /// already pledged and a die you cannot pay for are different sentences, and
    /// §6 forbids a bare error for either.
    ///
    /// **Nothing here draws.** A refusal returns before touching anything, and a
    /// success records a pledge; the dice are rolled at
    /// [`resolve`](Self::resolve), one face per pledge, in pledge order.
    ///
    /// **The caller spends.** `pool` is what the tower holds and this only says
    /// whether it is enough — the resource lives in `tower::Quintessence` now,
    /// shared with the forge, and a model that took from it would either need a
    /// `World` here or keep a second copy of one number.
    pub fn pledge(&mut self, die: Die, area: Area, pool: u32) -> Pledged {
        if !self.free(die) {
            return Pledged::Spent;
        }
        let cost = super::cost_of(die);
        if cost > pool {
            return Pledged::Short { cost };
        }
        self.pledges.push(Pledge { die, area });
        Pledged::Made { cost }
    }

    /// Roll everything pledged, and say what each area came to.
    ///
    /// **One draw per pledged die, in `pledges` order, unconditionally.** The
    /// order is the order the player pledged in, which is part of the replayed
    /// submissions — so two runs of one seed roll the same faces. Rolling *by
    /// area* instead would make the draw order depend on `Area::ALL`, which is
    /// fine today and would silently change every replay the day an area is
    /// added.
    fn resolve_pledges(&self, rngs: &mut Rngs) -> Strengths {
        let mut strengths = Strengths::default();
        for pledge in &self.pledges {
            let face = Roll::new(pledge.die, 0).resolve(rngs).face;
            *strengths.of_mut(pledge.area) += face;
        }
        strengths
    }

    /// Stage a modifier for the coming round.
    pub fn stage(&mut self, source: &str, effect: Effect) {
        self.staged.push(Modifier {
            source: source.to_owned(),
            effect,
        });
    }

    /// Add troops to the garrison — what `deploy` does.
    ///
    /// **`mustered` rises with them**, so reinforcing a worn line does not make
    /// it read as freshly wounded: the denominator is what the garrison has ever
    /// had, and a troop that arrives has been had.
    pub const fn reinforce(&mut self, count: u32) {
        self.garrison.count += count;
        self.garrison.vigour += count * VIGOUR;
        self.mustered += count;
    }

    /// Put fight back into the garrison — what `quaff` does.
    pub fn heal(&mut self, points: u32) {
        let ceiling = self.mustered * VIGOUR;
        self.garrison.mend(points, ceiling);
    }

    /// How far through the siege the player got, as a percentage.
    ///
    /// **What escrow is paid against** (§11.5): progress-scaled, so losing at
    /// 60% keeps something worth having rather than nothing. Measured by what
    /// the enemy has lost, because that is what the player actually did.
    #[must_use]
    pub const fn completion(&self) -> u32 {
        if self.arrived == 0 {
            return 100;
        }
        let felled = self.arrived.saturating_sub(self.enemy.count);
        (felled * 100) / self.arrived
    }

    /// Leave the enemy one round from breaking — `debug_siege`'s shortcut.
    ///
    /// **It thins the enemy and touches nothing else**, so the round that follows
    /// is a real round with real rolls: the garrison still has to tell, and on a
    /// bad enough draw it will not. A shortcut that *ended* the siege would be
    /// testing the shortcut.
    #[cfg(debug_assertions)]
    pub const fn give_away(&mut self) {
        self.enemy = Band {
            count: 1,
            vigour: 1,
        };
    }

    /// What the board draws.
    ///
    /// **The names and the intent travel with it**, which is what stops the
    /// painter having to know what a band is called — `orbs-render` may never
    /// depend on `orbs-sim`, and §19 records the lens's sheet shipping
    /// unlabelled because that was got the other way round.
    ///
    /// The odds go too, composed but undrawn, because §5.1's fairness rule is
    /// *show the odds before the commitment*: a player choosing whether to spend
    /// a potion has to be able to see what it buys.
    /// `pool` is what the **tower** holds, handed in for the same reason
    /// `pledge` takes it: the resource is shared with the forge now, and this
    /// module stays `World`-free.
    #[must_use]
    pub fn view(&self, tally: String, pool: u32) -> orbs_render::Rampart {
        orbs_render::Rampart {
            garrison: orbs_render::SiegeSide {
                name: GARRISON,
                troops: self.garrison.count,
                vigour: self.garrison.vigour,
                // **`mustered`, not the current count.** Against `count` the
                // garrison's bar could never fall below ~89% however many died —
                // `wound` keeps the two in lockstep — so a line cut from six to
                // two drew *full* one round before it routed. The enemy's bar was
                // already right; this is the same denominator.
                full: self.mustered * VIGOUR,
                chance: self.garrison_roll().chance(),
            },
            enemy: orbs_render::SiegeSide {
                name: ENEMY,
                troops: self.enemy.count,
                vigour: self.enemy.vigour,
                full: self.arrived * VIGOUR,
                chance: self.enemy_roll().chance(),
            },
            turns: self.turns,
            // **The whole allocation, drawn.** Three dice against four rows, so
            // one is always empty — and which one is the decision the board
            // exists to make visible.
            areas: Area::ALL
                .into_iter()
                .map(|area| orbs_render::Allocation {
                    name: area.word(),
                    dice: self
                        .pledged(area)
                        .into_iter()
                        .map(|die| die.word().to_owned())
                        .collect(),
                    range: self.range(area),
                    moot: !area.answers(self.intent),
                })
                .collect(),
            // **Everything held, with its price** — not just what is affordable.
            // The board's job is to show the bargain, and a die that vanished
            // when you could not pay for it would hide exactly the round where
            // the decision is hardest. `affordable()` is the *spell's* narrower
            // view; a person reads the numbers.
            coffer: self
                .coffer()
                .into_iter()
                .map(|die| (die.word().to_owned(), super::cost_of(die)))
                .collect(),
            quintessence: pool,
            intent: self.intent.word().to_owned(),
            tally,
        }
    }

    /// Compose the roll one attacker makes. **Public so the board can show the
    /// odds before the commitment**, which is the fairness rule.
    #[must_use]
    pub fn enemy_roll(&self) -> Roll {
        let mut roll = Roll::new(Die::D20, AGAINST);
        if self.intent.edge() != 0 {
            roll = roll.plus(self.intent.word(), Effect::Bonus(self.intent.edge()));
        }
        roll
    }

    /// Compose the roll one defender makes, with everything staged applied.
    #[must_use]
    pub fn garrison_roll(&self) -> Roll {
        let mut roll = Roll::new(Die::D20, AGAINST);
        // The Ley Line's edge first, named, so `peruse bailey.log` says the
        // wizard's own standing moved a roll before any potion did.
        if self.edge != 0 {
            roll = roll.plus("edge", Effect::Bonus(self.edge));
        }
        for modifier in &self.staged {
            roll = roll.plus(&modifier.source, modifier.effect);
        }
        roll
    }

    /// The same roll, with a resolved `line` behind it.
    ///
    /// **The bonus carries the area's name**, so `peruse bailey.log` says *why*
    /// a swing landed — rule 4, and the thing that makes an allocation legible
    /// after the fact rather than only before it.
    #[must_use]
    fn garrison_roll_with(&self, line: u32, intent: Intent) -> Roll {
        let mut roll = self.garrison_roll();
        // **Nothing when the garrison does not swing.** A line pledged against a
        // volley is wasted, and the board says so before the pledge — writing
        // the bonus in anyway would make the log claim an advantage that never
        // applied.
        if line > 0 && Area::Line.answers(intent) {
            roll = roll.plus(
                Area::Line.word(),
                Effect::Bonus(i32::try_from(line).unwrap_or(0)),
            );
        }
        roll
    }

    /// What one attacker must beat, with a resolved `buckler` raising it.
    #[must_use]
    fn enemy_roll_against(&self, buckler: u32) -> Roll {
        let mut roll = self.enemy_roll();
        // A buckler raises the target rather than lowering their die, which is
        // the same arithmetic and the honest description: the wall is higher.
        roll.against += i32::try_from(buckler).unwrap_or(0);
        roll
    }

    /// Resolve one round. **The only thing in this domain that advances state.**
    ///
    /// Returns every roll made, in order, so the caller can write each to the
    /// log with its die and face — rule 4, and what makes `peruse bailey.log` a
    /// genuine postmortem rather than a list of outcomes.
    ///
    /// **Both sides always roll their full complement**, with no early return
    /// once one side breaks. That is CLAUDE.md's determinism rule: a draw
    /// skipped because the round was already decided would make the stream
    /// depend on its own results.
    pub fn resolve(&mut self, rngs: &mut Rngs) -> Round {
        let intent = self.intent;

        // **The pledged dice roll first, before anything they modify.** They set
        // the round's terms, so they have to be known before a single attack is
        // composed — and doing it here rather than inside the loops keeps the
        // draw order a function of `pledges` alone.
        let strengths = self.resolve_pledges(rngs);

        let enemy_roll = self.enemy_roll_against(strengths.buckler);
        let garrison_roll = self.garrison_roll_with(strengths.line, intent);

        // The enemy strikes first, because the intent was announced last round
        // and the player has already had their turn to answer it.
        let attacking = intent.attackers(self.enemy.count);
        let mut struck = Vec::new();
        for _ in 0..attacking {
            struck.push(enemy_roll.resolve(rngs));
        }

        // ...and the garrison answers, unless this was a volley. **The rolls are
        // still drawn**, and then discarded, so the stream advances the same way
        // whichever intent came up — otherwise the intent would silently reorder
        // every subsequent draw.
        let answering = self.garrison.count;
        let mut answered = Vec::new();
        for _ in 0..answering {
            answered.push(garrison_roll.resolve(rngs));
        }

        let hits = |rolls: &[Landed]| -> u32 {
            u32::try_from(rolls.iter().filter(|landed| landed.tells).count()).unwrap_or(u32::MAX)
        };
        let taken = hits(&struck);
        let dealt = if intent.answered() {
            hits(&answered)
        } else {
            0
        };

        self.garrison.wound(taken);
        self.enemy.wound(dealt);

        // **A sortie, and it is the only thing here that wounds your own side on
        // purpose.** Favourable but paid in mettle: see `SORTIE_DEALT`.
        let sortied = strengths.sortie / SORTIE_DEALT;
        let spent = strengths.sortie / SORTIE_COST;
        if strengths.sortie > 0 {
            self.enemy.wound(sortied);
            self.garrison.wound(spent);
        }

        // **Succour last, after everything that wounds — and before the outcome
        // is decided.** Both halves are deliberate. Mending before the round
        // would heal damage that had not happened yet; mending *after* the
        // outcome would mean a garrison wounded to nothing was already lost, and
        // a pledged succour could never be a last stand. See [`Area::Succour`].
        //
        // What it actually put back is measured rather than assumed: `mend` caps
        // at full, so a big roll on a nearly-whole line is mostly wasted and the
        // sentence has to be able to say so.
        let before = self.garrison.vigour;
        if strengths.succour > 0 {
            let ceiling = self.mustered * VIGOUR;
            self.garrison.mend(strengths.succour, ceiling);
        }
        let mended = self.garrison.vigour.saturating_sub(before);

        self.staged.clear();
        self.pledges.clear();
        self.turns += 1;

        // Drawn unconditionally, before the outcome is checked, for the same
        // reason: an intent drawn only while the siege continues would make the
        // stream depend on whether it ended.
        self.intent = Intent::drawn(rngs);

        if self.enemy.routed() {
            self.outcome = Some(Outcome::Held);
        } else if self.garrison.routed() {
            self.outcome = Some(Outcome::Fallen);
        }

        Round {
            intent,
            struck,
            answered: if intent.answered() {
                answered
            } else {
                Vec::new()
            },
            taken,
            dealt,
            strengths,
            sortied,
            spent,
            mended,
            outcome: self.outcome,
        }
    }
}
