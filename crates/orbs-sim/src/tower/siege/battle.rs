//! [`Siege`] itself — the state, the readings' arithmetic, and `resolve` (§5.1).
//!
//! Everything else in the domain is instant; [`Siege::resolve`] is what `hold`
//! calls, and §5.0's *"no per-command tick cost"* rests on it being the only
//! thing here that advances the world.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use super::{
    AGAINST, ASSIGNED, Area, BASE_MOST, Band, ENEMY, FEWEST, GARRISON, Intent, MOST, Outcome, POOL,
    Pledge, Round, SORTIE_COST, SORTIE_DEALT, Strengths, THIN, VIGOUR,
};
use crate::rng::{RngStream, Rngs};
use crate::tower::dice::{Die, Effect, Landed, Modifier, Roll};

/// What happened when a die was pledged.
///
/// Three answers rather than a `bool`: a die already pledged is a mistake on
/// the board, a die you cannot pay for is the decision the domain is about.
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
/// A component on the rampart rather than a resource, so it saves through the
/// path every node saves through and `survey rampart` can read it.
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
    /// What the tower's standing was when the enemy came up the road.
    ///
    /// An opening snapshot like [`arrived`](Self::arrived): renown moves as the
    /// rounds resolve *and* once at the end, so a total read only at `settle`
    /// would omit what the exchanges cost. Measured against rather than
    /// accumulated into, so there is no running sum to keep in step.
    ///
    /// `Option` because an older save has no snapshot, and `settle` measures
    /// `now - standing` — nought reported the tower's whole renown as the
    /// winnings of a siege that was *lost*.
    #[serde(default)]
    pub standing: Option<u64>,
    /// What the garrison has ever had, counting reinforcements.
    ///
    /// The denominator: how worn a side is measures against what it once had,
    /// as the enemy's `completion` measures against `arrived`. Without it
    /// `hurt` was unreachable above two troops and shadowed by `few` below
    /// them, so the solver's `quaff mending` rung was dead code.
    #[serde(default)]
    pub mustered: u32,
    /// Modifiers the player has staged for the coming round.
    ///
    /// Cleared when the round resolves: an advantage that persisted would make
    /// the first turn the only one that mattered.
    pub staged: Vec<Modifier>,
    /// Dice pledged to areas for the coming round.
    ///
    /// Cleared when the round resolves, as [`staged`](Self::staged) is — what
    /// is scarce is the *round*, not the die.
    #[serde(default)]
    pub pledges: Vec<Pledge>,
    /// The tower's standing bonus on every answering roll — the Ley Line's
    /// `edge` grant, read once when the siege begins.
    ///
    /// On the siege rather than read per round, so the board's odds and the
    /// round's rolls read one number and a node taken mid-fight lands on the
    /// next siege.
    #[serde(default)]
    pub edge: i32,
    /// What a format-8 save recorded as this siege's private pool.
    ///
    /// A field rather than a lookup because `document::migrate` reads
    /// `node.siege` off the already-deserialised save, and without
    /// `deny_unknown_fields` serde would discard the format-8 value first.
    ///
    /// Read only by the 8 → 9 migration, which lifts it into
    /// [`tower::Quintessence`](crate::tower::Quintessence). Keep until 1.0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quintessence: Option<u32>,
    /// Set once the siege is over.
    pub outcome: Option<Outcome>,
    /// The tick the road is clear again, once this one has ended.
    ///
    /// On the siege rather than in a resource, so it travels in the save and a
    /// player cannot clear the cadence by quitting (§19).
    #[serde(default)]
    pub clear_at: Option<u64>,
}

impl Siege {
    /// Begin one. The garrison is always [`ASSIGNED`]; the enemy is drawn.
    ///
    /// A siege grants nothing: the pool is the tower's, so enchanting during
    /// one costs something rather than spending an allowance that expires.
    #[must_use]
    pub fn begin(rngs: &mut Rngs) -> Self {
        Self::begin_against(rngs, BASE_MOST)
    }

