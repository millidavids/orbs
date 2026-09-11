//! The scrivener against lines it was never taught.
//!
//! ```text
//! cargo run --release -p orbs-augury --example trials
//! ```
//!
//! `content/spell_trials.toml` holds the lines — literal, written to be unlike
//! the corpus, and held by a test to be nothing the reader was taught. This is
//! the report over them: by shape and by style, the betrayals apart, every miss
//! with *why*, and the whole scripts written through a real tower.
//!
//! **Inference only**, so it needs no GPU and no `train` feature.

use burn::backend::NdArray;
use orbs_augury::Scribe;
use orbs_sim::content::{TRIAL_SHAPES, TRIAL_STYLES, Trial, Trials, fold, with_extension};
use orbs_sim::{Sim, tower};

fn main() {
    let trials = Trials::builtin();
    println!("\nO.R.B.S. — the scrivener on trial\n");
    let Ok(scribe) = Scribe::<NdArray<f32>>::load(Default::default()) else {
        println!(
            "  no spell weights yet — `cargo run --release -p orbs-augury --example train --features train -- --spells`\n"
        );
        return;
    };

    let read: Vec<(&Trial, Option<String>)> = trials
        .lines()
        .iter()
        .map(|trial| (trial, scribe.reading(&trial.said)))
        .collect();
    println!("  {} lines, taught to nothing\n", read.len());

    println!("  by shape\n");
    for shape in TRIAL_SHAPES {
        let (passed, total) = tally(&read, |trial| trial.shape() == Some(shape));
        println!(
            "    {:>6.1}%   {shape:<14}{passed:>3} of {total}",
            percent(passed, total)
        );
    }
    let (passed, total) = tally(&read, |_| true);
    println!(
        "\n    {:>6.1}%   every line    {passed:>4} of {total}",
        percent(passed, total)
    );

    // **By style as well as by shape**, because a style is what a player has
    // and a shape is what the orb has. *"Contractions read 40%"* is a finding;
    // *"`if` reads 70%"* hides it.
    println!("\n  by style\n");
    for style in TRIAL_STYLES {
        let (passed, total) = tally(&read, |trial| trial.tags.iter().any(|tag| tag == style));
        if total > 0 {
            println!(
                "    {:>6.1}%   {style:<14}{passed:>3} of {total}",
                percent(passed, total)
            );
        }
    }

    // **Apart, and zero is the only acceptable count.** A miss leaves a spell
    // that faults where the player can see it; a betrayal runs one that means
    // the opposite of what was written, with nobody watching.
    let betrayed: Vec<&(&Trial, Option<String>)> = read
        .iter()
        .filter(|(trial, got)| trial.betrayed_by(got.as_deref()))
        .collect();
    println!(
        "\n  read as their opposite: {}   <- zero is the only acceptable count",
        betrayed.len()
    );
    for (trial, got) in &betrayed {
        println!(
            "    {:?} -> {}",
            trial.said,
            got.as_deref().unwrap_or_default()
        );
    }

    println!("\n  where it goes wrong, and why\n");
    for shape in TRIAL_SHAPES {
        for (trial, got) in read
            .iter()
            .filter(|(trial, got)| trial.shape() == Some(shape) && !trial.passes(got.as_deref()))
            .take(4)
        {
            println!(
                "    {:?} -> {}   (wanted {})",
                trial.said,
                got.as_deref().unwrap_or("<left as written>"),
                trial.reads.as_deref().unwrap_or("<left as written>"),
            );
            let why = scribe.consider(&trial.said);
            match why.gated {
                Some(gate) => println!("        gated: {gate}"),
                None => println!(
                    "        answering {} · top {:?} · slots {:?}",
                    why.answering, why.top, why.placed
                ),
            }
        }
    }

    scripts(&trials, &scribe);
    println!();
}

/// Every script written through a real tower's `write_spell_reading`, compiled
/// from what it stored, and checked against the room.
///
/// **The path a player's save takes**, not a loop over `reading`: the per-line
/// cache, the blank-and-comment rule and the byte-exact file all sit between
/// the editor and the program, and a report that skipped them would be
/// measuring something no player runs.
fn scripts(trials: &Trials, scribe: &Scribe<NdArray<f32>>) {
    println!("\n  whole spells, written through a tower\n");
    for script in trials.scripts() {
        let mut sim = Sim::new(3);
        let slug: String = script
            .name
            .chars()
            .map(|ch| if ch.is_alphanumeric() { ch } else { '_' })
            .collect();
        sim.write_spell_reading(&slug, &script.loose, scribe);
        sim.step();

        let filename = with_extension(&slug);
        let Some(node) = sim
            .world()
            .iter_entities()
            .find(|entity| {
                entity
                    .get::<tower::Name>()
                    .is_some_and(|name| name.0 == filename)
            })
            .map(|entity| entity.id())
        else {
            println!("    {:?}: the spell was never written", script.name);
            continue;
        };
        let compiled = tower::spell::source(sim.world(), node);
        let right = compiled
            .iter()
            .zip(&script.reads)
            .filter(|(got, wanted)| fold(got) == fold(wanted))
            .count();
        let faults = sim
            .read_spell(&script.domain, &compiled)
            .iter()
            .filter(|line| line.fault.is_some())
            .count();
        println!(
            "    {right:>2} of {:>2} lines   {}   {}",
            script.reads.len(),
            if faults == 0 { "compiles" } else { "faults  " },
            script.name,
        );
        for ((loose, got), wanted) in script.loose.iter().zip(&compiled).zip(&script.reads) {
            if fold(got) != fold(wanted) {
                println!("        {loose:?} -> {got:?}   (wanted {wanted:?})");
            }
        }
    }
}

/// How many of the chosen trials passed, and how many were chosen.
fn tally(read: &[(&Trial, Option<String>)], which: impl Fn(&Trial) -> bool) -> (usize, usize) {
    let chosen: Vec<&(&Trial, Option<String>)> =
        read.iter().filter(|(trial, _)| which(trial)).collect();
    let passed = chosen
        .iter()
        .filter(|(trial, got)| trial.passes(got.as_deref()))
        .count();
    (passed, chosen.len())
}

#[expect(
    clippy::cast_precision_loss,
    reason = "a suite is hundreds of lines, not 2^24"
)]
fn percent(part: usize, whole: usize) -> f32 {
    if whole == 0 {
        return 0.0;
    }
    part as f32 / whole as f32 * 100.0
}
