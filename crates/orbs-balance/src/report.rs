//! Turning runs into something a person or a spreadsheet can read.
//!
//! Two outputs, deliberately. **The table is for the eye** — §15's "work is done
//! when it has been looked at" applies to a balance sweep as much as to a
//! screen, and a wall of CSV is not looking. **The CSV is for the sweep**, where
//! the question is the shape of a curve over hours rather than one number.
//!
//! The reference rates are what a clean run of each policy **measured**, not
//! what DESIGN.md argues for in prose — see [`expected`], where the gap between
//! the two is the first thing this crate found. Printing a reference beside the
//! measurement is what makes the See-it line *"a sweep's curve and a hand-played
//! session agree"* something you can check rather than assert, and pinning it to
//! a reachable number is what stops it being a column nobody reads.

use std::fmt::Write as _;

use crate::drive::Run;

/// What each policy earns per tick, **as measured**, so drift shows up.
///
/// # These are not DESIGN.md's numbers, and the gap is the first finding
///
/// The design argues its rates from **recipe ticks alone** — clarity's 16 over
/// 94 is grind 8, digest 12, grind 8, mix 10, distil 56 — and a real loop also
/// pays a tick of queue latency per command, a `PURGE_TICKS` scour to clear each
/// byproduct, and a relight when the charcoal runs out. A hand-played clarity
/// reaches `experience 16` at **tick 123**, not 94, which is 0.130 rather than
/// 0.170; the looped policy settles at 0.140 once its first lap's setup is
/// amortised. Neither number is wrong — they measure different things — but only
/// one of them is reachable, and a reference nothing can hit is a reference
/// nobody checks.
///
/// So this table pins **what the harness produced on a clean run**, and
/// DESIGN.md keeps its idealisation with a §19 note saying what the loop costs on
/// top. That makes this a regression pin: a number moving here means the *game*
/// changed, which is exactly the alarm §16 wants.
const EXPECTED: [(&str, f64); 8] = [
    // 0.170 idealised (§19, "16, and why the anchor moved") against 0.140 looped.
    ("clarity", 0.140),
    // §10.1's claimed better play, measured **behind** the careless one — see
    // `Policy::DAMPED`. Pinned so that if damping ever does get ahead, the table
    // says so rather than nobody noticing.
    ("damped", 0.136),
    // §19: "the fastest experience in the game is the haste chain at 0.367/tick
    // against the flagship clarity's 0.170" — recorded there as a risk. The
    // idealised ratio is 2.16x; measured it is 1.82x. Still the risk, gentler.
    ("haste", 0.255),
    // §19: "1 experience per ~10 ticks for ever". The one rate the idealisation
    // and the loop agree on exactly, because the loop is two commands and has no
    // fire, no byproduct and nothing to scour.
    ("grind", 0.100),
    // **The number the lens redesign moved most, and the reason it is pinned at
    // all.** Automated scrying read 0.07 while the ward exchanged sigils, took
    // ~23 presses and paid twelve ticks for each; it reads 0.268 now — the same
    // 8 a solve, over ~12 presses at two ticks apiece.
    //
    // It is *unlike* the four above in one way worth knowing before reading it:
    // a press takes no production slot, so this is the only policy here that
    // never waits on the tower and never competes with a brew. The rate is
    // therefore **additive** to whatever the laboratory is doing rather than an
    // alternative to it, which is what §8 wants automation to buy and is not
    // something the table's single column can show.
    //
    // A real bound `breaking` measures ~0.18 over the same span, because §8's
    // interpreter spends a tick on every `if`, `else` and `end`. This is a
    // ceiling, exactly as `stacks` is — §19's *"the harness has no player"* puts
    // execution outside what a policy models.
    ("scrying", 0.268),
    // **The flattest column in the table, and the flatness is arithmetic rather
    // than luck.** A course of `n` wards costs `2^n` ticks and pays `n - 2`, so
    // three and four both come out at an eighth — and a policy keeping the walls
    // up only ever musters those two. All four seeds read 0.1249 to the digit,
    // where `stacks` swings 0.0067 to 0.0144 on the same four.
    //
    // Additive, like `scrying` and for the same reason: neither `muster` nor
    // `haul` takes the production slot, so this runs beside a brew rather than
    // instead of one. What is different is that this domain **pays twice** — a
    // finished course puts integrity back as well as earning — which is why the
    // rate sits under the flagship's 0.140 where scrying's sits over it.
    ("warding", 0.125),
    // **A ceiling, not a player**, and the arithmetic is exact: a figure is
    // twelve syllables four ticks apart, so a chant sung perfectly earns twelve
    // over forty-nine ticks and reads 0.243 on every seed. The draw is uniform
    // and the cost is the same whichever lane lands, so — like `warding` and
    // unlike `stacks` — there is nothing here for a seed to move.
    //
    // **It sits above the flagship on purpose**, beside `scrying` and for the
    // same reason: neither verb takes the production slot, so this is additive
    // rather than competing. Two things keep it honest that the column cannot
    // show. A *person* misses syllables and earns less; and a **spell cannot do
    // this at all** until the weave grants a second step, where every other
    // domain automates from the first. The policy measures the roof both are
    // under, which is what a policy is for.
    //
    // It is also the one entry that can make the tower **worse**: a regression
    // in the timing shows up here as a falling rate *and* as integrity draining,
    // and the pair is what to read.
    ("chanting", 0.243),
    // **The siege, and it was very nearly left unpinned for the wrong reason.**
    // The first measurement spread 0.043–0.085 across seeds, which read as dice
    // variance over the ~6 sieges a two-hour run fits — the argument `stacks`
    // makes, and it was written into the docs as such.
    //
    // It was not the dice. `fight_one` keyed *"have I already spent this round"*
    // on the byte length of a rendered prose line, so consecutive rounds collided
    // and the driver stopped using its arsenal at random. Keyed on `turns` the
    // spread tightened to **0.1225–0.1313 over five seeds**, tight enough to pin.
    //
    // **Then the driver learned to pledge dice and it settled at 0.114**, down
    // from that 0.128 midpoint — and the number moving *down* when the policy
    // started playing better is the part worth keeping. Allocating wins faster —
    // a pledged siege runs **6 rounds against about 10** — and costs more
    // commands to do it, and at three dice a round the second effect is larger.
    // The version that skipped the domain's central decision was measuring what
    // a player gets for *ignoring* the mechanic, which is not a ceiling worth
    // pinning at any spread.
    //
    // 0.114 sits under warding's 0.125 and clearly under clarity's 0.140, which
    // is where a domain that also mends the barrier *and* consumes the arsenal
    // belongs. It is gated by `siege::CADENCE` far more than by the loop's own
    // speed, so this pin is mostly watching that constant.
    //
    // **0.123 since pledging cost quintessence**, up from 0.114 — and the rate
    // rising when the domain got *harder* is the part worth understanding.
    // Nothing about a siege got cheaper: what changed is that the driver stops
    // asking for dice it cannot pay for, so the commands it used to spend being
    // refused now reach the arsenal ladder instead. The policy plays better
    // because the world tells it what it can afford.
    //
    // The spread narrowed with it, 23% of the mean to 17%, for the same reason —
    // a run's rate now depends less on how many refusals it happened to eat.
    //
    // **A single-seed sweep will still flag this sometimes, and the flag is not
    // a finding.** Measured over `agrees::SEEDS`: 0.1225, 0.1342, 0.1225,
    // 0.1138 — a **mean of 0.1232**. `--ticks 7200` fits only about six sieges,
    // so one badly-timed sabotage still moves a whole siege and the seed shows
    // through.
    //
    // This is why the pin is read by the **mean of four worlds** and not by the
    // column. `stacks` is the same problem and is left unpinned because a maze
    // is one sample per seed with nothing to average; a siege averages, so it is
    // pinned. Read `--why` before believing `<-- drifted` here: healthy costs are
    // cadence waits and sabotage notices, and anything else means the driver has
    // fallen out of phase.
    ("besieging", 0.123),
];

