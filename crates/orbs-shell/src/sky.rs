//! The sky over the orb, for the one word at the menu that asks about it.
//!
//! The wall clock, so the shell's and never the sim's (rule 3): nothing in a
//! tower can see it, so it cannot make two runs of one seed diverge. A dump is
//! a still photograph and reads no clock unless `ORBS_DATE` names a day.
//!
//! In UTC. A local date needs the zone, which `std` will not give without
//! `unsafe`, so the evening of the 31st can already be the 1st here. The moon
//! itself does not care.

use std::sync::atomic::{AtomicBool, Ordering};

/// Names the day to read the sky for, as `YYYY-MM-DD`. For a See-it line.
pub const DATE: &str = "ORBS_DATE";

static COVERED: AtomicBool = AtomicBool::new(false);

/// This process reads no clock, unless `ORBS_DATE` names a day.
///
/// Called by the dump, so no capture depends on when it was taken. One-way,
/// for `save::seal`'s reason.
pub fn cover() {
    COVERED.store(true, Ordering::Relaxed);
}

/// A day, counted from 1970-01-01.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Day(pub i64);

impl Day {
    /// The day `text` names, as `YYYY-MM-DD`.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let mut parts = text.trim().split('-');
        let year: i64 = parts.next()?.parse().ok()?;
        let month: i64 = parts.next()?.parse().ok()?;
        let day: i64 = parts.next()?.parse().ok()?;
        if parts.next().is_some() || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
            return None;
        }
        // Back again, or `2026-02-31` is quietly the 3rd of March.
        Some(Self(from_civil(year, month, day))).filter(|it| it.civil() == (year, month, day))
    }

    /// The year, month and day of the month.
    #[must_use]
    pub fn civil(self) -> (i64, i64, i64) {
        // Howard Hinnant's algorithm, in whole days: no float, no lossy cast.
        let z = self.0 + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z.rem_euclid(146_097);
        let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = doy - (153 * mp + 2) / 5 + 1;
        let month = if mp < 10 { mp + 3 } else { mp - 9 };
        let year = yoe + era * 400 + i64::from(month <= 2);
        (year, month, day)
    }

    /// The day of the week, Sunday first.
    #[must_use]
    pub const fn weekday(self) -> i64 {
        // 1970-01-01 was a Thursday.
        (self.0 + 4).rem_euclid(7)
    }
}

/// Days from 1970-01-01 to a civil date. Hinnant's inverse.
const fn from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let yoe = year - era * 400;
    let mp = (month + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Today, if anything may ask.
#[must_use]
pub fn tonight() -> Option<Day> {
    if let Some(named) = std::env::var(DATE)
        .ok()
        .filter(|text| !text.trim().is_empty())
    {
        return Day::parse(&named);
    }
    if COVERED.load(Ordering::Relaxed) {
        return None;
    }
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    i64::try_from(seconds / 86_400).ok().map(Day)
}

/// Where the moon is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Dark.
    New,
    /// Growing, and how many nights until it is full.
    Waxing(u32),
    /// Full.
    Full,
    /// Shrinking, and how many nights until it is new.
    Waning(u32),
}

/// A lunation, in days.
const SYNODIC: f64 = 29.530_588_853;

/// A known new moon — 2000-01-06 18:14 UTC — in days from 1970.
const KNOWN_NEW: f64 = 10_962.759_7;

/// How near a new or full moon counts as one.
///
/// The mean lunation drifts from the true one by up to about fourteen hours,
/// so a day either side is the least that names every eclipse right; a day and
/// a half is how people speak of it anyway.
const BAND: f64 = 1.5;

/// The moon on `day`, at noon UTC.
#[must_use]
pub fn phase(day: Day) -> Phase {
    let Ok(day) = i32::try_from(day.0) else {
        return Phase::New;
    };
    let age = (f64::from(day) + 0.5 - KNOWN_NEW).rem_euclid(SYNODIC);
    let half = SYNODIC / 2.0;
    if !(BAND..=SYNODIC - BAND).contains(&age) {
        Phase::New
    } else if (age - half).abs() < BAND {
        Phase::Full
    } else if age < half {
        Phase::Waxing(nights(half - age))
    } else {
        Phase::Waning(nights(SYNODIC - age))
    }
}

/// `days`, rounded to whole nights, without a lossy cast.
fn nights(days: f64) -> u32 {
    let mut whole = 0_u32;
    while f64::from(whole) + 0.5 < days && whole < 30 {
        whole += 1;
    }
    whole.max(1)
}

/// A night the orb has an opinion about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Occasion {
    /// Any other night.
    Ordinary,
    /// The 31st of October.
    Hallows,
    /// A Friday that is the 13th.
    Unlucky,
}

/// What kind of night `day` is.
#[must_use]
pub fn occasion(day: Day) -> Occasion {
    let (_, month, date) = day.civil();
    if (month, date) == (10, 31) {
        Occasion::Hallows
    } else if date == 13 && day.weekday() == 5 {
        Occasion::Unlucky
    } else {
        Occasion::Ordinary
    }
}

/// What `moon` answers at the menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reading {
    /// No sky to read — a dump with no `ORBS_DATE`.
    Unseen,
    /// The moon, and what kind of night it is.
    Seen(Phase, Occasion),
}

/// The sky tonight.
#[must_use]
pub fn reading() -> Reading {
    tonight().map_or(Reading::Unseen, |day| {
        Reading::Seen(phase(day), occasion(day))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn on(text: &str) -> Day {
        Day::parse(text).expect("a date")
    }

    #[test]
    fn a_date_survives_the_round_trip() {
        for text in ["1970-01-01", "2000-02-29", "2026-10-31", "1999-12-31"] {
            let (year, month, day) = on(text).civil();
            assert_eq!(format!("{year:04}-{month:02}-{day:02}"), text);
        }
        assert_eq!(on("1970-01-01").weekday(), 4, "1970 began on a Thursday");
        assert_eq!(Day::parse("2026-13-01"), None);
        assert_eq!(Day::parse("2026-02-31"), None, "rolled into March");
        assert_eq!(Day::parse("2026-02-29"), None, "2026 is no leap year");
        assert_eq!(Day::parse("yesterday"), None);
    }

    #[test]
    fn every_eclipse_is_named_rightly() {
        // A solar eclipse is a new moon and a lunar one is full, which makes
        // them dates no table of mine could have got wrong.
        for text in ["2017-08-21", "2023-10-14", "2024-04-08"] {
            assert_eq!(phase(on(text)), Phase::New, "{text}, a solar eclipse");
        }
        for text in ["2018-07-27", "2022-11-08", "2025-03-14", "2025-09-07"] {
            assert_eq!(phase(on(text)), Phase::Full, "{text}, a lunar eclipse");
        }
    }

    #[test]
    fn a_waxing_moon_counts_down_to_full() {
        // A week after the 2024-04-08 eclipse is first quarter.
        let Phase::Waxing(nights) = phase(on("2024-04-15")) else {
            panic!("a week after new was not waxing");
        };
        assert!((6..=9).contains(&nights), "{nights} nights from full");
    }

    #[test]
    fn the_nights_with_opinions() {
        assert_eq!(occasion(on("2026-10-31")), Occasion::Hallows);
        // 2026-02-13 is a Friday.
        assert_eq!(occasion(on("2026-02-13")), Occasion::Unlucky);
        // ...and 2026-04-13 a Monday.
        assert_eq!(occasion(on("2026-04-13")), Occasion::Ordinary);
    }
}
