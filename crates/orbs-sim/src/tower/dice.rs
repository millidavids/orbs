//! The seven dice, and a roll that is composed before it is resolved (§10).
//!
//! A wizard rolling dice in a tower is the joke the whole game is built on, so
//! the dice are the ones every player already knows: `d20`, `d100`, `d12`, `d10`,
//! `d8`, `d6`, `d4`. Naming them that way is the cheapest possible route to
//! making the arithmetic legible — nobody has to be taught what a d20 is.
//!
//! # Compose, then resolve — and this is the shape that cannot be retrofitted
//!
//! A [`Roll`] is *assembled* as a value — which die, what modifies it, what it is
//! against — and only then handed to [`Roll::resolve`], which draws. Everything
//! that wants to touch the dice touches the composed value; by the time the draw
//! happens there is nothing left to give it.
//!
//! ```text
//! Roll { die: D20, against: 11, modifiers: [ +2 mending, advantage haste ] }
//!         │
//!         └── resolve(&mut Rngs) ──▶ Landed { face: 14, total: 16, tells: true }
//! ```
//!
//! Five things this buys, in the order they matter:
//!
//! 1. **Arsenal items have somewhere to reach.** A potion that adds two, a scroll
//!    that rerolls, an enchantment that swaps a d8 for a d12 are all edits to a
//!    `Roll` before it resolves. **This is the expensive retrofit**: with the draw
//!    at the call site, every call site has to change to admit a modifier, and
//!    there is one per kind of attack.
//! 2. **Each modifier carries its source**, so the log says *why* — rule 4. The
//!    record holds `+2` *and* `mending`, and the view composes the sentence.
//! 3. **The draw count is a function of the composed roll, never of the
//!    outcome.** Advantage draws twice; a reroll draws again. Both are visible
//!    *before* resolution, so the number of draws is replayable — where a draw
//!    taken *because* the first one missed would make the stream depend on its own
//!    results. That is §19's `drift` defect, prevented by construction.
//! 4. **Odds are testable without a `Sim`.** A composed `Roll` is a pure value.
//! 5. **Tuning stays in TOML** (rule 6), because which die a thing rolls is
//!    authored rather than hardcoded.
//!
//! `tower::heat`'s `Quickened` is the counter-example worth reading: a real
//! world-effect reaching a number, with the halving applied at the call site. It
//! works because there is exactly one source and one effect, and it does not
//! generalise — a second source would need the call site to know about it. A
//! siege has many sources stacking onto one number.

use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::rng::{RngStream, Rngs};

/// One of D&D's seven, and the homage is the point.
///
/// **A closed table**, the shape `Syllable::ALL` and the lens's `SIGILS` already
/// have — so a new die is a variant and a row, and nothing derives one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Die {
    /// Four faces. The smallest swing in the game.
    D4,
    /// Six faces.
    D6,
    /// Eight faces.
    D8,
    /// Ten faces.
    D10,
    /// Twelve faces.
    D12,
    /// Twenty faces. The attack die, as it is everywhere else.
    D20,
    /// A hundred faces.
    ///
    /// **One die, not the percentile pair of d10s.** The pair is a *tabletop*
    /// affordance — it exists because nobody manufactures a hundred-sided solid —
    /// and reproducing it here would spend two draws to get one number, which
    /// doubles the stream cost of every percentile roll for a piece of physical
    /// trivia no player of this game can see. The homage is the *set* of seven,
    /// and the set is intact.
    D100,
}

impl Die {
    /// Every die, smallest first.
    pub const ALL: [Self; 7] = [
        Self::D4,
        Self::D6,
        Self::D8,
        Self::D10,
        Self::D12,
        Self::D20,
        Self::D100,
    ];

    /// How many faces.
    #[must_use]
    pub const fn faces(self) -> u32 {
        match self {
            Self::D4 => 4,
            Self::D6 => 6,
            Self::D8 => 8,
            Self::D10 => 10,
            Self::D12 => 12,
            Self::D20 => 20,
            Self::D100 => 100,
        }
    }

