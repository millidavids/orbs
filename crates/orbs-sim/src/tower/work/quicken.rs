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
/// **A placeholder, like every duration here**, and the balance CLI is what
/// sweeps it — `orbs-balance` being a ten-line stub is why that is a plan rather
/// than a fact.
pub const QUICKENED_TICKS: u64 = 300;

/// How much faster a quickened tool works.
///
/// Two, and a `u64` divisor rather than a float: durations are whole ticks and a
/// fractional rate would need a rounding rule nobody could predict from the
/// number. `max(1)` at the call site keeps a one-tick run from becoming a
/// no-tick one.
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
#[must_use]
pub fn quickened(world: &World, place: Entity) -> bool {
    let now = *world.resource::<Tick>();
    let domain = super::super::domain_of(world, place);
    [domain, Some(place)].into_iter().flatten().any(|node| {
        world
            .get::<Quickened>(node)
            .is_some_and(|haste| haste.remaining(now) > 0)
    })
}

/// Shorten `ticks` if `place` is quickened.
///
/// The one place the rate is applied, so the panel, the completion and every
/// duration a player sees come from one arithmetic.
#[must_use]
pub fn hastened(world: &World, place: Entity, ticks: u64) -> u64 {
    if quickened(world, place) {
        (ticks / QUICKENED_BY).max(1)
    } else {
        ticks
    }
}
