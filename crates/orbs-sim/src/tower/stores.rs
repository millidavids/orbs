//! How well stocked the tower is in a thing, measured as a **rate** (§11.5, §19).
//!
//! # Why a rate, and not a timestamp, and not a count
//!
//! §19 withdrew a perishable arsenal because `Stock::Counted(u32)` is *"one node
//! per kind, with a count — not one node per unit"*, so a timer *"either keeps a
//! thousand potions fresh off one restock — cheaper than playing normally — or
//! makes new stock unusable."*
//!
//! **A freshness clock keyed to *when one was last made* is that first branch
//! verbatim**, and a first draft of this feature was exactly that before the
//! objection was read properly. One `made:warding` would refresh five hundred
//! wardings, so a brew every twenty-five minutes would hold an unbounded hoard at
//! full strength for less work than playing.
//!
//! **A rate defeats it without touching the primitive.** What is asked is not
//! *when* did you last make one but *how many have you made lately* — so one
//! restock is a rate of one, which is thin at best, and holding a thousand is
//! worth exactly what your current industry is worth. **Total stock never enters
//! the arithmetic**, which is the property that makes this buildable on `Stock`
//! untouched: no per-unit batches, no rewrite, no save migration.
//!
//! # It does not decay while the game is closed, and that needs no code
//!
//! A tick is *"one real second while the window is open"*: the counter is
//! restored verbatim from a save and there is no wall clock anywhere in the sim,
//! because offline progression is a later phase. So a closed game advances
//! nothing and the stores are as they were left.
//!
//! **And when offline progression lands, they will thin across an absence
//! automatically** — catch-up advances the tick, and every expression below
//! starts spanning the gap with nothing rewritten. Correct now, correct later,
//! one arithmetic. That is deliberate rather than incidental.

use std::collections::BTreeMap;

use bevy_ecs::prelude::*;

/// How long a making counts toward the rate, in ticks.
///
/// Half an hour, against a siege cadence of twenty minutes and a clarity that
/// takes under two. Long enough that a player working by hand keeps the two or
/// three things they actually use, short enough that a full shelf wants a spell.
pub const WINDOW: u64 = 1_800;

/// Makings inside [`WINDOW`] to keep a store at full strength.
pub const FRESH_AT: usize = 3;

/// Makings inside [`WINDOW`] to keep a store worth anything at all.
pub const THIN_AT: usize = 1;

/// The most makings worth remembering for one name.
///
/// A ring rather than a growing list: nothing above [`FRESH_AT`] changes any
/// answer, so a spell hammering one recipe cannot grow the save. The oldest is
/// dropped, which is the right end — the rate is about *lately*.
const REMEMBERED: usize = FRESH_AT;

/// How well stocked the tower is in one thing.
///
/// **Three words, because the domain's readings are words.** `few`, `hurt` and
/// `outnumbered` are how this game says a derived fact, and a spell asks for them
/// by name — so a fraction here would be the odd one out and unaskable besides.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Supply {
    /// Made often enough to be worth its full authored strength.
    Fresh,
    /// Made, but not lately. Worth half, rounded down.
    Thin,
    /// Not made in long enough that the stores are out. Worth nothing.
    Spent,
}

impl Supply {
    /// The word the panel prints and a spell asks for.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Fresh => "fresh",
            Self::Thin => "thin",
            Self::Spent => "spent",
        }
    }

    /// Every word, for the lints that walk them.
    pub const ALL: [Self; 3] = [Self::Fresh, Self::Thin, Self::Spent];

    /// What `amount` of an authored effect is worth at this standing.
    ///
    /// **Half, rounded down, and nothing at all when spent.** A magnitude that
    /// halves to nought meets the existing *"a potion that would do nothing is
    /// refused rather than drunk"* guard, which is the right answer and needs no
    /// branch here.
    #[must_use]
    pub const fn scale(self, amount: u32) -> u32 {
        match self {
            Self::Fresh => amount,
            Self::Thin => amount / 2,
            Self::Spent => 0,
        }
    }

    /// What a **signed** magnitude is worth at this standing.
    ///
    /// `Effect::Bonus` may be negative — a `disadvantage`-shaped penalty is
    /// authored as one — so halving has to move it *toward nought* from either
    /// side rather than toward negative infinity. A thin curse is a weaker
    /// curse, not a worse one.
    #[must_use]
    pub const fn scale_signed(self, amount: i32) -> i32 {
        match self {
            Self::Fresh => amount,
            Self::Thin => amount / 2,
            Self::Spent => 0,
        }
    }

    /// Whether an effect carrying **no magnitude** still works at this standing.
    ///
    /// **A boolean effect has no half, so it works until it does not.**
    /// `advantage` is draw-twice-keep-the-best and `upgrade` is a bigger die;
    /// neither can be diluted, so the only honest answers are *yes* and *no*.
    ///
    /// **They survive `Thin`, and a first pass had them dropping there.** Two
    /// shipped spendables are `advantage`, and one making is a rate of one — so
    /// dropping at thin meant **a player who brewed a single `haste` could never
    /// use it**, which is the arsenal breaking a thing you made rather than
    /// pricing how much you make. The matrix test that walks every authored row
    /// caught it, which is what that test is for.
    #[must_use]
    pub const fn keeps_whole(self) -> bool {
        !matches!(self, Self::Spent)
    }
}