    /// What the player types and what the log prints — `d20`.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::D4 => "d4",
            Self::D6 => "d6",
            Self::D8 => "d8",
            Self::D10 => "d10",
            Self::D12 => "d12",
            Self::D20 => "d20",
            Self::D100 => "d100",
        }
    }

    /// Read one back from its word — how `dice.toml` names a die.
    ///
    /// **An unknown name fails the content load** rather than falling back, which
    /// is `materials.toml`'s tint rule: a silent default would make a typo
    /// indistinguishable from a deliberate choice.
    #[must_use]
    pub fn named(word: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|die| die.word() == word)
    }

    /// Roll it once. The only place in the game a face is drawn.
    fn draw(self, rngs: &mut Rngs) -> u32 {
        // `1..=faces`, because a die has no nought face and an off-by-one here
        // would be invisible in every aggregate and wrong in every single roll.
        rngs.stream(RngStream::Siege).random_range(1..=self.faces())
    }
}

/// Why a roll is not what the die alone would say.
///
/// **The source travels with the number**, which is rule 4 and is what makes
/// `peruse siege.log` a postmortem rather than a list of results: a player can
/// see which potion earned its place, and a rate test can tell a modifier that
/// works from one that silently does nothing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Modifier {
    /// What granted it — `mending`, `haste`, `fireball-scroll`.
    pub source: String,
    /// What it does.
    pub effect: Effect,
}

/// What a [`Modifier`] does to a roll.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effect {
    /// Add to the total. Negative narrows as well as widens.
    Bonus(i32),
    /// Draw twice, keep the better.
    Advantage,
    /// Draw twice, keep the worse.
    Disadvantage,
    /// Roll a bigger die instead. Ignored if it is not bigger.
    Upgrade(Die),
}

/// A roll, assembled and not yet drawn.
///
/// Build it, let anything that wants to edit it, then [`resolve`](Self::resolve).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Roll {
    /// Which die, before any [`Effect::Upgrade`].
    pub die: Die,
    /// What the total must reach to tell.
    pub against: i32,
    /// What is editing it, in the order it was applied.
    pub modifiers: Vec<Modifier>,
}

impl Roll {
    /// A bare roll of `die` against `against`.
    #[must_use]
    pub const fn new(die: Die, against: i32) -> Self {
        Self {
            die,
            against,
            modifiers: Vec::new(),
        }
    }

    /// Add a modifier, naming what granted it.
    #[must_use]
    pub fn plus(mut self, source: &str, effect: Effect) -> Self {
        self.modifiers.push(Modifier {
            source: source.to_owned(),
            effect,
        });
        self
    }

    /// Which die will actually be drawn, after upgrades.
    ///
    /// **The largest upgrade wins and a smaller one is ignored**, so stacking two
    /// buffs can never make a roll worse than either alone — the rule a player
    /// would assume, and the one that makes a second potion never a mistake.
    #[must_use]
    pub fn effective_die(&self) -> Die {
        self.modifiers
            .iter()
            .filter_map(|modifier| match modifier.effect {
                Effect::Upgrade(die) => Some(die),
                _ => None,
            })
            .fold(self.die, Ord::max)
    }

    /// The flat adjustment to the total.
    #[must_use]
    pub fn bonus(&self) -> i32 {
        self.modifiers
            .iter()
            .filter_map(|modifier| match modifier.effect {
                Effect::Bonus(n) => Some(n),
                _ => None,
            })
            .sum()
    }

    /// How many faces are drawn — **decided before the draw, always**.
    ///
    /// Advantage and disadvantage each mean two draws. Holding both means
    /// **neither**, which is the tabletop rule and is also the only answer that
    /// keeps the draw count a pure function of the composed roll.
    #[must_use]
    pub fn draws(&self) -> u32 {
        if self.edge().is_some() { 2 } else { 1 }
    }

    /// Which way the roll leans, if it leans at all.
    ///
    /// `Some(true)` is advantage, `Some(false)` disadvantage, `None` neither —
    /// **and holding both is `None`**, which is the tabletop rule and the only
    /// answer that keeps [`draws`](Self::draws) a pure function of the composed
    /// roll.
    ///
    /// **One predicate, asked by all three.** It was written out six times
    /// across `draws`, `resolve` and `chance`, which have to agree or the number
    /// of faces drawn, the face kept and the odds printed describe three
    /// different rolls. A fourth `Effect` that changed the draw count would mean
    /// editing three copies, and missing one desynchronises `RngStream::Siege`
    /// for the rest of the run — every replay past that point diverging, which
    /// is the failure this file's headline invariant exists to prevent.
    fn edge(&self) -> Option<bool> {
        let up = self
            .modifiers
            .iter()
            .any(|m| matches!(m.effect, Effect::Advantage));
        let down = self
            .modifiers
            .iter()
            .any(|m| matches!(m.effect, Effect::Disadvantage));
        (up != down).then_some(up)
    }

