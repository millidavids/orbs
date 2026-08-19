//! The See-it line, as a test: *a sweep's curve and a hand-played session agree*.
//!
//! ROADMAP's Phase 2 item asks for exactly that, and it is the one claim a
//! balance harness cannot be trusted without. If the synthetic player and a
//! person typing the same commands reach different numbers, every sweep after it
//! is measuring a game nobody plays — §13's divergence risk, arriving through
//! the instrument built to prevent it.
//!
//! The hand-played reference is CLAUDE.md's own worked line for a clarity, run
//! here through the same public `Sim` a dump uses.
//!
//! # Four claims, and two of them arrived late
//!
//! The first two tests below drive `Sim` directly and hold the *anchor*: one
//! clarity by hand earns exactly the 16 that §11.5's first threshold is derived
//! from, over more ticks than its recipes alone. They would both pass with this
//! crate's entire harness deleted, which is what the last two are for — they run
//! real [`Policy`]s through [`drive::run`] and compare what a sweep measures
//! against the hand-played number and against [`report::expected`]'s pins.

use orbs_balance::{drive, policy::Policy, report};
use orbs_sim::Sim;

/// How long a swept policy runs here.
///
/// **Two hours, and an hour was not enough.** A policy pays its first lap's setup
/// once and amortises it over the run, and the ambient sabotage surface
/// (`tower::sabotage`) costs a few minutes an hour — so at 3600 ticks a single
/// badly-timed swap moves the rate by more than the tolerance band.
///
/// **Length alone does not fix it, which this file used to claim it did.** The
/// note here said *"three seeds at this length were measured inside the band"*,
/// and that was not true of the tree it was written against: measured over seeds
/// 0, 3, 11 and 42, clarity spans 0.1244 to 0.1383 against a pinned 0.140 — a
/// seed-to-seed spread wider than the 10% band, so seed 3 fails and seed 0
/// passes on identical code. Sabotage is *part of the economy*, so that spread
/// is the game rather than noise; what is wrong is measuring it once.
///
/// [`SEEDS`] is the fix: the pin is held against the **mean** of several worlds.
const SPAN: u64 = 7200;

/// The worlds a pinned rate is averaged over.
///
/// Four, and chosen to include an unlucky one: seed 3 draws roughly twice the
/// sabotage of seed 0 over this span, and a pin that quietly excluded it would be
/// pinning a fair-weather economy. Averaging keeps the test sensitive to a real
/// regression — which moves every seed — while surviving the luck of any one.
const SEEDS: [u64; 4] = [0, 3, 11, 42];

/// The sampling interval. Coarse: nothing here reads the intermediate rows.
const EVERY: u64 = 600;

/// CLAUDE.md's worked clarity, verbatim in effect.
///
/// `meditate` is how a dump waits, and the numbers are one tick past each stage's
/// duration — 8-tick grind, 12-tick digest, 10-tick mix, 56-tick distil — which
/// is the same convention every See-it line in the project uses.
const BY_HAND: &[&str] = &[
    "attend laboratory",
    "kindle charcoal",
    "grind sage",
    "meditate 9",
    "empty mortar_and_pestle",
    "digest ground-sage",
    "meditate 14",
    // **`siphon` used to be here and is not a word any more** (§19): the
    // pipeline reaches into an unbusy instrument before the shelf, so `mix`
    // takes the tincture out of the bath itself. Left in, it was a fuzzy miss
    // costing the reference a tick and quietly widening the very anchor this
    // file exists to hold — a dead command inside the number everything else is
    // compared against.
    "grind rock-salt",
    "meditate 9",
    "empty mortar_and_pestle",
    "mix sage-tincture with ground-salt",
    "meditate 12",
    "distil clarified-draught",
    "meditate 60",
];

#[test]
fn one_clarity_by_hand_earns_exactly_sixteen() {
    // **The anchor 16 is derived from, not chosen against a clock** (§19):
    // grind 1, digest 2, grind 1, mix 4, distil 8. If this ever reads anything
    // else, `progression.toml` moved and §11.5's first threshold moved with it.
    let mut sim = Sim::new(0);
    for line in BY_HAND {
        sim.submit(line);
        sim.step();
    }

    assert_eq!(
        sim.experience(),
        16,
        "one clarity by hand no longer earns the threshold it defines",
    );
    assert_eq!(
        sim.concentration(),
        1,
        "16 no longer buys the first concentration slot",
    );
}

