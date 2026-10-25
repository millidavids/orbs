//! The expensive audit, and what limits the cheap one (DESIGN.md §8.1).
//!
//! `verify --all` scales with the tower, or it would be trivially correct to
//! open every siege with; `verify <target>` cools per surface, or four instant
//! checks are `--all` by another name.

use bevy_ecs::prelude::*;

use crate::tick::Tick;

/// One of §8.1's tamperable surfaces.
///
/// The cooldown is what makes looking at the wrong one cost something.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Surface {
    /// A `.log` file, whose lines can be forged (`sabotage::Log`).
    Log,
    /// A pile of stock, which can be swapped for something wearing its name.
    World,
    /// A `.spell` file, whose lines an enemy can rewrite (`tower::Held`).
    /// Siege-only (§5.1, pillar 4).
    Script,
    /// A bound spell's schedule, which an enemy can retime (`spell::Bound`).
    /// Siege-only; the text is perfect and only the clock lies.
    Clock,
}

impl Surface {
    /// Every surface that ships — all four, which closes §8.1's model.
    pub const ALL: [Self; 4] = [Self::Log, Self::World, Self::Script, Self::Clock];

    /// How many there are.
    pub const COUNT: usize = Self::ALL.len();

    /// Where this one sits in [`Cooling`]'s array.
    ///
    /// Free to renumber, unlike `RngStream::index`: the save is keyed by
    /// [`word`](Self::word).
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Log => 0,
            Self::World => 1,
            Self::Script => 2,
            Self::Clock => 3,
        }
    }

    /// The word a record carries for it — a table, not prose; the sentence is
    /// in `content/prose.toml`.
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
    /// `None` for a place or an instrument: still `verify`-able, just not
    /// rationed, since such a check can never find anything.
    #[must_use]
    pub fn of(world: &World, node: Entity) -> Option<Self> {
        if world.get::<super::sabotage::Log>(node).is_some() {
            return Some(Self::Log);
        }
        // A retimed spell first: its text is perfect, so filing it as `Script`
        // would leave `Clock` unreachable.
        if world.get::<super::Retimed>(node).is_some() {
            return Some(Self::Clock);
        }
        if world.get::<super::Held>(node).is_some() {
            return Some(Self::Script);
        }
        // The place, not the pile: `verify dispensary` looks one level down.
        let holds_stock = super::children_of(world, node)
            .into_iter()
            .any(|held| world.get::<super::Stock>(held).is_some());
        holds_stock.then_some(Self::World)
    }
}

/// How long one surface stays unavailable after a targeted check.
///
/// Short (§8.1): a real loss against a 56-tick brew, never the reason a player
/// stops looking. Per surface, not target, or the six logs walk in six ticks.
pub const COOLING: u64 = 20;

/// The floor on a full audit, before the tower is weighed — the base of a sum,
/// since §8.1 rules out a flat audit.
pub const AUDIT_BASE: u64 = 20;

/// What each held spell adds to a full audit: the player who automates hardest
/// pays most to check it, pillar 3 as a duration.
pub const AUDIT_PER_SPELL: u64 = 8;

/// How many logged lines add one tick to a full audit — divided, because the
/// stream is thousands deep by mid-game.
pub const AUDIT_PER_LINES: u64 = 20;

/// The longest a full audit will ever take, however large the tower grows.
///
/// A clamp, `LONGEST_BIDE`'s argument: past `MAX_MEDITATE` an audit is
/// indistinguishable from one that never lands.
pub const AUDIT_LONGEST: u64 = 600;

/// When each surface may be checked again.
///
/// A resource keyed by surface, not a component on every target. Saved, or
/// quitting would clear it.
#[derive(Resource, Debug, Clone, Default, PartialEq, Eq)]
pub struct Cooling {
    /// When each surface is next available, indexed by [`Surface::index`]. The
    /// save is keyed by name, so renumbering here remaps nothing.
    until: [Option<Tick>; Surface::COUNT],
}

impl Cooling {
    /// How long `surface` has left, or `None` when it is ready — ready on the
    /// tick it expires, the `>=` boundary every duration uses.
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

    /// What a save writes: the surfaces still cooling, and until when.
    ///
    /// Named pairs, because `toml` refuses a positional `None` and §15 wants a
    /// save a person can read. Only what is still cooling, since `spend` never
    /// clears a tick and stale rows are sentences that are not true.
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

    /// Read one back. A missing row is *nothing cooling*, and a surface this
    /// build does not know is dropped.
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

/// How long a full audit of this tower takes: §8.1's *bound scripts plus log
/// volume*, floored and capped.
///
/// Pure, so the number quoted before the audit is the number waited.
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

        assert_eq!(ticks_for(1, 0), AUDIT_BASE + AUDIT_PER_SPELL);
        assert!(ticks_for(4, 0) > ticks_for(1, 0));

        // ...and so is the log, by volume rather than by file.
        assert!(ticks_for(0, 1000) > bare);
        assert!(ticks_for(4, 3000) > ticks_for(4, 0));
    }

    /// A grown tower must cost materially more than a bare one, not merely more.
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

    #[test]
    fn an_audit_is_bounded() {
        assert_eq!(ticks_for(usize::MAX, usize::MAX), AUDIT_LONGEST);
    }

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

    /// One surface cooling leaves the other ready, which makes *which first* a
    /// decision rather than a wait.
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

        assert_eq!(Cooling::from_save(&[]), Cooling::default());
    }

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
        assert_eq!(cooling.left(Surface::Log, long_after), None);
    }
}
