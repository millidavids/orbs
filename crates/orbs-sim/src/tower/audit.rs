//! The expensive audit, and what limits the cheap one (DESIGN.md §8.1).
//!
//! §8.1 prices `verify`'s two forms against each other: `verify --all` scales
//! with the tower — bound scripts plus log volume — because a flat audit would
//! be trivially correct to open every siege with, and `verify <target>` carries
//! a short per-surface cooldown, because four free instant checks *are*
//! `verify --all` by another name.
//!
//! Neither shipped until now, so four instant checks audited the whole tower
//! for nothing. The ambient surfaces make it matter before the siege does:
//! `drift` poisons a log every ~300 ticks and `substitution` swaps a reagent
//! every ~3600, in the calm layer — so *is it the log lying, or the shelf?* was
//! already a choice that cost nothing.

use bevy_ecs::prelude::*;

use crate::tick::Tick;

/// One of §8.1's tamperable surfaces.
///
/// The two ambient ones shipped first; script text and trigger clocks waited
/// for the siege that produces them, because a variant nothing can ever be is a
/// cooldown nobody can ever be on. A poisoned log and a swapped pile are told
/// apart by *which* surface you look at, and the cooldown is what makes looking
/// at the wrong one cost something.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Surface {
    /// A `.log` file, whose lines can be forged (`sabotage::Log`).
    Log,
    /// A pile of stock, which can be swapped for something wearing its name.
    World,
    /// A `.spell` file, whose lines an enemy can rewrite (`tower::Held`).
    ///
    /// Siege-only by §5.1 — *"the enemy never touches scripts, schedules, or
    /// logs in the calm layer"*, which pillar 4 requires — so it has no ambient
    /// producer at all and only ever lies mid-siege.
    Script,
    /// A bound spell's schedule, which an enemy can retime (`spell::Bound`).
    ///
    /// Siege-only, for [`Script`](Self::Script)'s reason. It is the subtlest of
    /// the four: nothing about the spell's *text* is wrong, so `peruse` shows a
    /// correct spell and only the clock is lying.
    Clock,
}

impl Surface {
    /// Every surface that ships — all four, which closes §8.1's model.
    ///
    /// `Cooling` is keyed by *word*, so adding the siege's two renumbered
    /// nothing in a save.
    pub const ALL: [Self; 4] = [Self::Log, Self::World, Self::Script, Self::Clock];

    /// How many there are.
    pub const COUNT: usize = Self::ALL.len();

    /// Where this one sits in [`Cooling`]'s array.
    ///
    /// Private to that storage, unlike `RngStream::index`, which is a
    /// save-format commitment. Renumbering here is free because the save is
    /// keyed by [`word`](Self::word) instead.
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
    /// A table, not prose — `Verb::canonical`'s exemption. A field value `sift`
    /// matches and §14 speaks; the sentence around it is composed in
    /// `content/prose.toml` like every other.
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
    /// `None` for a place or an instrument: still `verify`-able and still
    /// answering `sound`, just not *rationed*, because a check that can never
    /// find anything is a cooldown with no information behind it.
    #[must_use]
    pub fn of(world: &World, node: Entity) -> Option<Self> {
        if world.get::<super::sabotage::Log>(node).is_some() {
            return Some(Self::Log);
        }
        // A spell is two surfaces, and a retimed one's text is perfect — so
        // categorising every spell as `Script` left `Clock` unreachable and
        // `to_save` could never emit a `["clock", n]` row. Retimed goes first.
        if world.get::<super::Retimed>(node).is_some() {
            return Some(Self::Clock);
        }
        if world.get::<super::Held>(node).is_some() {
            return Some(Self::Script);
        }
        // The place, not the pile: `verify dispensary` reports the shelf by
        // looking one level down, so asking the shelf itself for `Stock` would
        // answer `None` and leave the commonest world check unrationed.
        let holds_stock = super::children_of(world, node)
            .into_iter()
            .any(|held| world.get::<super::Stock>(held).is_some());
        holds_stock.then_some(Self::World)
    }
}

/// How long one surface stays unavailable after a targeted check.
///
/// §8.1 says *short*. Against a 56-tick clarity brew and a 4-tick purge, twenty
/// is a real loss without ever becoming the reason a player stops looking.
///
/// It rations the *surface*, never the target: §8.1's decision is *which
/// surface do I inspect first*, and a per-target cooldown would leave a player
/// free to walk the six domain logs in six ticks.
pub const COOLING: u64 = 20;