    /// Open a siege whose enemy may be drawn as large as `most`.
    ///
    /// Consumed *inside* the first draw rather than before it, so it cannot
    /// move either draw's position in the stream (§19); `begin(rngs)` is the
    /// old behaviour whole. `most` is clamped into [`FEWEST`]..=[`MOST`].
    #[must_use]
    pub fn begin_against(rngs: &mut Rngs, most: u32) -> Self {
        use rand::Rng;
        let most = most.clamp(FEWEST, MOST);
        // Drawn before the intent, unconditionally: order is part of the replay
        // contract, and swapping these two changes every existing seed.
        let enemy = rngs.stream(RngStream::Siege).random_range(FEWEST..=most);
        let intent = Intent::drawn(rngs);
        Self {
            turns: 0,
            garrison: Band::new(ASSIGNED),
            enemy: Band::new(enemy),
            intent,
            arrived: enemy,
            // Filled by `defend` from the world, as `edge` is: `begin` is a pure
            // function of the stream and the tower's standing is not in it.
            standing: None,
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
    /// Against [`mustered`](Self::mustered), never the current count: measured
    /// against `count` this was unreachable above two troops and shadowed by
    /// `few` below them.
    #[must_use]
    pub const fn hurt(&self) -> bool {
        self.garrison.count > 0 && self.garrison.vigour * 2 <= self.mustered * VIGOUR
    }

    /// The enemy has at least twice the garrison's numbers.
    ///
    /// A ratio the language cannot express, published as a word — why the
    /// readings exist rather than an arithmetic grammar.
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
    /// Held, not affordable. [`affordable`](Self::affordable) is the narrower
    /// question a spell gets, so the refusal can say *"you have it, you cannot
    /// pay for it"* rather than pretending the die is gone.
    #[must_use]
    pub fn coffer(&self) -> Vec<Die> {
        POOL.into_iter()
            .filter(|die| !self.pledges.iter().any(|pledge| pledge.die == *die))
            .collect()
    }

    /// Which dice are in the coffer *and* within what is left to spend.
    ///
    /// Published as a word because the natural sentence — `if the coffer has
    /// more quintessence than the d20` — is wrong: `watch.rs`'s comparison
    /// against another place is strict, so it excludes the die you can exactly
    /// afford. Derived, `if the coffer has d20` means *"I hold it and can pay
    /// for it"*.
    ///
    /// `pool` is handed in, which keeps this module `World`-free.
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
    /// §5.1 carried from the dice to the allocation: a player choosing where to
    /// gamble has to see what they are gambling. `(0, 0)` where nothing is.
    #[must_use]
    pub fn range(&self, area: Area) -> (u32, u32) {
        self.pledged(area)
            .into_iter()
            .fold((0, 0), |(low, high), die| (low + 1, high + die.faces()))
    }

    /// Pledge `die` to `area`, spending its cost.
    ///
    /// Two refusals the caller must tell apart — a die already pledged and a die
    /// you cannot pay for are different sentences (§6). Nothing here draws: the
    /// dice are rolled at [`resolve`](Self::resolve), in pledge order.
    ///
    /// The caller spends; `pool` is only what the tower holds. The resource
    /// lives in `tower::Quintessence`, shared with the forge, and taking from
    /// it here would need a `World`.
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
    /// One draw per pledged die, in `pledges` order: that order is part of the
    /// replayed submissions, and rolling *by area* would tie the draw order to
    /// `Area::ALL` and change every replay the day an area is added.
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
    /// `mustered` rises with them, so reinforcing a worn line does not make it
    /// read as freshly wounded.
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
    /// What escrow is paid against (§11.5), so losing at 60% keeps something
    /// worth having. Measured by what the enemy has lost.
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
    /// It thins the enemy and touches nothing else, so what follows is a real
    /// round the garrison can still lose. A shortcut that *ended* the siege
    /// would be testing the shortcut.
    #[cfg(debug_assertions)]
    pub const fn give_away(&mut self) {
        self.enemy = Band {
            count: 1,
            vigour: 1,
        };
    }

    /// What the board draws.
    ///
    /// The names and the intent travel with it, so the painter need not know
    /// what a band is called — `orbs-render` may never depend on `orbs-sim`
    /// (§19). The odds go too, because §5.1 is *show the odds before the
    /// commitment*. `pool` is handed in, for the reason `pledge` takes it.
    #[must_use]
    pub fn view(&self, tally: String, pool: u32) -> orbs_render::Rampart {
        orbs_render::Rampart {
            garrison: orbs_render::SiegeSide {
                name: GARRISON,
                troops: self.garrison.count,
                vigour: self.garrison.vigour,
                // `mustered`, not the current count: against `count` the bar
                // never fell below ~89%, so a line cut from six to two drew
                // full one round before it routed.
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
            // The whole allocation. Three dice against four rows, so one is
            // always empty — which one is the decision the board makes visible.
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
            // Everything held, with its price: a die that vanished when you
            // could not pay for it would hide the round where the decision is
            // hardest. `affordable()` is the spell's narrower view.
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

    /// Compose the roll one attacker makes. Public so the board can show the
    /// odds before the commitment (§5.1).
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
    /// The bonus carries the area's name, so `peruse bailey.log` says *why* a
    /// swing landed (rule 4).
    #[must_use]
    fn garrison_roll_with(&self, line: u32, intent: Intent) -> Roll {
        let mut roll = self.garrison_roll();
        // Nothing when the garrison does not swing: writing the bonus in anyway
        // would have the log claim an advantage that never applied.
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

    /// Resolve one round — the only thing in this domain that advances state.
    ///
    /// Returns every roll made, in order, so the caller can log each with its
    /// die and face (rule 4) and `peruse bailey.log` is a postmortem.
    ///
    /// Both sides always roll their full complement, with no early return: a
    /// draw skipped because the round was already decided would make the stream
    /// depend on its own results.
    pub fn resolve(&mut self, rngs: &mut Rngs) -> Round {
        let intent = self.intent;

        // The pledged dice roll first: they set the round's terms, and rolling
        // here rather than in the loops keeps the draw order a function of
        // `pledges`.
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

        // ...and the garrison answers, unless this was a volley. The rolls are
        // still drawn and discarded, so the intent cannot reorder later draws.
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

        // A sortie, the only thing here that wounds your own side on purpose.
        // Favourable but paid in mettle: see `SORTIE_DEALT`.
        let sortied = strengths.sortie / SORTIE_DEALT;
        let spent = strengths.sortie / SORTIE_COST;
        if strengths.sortie > 0 {
            self.enemy.wound(sortied);
            self.garrison.wound(spent);
        }

        // Succour last, after everything that wounds and before the outcome:
        // earlier would heal damage that had not happened, later would mean a
        // pledged succour could never be a last stand. See [`Area::Succour`].
        //
        // What it put back is measured, not assumed — `mend` caps at full, so a
        // big roll on a nearly-whole line is mostly wasted.
        let before = self.garrison.vigour;
        if strengths.succour > 0 {
            let ceiling = self.mustered * VIGOUR;
            self.garrison.mend(strengths.succour, ceiling);
        }
        let mended = self.garrison.vigour.saturating_sub(before);

        self.staged.clear();
        self.pledges.clear();
        self.turns += 1;

        // Drawn unconditionally, before the outcome is checked: an intent drawn
        // only while the siege continues would make the stream depend on
        // whether it ended.
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
