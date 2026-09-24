//! Turning runs into something a person or a spreadsheet can read.
//!
//! Two outputs. The table is for the eye — §15's "work is done when it has been
//! looked at" applies to a sweep too, and a wall of CSV is not looking. The CSV
//! is for the shape of a curve over hours.
//!
//! The reference rates are what a clean run measured, not what DESIGN.md argues
//! in prose (see [`expected`]) — pinning to a reachable number is what stops it
//! being a column nobody reads.

use std::fmt::Write as _;

use crate::drive::Run;

/// What each policy earns per tick, as measured, so drift shows up.
///
/// Not DESIGN.md's numbers: the design argues from recipe ticks alone, while a
/// real loop also pays a tick of queue latency per command, a `PURGE_TICKS`
/// scour per byproduct, and a relight. Hand-played clarity reaches
/// `experience 16` at tick 123, not 94, and the looped policy settles at 0.140
/// once its first lap is amortised.
///
/// So this pins what the harness produced on a clean run (§19). A number moving
/// here means the *game* changed, which is the alarm §16 wants.
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
    // The number the lens redesign moved most: 0.07 when the ward exchanged
    // sigils (~23 presses at twelve ticks each) against the same 8 over ~12
    // presses at two ticks apiece now.
    //
    // A press takes no production slot, so this is additive to whatever the
    // laboratory is doing — what §8 wants automation to buy, and not something
    // one column can show. A real bound `breaking` measures ~0.18, because §8's
    // interpreter spends a tick on every `if`, `else` and `end`; a ceiling, as
    // `stacks` is (§19's "the harness has no player").
    ("scrying", 0.268),
    // The flattest column, by arithmetic rather than luck: a course of `n` wards
    // costs `2^n` ticks and pays `n - 2`, so three and four both come out at an
    // eighth, and a policy keeping the walls up musters only those two. All four
    // seeds read 0.1249, where `stacks` swings 0.0067 to 0.0144.
    //
    // Additive like `scrying`, but this domain pays twice — a finished course
    // puts integrity back as well as earning — which is why the rate sits under
    // the flagship's 0.140 where scrying's sits over it.
    ("warding", 0.125),
    // Flatter still — 0.0060 to 0.0059 across four seeds — because output is
    // bounded by a resource regenerating on a clock and the lattice's one draw
    // does not reach the rate.
    //
    // The lowest rate, deliberately: enchanting's payoff is a mortar at half
    // time, realised in whatever room the tool is in, and the token 1 a fall
    // pays is for the slot it held. It earns something rather than nothing only
    // because 0.0000 is indistinguishable from a broken domain.
    //
    // The one policy that competes rather than adds: `anneal` is an operation,
    // so a tower binding charms is a tower not brewing — §10's scarcity for the
    // domain, unmeasured until this column.
    ("imbuing", 0.006),
    // The search, not a player, and deliberately one of the table's lowest
    // rates where the chant read 0.243 (§19): a reader of the temper holds a
    // beast in one call, while the shipped `taming` tries circles in order and
    // averages 68. This measures the second, which is what the economy sees.
    //
    // Its own band, thirty-five percent, measured rather than widened to taste:
    // since `0.15.4`'s turned wires, seeds 0–7, 11 and 42 read 0.0319 to 0.0550,
    // because a two-hour run holds only about forty beasts and one can take 1
    // call or 174. At fifteen percent two seeds of ten flagged on luck alone. A
    // halved menagerie is still flagged; `agrees.rs` holds the four-world mean,
    // 0.0414, to that band over √4. See [`WIDER`] and [`tolerance_of_mean`].
    //
    // Additive as `scrying` is, and the floor of what knowledge buys: a spell
    // pruning the keystone by `fervour` and sunwise by De Morgan averages 38
    // calls. A bound `taming` holds 27 beasts in two hours over four worlds,
    // about 80 troops, against the ~27 `besieging` spends.
    ("taming", 0.042),
    // Nearly left unpinned for the wrong reason: the first spread, 0.043–0.085,
    // was not dice variance but `fight_one` keying "already spent this round"
    // on a rendered line's byte length, so consecutive rounds collided. Keyed
    // on `turns`, the spread was 0.1225–0.1313 over five seeds.
    //
    // Then pledging dice took it to 0.114 — allocating wins faster (6 rounds
    // against about 10) and costs more commands, and at three dice a round the
    // second effect is larger. The version that skipped the domain's central
    // decision measured what a player gets for *ignoring* the mechanic.
    //
    // 0.123 since pledging cost quintessence: the driver stops asking for dice
    // it cannot pay for, so refused commands now reach the arsenal ladder, and
    // the spread narrowed from 23% of the mean to 17%. It is gated by
    // `siege::CADENCE` more than by the loop's own speed, so this pin mostly
    // watches that constant.
    //
    // A single-seed sweep still flags this sometimes and the flag is not a
    // finding: over `agrees::SEEDS` it reads 0.1225, 0.1342, 0.1225, 0.1138, a
    // mean of 0.1232, because `--ticks 7200` fits only about six sieges. Hence
    // the pin is read by the mean of four worlds. Read `--why` before believing
    // `<-- drifted`: healthy costs are cadence waits and sabotage notices.
    ("besieging", 0.123),
];

/// How far a measurement may sit from its expectation before it is called out.
///
/// A tenth, now that the references are measured rather than argued. It was a
/// quarter while they came from prose, which would let clarity fall to 0.105 —
/// below the standing grind loop — without a word.
///
/// `stacks` is absent from [`EXPECTED`] because a maze is generated per seed and
/// one run is one sample, so pinning it would pin a seed. Sweep several
/// `--seed`s instead.
///
/// `bound` is absent because its absolute rate moves with a world's luck at
/// sabotage as `grind` does (0.0814 on seed 3 against 0.0910 on seed 0), while
/// the quotient of the two is 0.910 on every seed — a property of the script
/// engine rather than of the tower. `tests/agrees.rs` pins the quotient.
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
        if run.reasons.is_empty() {
            continue;
        }
        let _ = writeln!(out, "\n{} — {} cost", run.policy, run.cost);
        let mut ranked: Vec<_> = run.reasons.iter().collect();
        ranked.sort_by(|(a_why, a), (b_why, b)| b.cmp(a).then(a_why.cmp(b_why)));
        for (why, count) in ranked.iter().take(6) {
            let _ = writeln!(out, "  {count:>5}  {why}");
        }
    }
    out
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
