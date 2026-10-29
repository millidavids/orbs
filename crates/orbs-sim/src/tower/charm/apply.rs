//! A charm on a node, and the one question every read site asks.
//!
//! An interval, never a countdown (§19): [`Charm`] stores when it began and how
//! long it lasts, so a collapsed `meditate` behaves like ticks watched. With no
//! expiry system an expired charm is still present, so `With<Charmed>` is wrong
//! everywhere and [`charmed`] asks the clock instead.
//!
//! One component holding a `Vec`, not five component types, in registration
//! order and never sorted, or a save would reorder it silently.

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
    /// Saturating both ways: a charm laid in the future (only a hand-edited
    /// save does that) reads as fully remaining, one long expired as nought.
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
    /// Refreshes rather than stacks, so a maintenance loop does not grow the
    /// `Vec` every lap (§19), and keeps its position, so a save's order does not
    /// depend on renewal.
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
/// Looks at the node and the domain it stands in: a `quickening-scroll` hangs
/// its state on the room while `imbue` names one tool, and both must reach the
/// same answer at `work::begin` (§19).
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

/// The most ticks any tool in the tower has left, of every kind at once.
///
/// The longest, so `ebbing` fires when the last runs out rather than relaying a
/// charm with life left. One pass for all five, since this runs on a tick. What
/// the forge publishes and the rail's forge box reads (§19, *the tower speaks
/// up*), so the two cannot disagree.
#[must_use]
pub fn longest_of_each(world: &World) -> [u64; Kind::ALL.len()] {
    let now = *world.resource::<Tick>();
    let mut out = [0u64; Kind::ALL.len()];
    let Some(mut held) = world.try_query::<&Charmed>() else {
        return out;
    };
    for charmed in held.iter(world) {
        for kind in Kind::ALL {
            let left = charmed.left(kind, now);
            let slot = &mut out[kind as usize];
            if left > *slot {
                *slot = left;
            }
        }
    }
    out
}

/// The charm nearest lapsing, of the kinds the tower holds, by
/// [`longest_of_each`]: so its word is the one the forge publishes for it.
#[must_use]
pub fn soonest(world: &World) -> Option<(Kind, u64)> {
    let longest = longest_of_each(world);
    Kind::ALL
        .into_iter()
        .map(|kind| (kind, longest[kind as usize]))
        .filter(|(_, left)| *left > 0)
        .min_by_key(|(_, left)| *left)
}

/// Ticks left of `kind` on `place` or its domain — the larger of the two.
///
/// What the panel's decay row and the `graced` reading are drawn from. The
/// larger because both are in force; the shorter would hit nought while the
/// effect was still working.
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