/// How far a measurement may sit from its expectation before it is called out.
///
/// **A tenth, now that the references are measured rather than argued.** It was
/// a quarter while they came from prose, which was the right width for numbers
/// that were never precise and the wrong one for a regression pin — a quarter
/// would let clarity fall to 0.105, below the standing grind loop, without a
/// word.
///
/// `stacks` is deliberately absent from [`EXPECTED`]: a maze is generated per
/// seed and one run is one sample, so pinning it would pin a seed rather than a
/// rate. Sweep it across several `--seed`s instead.
///
/// **`bound` is absent for the opposite reason** — not because one run says too
/// little, but because its absolute rate is the wrong thing to hold. It moves
/// with a world's luck at sabotage exactly as `grind` does (0.0814 on seed 3
/// against 0.0910 on seed 0), while the quotient of the two is 0.910 on every
/// seed measured, because that is a property of the script engine rather than of
/// the tower. `tests/agrees.rs` pins the quotient; this table reports the rate.
const TOLERANCE: f64 = 0.10;

/// The summary table.
#[must_use]
pub fn table(runs: &[Run]) -> String {
    let mut out = String::new();
    out.push_str("policy      seed    ticks      xp  conc    xp/tick   landed  cost  expected\n");
    out.push_str(
        "--------------------------------------------------------------------------------\n",
    );

    for run in runs {
        let last = run.last();
        let rate = run.rate();
        let note = match expected(run.policy) {
            Some(want) if off_by(rate, want) => format!("  {want:.3}  <-- drifted"),
            Some(want) => format!("  {want:.3}"),
            None => String::new(),
        };
        let _ = writeln!(
            out,
            "{:<10} {:>4} {:>8} {:>7} {:>5} {:>10.4} {:>8} {:>8}{}",
            run.policy,
            run.seed,
            last.tick,
            last.experience,
            last.concentration,
            rate,
            run.landed,
            run.cost,
            note,
        );
    }
    out
}

