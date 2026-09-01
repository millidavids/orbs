//! The expensive audit, and what limits the cheap one (DESIGN.md §8.1).
//!
//! §8.1 gives `verify` two forms and prices them against each other:
//!
//! > **`verify --all` is Production-class, and its duration scales with the
//! > tower.** Auditing everything takes time proportional to bound scripts plus
//! > log volume … **A flat 30-second audit would be trivially correct to open
//! > every siege with, "which surface do I inspect first" would stop being a
//! > decision, and the four-surface model would collapse on move one.**
//!
//! > **`verify <target>` carries a short per-surface cooldown.** There are only
//! > four surfaces; four free instant checks *are* `verify --all` by another
//! > name.
//!
//! Neither shipped. `sabotage::verify`'s own doc has said *"that arrives with
//! the remaining surfaces in Phase 1"* since Phase 0, and **Phase 1 closed
//! without it** — so until now four instant checks audited the whole tower for
//! nothing, which is the collapse §8.1 names. This is that debt, paid.
//!
//! # Why it is worth paying now rather than with the siege
//!
//! The adversarial surfaces are siege-only (§5.1), but the *environmental* ones
//! are not: `sabotage::drift` poisons a log every ~300 ticks and
//! `sabotage::substitution` swaps a base reagent every ~3600, in the calm layer,
//! today. A player who notices something is wrong already has the choice §8.1
//! describes — **is it the log lying, or the shelf?** — and until this file
//! existed that choice cost nothing, so it was not a choice.

use bevy_ecs::prelude::*;

use crate::tick::Tick;

/// One of §8.1's tamperable surfaces.
///
/// # Two, where §8.1 names four
///
/// *World state* and *logs* ship; *script text* and *trigger clocks* have no
/// producer — §19 moved them to Phase 8, *"where their producer is"*. They are
/// deliberately **not** enumerated here ahead of that: a variant nothing can
/// ever be is a cooldown nobody can ever be on, and CLAUDE.md's standing
/// objection to an API with no callers applies to an enum arm as much as to a
/// function.
///
/// Adding the other two is then one variant each plus one arm in [`Surface::of`], which
/// is the shape [`chant::readings`](super::chant::readings) grows by.
///
/// **Two is already a decision**, which is the thing to check before dismissing
/// it as too few: a poisoned log and a swapped pile are the two ways the tower
/// lies to you, they are told apart by *which* surface you look at, and the
/// cooldown is what makes looking at the wrong one cost something.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Surface {
    /// A `.log` file, whose lines can be forged (`sabotage::Log`).
    Log,
    /// A pile of stock, which can be swapped for something wearing its name.
    World,
    /// A `.spell` file, whose lines an enemy can rewrite (`tower::Held`).
    ///
    /// **Siege-only, and that is §5.1 rather than an omission.** *"The enemy
    /// never touches scripts, schedules, or logs in the calm layer. Phase A
    /// stays genuinely safe, which pillar 4 requires."* So this surface has no
    /// ambient producer at all — `drift` and `substitution` cannot reach it —
    /// and it only ever lies while a siege is being fought.
    Script,
    /// A bound spell's schedule, which an enemy can retime (`spell::Bound`).
    ///
    /// Siege-only, for [`Script`](Self::Script)'s reason. It is the subtlest of
    /// the four: nothing about the spell's *text* is wrong, so `peruse` shows a
    /// correct spell and only the clock is lying.
    Clock,
}

impl Surface {
    /// Every surface that ships.
    ///
    /// **All four now**, which closes §8.1's model — the two ambient ones and
    /// the two the siege brought. `Cooling` is keyed by *word*, so adding these
    /// renumbered nothing in a save.
    pub const ALL: [Self; 4] = [Self::Log, Self::World, Self::Script, Self::Clock];

    /// How many there are.
    pub const COUNT: usize = Self::ALL.len();

