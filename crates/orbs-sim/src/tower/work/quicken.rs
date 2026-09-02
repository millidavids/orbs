//! A domain working at double speed for a while.
//!
//! What a `quickening-scroll` sets. It began as a one-shot on the run in hand —
//! halve what is left, refuse if nothing is running — and that was the wrong
//! shape twice over: the scroll was unusable at exactly the moment a player would
//! reach for one (before starting a brew, to make the brew quick), and it made
//! four walks of the stacks buy a single stage rather than a stretch of work.
//!
//! # An interval, like the fire
//!
//! `Burning` is *"a pure function of the tick, so fuel survives `meditate`"*, and
//! this is the same thing said about speed: [`Quickened`] stores when it began
//! and how long it lasts, and [`quickened`] compares that against the clock. No
//! system ticks it down, nothing has to notice it expiring, and hundreds of ticks
//! collapsed inside one `step` behave exactly like hundreds watched.
//!
//! # Checked when a run starts, like heat
//!
//! §10.1 checks the athanor when a run **begins** and lets the run finish even if
//! the fire dies under it, because *"pausing would be the countdown §19 refused,
//! spoiling would cost progress against §11.5's never ruinous, only slower"*.
//! Speed follows heat: a run that starts inside the window is short, and stays
//! short even if the window closes while it works.
//!
//! That also keeps `Working` an interval set once, which is the property
//! `meditate` idempotence rests on. A speed-up applied per tick would be a
//! countdown wearing a multiplier.

use bevy_ecs::prelude::*;

use crate::tick::Tick;

/// How long one scroll quickens a domain, in ticks.
///
/// **Five minutes**, since §5.0 makes a tick one real second while the window is
/// open. That is squarely in §11.5's Production band (3–10 minutes), which is
/// what a window covering a *stretch of work* rather than a single stage should
/// be measured against.
///
/// For scale: a clarity walked end to end is 94 ticks of work, so this is about
/// three brews' worth — generous, and deliberately so. What a scroll costs is
/// four walks of the stacks, which is thousands of ticks of walking, and a window
/// that covered one stage made that trade absurd. The lever if it runs hot is
/// this line.
///
/// **A placeholder, like every duration here**, and `orbs-balance` is what
/// sweeps it. That sentence used to end *"`orbs-balance` being a ten-line stub
/// is why that is a plan rather than a fact"*; the harness is real, so the
/// sweep is available and this number has simply not been through one.
pub const QUICKENED_TICKS: u64 = 300;

/// How much faster a quickened tool works.
///
/// Two, and a `u64` divisor rather than a float: durations are whole ticks and a
/// fractional rate would need a rounding rule nobody could predict from the
/// number.
///
/// **Two call sites floor it in opposite directions, deliberately.**
/// `hastened` takes `max(1)`, so a one-tick run does not become a no-tick one;
/// [`hurried_from`] refuses to move a run with nothing left, because there the
/// same flooring would land it *later*. One rate, two guards — and both of them
/// in this module, which is the whole reason the second was worth moving here.
pub const QUICKENED_BY: u64 = 2;

/// A place whose tools are working at double speed.
///
/// On the **domain**, not on an instrument: what the scroll buys is a stretch of
/// the laboratory being quick, and hanging it on one tool would mean choosing
/// which — a decision the player did not make and could not see.
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
/// Takes an **instrument** and looks up, because that is what the callers have:
/// `begin` knows the tool a run is starting on, and the state belongs to the room
/// the tool is in.
/// **Two sources now, and that is the whole of Phase 9's architecture.** A
/// `quickening-scroll` sets [`Quickened`] on the room; the forge lays
/// `Charm(Hurried)` on one tool. `tower::dice` predicted exactly this — *"a
/// second source would need the call site to know about it"* — so the call site
/// is not told: it asks here, and this composes. Adding a third source is a line
/// in this function rather than an edit at `begin`.
///
/// **Either, not both-stacked.** Two halvings would be a quartering, and §11.5's
/// durations are not built for it — what a second source buys is *another way to
/// get* the window, not a deeper one. The charm's own value is its duration.
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
/// The one place the rate is applied to a run **about to start**, so the panel,
/// the completion and every duration a player sees come from one arithmetic.
/// A run already in flight is [`hurried_from`], which is a different guard.
#[must_use]
pub fn hastened(world: &World, place: Entity, ticks: u64) -> u64 {
    if quickened(world, place) {
        (ticks / QUICKENED_BY).max(1)
    } else {
        ticks
    }
}

/// Where a run **already in flight** should end, once its room is quickened.
///
/// **Not `hastened`, and the difference is a bug that shipped once.**
/// `hastened` halves a *fresh* duration and floors at one, so a one-tick run
/// does not become a no-tick one. An interval already running needs the opposite
/// guard: `commands` runs before `tower::finish` in `Sim::advance`, so a scroll
/// spent on the exact tick a run would land sees nothing left — and
/// `(0 / QUICKENED_BY).max(1)` is 1, which pushes `ends` a tick *past* where it
/// already was. **A scroll that makes the thing it hurries land later is the one
/// outcome it must never have.**
///
/// Halved from **now** rather than from the start: halving the whole interval
/// would refund time already spent and, past the half-way point, land the end in
/// the past.
///
/// `None` when the run is too close to landing to be worth moving — which is
/// what keeps this *only ever earlier*.
///
/// It lived in `execute::scroll` and the rate was written out there a second
/// time. One rate, two guards, and both of them here: a reader comparing the
/// pair can see that the difference is deliberate, which is the whole reason the
/// duplicate was worth moving rather than deleting.
#[must_use]
pub const fn hurried_from(now: Tick, ends: Tick) -> Option<Tick> {
    let left = ends.get().saturating_sub(now.get());
    if left > 1 {
        // **No floor, and there must not be one.** `left > 1` means `left >= 2`,
        // so the quotient is at least one and a `max(1)` here would be dead —
        // worse, it would read as a live guard of exactly the shape this
        // function was extracted to *refuse*, so a later reader widening the
        // test to `left > 0` would reintroduce the landing-a-tick-later bug
        // under the cover of a floor that looked deliberate.
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
    /// **A scroll may never make the thing it hurries land later.** The bug it
    /// replaces was reachable on exactly one tick — the one a run was about to
    /// land on — which is why a sampled test would have missed it and a sweep
    /// does not.
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

    /// Halved from **now**, never from the start — the other half of the rule.
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
