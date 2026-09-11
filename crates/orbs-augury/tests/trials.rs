//! No trial is ever read as its opposite.
//!
//! `content/spell_trials.toml` marks, for its hazard lines, the readings that
//! would run a spell meaning the opposite of what was written — a dropped `not`,
//! a dropped `or` clause, a number that changed. A miss leaves a spell that
//! faults where the player can see it; one of these runs, unattended, doing the
//! wrong thing. So this is a hard failure rather than a figure in a report, and
//! it is the one claim about the trained reader the suite makes as a test.
//!
//! **Skipped when this checkout has no spell weights**, for the reason every
//! test of a trained reader is: weights are a gitignored build artefact, and
//! `cargo test --workspace` must pass on a machine that has never trained.

use burn::backend::NdArray;
use orbs_augury::Scribe;
use orbs_sim::content::Trials;

#[test]
fn no_trial_is_ever_read_as_its_opposite() {
    let Ok(scribe) = Scribe::<NdArray<f32>>::load(Default::default()) else {
        println!(
            "no spell weights; run `cargo run --release -p orbs-augury --example train --features train -- --spells`"
        );
        return;
    };
    let betrayed: Vec<String> = Trials::builtin()
        .lines()
        .iter()
        .filter_map(|trial| {
            let got = scribe.reading(&trial.said);
            trial
                .betrayed_by(got.as_deref())
                .then(|| format!("{:?} -> {got:?}", trial.said))
        })
        .collect();
    assert!(
        betrayed.is_empty(),
        "{} trials were read as their opposite: {betrayed:#?}",
        betrayed.len(),
    );
}