    /// Where this one sits in [`Cooling`]'s array.
    ///
    /// **Private to the cooldown's storage**, unlike `RngStream::index`, which
    /// is a save-format commitment. Nothing outside this module can see the
    /// number, so renumbering is free — which is why the save is keyed by
    /// [`word`](Self::word) instead.
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Log => 0,
            Self::World => 1,
            Self::Script => 2,
            Self::Clock => 3,
        }
    }

    /// The word a record carries for it.
    ///
    /// A **table, not prose** — the same exemption `Verb::canonical` has. It is
    /// a field value that `sift` matches and §14 speaks, not an authored
    /// sentence, and the sentence around it is composed in `content/prose.toml`
    /// like every other.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Log => "log",
            Self::World => "world",
            Self::Script => "script",
            Self::Clock => "clock",
        }
    }

    /// Which surface a node belongs to.
    ///
    /// `None` for a node that is neither — a spell, a place, an instrument.
    /// Those are still `verify`-able and still answer `sound`; what they are not
    /// is *rationed*, because rationing a check that can never find anything
    /// would be a cooldown with no information behind it.
    #[must_use]
    pub fn of(world: &World, node: Entity) -> Option<Self> {
        if world.get::<super::sabotage::Log>(node).is_some() {
            return Some(Self::Log);
        }
        // **A spell is two surfaces at once**, and the text is the one a
        // targeted `verify` reaches: a player checking a spell is reading it.
        // The clock is checked by looking at what is *bound*, which is the
        // audit's business rather than a target's.
        // **A spell is two surfaces, and which one it *is* depends on how it was
        // reached.** A retimed spell's text is perfect — that is what makes the
        // clock the subtlest of the four — so categorising every spell as
        // `Script` left `Clock` unreachable: `Cooling.until[3]` was never
        // written, `to_save` could never emit a `["clock", n]` row, and
        // `Surface::ALL`'s claim to *close* §8.1's model was one surface short.
        //
        // That is exactly the objection this file raises two paragraphs up for
        // why `Script` and `Clock` were withheld in the first place — *a variant
        // nothing can ever be is a cooldown nobody can ever be on*.
        if world.get::<super::Retimed>(node).is_some() {
            return Some(Self::Clock);
        }
        if world.get::<super::Held>(node).is_some() {
            return Some(Self::Script);
        }
        // **The place, not the pile.** `sabotage::verify` reports a shelf by
        // looking one level down at what stands in it, so the surface a player
        // is checking when they type `verify dispensary` is the world — and
        // asking the shelf itself for a `Stock` would answer `None` and leave
        // the commonest world-surface check unrationed.
        let holds_stock = super::children_of(world, node)
            .into_iter()
            .any(|held| world.get::<super::Stock>(held).is_some());
        holds_stock.then_some(Self::World)
    }
}

/// How long one surface stays unavailable after a targeted check.
///
/// **Short, which is §8.1's own word**, and measured against what else a tick
/// buys: a clarity brew is 56 ticks and a purge is 4, so twenty seconds is long
/// enough that checking the wrong surface is a real loss and short enough that
/// it never becomes the reason a player stops looking.
///
/// It rations the *surface*, never the target. `verify laboratory.log` puts
/// every log beyond reach, because §8.1's decision is *which surface do I
/// inspect first* — and a per-target cooldown would leave a player free to walk
/// the six domain logs in six ticks, which is the collapse this exists to stop
/// wearing a longer name.
pub const COOLING: u64 = 20;

/// The floor on a full audit, before the tower is weighed.
///
/// §8.1 rules out *"a flat 30-second audit"* explicitly, so this is the *base*
/// of a sum rather than the answer: a bare tower audits in about this, and every
/// spell bound and every line logged pushes it up from here.
pub const AUDIT_BASE: u64 = 20;

/// What each held spell adds to a full audit.
///
/// §8.1: *"time proportional to **bound scripts** plus log volume"*. A bound
/// spell is a surface the enemy will eventually be able to rewrite (§8.1's
/// script-text row, Phase 8), so auditing one is real work — and it means the
/// player who automates hardest pays most to check their automation, which is
/// pillar 3 stated as a duration.
pub const AUDIT_PER_SPELL: u64 = 8;

/// How many logged lines add one tick to a full audit.
///
/// The *"plus log volume"* half. Divided rather than multiplied because the
/// stream is thousands of records deep by mid-game and a tick each would make
/// the audit longer than the session.
pub const AUDIT_PER_LINES: u64 = 20;

/// The longest a full audit will ever take, however large the tower grows.
///
/// **A clamp rather than a refusal**, on `LONGEST_BIDE`'s argument: the number
/// is a product of the player's own sprawl and there is no honest place to
/// refuse it, but an audit longer than `MAX_MEDITATE` would be indistinguishable
/// from one that never lands. Ten minutes is far past any tower the balance
/// harness has swept and still bounded.
pub const AUDIT_LONGEST: u64 = 600;

