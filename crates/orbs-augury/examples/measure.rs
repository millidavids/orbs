//! The trained reader, on the same holdout the parser and the grammar are
//! measured against.
//!
//! ```text
//! cargo run --release -p orbs-augury --example measure --features train
//! ```
//!
//! **The fourth line of `orbs-sim`'s `--bench`, and it has to live here.**
//! `orbs-sim` must never depend on this crate (CLAUDE.md rule 1), so the bench
//! that measures the matcher cannot reach the model. The scoring rule is
//! deliberately the same: a reading counts when the command it produces
//! *resolves to the canonical form the corpus meant*, which is what
//! `Sim::submit_reading` would do with it.

use burn::backend::NdArray;
use orbs_augury::Trained;
use orbs_sim::augur::{Augur as _, Grammar};
use orbs_sim::content::{Phrasings, corpus_scene};
use orbs_sim::parser::{Mode, Resolution, Scene, resolve};

fn main() {
    let scene = corpus_scene();
    let phrasings = Phrasings::builtin();
    let holdout = phrasings.holdout(&scene);

    println!("\nO.R.B.S. — what the trained reader adds\n");
    println!("  {} phrasings, taught to nothing\n", holdout.len());

    let Ok(reader) = Trained::<NdArray<f32>>::load(Default::default()) else {
        println!(
            "  no weights yet — `cargo run --release -p orbs-augury --example train --features train`\n"
        );
        return;
    };
    let grammar = Grammar::builtin();

    let mut parser_read = 0usize;
    let mut model_read = 0usize;
    let mut together = 0usize;
    let mut refused = 0usize;
    let mut wrong: Vec<String> = Vec::new();

    for example in &holdout {
        let by_parser = reaches(&example.said, &example.canonical, &scene);

        let read = reader.read(&example.said);
        if read.is_empty() {
            refused += 1;
        }
        let by_model = read
            .iter()
            .any(|echo| reaches(echo, &example.canonical, &scene));

        let by_grammar = grammar
            .read(&example.said)
            .iter()
            .any(|echo| reaches(echo, &example.canonical, &scene));

        parser_read += usize::from(by_parser);
        model_read += usize::from(by_model);
        together += usize::from(by_parser || by_model || by_grammar);

        if !by_model
            && wrong.len() < 8
            && let Some(echo) = read.first()
        {
            wrong.push(format!(
                "{:?} -> {echo}   (wanted {})",
                example.said, example.canonical
            ));
        }
    }

    let total = holdout.len().max(1);
    println!(
        "    today's parser      {:>6.1}%",
        percent(parser_read, total)
    );
    println!(
        "    the trained reader  {:>6.1}%",
        percent(model_read, total)
    );
    println!(
        "    all three together  {:>6.1}%   <- what shipping it buys",
        percent(together, total),
    );
    // **Two populations, scored opposite ways round.** On commands a refusal is
    // a miss; on sentences that ask for nothing it is the right answer. Reading
    // one number for both is how *"it refused 0.0%"* got reported as a failure
    // when, on the command holdout, nought is exactly right.
    let refusals = phrasings.refused_holdout(&scene);
    let correctly_refused = refusals
        .iter()
        .filter(|line| reader.read(line).is_empty())
        .count();

    println!(
        "\n  and on {} sentences that ask for nothing:\n",
        refusals.len()
    );
    println!(
        "    correctly refused   {:>6.1}%   <- the reject class",
        percent(correctly_refused, refusals.len().max(1)),
    );
    println!(
        "    wrongly refused     {:>6.1}%   <- commands it would not answer\n",
        percent(refused, total),
    );

    // **The number that decides whether a worker thread is needed at all.** §6
    // requires the echo be immediate — *"a terminal that takes a second to
    // answer reads as broken"* — and the plan assumed a reader slow enough to
    // need a thread, a channel and a deadline. One short sentence on the CPU
    // backend may simply not be.
    let sentences: Vec<&str> = holdout.iter().take(200).map(|e| e.said.as_str()).collect();
    let started = std::time::Instant::now();
    for line in &sentences {
        let _ = reader.read(line);
    }
    let each = started.elapsed() / u32::try_from(sentences.len()).unwrap_or(1);
    println!("  one line takes {each:?} on the cpu backend");

    // ...and the same on the GPU, because the plan assumed that was the faster
    // place to read and it is worth knowing whether it is. A batch of one over
    // 32 tokens is a very different workload from a training epoch.
    if let Ok(gpu) = Trained::<burn::backend::Wgpu>::load(Default::default()) {
        for line in sentences.iter().take(20) {
            let _ = gpu.read(line); // warm the shaders before timing them
        }
        let started = std::time::Instant::now();
        for line in &sentences {
            let _ = gpu.read(line);
        }
        let each = started.elapsed() / u32::try_from(sentences.len()).unwrap_or(1);
        println!("  one line takes {each:?} on the gpu backend\n");
    } else {
        println!();
    }

    println!("  where it goes wrong:\n");
    for line in &wrong {
        println!("    {line}");
    }
    println!();
}

/// Whether `said` resolves to the command `meant` names — the bench's rule.
fn reaches(said: &str, meant: &str, scene: &Scene) -> bool {
    matches!(
        resolve(said, scene, Mode::Calm),
        Resolution::Resolved { ref intent, .. } if intent.echo() == meant
    )
}

#[expect(
    clippy::cast_precision_loss,
    reason = "a holdout is thousands of lines, not 2^24"
)]
fn percent(part: usize, whole: usize) -> f32 {
    if whole == 0 {
        return 0.0;
    }
    part as f32 / whole as f32 * 100.0
}
