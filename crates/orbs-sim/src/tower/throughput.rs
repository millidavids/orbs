//! How fast the tower makes things, lately (§19, *the tower speaks up*).
//!
//! A [`Ring`] per tally key, fed at the one door every completion passes, and
//! the experience total at each credit. `Stores` answers *is it fresh*; this
//! answers *how fast*, which is the question `status rates` and a forecast ask.

use std::collections::BTreeMap;

use bevy_ecs::prelude::*;

use super::ring::{Pace, Ring};

/// The window a rate is measured over, in ticks: an hour, the sentence's unit.
pub const HOUR: u64 = 3_600;

/// How long a tower must have been watched before one making is a rate.
///
/// Five minutes: under it, one brew is a point, and "12 an hour" off a single
/// one would be a promise.
pub const SETTLE: u64 = 300;

/// The most makings worth remembering per key. Enough for a busy hour's pace;
/// a spell hammering one recipe cannot grow the save past it.
const REMEMBERED: usize = 64;

/// Ticks a minute: what a making the ring let go is counted to.
const MINUTE: u64 = 60;

/// How fast, as `status rates` says it. Not `content::Rate`, a supply's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hourly {
    /// Too little has been watched to say.
    Unknown,
    /// Nothing made this hour.
    None,
    /// So many an hour.
    PerHour(u64),
}

/// The tower's makings, lately, and its experience as it was earned.
///
/// `since` is when it began watching: tick nought for a new tower, the tick an
/// older save loaded into. Saved, because the window is in ticks and none run
/// while the orb is closed.
#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct Throughput {
    made: BTreeMap<String, Ring>,
    /// Makings a key's ring let go while still inside the hour, as a minute
    /// and a count: the hour is counted whole without keeping every tick.
    spilled: BTreeMap<String, Vec<(u64, u64)>>,
    /// The tick and the experience total at each credit, oldest first.
    earned: Vec<(u64, u64)>,
    since: u64,
}

impl Throughput {
    /// Record a completion counted under `key` at `now`.
    pub fn made(&mut self, key: &str, now: u64) {
        let let_go = self
            .made
            .entry(key.to_owned())
            .or_default()
            .push(now, HOUR, REMEMBERED);
        if let Some(minutes) = self.spilled.get_mut(key) {
            minutes.retain(|(minute, _)| counts(*minute, now));
        }
        if let Some(at) = let_go {
            let minutes = self.spilled.entry(key.to_owned()).or_default();
            match minutes.last_mut() {
                Some((minute, count)) if *minute == at / MINUTE => *count += 1,
                _ => minutes.push((at / MINUTE, 1)),
            }
        }
    }

    /// Record that experience stood at `total` at `now`.
    ///
    /// A total, not the amount: a pace of experience is an amount over time,
    /// which a [`Ring`] of ticks cannot carry.
    pub fn earned(&mut self, now: u64, total: u64) {
        self.earned.push((now, total));
        super::ring::bound(&mut self.earned, |(at, _)| *at, now, HOUR, REMEMBERED);
    }

    /// How many `key` were made in the last hour of ticks, over as much of it
    /// as was watched.
    ///
    /// The ring's newest and the minutes it let go count together, so a fast
    /// room is not pinned at what the ring holds, and the 64th making moves
    /// the rate no more than the 63rd did.
    #[must_use]
    pub fn rate(&self, key: &str, now: u64) -> Hourly {
        let start = self.since.max(now.saturating_sub(HOUR));
        let kept = self.made.get(key).map_or(0, |ring| ring.within(now, HOUR));
        let spilled: u64 = self.spilled.get(key).map_or(0, |minutes| {
            minutes
                .iter()
                .filter(|(minute, _)| counts(*minute, now))
                .map(|(_, count)| count)
                .sum()
        });
        let span = now.saturating_sub(start);
        let made = u64::try_from(kept)
            .unwrap_or(u64::MAX)
            .saturating_add(spilled);
        if made < 2 && span < SETTLE {
            Hourly::Unknown
        } else if made == 0 {
            Hourly::None
        } else {
            Hourly::PerHour(made.saturating_mul(HOUR) / span.max(1))
        }
    }

    /// How fast `key` has been made, from its gaps. `None` under two.
    #[must_use]
    pub fn pace(&self, key: &str) -> Option<Pace> {
        self.made.get(key).and_then(Ring::pace)
    }

    /// How fast experience has been earned, from the credits remembered.
    #[must_use]
    pub fn earning(&self) -> Option<Pace> {
        let (first, last) = (self.earned.first()?, self.earned.last()?);
        Pace::of(
            last.0.saturating_sub(first.0),
            last.1.saturating_sub(first.1),
        )
    }

    /// Begin watching at `tick`, for a save written before this was kept.
    pub(crate) fn begin(&mut self, tick: u64) {
        *self = Self {
            since: tick,
            ..Self::default()
        };
    }

    /// What a save keeps: when watching began, every key's ticks, the minutes
    /// its ring let go, and the experience credits.
    #[must_use]
    pub fn parts(&self) -> ThroughputParts<'_> {
        ThroughputParts {
            since: self.since,
            made: self
                .made
                .iter()
                .map(|(key, ring)| (key.as_str(), ring.ticks()))
                .collect(),
            spilled: self
                .spilled
                .iter()
                .filter(|(_, minutes)| !minutes.is_empty())
                .map(|(key, minutes)| (key.as_str(), minutes.as_slice()))
                .collect(),
            earned: &self.earned,
        }
    }

    /// Put it back, for a save.
    pub(crate) fn restore(&mut self, since: u64, made: Makings, earned: Vec<(u64, u64)>) {
        self.since = since;
        self.made = made
            .ticks
            .into_iter()
            .map(|(key, ticks)| (key, Ring::from_ticks(ticks)))
            .collect();
        self.spilled = made.spilled;
        self.earned = earned;
    }
}