    /// Draw it.
    ///
    /// **Every path draws [`draws`](Self::draws) times, with no early return.**
    /// That is CLAUDE.md's determinism rule stated as code: a draw skipped
    /// because the first one already settled the question would make the stream
    /// depend on its own results, and every replay past that point would diverge.
    pub fn resolve(&self, rngs: &mut Rngs) -> Landed {
        let die = self.effective_die();
        let faces: Vec<u32> = (0..self.draws()).map(|_| die.draw(rngs)).collect();

        let face = match self.edge() {
            Some(true) => faces.iter().copied().max().unwrap_or(1),
            Some(false) => faces.iter().copied().min().unwrap_or(1),
            None => faces.first().copied().unwrap_or(1),
        };

        let total = i32::try_from(face).unwrap_or(i32::MAX) + self.bonus();
        Landed {
            die,
            face,
            faces,
            total,
            against: self.against,
            tells: total >= self.against,
        }
    }

    /// The chance this tells, as a percentage, without drawing anything.
    ///
    /// **Shown before the commitment, which is the XCOM bargain and the rule that
    /// keeps a rolled outcome fair**: a decision under known risk is a decision;
    /// a surprise is not. §5.1 wants the skill to be reading the board, so the
    /// board has to be readable.
    /// **Integer arithmetic throughout**, deliberately. A probability in floats
    /// is the obvious way to write this and it costs three casts back to `u32`,
    /// each of which is a lint about truncation and sign that has to be either
    /// silenced or wrapped. Counting *faces* instead is exact, needs no cast at
    /// all, and is closer to what the thing actually is: a die has a whole
    /// number of faces and some whole number of them tell.
    #[must_use]
    pub fn chance(&self) -> u32 {
        // **Asked once.** It was called twice here, and the second call is the
        // kind of thing that stops matching the first the day an `Upgrade` is
        // conditional.
        let die = self.effective_die();
        let faces = u64::from(die.faces());
        let bonus = self.bonus();
        // How many single faces would tell.
        let telling = (1..=die.faces())
            .filter(|face| i32::try_from(*face).unwrap_or(i32::MAX) + bonus >= self.against)
            .count();
        let telling = u64::try_from(telling).unwrap_or(0);
        let missing = faces - telling;

        // Advantage tells unless *both* draws miss; disadvantage needs both.
        // Over two draws the sample space is `faces * faces`, so both cases are
        // exact in integers. `edge` is the same predicate `draws` and `resolve`
        // ask, which is what keeps the odds a statement about the roll that will
        // actually be made.
        let (told, over) = match self.edge() {
            None => (telling, faces),
            Some(true) => (faces * faces - missing * missing, faces * faces),
            Some(false) => (telling * telling, faces * faces),
        };

        // Rounded to nearest, so 50.5% reads as 51 rather than 50 — the board
        // is a decision aid and a systematic downward bias would understate
        // every choice it is there to inform.
        u32::try_from((told * 200 + over) / (over * 2)).unwrap_or(100)
    }
}

