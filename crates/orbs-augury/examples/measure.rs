//! The trained readers, on the same holdouts the parser and the grammar are
//! measured against.
//!
//! ```text
//! cargo run --release -p orbs-augury --example measure --features train
//! cargo run --release -p orbs-augury --example measure --features train -- --spells
//! cargo run --release -p orbs-augury --example measure --features train -- --reader target/seeds/try/reader-7 --scores
//! ```
//!
//! The fourth line of `orbs-sim`'s `--bench`, and it has to live here: rule 1
//! forbids `orbs-sim` depending on this crate, so the bench that measures the
//! matcher cannot reach the model. The scoring rule is deliberately the same — a
//! reading counts when the command it produces resolves to the canonical form
//! the corpus meant, which is what `Sim::submit_reading` would do.
//!
//! `--spells` measures the scrivener instead, stricter for §9999's reason — *"a
//! spell resolves with nobody watching"*. A statement counts only when the line
//! it produces *is* the canonical one; a command line inside a spell is scored
//! the prompt's way, because that is the reader it is handed to.
//!
//! `--reader` and `--scribe` measure weights other than the shipped ones, each a
//! path without its `.bin`, and `--scores` prints nothing but the numbers — one
//! `name<TAB>value` a line — for `scripts/seeds.sh` to average across seeds. The
//! spell register takes both, because its command lines are handed to a prompt
//! reader and two spell readers can only be compared through the same one.

use burn::backend::NdArray;
use orbs_augury::cli::{missing, weights};
use orbs_augury::trained::{SCRIBE_WEIGHTS, WEIGHTS};
use orbs_augury::{Register, Scribe, Trained};
use orbs_sim::augur::{Augur as _, Grammar, Scrivener as _};
use orbs_sim::content::{Phrasings, corpus_scene};
use orbs_sim::parser::{Mode, Resolution, Scene, resolve};

fn main() {
    let scores = std::env::args().any(|arg| arg == "--scores");
    match Register::asked() {
        Register::Verbs => prompt(scores),
        Register::Spells => spells(scores),
    }
}

/// What the scrivener makes of lines nothing taught it.
fn spells(scores: bool) {
    let scene = corpus_scene();
    let spellings = Phrasings::spellings();
    let statements = spellings.holdout_by_entry(&scene);
    let shapes = orbs_augury::shapes();

    if !scores {
        println!("\nO.R.B.S. — what the scrivener adds\n");
        println!(
            "  {} statements over {} shapes, taught to nothing\n",
            statements.len(),
            shapes.len(),
        );
    }

    let Ok(scribe) = Scribe::<NdArray<f32>>::load_from(
        &weights("--scribe", SCRIBE_WEIGHTS),
        &weights("--reader", WEIGHTS),
        Default::default(),
    ) else {
        missing(
            scores,
            "spell weights",
            "cargo run --release -p orbs-augury --example train --features train -- --spells",
        );
        return;
    };

    // Per shape, because one number hides the shapes with nothing to expand
    // over: `end` and `else` are twenty authored sentences apiece against `if
    // {place} has {reagent}`'s sixteen thousand.
    let mut right = vec![0usize; shapes.len()];
    let mut seen = vec![0usize; shapes.len()];
    // One misreading per shape, so the list is a spread rather than eight
    // expansions of whichever shape happens to be widest.
    let mut wrong: Vec<Option<String>> = vec![None; shapes.len()];
    for (class, example) in &statements {
        let read = scribe.read(&example.said);
        seen[*class] += 1;
        if read.as_deref() == Some(example.canonical.as_str()) {
            right[*class] += 1;
        } else if wrong[*class].is_none() {
            wrong[*class] = Some(format!(
                "{:?} -> {}   (wanted {})",
                example.said,
                read.as_deref().unwrap_or("<left as written>"),
                example.canonical,
            ));
        }
    }

    for (at, shape) in shapes.iter().enumerate() {
        let rate = percent(right[at], seen[at].max(1));
        if scores {
            println!("{shape}\t{rate:.1}");
        } else {
            println!("    {rate:>6.1}%   {shape}   ({} held back)", seen[at]);
        }
    }
    // The mean over shapes, and the one to read. A holdout expands over the noun
    // tables, so `if {place} has {reagent}` is 86% of these lines — the same
    // arithmetic that made `move` 47% of the prompt's corpus.
    let each = mean(
        &(0..shapes.len())
            .map(|at| percent(right[at], seen[at].max(1)))
            .collect::<Vec<f32>>(),
    );
    let every = percent(right.iter().sum(), statements.len().max(1));

    // A command line in a spell is the prompt reader's, so it is scored the
    // prompt's way: does what comes back *resolve* to what the corpus meant.
    let commands = Phrasings::builtin().holdout(&scene);
    let sampled: Vec<_> = commands.iter().step_by(7).collect();
    let read = sampled
        .iter()
        .filter(|example| {
            scribe
                .read(&example.said)
                .is_some_and(|line| reaches(&line, &example.canonical, &scene))
        })
        .count();
    let commanded = percent(read, sampled.len().max(1));

    if scores {
        println!("the average shape\t{each:.1}");
        println!("every line\t{every:.1}");
        println!("command lines\t{commanded:.1}");
    } else {
        println!("\n    {each:>6.1}%   the average shape   <- the number to read");
        println!("    {every:>6.1}%   every line, which the widest shape dominates");
        println!(
            "    {commanded:>6.1}%   command lines, on {} of the prompt's own holdout",
            sampled.len(),
        );
        // ...and the half that matters more here than at the prompt. A spell
        // runs unattended, so a line left alone is recoverable and a line read
        // wrongly is not.
        println!("\n  and on the lines that must come back untouched:\n");
    }

    // Two populations again, failing differently: an already-canonical statement
    // is caught by `reads_cleanly` before the model is consulted, where a
    // sentence asking for nothing reaches the model and is the refusal head's.
    let untouched = |what: &str, lines: &[String]| {
        let left = lines
            .iter()
            .filter(|line| scribe.read(line).is_none())
            .count();
        let rate = percent(left, lines.len().max(1));
        if scores {
            println!("untouched, {what}\t{rate:.1}");
            return;
        }
        println!("    {rate:>6.1}%   {what} ({} lines)", lines.len());
        for line in lines
            .iter()
            .filter(|line| scribe.read(line).is_some())
            .take(3)
        {
            println!("             rewrote {line:?} -> {:?}", scribe.read(line));
        }
    };
    let mut canonical = spellings.refused(&scene);
    canonical.extend(spellings.refused_holdout(&scene));
    untouched("already a statement", &canonical);
    let mut nothing = Phrasings::builtin().refused_holdout(&scene);
    nothing.extend(Phrasings::builtin().refused(&scene));
    untouched("asking for nothing at all", &nothing);

    if scores {
        return;
    }

    let started = std::time::Instant::now();
    for (_, example) in statements.iter().take(200) {
        let _ = scribe.read(&example.said);
    }
    let each = started.elapsed() / u32::try_from(statements.len().clamp(1, 200)).unwrap_or(1);
    println!("\n  one line takes {each:?} on the cpu backend\n");

    println!("  where it goes wrong, one shape at a time:\n");
    for line in wrong.iter().flatten() {
        println!("    {line}");
    }
    println!();
}

