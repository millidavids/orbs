//! Turning runs into something a person or a spreadsheet can read.
//!
//! A table for the eye (§15) and a CSV for curves. The references are
//! measured, not argued ([`expected`]).

use std::fmt::Write as _;

use crate::drive::Run;

/// What each policy earns per tick, as measured, so drift shows up.
///
/// A clean run's measurement, not the design's recipe arithmetic, which omits
/// queue latency, scours and relights (§19). A move means the game changed.
const EXPECTED: [(&str, f64); 9] = [
    // 0.170 idealised (§19, "16, and why the anchor moved") against 0.140 looped.
    ("clarity", 0.140),
    // §10.1's claimed better play, measured behind the careless one — see
    // `Policy::DAMPED`. Pinned so the table says so if damping ever gets ahead.
    ("damped", 0.136),
    // §19 records the haste chain ahead of the flagship as a risk. The
    // idealised ratio is 2.16x; measured it is 1.82x. Still the risk, gentler.
    ("haste", 0.255),
    // §19's "1 experience per ~10 ticks for ever", and the one rate the
    // idealisation and the loop agree on exactly — two commands, no fire, no
    // byproduct, nothing to scour.
    ("grind", 0.100),
    // Additive, taking no slot (§8). A ceiling: a bound `breaking` reads ~0.18
    // (§19).
    ("scrying", 0.268),
    // Flat by arithmetic: courses of three and four both pay an eighth.
    // Additive, and it pays twice, restoring integrity too.
    ("warding", 0.125),
    // Flat, bounded by a regenerating resource. Lowest on purpose: the payoff
    // is the charm, and 0.0000 would look broken. It competes for the slot
    // (§10).
    ("imbuing", 0.006),
    // The search, not a player, so low (§19). A 35% band, measured: about
    // forty beasts a run, each 1 to 174 calls ([`WIDER`],
    // [`tolerance_of_mean`]). Additive; the garrison it feeds is §19's
    // *Supply*.
    ("taming", 0.042),
    // Mostly `siege::CADENCE`'s. About six sieges a run, so one seed may flag;
    // the pin is the four-world mean, 0.1232. Read `--why` before
    // `<-- drifted`.
    ("besieging", 0.123),
];

/// How far a measurement may sit from its expectation before it is called out.
///
/// A tenth, since the references are measured. Not `stacks`, one maze a seed,
/// nor `bound`, whose quotient over `grind` `tests/agrees.rs` pins instead.
const TOLERANCE: f64 = 0.10;

/// The policies whose single-seed spread is measured wider than [`TOLERANCE`],
/// and the band each is held to instead.
///
/// A table of its own rather than a third column every pin fills in. A wider
/// band claims the draw moves the rate, and the comment beside the pin argues
/// it; nothing here is wider than its worst measured seed needs.
const WIDER: [(&str, f64); 1] = [("taming", 0.35)];

/// The summary table.
#[must_use]
pub fn table(runs: &[Run]) -> String {
    let mut out = String::new();
    out.push_str(
        "policy      seed    ticks      xp  conc    xp/tick   landed  cost  renown  expected\n",
    );
    out.push_str(
        "----------------------------------------------------------------------------------------\n",
    );

    for run in runs {
        let last = run.last();
        let rate = run.rate();
        let note = match expected(run.policy) {
            Some(want) if off_by(rate, want, tolerance(run.policy)) => {
                format!("  {want:.3}  <-- drifted")
            }
            Some(want) => format!("  {want:.3}"),
            None => String::new(),
        };
        let _ = writeln!(
            out,
            "{:<10} {:>4} {:>8} {:>7} {:>5} {:>10.4} {:>8} {:>5} {:>7}{}",
            run.policy,
            run.seed,
            last.tick,
            last.experience,
            last.concentration,
            rate,
            run.landed,
            run.cost,
            last.renown,
            note,
        );
    }
    out
}

/// The curve, one row per sample, for every run in the sweep.
#[must_use]
pub fn csv(runs: &[Run]) -> String {
    let mut out = String::from("policy,seed,tick,experience,concentration,renown\n");
    for run in runs {
        for sample in &run.samples {
            let _ = writeln!(
                out,
                "{},{},{},{},{},{}",
                run.policy,
                run.seed,
                sample.tick,
                sample.experience,
                sample.concentration,
                sample.renown,
            );
        }
    }
    out
}