/// When each surface may be checked again.
///
/// **A resource keyed by surface, not a component on the target.** The thing
/// being rationed is the *surface* — see [`COOLING`] — so hanging it on a node
/// would mean writing it to every log in the tower and keeping six copies in
/// step.
///
/// It travels in the save. A cooldown that reset on load would be a cooldown a
/// player could clear by quitting, which is the shape §19 calls an exploit that
/// then needs its own rule.
#[derive(Resource, Debug, Clone, Default, PartialEq, Eq)]
pub struct Cooling {
    /// When each surface is next available, indexed by [`Surface::index`].
    ///
    /// **An array, where this was a named field per surface.** At two surfaces
    /// the fields were clearer; at four they were the same `match` written out
    /// three times — `spend`, `from_save` and `at` — which is three places for a
    /// fifth surface to be forgotten in. The *save* is still keyed by name, and
    /// that is the part that had to be: an index here is private and can be
    /// renumbered freely, where a renumbered save would silently remap a
    /// player's cooldowns.
    until: [Option<Tick>; Surface::COUNT],
}

impl Cooling {
    /// How long `surface` has left, or `None` when it is ready.
    ///
    /// **Zero reads as ready**, so the tick a cooldown expires on is a tick the
    /// player may check — the same `>=` boundary `bide` and `Working` both use,
    /// and the one that stops a twenty-tick wait being a twenty-one-tick one.
    #[must_use]
    pub fn left(&self, surface: Surface, now: Tick) -> Option<u64> {
        let until = self.at(surface)?;
        let left = until.get().saturating_sub(now.get());
        (left > 0).then_some(left)
    }

    /// Start `surface` cooling from `now`.
    pub const fn spend(&mut self, surface: Surface, now: Tick) {
        self.until[surface.index()] = Some(Tick::new(now.get().saturating_add(COOLING)));
    }

    /// What a save writes: the surfaces that *are* cooling, and until when.
    ///
    /// **Pairs rather than a slot per surface, and TOML is why.** The obvious
    /// shape is `Vec<Option<u64>>` positional on [`Surface::ALL`], and `toml`
    /// refuses it outright — *"unsupported None value"*, because the format has
    /// no null. Naming each surface is also the better file: §15 wants a save a
    /// person can read, and `[["log", 120]]` says what a bare `[null, 120]`
    /// cannot.
    ///
    /// It means the order here carries no meaning, which is the point — and it
    /// was the point: the siege added two surfaces and this renumbered nothing,
    /// so a save written before them still reads exactly right.
    ///
    /// **Only what is still cooling, which is what the heading says and what it
    /// did not do.** `spend` stores a tick and nothing ever clears it, so this
    /// filtered on *ever checked* rather than *still cooling* — and once a player
    /// had run `verify` on each of the four surfaces, every autosave from then
    /// on carried four rows whose ticks were thousands of ticks in the past.
    /// [`left`](Self::left) already draws that line for the reader; this now
    /// draws it for the writer.
    #[must_use]
    pub fn to_save(&self, now: Tick) -> Vec<(String, u64)> {
        Surface::ALL
            .into_iter()
            .filter_map(|surface| {
                self.left(surface, now)?;
                self.at(surface)
                    .map(|until| (surface.word().to_owned(), until.get()))
            })
            .collect()
    }

    /// Read one back.
    ///
    /// **A missing row is not an error**, which is the same tolerance every
    /// other `from_save` here has: a document written before this existed has no
    /// table at all, and the honest reading of that is *nothing is cooling*. A
    /// surface the build no longer knows is dropped for the same reason.
    #[must_use]
    pub fn from_save(saved: &[(String, u64)]) -> Self {
        let mut cooling = Self::default();
        for (word, until) in saved {
            let Some(surface) = Surface::ALL
                .into_iter()
                .find(|surface| surface.word() == word)
            else {
                continue;
            };
            cooling.until[surface.index()] = Some(Tick::new(*until));
        }
        cooling
    }

    const fn at(&self, surface: Surface) -> Option<Tick> {
        self.until[surface.index()]
    }
}

