//! A domain working at double speed for a while.
//!
//! What a `quickening-scroll` sets. A one-shot on the run in hand was the wrong
//! shape twice over: unusable at the moment a player reaches for one (before
//! starting a brew), and four walks of the stacks bought a single stage.
//!
//! An interval, like the fire. [`Quickened`] stores when it began and how long
//! it lasts and [`quickened`] compares that against the clock, so nothing ticks
//! it down and ticks collapsed inside one `step` behave like ticks watched.
//!
//! Checked when a run starts, like heat (§10.1): a run that starts inside the
//! window is short and stays short even if the window closes under it. That
//! also keeps `Working` an interval set once, which is what `meditate`
//! idempotence rests on.

use bevy_ecs::prelude::*;

use crate::tick::Tick;

/// How long one scroll quickens a domain, in ticks.
///
/// Five minutes at §5.0's one tick a second, inside §11.5's Production band
/// (3–10 minutes). A clarity end to end is 94 ticks, so this is three brews'
/// worth — generous, because a scroll costs four walks of the stacks.
///
/// A placeholder, like every duration here. `orbs-balance` can sweep it; this
/// number has not been through one.
pub const QUICKENED_TICKS: u64 = 300;

/// How much faster a quickened tool works.
///
/// Two, and a `u64` divisor rather than a float: durations are whole ticks and a
/// fractional rate would need a rounding rule nobody could predict from the
/// number.
///
/// Two call sites floor it in opposite directions, deliberately: `hastened`
/// takes `max(1)` so a one-tick run does not become a no-tick one, and
/// [`hurried_from`] refuses to move a run with nothing left, where the same
/// flooring would land it *later*. Both guards live here so the pair is
/// readable side by side.
pub const QUICKENED_BY: u64 = 2;

/// A place whose tools are working at double speed.
///
/// On the domain, not an instrument: hanging it on one tool would mean choosing
/// which, a decision the player did not make and could not see.
#[derive(Component, Debug, Clone, Copy)]
pub struct Quickened {
    /// When the scroll was unrolled.
    pub from: Tick,
    /// How long it lasts, in ticks.
    pub ticks: u64,
}

impl Quickened {
    /// Ticks of haste left at `now`.
    #[must_use]
    pub const fn remaining(&self, now: Tick) -> u64 {
        let spent = now.get().saturating_sub(self.from.get());
        self.ticks.saturating_sub(spent)
    }
}

/// Whether the domain `place` stands in is quickened right now.
///
/// Takes an instrument and looks up, because that is what callers have: `begin`
/// knows the tool, and the state belongs to the room it is in.
///
/// Two sources — a `quickening-scroll` sets [`Quickened`] on the room, the
/// forge lays `Charm(Hurried)` on one tool — and the call site is told about
/// neither. A third is a line here rather than an edit at `begin`.
///
/// Either, not both-stacked: two halvings would be a quartering, and §11.5's
/// durations are not built for it. A second source is another way to get the
/// window, not a deeper one.
#[must_use]
pub fn quickened(world: &World, place: Entity) -> bool {
    let now = *world.resource::<Tick>();
    let domain = super::super::domain_of(world, place);
    let scroll = [domain, Some(place)].into_iter().flatten().any(|node| {
        world
            .get::<Quickened>(node)
            .is_some_and(|haste| haste.remaining(now) > 0)
    });
    scroll || super::super::charmed(world, place, super::super::charm::Kind::Hurried)
}

/// Shorten `ticks` if `place` is quickened.
///
/// The one place the rate is applied to a run about to start, so the panel, the
/// completion and every duration a player sees share one arithmetic. A run
/// already in flight is [`hurried_from`], a different guard.
#[must_use]
pub fn hastened(world: &World, place: Entity, ticks: u64) -> u64 {
    if quickened(world, place) {
        (ticks / QUICKENED_BY).max(1)
    } else {
        ticks
    }
}

/// Where a run already in flight should end, once its room is quickened.
///
/// Not `hastened`, and the difference shipped as a bug once: `commands` runs
/// before `tower::finish`, so a scroll spent on the tick a run would land sees
/// nothing left, and `(0 / QUICKENED_BY).max(1)` is 1 — a tick *past* where
/// `ends` already was. A scroll must never make what it hurries land later.
///
/// Halved from now rather than from the start, which would refund time already
/// spent and, past the half-way point, land the end in the past. `None` when
/// the run is too close to landing to move, which is what keeps this only ever
/// earlier.
#[must_use]
pub const fn hurried_from(now: Tick, ends: Tick) -> Option<Tick> {
    let left = ends.get().saturating_sub(now.get());
    if left > 1 {
        // No floor, and there must not be one: `left > 1` makes the quotient at
        // least one, so a `max(1)` would be dead — and would look like a live
        // guard, letting a later reader widen the test to `left > 0` and bring
        // the landing-a-tick-later bug back under cover.
        Some(Tick::new(now.get() + left / QUICKENED_BY))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The property the whole guard exists for, swept rather than sampled.
    ///
    /// A scroll may never make what it hurries land later. The bug it replaces
    /// was reachable on exactly one tick, so a sampled test would miss it.
    #[test]
    fn hurrying_a_run_never_lands_it_later() {
        for ends in 0..200u64 {
            for now in 0..=ends {
                let Some(moved) = hurried_from(Tick::new(now), Tick::new(ends)) else {
                    continue;
                };
                assert!(
                    moved.get() <= ends,
                    "hurrying a run ending at {ends} from {now} moved it to {} — later",
                    moved.get()
                );
                assert!(
                    moved.get() >= now,
                    "hurrying a run ending at {ends} from {now} landed it in the past"
                );
            }
        }
    }

    /// The case the guard is *for*: nothing left to halve.
    ///
    /// `commands` runs before `tower::finish`, so a scroll spent on the landing
    /// tick sees this. Under `hastened`'s flooring it became one tick *later*.
    #[test]
    fn a_run_about_to_land_is_left_alone() {
        assert_eq!(hurried_from(Tick::new(10), Tick::new(10)), None);
        assert_eq!(hurried_from(Tick::new(10), Tick::new(11)), None);
        // ...and one tick further on there is something to take.
        assert_eq!(
            hurried_from(Tick::new(10), Tick::new(12)),
            Some(Tick::new(11))
        );
    }

    /// Halved from now, never from the start — the other half of the rule.
    ///
    /// A run that began at 0 and ends at 100, hurried at 80, has 20 left and
    /// must land at 90. Halving the *interval* would land it at 50, in the past.
    #[test]
    fn a_run_is_halved_from_now_and_not_from_its_start() {
        assert_eq!(
            hurried_from(Tick::new(80), Tick::new(100)),
            Some(Tick::new(90))
        );
    }
}