/// What the prompt's reader makes of lines nothing taught it.
fn prompt(scores: bool) {
    let scene = corpus_scene();
    let phrasings = Phrasings::builtin();
    let holdout = phrasings.holdout(&scene);

    if !scores {
        println!("\nO.R.B.S. — what the trained reader adds\n");
        println!("  {} phrasings, taught to nothing\n", holdout.len());
    }

    let reading = weights("--reader", WEIGHTS);
    let Ok(reader) = Trained::<NdArray<f32>>::load_from(&reading, Default::default()) else {
        missing(
            scores,
            "weights",
            "cargo run --release -p orbs-augury --example train --features train",
        );
        return;
    };
    let grammar = Grammar::builtin();

    let mut parser_read = 0usize;
    let mut model_read = 0usize;
    let mut model_ran = 0usize;
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
        // And the one the orb would actually run — the reading
        // `Sim::submit_reading` and the bench both take. The line above counts a
        // hit when any of four readings reaches the command, which `orbs-sim`'s
        // bench calls flattering. Both are printed: the looser one is what every
        // number in §19 before `0.14.11` was measured with.
        model_ran += usize::from(
            orbs_sim::parser::reading_to_run(&read, &scene, Mode::Calm)
                .is_some_and(|line| reaches(line, &example.canonical, &scene)),
        );

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
    // Two populations, scored opposite ways round: on commands a refusal is a
    // miss, on sentences that ask for nothing it is the right answer. One number
    // for both is how *"it refused 0.0%"* got reported as a failure.
    let refusals = phrasings.refused_holdout(&scene);
    let correctly_refused = refusals
        .iter()
        .filter(|line| reader.read(line).is_empty())
        .count();

    if scores {
        println!("the trained reader\t{:.1}", percent(model_read, total));
        println!("the reading it runs\t{:.1}", percent(model_ran, total));
        println!("all three together\t{:.1}", percent(together, total));
        println!(
            "correctly refused\t{:.1}",
            percent(correctly_refused, refusals.len().max(1))
        );
        println!("wrongly refused\t{:.1}", percent(refused, total));
        return;
    }

    println!(
        "    today's parser      {:>6.1}%",
        percent(parser_read, total)
    );
    println!(
        "    the trained reader  {:>6.1}%   <- any of its readings reaches it",
        percent(model_read, total)
    );
    println!(
        "    the reading it runs {:>6.1}%   <- the one `submit_reading` would take",
        percent(model_ran, total)
    );
    println!(
        "    all three together  {:>6.1}%   <- what shipping it buys",
        percent(together, total),
    );
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

    // The number that decides whether a worker thread is needed at all. §6 wants
    // the echo immediate — *"a terminal that takes a second to answer reads as
    // broken"* — and the plan assumed a reader slow enough to need a thread, a
    // channel and a deadline. One short sentence on the CPU backend may not be.
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
    if let Ok(gpu) = Trained::<burn::backend::Wgpu>::load_from(&reading, Default::default()) {
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

/// The average of some percentages, weighting each equally.
#[expect(clippy::cast_precision_loss, reason = "eleven shapes, not 2^24")]
fn mean(of: &[f32]) -> f32 {
    if of.is_empty() {
        return 0.0;
    }
    of.iter().sum::<f32>() / of.len() as f32
}