/// The floor on a full audit, before the tower is weighed.
///
/// §8.1 rules out *"a flat 30-second audit"*, so this is the base of a sum
/// rather than the answer: a bare tower audits in about this, and every spell
/// bound and every line logged pushes it up from here.
pub const AUDIT_BASE: u64 = 20;

/// What each held spell adds to a full audit.
///
/// §8.1's *"proportional to bound scripts"*. A bound spell is a surface the
/// enemy can rewrite, so auditing one is real work — and the player who
/// automates hardest pays most to check their automation, which is pillar 3
/// stated as a duration.
pub const AUDIT_PER_SPELL: u64 = 8;

/// How many logged lines add one tick to a full audit.
///
/// The *"plus log volume"* half. Divided rather than multiplied: the stream is
/// thousands of records deep by mid-game and a tick each would make the audit
/// longer than the session.
pub const AUDIT_PER_LINES: u64 = 20;

/// The longest a full audit will ever take, however large the tower grows.
///
/// A clamp rather than a refusal, on `LONGEST_BIDE`'s argument: the number is
/// the player's own sprawl and there is no honest place to refuse it, but an
/// audit longer than `MAX_MEDITATE` is indistinguishable from one that never
/// lands. Ten minutes is past any tower the harness has swept, and bounded.
pub const AUDIT_LONGEST: u64 = 600;

/// When each surface may be checked again.
///
/// A resource keyed by surface, not a component on the target: [`COOLING`]
/// rations the *surface*, so hanging it on a node means writing it to every log
/// in the tower and keeping six copies in step.
///
/// It travels in the save — a cooldown that reset on load is one a player
/// clears by quitting.
#[derive(Resource, Debug, Clone, Default, PartialEq, Eq)]
pub struct Cooling {
    /// When each surface is next available, indexed by [`Surface::index`].
    ///
    /// An array rather than a field per surface: at four, the fields were the
    /// same `match` written out in `spend`, `from_save` and `at` — three places
    /// to forget a fifth surface in. The save stays keyed by name, because a
    /// renumbered save would silently remap a player's cooldowns.
    until: [Option<Tick>; Surface::COUNT],
}

impl Cooling {
    /// How long `surface` has left, or `None` when it is ready.
    ///
    /// Zero reads as ready, so the tick a cooldown expires on is a tick the
    /// player may check — `bide`'s and `Working`'s `>=` boundary, which stops a
    /// twenty-tick wait being a twenty-one-tick one.
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
    /// Pairs rather than a `Vec<Option<u64>>` positional on [`Surface::ALL`]:
    /// `toml` refuses that outright — *"unsupported None value"* — and §15
    /// wants a save a person can read, where `[["log", 120]]` says what a bare
    /// `[null, 120]` cannot. Order carries no meaning, so the siege's two
    /// surfaces renumbered nothing and an older save still reads right.
    ///
    /// Only what is *still* cooling: `spend` stores a tick and nothing clears
    /// it, so filtering on *ever checked* put four rows thousands of ticks
    /// stale in every autosave once a player had `verify`'d each surface.
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
    /// A missing row is not an error, the tolerance every `from_save` here has:
    /// a document written before this existed has no table, and the honest
    /// reading of that is *nothing is cooling*. A surface the build no longer
    /// knows is dropped for the same reason.
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
/// §8.1's *"proportional to bound scripts plus log volume"*, floored and
/// capped. A pure function of two counts, so the number quoted before the audit
/// starts is the number waited, and a test can assert the curve without running
/// one.
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

    /// §8.1's curve: a bare tower is quick, and grows with the two things named.
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

    /// §8.1 rules out a flat audit by name, so a grown tower must cost
    /// materially more than a bare one rather than merely more.
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

    /// The tick it expires on is a tick you may check — the `>=` boundary every
    /// other duration in the game uses.
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

    /// One surface cooling leaves the other ready, which is what makes *"which
    /// surface do I inspect first"* a decision rather than a wait.
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

    /// An expired cooldown is not written at all.
    ///
    /// `spend` never clears its tick, so `to_save` filtered on *ever checked*
    /// and a player who had `verify`'d each surface carried four stale rows in
    /// every autosave. §15 wants a save a person can read, and four dead rows
    /// are four sentences that are not true.
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