/// What a save keeps of a [`Throughput`].
pub struct ThroughputParts<'a> {
    /// The tick it began watching.
    pub since: u64,
    /// Each key's remembered ticks.
    pub made: Vec<(&'a str, &'a [u64])>,
    /// Each key's let-go minutes and their counts.
    pub spilled: Vec<(&'a str, &'a [(u64, u64)])>,
    /// The experience credits.
    pub earned: &'a [(u64, u64)],
}

/// Each key's makings, as a save gives them back.
#[derive(Debug, Default)]
pub struct Makings {
    /// The remembered ticks.
    pub ticks: BTreeMap<String, Vec<u64>>,
    /// The minutes the ring let go, and how many in each.
    pub spilled: BTreeMap<String, Vec<(u64, u64)>>,
}

/// Whether a minute's makings still count at `now`: until its last tick has
/// left the hour, so the count is whole to the minute.
const fn counts(minute: u64, now: u64) -> bool {
    let last = minute.saturating_mul(MINUTE).saturating_add(MINUTE - 1);
    super::ring::inside(last, now, HOUR)
}

/// Feed one completion's keys and the experience it left, at the tally's door.
pub(crate) fn record(world: &mut World, keys: &[String], earned: u64) {
    let now = world.resource::<crate::tick::Tick>().get();
    let total = world.resource::<super::Experience>().get();
    let mut throughput = world.resource_mut::<Throughput>();
    for key in keys {
        throughput.made(key, now);
    }
    if earned > 0 {
        throughput.earned(now, total);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_making_is_too_soon_to_say_and_two_are_a_rate() {
        let mut throughput = Throughput::default();
        throughput.made("potion", 60);
        assert_eq!(throughput.rate("potion", 120), Hourly::Unknown);
        throughput.made("potion", 130);
        // Two in 130 ticks, which is fifty-five an hour.
        assert_eq!(throughput.rate("potion", 130), Hourly::PerHour(55));
    }

    #[test]
    fn a_watched_hour_with_nothing_in_it_is_none() {
        let throughput = Throughput::default();
        assert_eq!(throughput.rate("potion", SETTLE - 1), Hourly::Unknown);
        assert_eq!(throughput.rate("potion", SETTLE), Hourly::None);
    }

    #[test]
    fn the_rate_counts_only_the_last_hour_of_ticks() {
        let mut throughput = Throughput::default();
        for at in [0, 10, 20, 4_000, 4_010] {
            throughput.made("potion", at);
        }
        assert_eq!(throughput.rate("potion", 4_010), Hourly::PerHour(2));
    }

    #[test]
    fn a_fast_room_is_not_pinned_at_what_the_ring_holds() {
        // One every two ticks is 1,800 an hour. Counting the ring alone read
        // 64, its cap, for the lens and the sanctum in every sweep.
        let mut throughput = Throughput::default();
        for at in (0..2_000).step_by(2) {
            throughput.made("at:prism", at);
        }
        let Hourly::PerHour(rate) = throughput.rate("at:prism", 1_998) else {
            panic!("a busy room read no rate");
        };
        assert!((1_700..=1_900).contains(&rate), "{rate} an hour");
        // ...and it falls once the making stops, rather than holding, until
        // the hour has nothing in it.
        let Hourly::PerHour(later) = throughput.rate("at:prism", 3_000) else {
            panic!("a stopped room read no rate");
        };
        assert!(
            later < rate,
            "{later} an hour a thousand ticks after the last"
        );
        assert_eq!(
            throughput.rate("at:prism", 1_998 + HOUR + MINUTE),
            Hourly::None
        );
    }

    #[test]
    fn the_making_that_fills_the_ring_moves_the_rate_no_more_than_any_other() {
        // Idle a while, then one every two ticks: the 64th making once took the
        // rate from 201 an hour to 1,828, with the pace unchanged.
        let mut throughput = Throughput::default();
        let mut last = None;
        for at in (1_000..1_400).step_by(2) {
            throughput.made("at:prism", at);
            let Hourly::PerHour(rate) = throughput.rate("at:prism", at) else {
                continue;
            };
            if let Some(before) = last {
                assert!(
                    rate.abs_diff(before) <= before / 10 + 10,
                    "{before} an hour, then {rate} at tick {at}"
                );
            }
            last = Some(rate);
        }
    }

    #[test]
    fn a_tower_loaded_from_an_older_save_measures_from_the_load() {
        let mut throughput = Throughput::default();
        throughput.begin(10_000);
        assert_eq!(throughput.rate("potion", 10_100), Hourly::Unknown);
        throughput.made("potion", 10_200);
        throughput.made("potion", 10_250);
        assert_eq!(throughput.rate("potion", 10_250), Hourly::PerHour(28));
    }

    #[test]
    fn a_pace_is_the_gap_between_makings() {
        // `pace` takes no clock, so it cannot move while nothing is made;
        // `tests/rates.rs` holds that across 600 idle ticks.
        let mut throughput = Throughput::default();
        throughput.made("potion", 100);
        throughput.made("potion", 160);
        assert_eq!(throughput.pace("potion"), Some(Pace { ticks: 60, made: 1 }));
    }

    #[test]
    fn experience_is_paced_by_the_amount_not_the_count() {
        let mut throughput = Throughput::default();
        throughput.earned(0, 10);
        throughput.earned(30, 16);
        throughput.earned(90, 40);
        assert_eq!(
            throughput.earning(),
            Some(Pace {
                ticks: 90,
                made: 30
            })
        );
    }
}
