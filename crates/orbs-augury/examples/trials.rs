//! The scrivener against lines it was never taught.
//!
//! ```text
//! cargo run --release -p orbs-augury --example trials
//! cargo run --release -p orbs-augury --example trials -- --scribe target/seeds/try/scribe-7 --scores
//! ```
//!
//! A report over `content/spell_trials.toml`, lines a test holds untaught: by
//! shape and style, every miss with why, and whole scripts through a real
//! tower. Inference only. Flags as in `measure`.

use burn::backend::NdArray;
use orbs_augury::Scribe;
use orbs_augury::cli::{percent, scribe};
use orbs_sim::content::{TRIAL_SHAPES, TRIAL_STYLES, Trial, Trials, fold, with_extension};
use orbs_sim::{Sim, tower};

fn main() {
    let scores = std::env::args().any(|arg| arg == "--scores");
    let trials = Trials::builtin();
    if !scores {
        println!("\nO.R.B.S. — the scrivener on trial\n");
    }
    let Some(scribe) = scribe(scores) else {
        return;
    };

    let read: Vec<(&Trial, Option<String>)> = trials
        .lines()
        .iter()
        .map(|trial| (trial, scribe.reading(&trial.said)))
        .collect();
    if !scores {
        println!("  {} lines, taught to nothing\n", read.len());
        println!("  by shape\n");
    }
    for shape in TRIAL_SHAPES {
        let (passed, total) = tally(&read, |trial| trial.shape() == Some(shape));
        if scores {
            println!("{shape}\t{:.1}", percent(passed, total));
        } else {
            println!(
                "    {:>6.1}%   {shape:<14}{passed:>3} of {total}",
                percent(passed, total)
            );
        }
    }
    let (passed, total) = tally(&read, |_| true);
    if scores {
        println!("every line\t{:.1}", percent(passed, total));
    } else {
        println!(
            "\n    {:>6.1}%   every line    {passed:>4} of {total}",
            percent(passed, total)
        );
        // By style as well as by shape: a style is what a player has, a shape
        // what the orb has. *"Contractions read 40%"* is a finding;
        // *"`if` reads 70%"* hides it.
        println!("\n  by style\n");
    }
    for style in TRIAL_STYLES {
        let (passed, total) = tally(&read, |trial| trial.tags.iter().any(|tag| tag == style));
        if total == 0 {
            continue;
        }
        if scores {
            println!("style: {style}\t{:.1}", percent(passed, total));
        } else {
            println!(
                "    {:>6.1}%   {style:<14}{passed:>3} of {total}",
                percent(passed, total)
            );
        }
    }

    // Counted apart, and zero is the only acceptable count: a miss leaves a
    // spell that faults where the player can see it, a betrayal runs one that
    // means the opposite of what was written.
    let betrayed: Vec<&(&Trial, Option<String>)> = read
        .iter()
        .filter(|(trial, got)| trial.betrayed_by(got.as_deref()))
        .collect();
    if scores {
        println!("read as their opposite\t{}", betrayed.len());
        return;
    }
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
/// The path a player's save takes, not a loop over `reading`: the per-line
/// cache, the blank-and-comment rule and the byte-exact file all sit between
/// the editor and the program.
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
