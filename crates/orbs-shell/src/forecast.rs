//! How long until the next station, in the words the road and the ley gauge
//! draw (§19, *the tower speaks up*).
//!
//! Coarse on purpose — *~3 min*, not *2 minutes 47* — as the resume line is: a
//! pace is a few makings' average, and a precise figure would claim more.

use orbs_sim::tower::Pace;
use orbs_sim::{Ahead, Line, Prose, Toward};

/// Minutes under which a forecast is in minutes; past it, hours.
const IN_MINUTES: u64 = 100;

/// `left` more at `pace`, in words — or `None` with no pace or nothing left.
#[must_use]
pub fn eta(prose: &Prose, pace: Option<Pace>, left: u64) -> Option<String> {
    let pace = pace.filter(|_| left > 0)?;
    Some(words(prose, pace.eta(left)))
}

/// The road's: its next deed's remaining count at that deed's pace.
#[must_use]
pub fn road(prose: &Prose, line: &Line, pace: Option<Pace>) -> Option<String> {
    let next = line.next()?;
    eta(
        prose,
        pace,
        u64::from(next.needed.saturating_sub(next.done)),
    )
}

/// The ley gauge's: experience still owed to the next station, at the pace it
/// has been earned. None for a track with nothing ahead.
#[must_use]
pub fn ley(
    prose: &Prose,
    station: Toward,
    experience: u64,
    earning: Option<Pace>,
) -> Option<String> {
    let Ahead::Tier(at) = station.ahead else {
        return None;
    };
    eta(prose, earning, at.saturating_sub(experience))
}

/// Ticks, which are seconds, as a coarse span.
fn words(prose: &Prose, ticks: u64) -> String {
    if ticks < 60 {
        return prose.line("eta_soon", &[]);
    }
    let minutes = ticks.div_ceil(60);
    if minutes < IN_MINUTES {
        return prose.line("eta_minutes", &[("count", &minutes.to_string())]);
    }
    let hours = (minutes + 30) / 60;
    prose.line("eta_hours", &[("count", &hours.to_string())])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_eta_is_the_pace_times_what_is_left_rounded_up() {
        let prose = Prose::builtin();
        // (ticks per making, makings left, what it reads).
        for (ticks, left, want) in [
            (60, 3, Some("~3 min")),
            // 61 ticks is two minutes, not one.
            (61, 1, Some("~2 min")),
            (20, 1, Some("under a minute")),
            (600, 30, Some("~5 h")),
            (60, 0, None),
        ] {
            let pace = Pace { ticks, made: 1 };
            assert_eq!(
                eta(&prose, Some(pace), left).as_deref(),
                want,
                "{ticks}t, {left} left"
            );
        }
        assert_eq!(eta(&prose, None, 3), None, "no pace is no forecast");
    }
}
