//! A domain working faster for a while.
//!
//! What a quickening supply sets, one window a draw (§19, *Supply*). An
//! interval, like the fire: [`Quickened`] stores when it began, how long it
//! lasts and by how much, and [`quickened_by`] compares that against the clock,
//! so nothing ticks it down and ticks collapsed inside one `step` behave like
//! ticks watched.
//!
//! Checked when a run starts, like heat (§10.1): a run that starts inside the
//! window is short and stays short even if the window closes under it. That
//! also keeps `Working` an interval set once, which is what `meditate`
//! idempotence rests on.

use bevy_ecs::prelude::*;

use crate::tick::Tick;

/// How much faster a `hurried` charm works, and what an older save's window
/// was worth before a window said.
///
/// A `u64` divisor rather than a float: durations are whole ticks and a
/// fractional rate would need a rounding rule nobody could predict.
pub const QUICKENED_BY: u64 = 2;

/// A place whose tools are working faster.
///
/// On the domain, not an instrument: hanging it on one tool would mean choosing
/// which, a decision the player did not make and could not see.
#[derive(Component, Debug, Clone, Copy)]
pub struct Quickened {
    /// When the draw unrolled it.
    pub from: Tick,
    /// How long it lasts, in ticks.
    pub ticks: u64,
    /// What a run's time is divided by: 2 is half, 3 a third.
    pub by: u64,
}

impl Quickened {
    /// Ticks of haste left at `now`.
    #[must_use]
    pub const fn remaining(&self, now: Tick) -> u64 {
        let spent = now.get().saturating_sub(self.from.get());
        self.ticks.saturating_sub(spent)
    }
}

/// How much faster the domain `place` stands in works right now, if at all.
///
/// Takes an instrument and looks up, because that is what callers have: `begin`
/// knows the tool, and the state belongs to the room it is in.
///
/// Two sources — a quickening supply's window on the room, a `hurried` charm on
/// one tool — and the larger applies. Two halvings would be a quartering, and
/// §11.5's durations are not built for it.
#[must_use]
pub fn quickened_by(world: &World, place: Entity) -> Option<u64> {
    let now = *world.resource::<Tick>();
    let domain = super::super::domain_of(world, place);
    let scroll = [domain, Some(place)]
        .into_iter()
        .flatten()
        .filter_map(|node| world.get::<Quickened>(node))
        .filter(|haste| haste.remaining(now) > 0)
        .map(|haste| haste.by)
        .max();
    let charm = super::super::charmed(world, place, super::super::charm::Kind::Hurried)
        .then_some(QUICKENED_BY);
    scroll.max(charm).filter(|by| *by > 1)
}

/// Whether the domain `place` stands in is quickened right now.
#[must_use]
pub fn quickened(world: &World, place: Entity) -> bool {
    quickened_by(world, place).is_some()
}

/// Shorten `ticks` if `place` is quickened.
///
/// The one place the rate is applied to a run about to start, so the panel, the
/// completion and every duration a player sees share one arithmetic. A run
/// already in flight is [`hurried_from`], a different guard.
#[must_use]
pub fn hastened(world: &World, place: Entity, ticks: u64) -> u64 {
    quickened_by(world, place).map_or(ticks, |by| (ticks / by).max(1))
}

/// Where a run in flight `already` that quick should end, once its room is
/// quickened by `by`: from now, and only by what `by` adds, so the larger
/// applies and never both.
///
/// `None` when that adds nothing or the run is too close to landing to move.
/// Not `hastened`: a run due this tick has nothing left, and its floor of one
/// would land it a tick late.
#[must_use]
pub fn hurried_from(now: Tick, ends: Tick, by: u64, already: u64) -> Option<Tick> {
    let left = ends.get().saturating_sub(now.get());
    let already = already.max(1);
    (by > already && left > 1).then(|| Tick::new(now.get() + (left * already / by).max(1)))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The property the whole guard exists for, swept rather than sampled.
    ///
    /// Quickening may never make what it hurries land later. The bug it
    /// replaces was reachable on exactly one tick, so a sampled test would miss
    /// it.
    #[test]
    fn hurrying_a_run_never_lands_it_later() {
        for already in 1..=3 {
            for by in 2..=4 {
                for ends in 0..200u64 {
                    for now in 0..=ends {
                        let Some(moved) =
                            hurried_from(Tick::new(now), Tick::new(ends), by, already)
                        else {
                            continue;
                        };
                        assert!(
                            moved.get() < ends,
                            "hurrying a run ending at {ends} from {now} by {by} over \
                             {already} moved it to {}",
                            moved.get()
                        );
                        assert!(
                            moved.get() > now,
                            "hurrying a run ending at {ends} from {now} by {by} over \
                             {already} landed it now"
                        );
                    }
                }
            }
        }
    }

    /// The case the guard is *for*: nothing left to hurry.
    #[test]
    fn a_run_about_to_land_is_left_alone() {
        assert_eq!(hurried_from(Tick::new(10), Tick::new(10), 2, 1), None);
        assert_eq!(hurried_from(Tick::new(10), Tick::new(11), 2, 1), None);
        assert_eq!(
            hurried_from(Tick::new(10), Tick::new(12), 2, 1),
            Some(Tick::new(11))
        );
    }

    /// Divided from now, never from the start.
    ///
    /// A run that began at 0 and ends at 100, hurried at 80, has 20 left and
    /// lands at 90 at half, 86 at a third.
    #[test]
    fn a_run_is_hurried_from_now_and_not_from_its_start() {
        assert_eq!(
            hurried_from(Tick::new(80), Tick::new(100), 2, 1),
            Some(Tick::new(90))
        );
        assert_eq!(
            hurried_from(Tick::new(80), Tick::new(100), 3, 1),
            Some(Tick::new(86))
        );
    }

    /// The larger applies and never both: a run started at half is not halved
    /// again, and a third over a half takes it from 20 left to 13.
    #[test]
    fn a_hurried_run_is_hurried_only_by_what_more_adds() {
        assert_eq!(hurried_from(Tick::new(80), Tick::new(100), 2, 2), None);
        assert_eq!(hurried_from(Tick::new(80), Tick::new(100), 2, 3), None);
        assert_eq!(
            hurried_from(Tick::new(80), Tick::new(100), 3, 2),
            Some(Tick::new(93))
        );
    }
}
