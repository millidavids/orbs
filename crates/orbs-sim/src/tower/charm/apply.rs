//! A charm on a node, and the one question every read site asks.
//!
//! # An interval, never a countdown
//!
//! `Burning` is *"a pure function of the tick, so fuel survives `meditate`"*, and
//! a charm is the same claim: [`Charm`] stores when it began and how long it
//! lasts, and [`charmed`] compares that against the clock. **No system ticks it
//! down and nothing has to notice it expiring**, so hundreds of ticks collapsed
//! inside one `step` behave exactly like hundreds watched. Four modules argue
//! for this shape independently — `heat`, `quicken`, `work::slot` and
//! `save::node` — and a charm that decayed per tick would be the countdown §19
//! refused, wearing a multiplier.
//!
//! # So `Charmed` can never be a query filter
//!
//! The consequence of having no expiry system: **an expired charm is still a
//! present component.** `With<Charmed>` is therefore wrong at every read site,
//! and each one needs the clock. [`charmed`] is that question asked once, so no
//! call site has to remember.
//!
//! # Why one component holding many, rather than one per charm
//!
//! `bevy_ecs` allows one component of a type per entity, so five charms as five
//! types would be five queries, five save fields and five liveness tests. They
//! are one `Vec` in registration order — **never sorted**, because insertion
//! order is the parse everywhere else in this tower and a charm list that
//! reordered itself across a save would be a replay divergence with no symptom.

use bevy_ecs::prelude::*;

use crate::tick::Tick;

use super::Kind;

/// One enchantment, running.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Charm {
    /// What it does.
    pub kind: Kind,
    /// When it was laid.
    pub from: Tick,
    /// How long it lasts, in ticks.
    pub ticks: u64,
}

impl Charm {
    /// Ticks of this charm left at `now`.
    ///
    /// Saturating in both directions: a charm laid in the future (which only a
    /// hand-edited save can produce) reads as fully remaining rather than
    /// panicking, and one long expired reads as nought rather than wrapping.
    #[must_use]
    pub const fn remaining(&self, now: Tick) -> u64 {
        let spent = now.get().saturating_sub(self.from.get());
        self.ticks.saturating_sub(spent)
    }

    /// Whether this charm is still doing anything at `now`.
    #[must_use]
    pub const fn live(&self, now: Tick) -> bool {
        self.remaining(now) > 0
    }
}

/// Every charm laid on one node, live or long expired.
#[derive(Component, Debug, Clone, Default)]
pub struct Charmed(pub Vec<Charm>);

impl Charmed {
    /// Lay one, replacing any charm of the same kind.
    ///
    /// **Re-laying refreshes rather than stacking**, which is what makes the
    /// maintenance spell of Phase 9's fourth box writable: `repeat until` keeps
    /// a charm alive by laying it again, and a stacking rule would make that loop
    /// grow the `Vec` once a lap for ever — a save that swells by a row a second,
    /// which is the shape §19 records for runaway `part` recursion.
    ///
    /// Order is otherwise untouched: a refreshed charm keeps its position rather
    /// than moving to the end, so the list a save writes does not depend on how
    /// recently each was renewed.
    pub fn lay(&mut self, charm: Charm) {
        match self.0.iter_mut().find(|held| held.kind == charm.kind) {
            Some(held) => *held = charm,
            None => self.0.push(charm),
        }
    }

    /// Ticks left of one kind at `now`, or nought if it is not held.
    #[must_use]
    pub fn left(&self, kind: Kind, now: Tick) -> u64 {
        self.0
            .iter()
            .find(|charm| charm.kind == kind)
            .map_or(0, |charm| charm.remaining(now))
    }

    /// The charms still doing something at `now`, in the order they were laid.
    pub fn live(&self, now: Tick) -> impl Iterator<Item = &Charm> {
        self.0.iter().filter(move |charm| charm.live(now))
    }
}

/// Whether `place` is under a live charm of `kind` right now.
///
/// **Looks at the node and the domain it stands in**, which is `quicken`'s rule
/// and the reason a `quickening-scroll` can grant one: the scroll buys *a stretch
/// of the laboratory being quick* and hangs its state on the room, while `imbue`
/// names one tool. Both have to reach the same answer at `work::begin`, or the
/// two sources §19 warns about would disagree at the call site — which is the
/// exact failure this composition exists to prevent.
#[must_use]
pub fn charmed(world: &World, place: Entity, kind: Kind) -> bool {
    let now = *world.resource::<Tick>();
    let domain = crate::tower::domain_of(world, place);
    [domain, Some(place)].into_iter().flatten().any(|node| {
        world
            .get::<Charmed>(node)
            .is_some_and(|held| held.left(kind, now) > 0)
    })
}

/// Ticks left of `kind` on `place` or its domain — the larger of the two.
///
/// What the panel's decay row and the `graced` reading are drawn from. The
/// larger rather than the nearer, because both are really in force and reporting
/// the shorter would have the row hit nought while the effect was still working.
#[must_use]
pub fn charm_left(world: &World, place: Entity, kind: Kind) -> u64 {
    let now = *world.resource::<Tick>();
    let domain = crate::tower::domain_of(world, place);
    [domain, Some(place)]
        .into_iter()
        .flatten()
        .filter_map(|node| world.get::<Charmed>(node))
        .map(|held| held.left(kind, now))
        .max()
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn charm(kind: Kind, from: u64, ticks: u64) -> Charm {
        Charm {
            kind,
            from: Tick::new(from),
            ticks,
        }
    }

    #[test]
    fn a_charm_is_a_pure_function_of_the_clock() {
        let held = charm(Kind::Hurried, 100, 50);
        assert_eq!(held.remaining(Tick::new(100)), 50);
        assert_eq!(held.remaining(Tick::new(125)), 25);
        assert_eq!(held.remaining(Tick::new(150)), 0);
        // ...and stays nought rather than wrapping.
        assert_eq!(held.remaining(Tick::new(9_000)), 0);
        assert!(!held.live(Tick::new(150)));
    }

    /// The maintenance spell's whole shape: laying again must not stack.
    #[test]
    fn re_laying_a_charm_refreshes_it_rather_than_stacking() {
        let mut held = Charmed::default();
        for lap in 0..100 {
            held.lay(charm(Kind::Hurried, lap, 10));
        }
        assert_eq!(held.0.len(), 1, "a maintenance loop grew the list");
        assert_eq!(held.left(Kind::Hurried, Tick::new(99)), 10);
    }

    /// ...and a refresh does not move it, so a save's row order is stable.
    #[test]
    fn refreshing_one_charm_leaves_the_order_alone() {
        let mut held = Charmed::default();
        held.lay(charm(Kind::Hurried, 0, 10));
        held.lay(charm(Kind::Whetted, 0, 10));
        held.lay(charm(Kind::Hurried, 5, 10));
        let order: Vec<Kind> = held.0.iter().map(|charm| charm.kind).collect();
        assert_eq!(order, vec![Kind::Hurried, Kind::Whetted]);
    }

    /// Absent is nought here too, which is the tower's rule everywhere else.
    #[test]
    fn a_charm_never_laid_has_nothing_left() {
        let held = Charmed::default();
        assert_eq!(held.left(Kind::Shielded, Tick::new(0)), 0);
    }

    #[test]
    fn only_live_charms_are_walked() {
        let mut held = Charmed::default();
        held.lay(charm(Kind::Hurried, 0, 10));
        held.lay(charm(Kind::Whetted, 0, 100));
        let live: Vec<Kind> = held.live(Tick::new(50)).map(|charm| charm.kind).collect();
        assert_eq!(live, vec![Kind::Whetted]);
    }
}
