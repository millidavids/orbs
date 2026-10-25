//! The See-it line, as a test: *a sweep's curve and a hand-played session agree*.
//!
//! Without it every sweep may measure a game nobody plays (§13). The first two
//! tests hold the anchor, a hand-played clarity's 16; the last two run real
//! [`Policy`]s through [`drive::run`] against it and [`report::expected`].

use orbs_balance::{drive, policy::Policy, report};
use orbs_sim::Sim;

/// How long a swept policy runs here.
///
/// Two hours, so one swap cannot move a rate past the band. The seed spread is
/// the game (0.1244 to 0.1383 for clarity), so the pin holds the mean over
/// [`SEEDS`].
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

/// The curve every pinned rate below was measured against.
///
/// Baseline, and named rather than passed anonymously: the numbers in
/// `report::EXPECTED` were taken against the authored curve, so a test sweeping
/// a different one compares a measurement to a pin that never described it.
/// `bound` would move most, since it earns its first slot by hand and a longer
/// game is a longer hand-played prefix inside the same budget.
const MEASURED: orbs_sim::content::Length = orbs_sim::content::Length::Baseline;

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
    // `siphon` used to be here and is not a word any more (§19): the pipeline
    // reaches into an unbusy instrument before the shelf, so `mix` takes the
    // tincture out of the bath. Left in it was a fuzzy miss costing the
    // reference a tick — a dead command inside the anchor everything else is
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
    // The anchor 16 is derived from, not chosen against a clock (§19): grind 1,
    // digest 2, grind 1, mix 4, distil 8. Anything else means
    // `progression.toml` moved and §11.5's first threshold with it.
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
    // Not equal — one brew pays its setup once (0.130), the loop amortises it
    // (0.140) — but the same loop: within a fifth, the policy never behind.
    let mut sim = Sim::new(0);
    for line in BY_HAND {
        sim.submit(line);
        sim.step();
    }
    #[allow(clippy::cast_precision_loss)]
    let by_hand = sim.experience() as f64 / sim.tick().get() as f64;

    let policy = Policy::named("clarity").expect("the flagship policy exists");
    let swept = drive::run(policy, 0, SPAN, EVERY, MEASURED).rate();

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

/// What a bound spell keeps of the same loop played by hand.
///
/// One tick in eleven, the same on every world. `grind` and `bound` issue the
/// same two commands for ever, so everything about them is equal except who is
/// typing, and the gap is the tick a binding spends re-casting a spell that ran
/// off the end. Over seeds 0, 3, 11 and 42 the ratio is 0.910, 0.911, 0.910,
/// 0.910 — the absolute rates move with a world's luck and this does not,
/// because it is a property of the script engine rather than of the tower.
const AUTOMATION_KEEPS: f64 = 0.910;

#[test]
fn a_bound_spell_keeps_a_known_fraction_of_the_loop_it_automates() {
    // §19's *a step still costs a tick* is a claim about this number. The
    // quotient, since the rate follows a world's luck; a language change that
    // moves it should report here.
    for seed in SEEDS {
        let grind = drive::run(
            Policy::named("grind").expect("the rate floor exists"),
            seed,
            SPAN,
            EVERY,
            MEASURED,
        )
        .rate();
        let bound = drive::run(
            Policy::named("bound").expect("the bound-spell policy exists"),
            seed,
            SPAN,
            EVERY,
            MEASURED,
        )
        .rate();

        assert!(
            bound < grind,
            "seed {seed}: a bound spell ({bound:.4}) out-earned the same loop \
             played by hand ({grind:.4}) — §8 charges a tick per step, so \
             automation cannot be the faster of the two",
        );
        let kept = bound / grind;
        assert!(
            (kept - AUTOMATION_KEEPS).abs() < 0.02,
            "seed {seed}: a bound spell kept {kept:.3} of the hand-played loop \
             against a pinned {AUTOMATION_KEEPS:.3} — the script engine's \
             overhead moved ({bound:.4} against {grind:.4})",
        );
    }
}