/// A roll, drawn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Landed {
    /// Which die was actually drawn, after upgrades.
    pub die: Die,
    /// The face that counted.
    pub face: u32,
    /// Every face drawn, in order — two under advantage, one otherwise.
    ///
    /// Kept so the log can say *"18 and 4, kept 18"*: a player who is told only
    /// the kept face cannot see that a potion did anything.
    pub faces: Vec<u32>,
    /// Face plus bonuses.
    pub total: i32,
    /// What it had to reach.
    pub against: i32,
    /// Whether it did.
    pub tells: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rngs() -> Rngs {
        Rngs::from_seed(11)
    }

    #[test]
    fn the_seven_are_the_seven() {
        // The homage, asserted — a die added or removed should be a decision,
        // not a diff nobody noticed.
        let faces: Vec<u32> = Die::ALL.into_iter().map(Die::faces).collect();
        assert_eq!(faces, vec![4, 6, 8, 10, 12, 20, 100]);
        let words: Vec<&str> = Die::ALL.into_iter().map(Die::word).collect();
        assert_eq!(words, vec!["d4", "d6", "d8", "d10", "d12", "d20", "d100"]);
    }

    #[test]
    fn every_die_names_itself_and_reads_back() {
        for die in Die::ALL {
            assert_eq!(
                Die::named(die.word()),
                Some(die),
                "{} not round-tripped",
                die.word()
            );
        }
        assert_eq!(
            Die::named("d7"),
            None,
            "a die nobody has should not resolve"
        );
    }

    #[test]
    fn a_face_is_never_nought_and_never_over_the_die() {
        let mut rngs = rngs();
        for die in Die::ALL {
            for _ in 0..500 {
                let face = die.draw(&mut rngs);
                assert!(
                    (1..=die.faces()).contains(&face),
                    "{} rolled {face}",
                    die.word(),
                );
            }
        }
    }

    #[test]
    fn the_draw_count_is_decided_before_the_draw() {
        // **The property the whole shape exists for.** A roll that drew again
        // *because* it missed would make the stream depend on its own results —
        // §19's `drift` defect — so the count must be readable from the composed
        // value alone.
        assert_eq!(Roll::new(Die::D20, 11).draws(), 1);
        assert_eq!(
            Roll::new(Die::D20, 11)
                .plus("haste", Effect::Advantage)
                .draws(),
            2,
        );
        // Both at once is neither, which is the tabletop rule and keeps the
        // count a pure function.
        assert_eq!(
            Roll::new(Die::D20, 11)
                .plus("haste", Effect::Advantage)
                .plus("mire", Effect::Disadvantage)
                .draws(),
            1,
        );
    }

    #[test]
    fn resolving_consumes_exactly_the_draws_it_promised() {
        // Same seed, two rolls: the second must start where the first left off,
        // by exactly the promised number of draws and never by outcome.
        for roll in [
            Roll::new(Die::D20, 11),
            Roll::new(Die::D20, 11).plus("haste", Effect::Advantage),
            Roll::new(Die::D20, 99).plus("haste", Effect::Advantage),
        ] {
            let mut a = rngs();
            let landed = roll.resolve(&mut a);
            assert_eq!(
                u32::try_from(landed.faces.len()).unwrap_or(u32::MAX),
                roll.draws(),
                "a roll drew a different number of faces than it promised",
            );
        }
    }

    #[test]
    fn advantage_keeps_the_better_and_disadvantage_the_worse() {
        let mut rngs = rngs();
        for _ in 0..200 {
            let up = Roll::new(Die::D20, 11)
                .plus("haste", Effect::Advantage)
                .resolve(&mut rngs);
            assert_eq!(up.face, up.faces.iter().copied().max().unwrap_or(0));
            let down = Roll::new(Die::D20, 11)
                .plus("mire", Effect::Disadvantage)
                .resolve(&mut rngs);
            assert_eq!(down.face, down.faces.iter().copied().min().unwrap_or(0));
        }
    }

    #[test]
    fn the_largest_upgrade_wins_so_a_second_buff_is_never_a_mistake() {
        let roll = Roll::new(Die::D6, 4)
            .plus("keen", Effect::Upgrade(Die::D12))
            .plus("dull", Effect::Upgrade(Die::D8));
        assert_eq!(roll.effective_die(), Die::D12);
        // ...and an upgrade smaller than the base die changes nothing.
        let roll = Roll::new(Die::D20, 11).plus("dull", Effect::Upgrade(Die::D4));
        assert_eq!(roll.effective_die(), Die::D20);
    }

    #[test]
    fn a_bonus_moves_the_total_and_not_the_face() {
        let mut rngs = rngs();
        let landed = Roll::new(Die::D20, 11)
            .plus("mending", Effect::Bonus(2))
            .resolve(&mut rngs);
        assert_eq!(landed.total, i32::try_from(landed.face).unwrap_or(0) + 2);
    }

    #[test]
    fn the_stated_chance_is_the_chance_it_actually_lands() {
        // **The instrument the odds-altering arsenal needs.** Without this a
        // modifier that silently does nothing is invisible — so the claim on
        // screen is checked against the dice, not against itself.
        let mut rngs = rngs();
        for roll in [
            Roll::new(Die::D20, 11),
            Roll::new(Die::D20, 15),
            Roll::new(Die::D6, 4),
            Roll::new(Die::D20, 11).plus("mending", Effect::Bonus(3)),
            Roll::new(Die::D20, 11).plus("haste", Effect::Advantage),
            Roll::new(Die::D20, 11).plus("mire", Effect::Disadvantage),
        ] {
            let runs: u64 = 20_000;
            let told = (0..runs).filter(|_| roll.resolve(&mut rngs).tells).count();
            // Integers, like `chance` itself — the two have to be comparable and
            // a float here would reintroduce the casts that function avoids.
            let measured = (u64::try_from(told).unwrap_or(0) * 100) / runs;
            let claimed = u64::from(roll.chance());
            assert!(
                measured.abs_diff(claimed) < 2,
                "{roll:?} claims {claimed}% and lands {measured}%",
            );
        }
    }

    #[test]
    fn a_certainty_and_an_impossibility_are_said_plainly() {
        // 0 and 100 have to be exact, or the board tells a player a sure thing
        // might fail.
        assert_eq!(Roll::new(Die::D20, 0).chance(), 100);
        assert_eq!(Roll::new(Die::D20, 21).chance(), 0);
        // A bonus that makes an impossibility certain.
        assert_eq!(
            Roll::new(Die::D20, 21)
                .plus("m", Effect::Bonus(20))
                .chance(),
            100,
        );
        // ...and one that only reaches it on the best face. Against 21 with +1,
        // a natural 20 is the single telling face — 5%, not certainty, and this
        // assertion said 100 while the arithmetic said 5.
        assert_eq!(
            Roll::new(Die::D20, 21).plus("m", Effect::Bonus(1)).chance(),
            5,
        );
    }

    #[test]
    fn every_face_of_every_die_comes_up() {
        // **A die that never rolls its highest face is a die with a silent
        // off-by-one**, and an aggregate cannot see it: a d20 rolling 1..=19
        // has a mean of 10 against a true 10.5, which is well inside any
        // tolerance a rate test would use. Coverage is what catches it.
        let mut rngs = rngs();
        for die in Die::ALL {
            let mut seen = std::collections::BTreeSet::new();
            // Enough draws that a fair die is overwhelmingly likely to have
            // shown every face; a coupon-collector bound for d100 is ~519.
            for _ in 0..(die.faces() * 60) {
                seen.insert(die.draw(&mut rngs));
            }
            let missing: Vec<u32> = (1..=die.faces()).filter(|f| !seen.contains(f)).collect();
            assert!(
                missing.is_empty(),
                "{} never rolled {missing:?}",
                die.word(),
            );
        }
    }

    #[test]
    fn every_die_is_roughly_fair() {
        // Not a rigorous test of the generator — `ChaCha8` is not what is under
        // test — but enough to catch a modulo bias or a truncated range, which
        // would show as one face appearing far more or less than its share.
        let mut rngs = rngs();
        for die in Die::ALL {
            let runs = u64::from(die.faces()) * 400;
            let mut counts = vec![0_u64; die.faces() as usize + 1];
            for _ in 0..runs {
                counts[die.draw(&mut rngs) as usize] += 1;
            }
            let want = runs / u64::from(die.faces());
            for face in 1..=die.faces() {
                let got = counts[face as usize];
                assert!(
                    got * 100 > want * 60 && got * 100 < want * 160,
                    "{} face {face} came up {got} times against an expected {want}",
                    die.word(),
                );
            }
        }
    }

    #[test]
    fn the_stated_chance_matches_every_die_and_not_only_the_d20() {
        // `the_stated_chance_is_the_chance_it_actually_lands` checks a handful
        // of rolls, all but one of them a d20. The board prints odds for
        // whatever die a modifier upgraded to, so every die's arithmetic has to
        // be right — a `d100` reading 50% and landing 5% is the shape that would
        // hide here.
        let mut rngs = rngs();
        for die in Die::ALL {
            for against in [1_i32, 2, i32::try_from(die.faces()).unwrap_or(20) / 2] {
                let roll = Roll::new(die, against);
                let runs: u64 = 8_000;
                let told = (0..runs).filter(|_| roll.resolve(&mut rngs).tells).count();
                let measured = (u64::try_from(told).unwrap_or(0) * 100) / runs;
                let claimed = u64::from(roll.chance());
                assert!(
                    measured.abs_diff(claimed) <= 2,
                    "{} against {against} claims {claimed}% and lands {measured}%",
                    die.word(),
                );
            }
        }
    }

    #[test]
    fn the_same_seed_rolls_the_same_dice() {
        // Rule 3, at the level this domain touches it.
        let roll = Roll::new(Die::D20, 11).plus("haste", Effect::Advantage);
        let mut a = rngs();
        let mut b = rngs();
        for _ in 0..50 {
            assert_eq!(roll.resolve(&mut a), roll.resolve(&mut b));
        }
    }
}