/// When the tower last made each thing, lately.
///
/// One short, bounded list per name. `BTreeMap` for `Tally`'s reason: a save is
/// read by people, and key order is what makes a diff legible.
#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct Stores(BTreeMap<String, Vec<u64>>);

impl Stores {
    /// Record that one `named` was made at `now`.
    pub fn made(&mut self, named: &str, now: u64) {
        let makings = self.0.entry(named.to_owned()).or_default();
        makings.push(now);
        prune(makings, now);
        // Keep the newest, drop the oldest: the rate is about *lately*, and
        // nothing past `FRESH_AT` changes an answer.
        if makings.len() > REMEMBERED {
            let excess = makings.len() - REMEMBERED;
            makings.drain(..excess);
        }
    }

    /// How many `named` were made inside [`WINDOW`] of `now`.
    #[must_use]
    pub fn rate(&self, named: &str, now: u64) -> usize {
        self.0
            .get(named)
            .map(|makings| makings.iter().filter(|at| inside(**at, now)).count())
            .unwrap_or_default()
    }

    /// How well stocked the tower is in `named`.
    #[must_use]
    pub fn store(&self, named: &str, now: u64) -> Supply {
        match self.rate(named, now) {
            rate if rate >= FRESH_AT => Supply::Fresh,
            rate if rate >= THIN_AT => Supply::Thin,
            _ => Supply::Spent,
        }
    }

    /// Every name and the ticks it was made at, in key order — for a save.
    pub fn entries(&self) -> impl Iterator<Item = (&str, &[u64])> {
        self.0
            .iter()
            .map(|(name, makings)| (name.as_str(), makings.as_slice()))
    }

    /// Put stores back, for a save.
    pub(crate) fn restore(&mut self, entries: impl IntoIterator<Item = (String, Vec<u64>)>) {
        self.0 = entries.into_iter().collect();
    }

    /// Whether anything has ever been recorded.
    ///
    /// **What tells a fresh tower from an old save.** A document written before
    /// this existed says nothing, and reading that as *every store is spent*
    /// would empty a returning player's arsenal — so `restore` stamps instead.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Whether a making at `at` still counts at `now`.
const fn inside(at: u64, now: u64) -> bool {
    now.saturating_sub(at) < WINDOW
}

/// Drop makings that have fallen out of the window.
fn prune(makings: &mut Vec<u64>, now: u64) {
    makings.retain(|at| inside(*at, now));
}

/// Record that one `named` was made, now.
pub fn made(world: &mut World, named: &str) {
    let now = world.resource::<crate::tick::Tick>().get();
    world.resource_mut::<Stores>().made(named, now);
}

/// How well stocked the tower is in `named`, now.
#[must_use]
pub fn supply_of(world: &World, named: &str) -> Supply {
    let now = world.resource::<crate::tick::Tick>().get();
    world.resource::<Stores>().store(named, now)
}

/// The set an arsenal item belongs to, so a spell can walk them.
///
/// **Singular, because the word names the cursor as well as the set** — `for
/// each store` binds `store`, and the body reads `if store has thin`.
pub const STORE: &str = "store";

/// The words an arsenal item answers `has` with.
#[must_use]
pub fn readings() -> Vec<&'static str> {
    Supply::ALL.into_iter().map(Supply::word).collect()
}