#[test]
fn the_bound_policy_actually_gets_a_spell_bound() {
    // A policy that silently never binds would still report a rate, and it would
    // be the hand-played one — the failure mode this instrument exists to avoid,
    // arriving inside the instrument. It happened once: `Sim::bound` answers
    // `tending.spell` where the policy names `tending`, so the driver re-issued
    // `bind` every free tick and a two-hour sweep carried 1,920 refusals.
    let run = drive::run(
        Policy::named("bound").expect("the bound-spell policy exists"),
        0,
        SPAN,
        EVERY,
        MEASURED,
    );

    assert!(
        !run.reasons
            .keys()
            .any(|why| why.contains("already holding")),
        "the driver kept asking the orb to bind a spell it was already holding: \
         {:?}",
        run.reasons,
    );
    assert!(
        run.reasons.keys().any(|why| why.contains("waits")),
        "nothing in the run waited on an instrument, so no spell was running: \
         {:?}",
        run.reasons,
    );
}

#[test]
fn every_pinned_rate_is_one_a_sweep_still_reaches() {
    // `report::EXPECTED` made a failing pin, not a column somebody must read.
    // Its first catch was the charcoal swap.
    for policy in Policy::ALL {
        let Some(want) = report::expected(policy.name) else {
            // `stacks` is deliberately unpinned: a maze is generated per seed, so
            // one run is one sample. `report::EXPECTED` says so.
            continue;
        };
        // Averaged over worlds, not measured in one: a single hardcoded seed
        // made this test's sensitivity worse than the spread it ignored — seed 3
        // leaves clarity, damped and grind outside the band while seed 0 passes,
        // so any regression smaller than that gap was invisible.
        let runs: Vec<_> = SEEDS
            .iter()
            .map(|seed| drive::run(policy, *seed, SPAN, EVERY, MEASURED))
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
            !report::off_by(
                mean,
                want,
                report::tolerance_of_mean(policy.name, SEEDS.len())
            ),
            "{} averaged {mean:.4} over {} worlds against its pinned {want:.4} — \
             either the game changed or the policy has. Per seed: {}. {cost} \
             commands cost something across all of them",
            policy.name,
            SEEDS.len(),
            spread.join(", "),
        );
    }
}

/// The scrolls' question, as a number: a quickening scroll is worth using, so
/// what it buys the laboratory beats what the archive spends making it (§19,
/// *Supply*).
///
/// The archive's cost to the laboratory is the lectern's run alone: a walk of
/// the stacks takes no slot. Priced in the flagship's own experience, so the
/// two sides are one unit. Averaged over [`SEEDS`], as every pin here is.
#[test]
fn quickening_pays_more_than_the_archive_spends_on_it() {
    const SCROLL: &str = "quickening-scroll";
    let clarity = Policy::named("clarity").expect("the flagship ships");
    let quickened = Policy::named("quickened").expect("the quickened policy ships");
    let recipes = orbs_sim::content::Recipes::builtin();
    let lectern = recipes
        .for_instrument("lectern")
        .first()
        .expect("the lectern has a recipe");
    #[allow(clippy::cast_precision_loss)]
    let ticks_a_scroll = lectern.ticks as f64 / f64::from(lectern.yields);

    let mut gained = 0.0;
    let mut spent = 0.0;
    for seed in SEEDS {
        let plain = drive::run(clarity, seed, SPAN, EVERY, MEASURED);
        let quick = drive::run(quickened, seed, SPAN, EVERY, MEASURED);
        let given = quick.given.get(SCROLL).copied().unwrap_or(0);
        let used = given - quick.held.get(SCROLL).copied().unwrap_or(0);
        assert!(used > 0, "seed {seed}: no scroll was used");
        #[allow(clippy::cast_precision_loss)]
        let more = quick.last().experience as f64 - plain.last().experience as f64;
        gained += more / f64::from(used);
        spent += ticks_a_scroll * plain.rate();
    }
    assert!(
        gained > spent,
        "a quickening scroll buys {gained:.2} experience over {} worlds and costs \
         {spent:.2} to make",
        SEEDS.len(),
    );
}