#[test]
fn a_hand_played_clarity_costs_more_ticks_than_its_recipes_do() {
    // The finding that moved this crate's reference table off DESIGN.md's
    // numbers. The recipes total 94 ticks; a session that actually types the
    // commands pays queue latency on every one of them and lands near 123.
    //
    // Asserted as a *range* rather than a value, because the exact figure is a
    // property of the drive model and not of the design: what matters is that
    // the gap is real and one-directional.
    let mut sim = Sim::new(0);
    for line in BY_HAND {
        sim.submit(line);
        sim.step();
    }

    let recipes = 8 + 12 + 8 + 10 + 56;
    let played = sim.tick().get();
    assert!(
        played > recipes,
        "a played clarity ({played}) cost no more than its recipes ({recipes}) — \
         the queue latency this crate's reference table accounts for has gone",
    );
    assert!(
        played < recipes * 2,
        "a played clarity ({played}) cost more than twice its recipes ({recipes}) — \
         something is waiting that did not used to",
    );
}

#[test]
fn a_swept_clarity_and_a_hand_played_one_agree() {
    // **The See-it line this file is named for, and neither test above held it.**
    // Both drove `Sim` by hand and would have passed with the whole harness
    // deleted — so the one claim a balance harness cannot be trusted without was
    // the one claim nothing checked.
    //
    // The two numbers are not equal and must not be asserted equal: a hand-played
    // clarity pays its setup once over 16 experience (0.130), where the looped
    // policy amortises the same setup over dozens of laps (0.140). What has to
    // hold is that they measure the *same loop* — within a fifth of each other,
    // and the policy never behind the single brew that has no laps to amortise.
    let mut sim = Sim::new(0);
    for line in BY_HAND {
        sim.submit(line);
        sim.step();
    }
    #[allow(clippy::cast_precision_loss)]
    let by_hand = sim.experience() as f64 / sim.tick().get() as f64;

    let policy = Policy::named("clarity").expect("the flagship policy exists");
    let swept = drive::run(policy, 0, SPAN, EVERY).rate();

    assert!(
        swept >= by_hand,
        "the looped policy ({swept:.4}) earns less per tick than one brew by hand \
         ({by_hand:.4}) — a loop that amortises its setup cannot be the slower of \
         the two, so the policy has fallen out of phase with the tower",
    );
    assert!(
        (swept - by_hand).abs() < by_hand * 0.20,
        "a swept clarity ({swept:.4}) and a hand-played one ({by_hand:.4}) no \
         longer agree — one of the two has drifted, and every sweep after this is \
         measuring a game nobody plays",
    );
}

#[test]
fn every_pinned_rate_is_one_a_sweep_still_reaches() {
    // **The pin, made load-bearing.** `report::EXPECTED` is documented as a
    // regression pin, but nothing failed when a measurement left its band — the
    // `<-- drifted` marker is a pin only for as long as somebody is reading the
    // column. This reads it.
    //
    // It caught the first thing it was pointed at: the ambient reagent swap
    // shipped taking the alphabetically-first endless pile, which is `charcoal`,
    // so every session lost its fire and clarity fell from 0.140 to 0.074. Four
    // policies, all four flagged, and the suite was green.
    for policy in Policy::ALL {
        let Some(want) = report::expected(policy.name) else {
            // `stacks` is deliberately unpinned: a maze is generated per seed, so
            // one run is one sample. `report::EXPECTED` says so.
            continue;
        };
        // **Averaged over worlds, not measured in one.** A single hardcoded seed
        // made this test's sensitivity worse than the spread it was ignoring:
        // seed 3 leaves clarity, damped and grind all outside the band while
        // seed 0 passes, so any real regression smaller than that gap was
        // invisible and any seed change was a false alarm.
        let runs: Vec<_> = SEEDS
            .iter()
            .map(|seed| drive::run(policy, *seed, SPAN, EVERY))
            .collect();
        #[allow(clippy::cast_precision_loss)]
        let mean = runs.iter().map(drive::Run::rate).sum::<f64>() / runs.len() as f64;
        let spread: Vec<String> = runs
            .iter()
            .zip(SEEDS)
            .map(|(run, seed)| format!("{seed}:{:.4}", run.rate()))
            .collect();
        let cost: usize = runs.iter().map(|run| run.cost).sum();
        assert!(
            !report::off_by(mean, want),
            "{} averaged {mean:.4} over {} worlds against its pinned {want:.4} — \
             either the game changed or the policy has. Per seed: {}. {cost} \
             commands cost something across all of them",
            policy.name,
            SEEDS.len(),
            spread.join(", "),
        );
    }
}