/// Keep every arsenal item's group and reading in step with its store.
///
/// # Why this is a system and not a `publish` call
///
/// Every other reading in the tower is republished by the thing that *changed*
/// it — `defend::publish` after a round, `erode` after wear. A store has no such
/// moment: it changes because **time passed**, and nothing else happened. So the
/// clock is what has to notice, and this is the one place that does.
///
/// **It writes only when the word changes.** Despawning and respawning a reading
/// every tick would issue new `NodeId`s every tick, and §19 makes insertion order
/// *the parse* — a live-watched hour and a `meditate`-collapsed one must issue
/// identical ids in identical order. Idle ticks touch nothing.
///
/// The group marker is idempotent and goes on regardless, which is what lets a
/// spell walk an arsenal it has never surveyed.
pub fn stocktake(world: &mut World) {
    let Some(arsenal) = super::keep(world) else {
        return;
    };
    let now = world.resource::<Stores>().clone();
    let tick = world.resource::<crate::tick::Tick>().get();
    for node in super::children_of(world, arsenal) {
        let Some(named) = world.get::<super::Name>(node).map(|name| name.0.clone()) else {
            continue;
        };
        // A log is a child of the arsenal too, and it has no store.
        if world.get::<super::Stock>(node).is_none() {
            continue;
        }
        world
            .entity_mut(node)
            .insert(super::Grouped(STORE.to_owned()));

        let want = now.store(&named, tick).word();
        let said: Vec<Entity> = super::children_of(world, node)
            .into_iter()
            .filter(|child| {
                world
                    .get::<super::Name>(*child)
                    .is_some_and(|name| readings().contains(&name.0.as_str()))
            })
            .collect();
        if said.len() == 1
            && world
                .get::<super::Name>(said[0])
                .is_some_and(|name| name.0 == want)
        {
            continue;
        }
        for stale in said {
            world.entity_mut(stale).despawn();
        }
        super::build::raise_reading(world, node, want);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_restock_is_never_fresh_however_much_it_restocks() {
        // **The hole this whole file exists to close.** §19 withdrew a
        // perishable arsenal because a timer *"keeps a thousand potions fresh
        // off one restock — cheaper than playing normally"*. A rate cannot: one
        // making is one, whatever it stocked.
        let mut stores = Stores::default();
        stores.made("warding", 0);
        assert_eq!(stores.store("warding", 0), Supply::Thin);
        assert_eq!(stores.store("warding", 10), Supply::Thin);
    }

    #[test]
    fn the_rate_is_what_reads_and_the_pile_is_never_asked_about() {
        // Total stock does not appear in this file's arithmetic at all, which is
        // what lets `Stock::Counted` stay exactly as it is. Three makings is
        // fresh whether the shelf holds three or three hundred.
        let mut stores = Stores::default();
        for at in 0..FRESH_AT as u64 {
            stores.made("warding", at);
        }
        assert_eq!(stores.store("warding", 0), Supply::Fresh);
    }

    #[test]
    fn a_store_thins_and_then_runs_out_as_the_window_passes() {
        let mut stores = Stores::default();
        for at in 0..FRESH_AT as u64 {
            stores.made("warding", at);
        }
        assert_eq!(stores.store("warding", 0), Supply::Fresh);
        // **Thinning is the middle of it, not the end.** The oldest makings fall
        // out first, so a store passes through `Thin` on its way — which is what
        // gives a player warning rather than a cliff.
        assert_eq!(stores.store("warding", WINDOW), Supply::Thin);
        // ...and past the newest making's own window, it is out. `WINDOW + 1` is
        // *not* far enough and a first draft of this test said it was: the last
        // making was at tick 2, so it is still counted until 1,802.
        assert_eq!(stores.store("warding", WINDOW + 10), Supply::Spent);
    }

    #[test]
    fn a_thing_never_made_is_spent() {
        let stores = Stores::default();
        assert_eq!(stores.store("warding", 0), Supply::Spent);
    }

    #[test]
    fn what_is_remembered_is_bounded_however_hard_a_spell_hammers_it() {
        // A bound spell can make thousands. The save may not grow with them.
        let mut stores = Stores::default();
        for at in 0..1_000 {
            stores.made("clarity", at);
        }
        let (_, makings) = stores.entries().next().expect("something was made");
        assert!(
            makings.len() <= REMEMBERED,
            "a thousand makings kept {} entries",
            makings.len(),
        );
        assert_eq!(stores.store("clarity", 1_000), Supply::Fresh);
    }

    #[test]
    fn a_boolean_effect_has_no_half_so_it_works_until_it_does_not() {
        // `advantage` is draw-twice; there is no halving it. **It survives
        // `Thin`**, because one making is a rate of one — and dropping it there
        // would mean a player who brewed a single `haste` could never use it.
        assert!(Supply::Fresh.keeps_whole());
        assert!(Supply::Thin.keeps_whole());
        assert!(!Supply::Spent.keeps_whole());
        // ...and a magnitude halves, rounding down into the existing
        // "would do nothing, so it is refused and kept" guard.
        assert_eq!(Supply::Fresh.scale(3), 3);
        assert_eq!(Supply::Thin.scale(3), 1);
        assert_eq!(Supply::Thin.scale(1), 0);
        assert_eq!(Supply::Spent.scale(9), 0);
    }
}