/// How long a full audit of this tower takes.
///
/// §8.1's *"proportional to bound scripts plus log volume"*, as a sum with a
/// floor and a ceiling. It is deliberately a **pure function of two counts**, so
/// the number a player is quoted before the audit starts is the number they
/// wait, and a test can assert the curve without running one.
#[must_use]
pub fn ticks_for(spells: usize, logged: usize) -> u64 {
    let spells = u64::try_from(spells).unwrap_or(u64::MAX);
    let logged = u64::try_from(logged).unwrap_or(u64::MAX);
    AUDIT_BASE
        .saturating_add(spells.saturating_mul(AUDIT_PER_SPELL))
        .saturating_add(logged / AUDIT_PER_LINES)
        .min(AUDIT_LONGEST)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The curve §8.1 asks for**, in one assertion: a bare tower is quick, and
    /// it grows with the two things named.
    #[test]
    fn an_audit_grows_with_the_tower_it_audits() {
        let bare = ticks_for(0, 0);
        assert_eq!(bare, AUDIT_BASE);

        // Each bound spell is a surface to walk.
        assert_eq!(ticks_for(1, 0), AUDIT_BASE + AUDIT_PER_SPELL);
        assert!(ticks_for(4, 0) > ticks_for(1, 0));

        // ...and so is the log, by volume rather than by file.
        assert!(ticks_for(0, 1000) > bare);
        assert!(ticks_for(4, 3000) > ticks_for(4, 0));
    }

    /// **§8.1 rules out a flat audit by name**, so the test is that a grown
    /// tower costs materially more than a bare one rather than merely more.
    #[test]
    fn a_grown_tower_is_not_a_flat_thirty_seconds() {
        let bare = ticks_for(0, 0);
        let grown = ticks_for(4, 3000);
        assert!(
            grown > bare * 4,
            "a grown tower audits in {grown} against a bare {bare}, which is \
             close enough to flat that opening every siege with it would be \
             trivially correct",
        );
    }

    /// Bounded however far the tower sprawls — `LONGEST_BIDE`'s argument.
    #[test]
    fn an_audit_is_bounded() {
        assert_eq!(ticks_for(usize::MAX, usize::MAX), AUDIT_LONGEST);
    }

    /// **The tick it expires on is a tick you may check**, which is the `>=`
    /// boundary every other duration in the game uses.
    #[test]
    fn a_cooldown_ends_on_the_tick_it_names() {
        let mut cooling = Cooling::default();
        assert_eq!(cooling.left(Surface::Log, Tick::new(0)), None);

        cooling.spend(Surface::Log, Tick::new(100));
        assert_eq!(cooling.left(Surface::Log, Tick::new(100)), Some(COOLING));
        assert_eq!(cooling.left(Surface::Log, Tick::new(119)), Some(1));
        assert_eq!(cooling.left(Surface::Log, Tick::new(120)), None);
        assert_eq!(cooling.left(Surface::Log, Tick::new(9999)), None);
    }

    /// **One surface cooling leaves the other ready**, which is the whole of
    /// what makes *"which surface do I inspect first"* a decision rather than a
    /// wait.
    #[test]
    fn surfaces_cool_apart() {
        let mut cooling = Cooling::default();
        cooling.spend(Surface::Log, Tick::new(10));
        assert!(cooling.left(Surface::Log, Tick::new(10)).is_some());
        assert_eq!(cooling.left(Surface::World, Tick::new(10)), None);
    }

    #[test]
    fn cooling_survives_a_save() {
        let mut cooling = Cooling::default();
        cooling.spend(Surface::World, Tick::new(42));
        assert_eq!(Cooling::from_save(&cooling.to_save(Tick::new(42))), cooling,);

        // A document written before this existed has no row, and nothing is
        // cooling — which is the honest reading of an absence.
        assert_eq!(Cooling::from_save(&[]), Cooling::default());
    }

    /// **An expired cooldown is not written at all.**
    ///
    /// `spend` stores a tick and nothing clears it, so `to_save` filtered on
    /// *ever checked* — and a player who had run `verify` once on each surface
    /// carried four rows thousands of ticks stale in every autosave from then
    /// on. §15 wants a save a person can read, and four dead rows are four
    /// sentences that are not true.
    #[test]
    fn a_cooldown_that_has_run_out_is_not_saved() {
        let mut cooling = Cooling::default();
        cooling.spend(Surface::Log, Tick::new(10));
        assert_eq!(
            cooling.to_save(Tick::new(10)).len(),
            1,
            "a live cooldown was dropped",
        );
        let long_after = Tick::new(10 + COOLING + 1);
        assert!(
            cooling.to_save(long_after).is_empty(),
            "an expired cooldown was written: {:?}",
            cooling.to_save(long_after),
        );
        // ...and it reads back as nothing cooling, which it already was.
        assert_eq!(cooling.left(Surface::Log, long_after), None);
    }
}