/// Why each policy's commands were turned down, commonest first.
///
/// The half of a sweep that says whether to believe the other half: a rate
/// measured while a third of the loop is refused is a real number about a loop
/// nobody wrote.
#[must_use]
pub fn why(runs: &[Run]) -> String {
    let mut out = String::new();
    for run in runs {
        // Where the loop ended up waiting: `status rates`, as the run left it.
        // Only rooms that made something or are held, or six idle rows bury
        // the one that matters.
        let rooms: Vec<String> = run.yields.iter().filter_map(yielded).collect();
        if run.reasons.is_empty()
            && run.given.is_empty()
            && !run.held.contains_key("fragment")
            && rooms.is_empty()
        {
            continue;
        }
        let _ = writeln!(out, "\n{} — {} cost", run.policy, run.cost);
        let mut ranked: Vec<_> = run.reasons.iter().collect();
        ranked.sort_by(|(a_why, a), (b_why, b)| b.cmp(a).then(a_why.cmp(b_why)));
        for (why, count) in ranked.iter().take(6) {
            let _ = writeln!(out, "  {count:>5}  {why}");
        }
        // What a supply-spending policy was handed and what it spent, and
        // the fragments the archive held, which is gleaning's yield.
        for (name, given) in &run.given {
            let left = run.held.get(name).copied().unwrap_or(0);
            let spent = given.saturating_sub(left);
            let _ = writeln!(out, "  {name}: {given} given, {spent} spent");
        }
        if let Some(fragments) = run.held.get("fragment") {
            let _ = writeln!(out, "  fragment: {fragments} held");
        }
        for line in rooms {
            let _ = writeln!(out, "  {line}");
        }
    }
    out
}

/// One room's rate and wait, for `--why`; `None` for a room with nothing to say.
fn yielded(one: &orbs_sim::tower::Yield) -> Option<String> {
    use orbs_sim::tower::{Hourly, Wait};
    let rate = match one.rate {
        Hourly::PerHour(count) => format!("{count} an hour"),
        Hourly::None | Hourly::Unknown => {
            if !matches!(one.wait, Wait::Held { .. }) {
                return None;
            }
            "none".to_owned()
        }
    };
    let wait = match &one.wait {
        Wait::At {
            instrument, state, ..
        } => format!("{} at {instrument}", state.label()),
        Wait::Held { spell, .. } => format!("{spell} held"),
        Wait::Nothing => "nothing".to_owned(),
    };
    Some(format!("{}: {rate}, {wait}", one.room))
}

/// What a policy is pinned to earn per tick, if anything.
///
/// Public so a test can hold the pin: `<-- drifted` is a regression pin only
/// while somebody reads the column, and `tests/agrees.rs` reads it every run.
#[must_use]
pub fn expected(policy: &str) -> Option<f64> {
    EXPECTED
        .iter()
        .find(|(name, _)| *name == policy)
        .map(|(_, rate)| *rate)
}

/// How far a policy's rate may sit from its pin: a tenth (`TOLERANCE`), unless
/// its measured seeds needed more (`WIDER`, where each wider band is argued).
#[must_use]
pub fn tolerance(policy: &str) -> f64 {
    WIDER
        .iter()
        .find(|(name, _)| *name == policy)
        .map_or(TOLERANCE, |(_, band)| *band)
}

/// How far a policy's rate averaged over `worlds` seeds may sit from its pin.
///
/// Narrower than [`tolerance`] wherever that band was widened for one seed's
/// luck: a mean of `n` runs spreads about `1/√n` as far, so `taming`'s
/// thirty-five percent is seventeen and a half over four worlds — held to the
/// whole band, a menagerie earning thirty percent less averaged inside it.
/// Never below `TOLERANCE`.
#[must_use]
pub fn tolerance_of_mean(policy: &str, worlds: usize) -> f64 {
    #[allow(clippy::cast_precision_loss)]
    let worlds = worlds.max(1) as f64;
    (tolerance(policy) / worlds.sqrt()).max(TOLERANCE)
}