/// The curve, one row per sample, for every run in the sweep.
#[must_use]
pub fn csv(runs: &[Run]) -> String {
    let mut out = String::from("policy,seed,tick,experience,concentration\n");
    for run in runs {
        for sample in &run.samples {
            let _ = writeln!(
                out,
                "{},{},{},{},{}",
                run.policy, run.seed, sample.tick, sample.experience, sample.concentration,
            );
        }
    }
    out
}

/// Why each policy's commands were turned down, commonest first.
///
/// **The half of a sweep that says whether to believe the other half.** A rate
/// measured while a third of the loop is being refused is a real number about a
/// loop nobody wrote, and the only way to tell the two apart is to read the
/// sentences the tower was saying at the time.
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
/// **Public so a test can hold the pin, and that is not test-only API.** The
/// number is already on screen in every [`table`] a person reads; what was missing
/// is anything that *fails* when a measurement leaves the band. A `<-- drifted`
/// marker in a column is a regression pin only for as long as somebody is looking
/// at the column, and `tests/agrees.rs` is what looks at it every run.
#[must_use]
pub fn expected(policy: &str) -> Option<f64> {
    EXPECTED
        .iter()
        .find(|(name, _)| *name == policy)
        .map(|(_, rate)| *rate)
}

/// Whether a measured rate has left its expectation's tolerance band.
///
/// Public for the same reason [`expected`] is: the band is half the pin, and a
/// test asserting one without the other would invent a second tolerance.
#[must_use]
pub fn off_by(measured: f64, want: f64) -> bool {
    (measured - want).abs() > want * TOLERANCE
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
        // Reachable with `--ticks 0`, and dividing by nought here would take the
        // harness down over a question nobody asked.
        assert!((run_of("grind", 0, 0).rate() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn drift_is_flagged_and_agreement_is_not() {
        // The whole point of the reference column: a policy that has wandered
        // out of its band says so in the table rather than in a later phase.
        assert!(!off_by(0.140, 0.140));
        assert!(!off_by(0.133, 0.140), "inside a tenth, so not drift");
        assert!(
            off_by(0.100, 0.140),
            "clarity down to the grind loop, unflagged"
        );
    }

    #[test]
    fn the_flagship_still_beats_the_standing_grind_loop() {
        // §11.5's whole shape in one assertion: finishing the chain must pay
        // better than repeating its first step for ever. The tolerance band is
        // set so that a clarity which fell this far would be flagged, and this
        // is the sentence that says why anyone cares.
        let clarity = expected("clarity").expect("clarity is pinned");
        let grind = expected("grind").expect("grind is pinned");
        assert!(clarity > grind, "{clarity} is not ahead of {grind}");
    }

    #[test]
    fn every_expected_policy_is_one_the_harness_can_run() {
        // A reference for a policy nobody can select would never be checked, and
        // would look exactly like a policy that always agrees.
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