/// Whether a measured rate has left its expectation's `tolerance` band.
///
/// Public for [`expected`]'s reason: the band is half the pin. It is passed in,
/// from [`tolerance`], so the table and the test cannot hold one policy to two
/// widths.
#[must_use]
pub fn off_by(measured: f64, want: f64, tolerance: f64) -> bool {
    (measured - want).abs() > want * tolerance
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drive::Sample;

    fn run_of(policy: &'static str, tick: u64, experience: u64) -> Run {
        Run {
            policy,
            seed: 0,
            samples: vec![Sample {
                tick,
                experience,
                concentration: 0,
                renown: 0,
            }],
            cost: 0,
            landed: 0,
            reasons: std::collections::BTreeMap::new(),
            held: std::collections::BTreeMap::new(),
            given: std::collections::BTreeMap::new(),
            yields: Vec::new(),
        }
    }

    #[test]
    fn a_rate_is_experience_over_ticks() {
        let run = run_of("clarity", 94, 16);
        assert!((run.rate() - 0.170).abs() < 0.001, "{}", run.rate());
    }

    #[test]
    fn a_run_of_no_length_has_no_rate_rather_than_a_panic() {
        // Reachable with `--ticks 0`.
        assert!((run_of("grind", 0, 0).rate() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn drift_is_flagged_and_agreement_is_not() {
        // A policy out of its band says so in the table, not a phase later.
        let band = tolerance("clarity");
        assert!(!off_by(0.140, 0.140, band));
        assert!(!off_by(0.133, 0.140, band), "inside a tenth, so not drift");
        assert!(
            off_by(0.100, 0.140, band),
            "clarity down to the grind loop, unflagged"
        );
    }

    /// `taming`'s band is its measured seeds and no wider: the lowest and
    /// highest of ten read inside it, and a halved menagerie is still out.
    #[test]
    fn the_menageries_wider_band_holds_its_seeds_and_still_flags_a_halving() {
        let want = expected("taming").expect("taming is pinned");
        let band = tolerance("taming");
        assert!(band > TOLERANCE, "taming has no band of its own");
        for seed_rate in [0.0319, 0.0550] {
            assert!(!off_by(seed_rate, want, band), "{seed_rate} flagged");
        }
        assert!(off_by(want / 2.0, want, band), "a halved menagerie passed");

        // A mean is held tighter than a seed: four worlds earning thirty
        // percent less sat inside the single-seed band.
        let mean_band = tolerance_of_mean("taming", 4);
        assert!(mean_band < band, "a mean held to a single seed's band");
        assert!(off_by(want * 0.7, want, mean_band), "a 30% cut passed");
        assert!(
            !off_by(0.0414, want, mean_band),
            "the measured mean flagged"
        );
        assert!(
            (tolerance_of_mean("clarity", 4) - TOLERANCE).abs() < f64::EPSILON,
            "a tenth was loosened or tightened by averaging"
        );
        assert!(
            WIDER
                .iter()
                .all(|(name, _)| EXPECTED.iter().any(|(pinned, _)| pinned == name)),
            "a wider band for a policy nothing pins",
        );
    }

    #[test]
    fn the_flagship_still_beats_the_standing_grind_loop() {
        // §11.5's shape: finishing the chain must pay better than repeating its
        // first step for ever.
        let clarity = expected("clarity").expect("clarity is pinned");
        let grind = expected("grind").expect("grind is pinned");
        assert!(clarity > grind, "{clarity} is not ahead of {grind}");
    }

    #[test]
    fn every_expected_policy_is_one_the_harness_can_run() {
        // An unselectable policy's reference is never checked, and looks
        // exactly like one that always agrees.
        for (name, _) in EXPECTED {
            assert!(
                crate::policy::Policy::named(name).is_some(),
                "no policy named {name}",
            );
        }
    }

    #[test]
    fn the_csv_carries_every_sample_of_every_run() {
        let runs = vec![run_of("clarity", 94, 16), run_of("grind", 40, 4)];
        let csv = csv(&runs);
        assert_eq!(csv.lines().count(), 3, "header plus one row per sample");
        assert!(csv.contains("clarity,0,94,16,0"));
        assert!(csv.contains("grind,0,40,4,0"));
    }
}
