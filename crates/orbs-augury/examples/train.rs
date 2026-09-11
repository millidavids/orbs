#![expect(
    clippy::cast_precision_loss,
    reason = "epoch and batch counts are small integers; these are report figures"
)]
//! Train a reader, on this machine's GPU, from this game's own content.
//!
//! ```text
//! cargo run --release -p orbs-augury --example train --features train
//! cargo run --release -p orbs-augury --example train --features train -- --spells
//! cargo run --release -p orbs-augury --example train --features train -- --epochs 40 --cpu
//! ```
//!
//! **An example rather than a test**, because it needs a GPU and minutes, and
//! `cargo test --workspace` must run on a machine with neither.
//!
//! # Two registers, one trainer
//!
//! `--spells` trains the reader for `.spell` files instead of the one for the
//! prompt. Everything below is shared — the loop, the weighting, the selection
//! rule, the scoring — because the two ask the identical question of an
//! identical sentence over a different set of answers. What differs is gathered
//! in [`Corpus`], and a second trainer would be the two-expressions-of-one-rule
//! defect §19 records more often than any other.
//!
//! # What it learns from
//!
//! `content/phrasings.toml` and `content/spellings.toml`, expanded over
//! everything the content tables name — and nothing else. No pretrained weights,
//! no pretrained embeddings, no downloaded tokenizer, no other model's outputs
//! (DESIGN.md §19, *the augury*). The vocabulary is assembled from the game's
//! own tables, so there is no artefact here whose provenance is anywhere but
//! this repository.
//!
//! # What the numbers mean
//!
//! **Holdout accuracy is the only one worth reading.** The corpus is what the
//! reader is shown; scoring on it says the optimiser worked, not that anything
//! was learned. `holdout` is taught to nothing, here as everywhere.
//!
//! Tagging is scored over **real words only** — a five-word sentence sits in a
//! thirty-two row buffer, and counting padding would report seven-eighths of a
//! score for predicting nothing.

use burn::backend::{Autodiff, NdArray, Wgpu};
use burn::module::AutodiffModule;
use burn::nn::loss::CrossEntropyLossConfig;
use burn::optim::{AdamConfig, GradientsParams, Optimizer};
use burn::prelude::*;
use burn::record::{BinFileRecorder, FullPrecisionSettings};

use orbs_augury::{Batch, Corpus, Reader, ReaderConfig, Register, Sample, Vocabulary};

/// How many sentences the reader sees at once.
const BATCH: usize = 64;

fn main() {
    let epochs = numbered("--epochs").unwrap_or(30);
    let cpu = std::env::args().any(|arg| arg == "--cpu");
    let register = Register::asked();

    println!(
        "\nO.R.B.S. — teaching the orb to read {}\n",
        register.name()
    );

    let vocabulary = Vocabulary::builtin();
    let corpus = register.corpus(&vocabulary);

    println!(
        "  {} rows, {} answers, {} to learn from ({} of them refusals), {} held back\n",
        vocabulary.rows(),
        register.classes(),
        corpus.learn.len(),
        corpus.refusals,
        corpus.holdout.len(),
    );

    if cpu {
        run::<Autodiff<NdArray<f32>>>(
            &burn::backend::ndarray::NdArrayDevice::default(),
            register,
            &corpus,
            &vocabulary,
            epochs,
        );
    } else {
        run::<Autodiff<Wgpu>>(
            &burn::backend::wgpu::WgpuDevice::default(),
            register,
            &corpus,
            &vocabulary,
            epochs,
        );
    }
}

/// The narrowest and widest a computed weight may be.
///
/// **Raw inverse frequency is unusable here.** The rarest verb has a handful of
/// examples against the commonest's thousands, so its unclamped weight is in the
/// hundreds and one such sentence in a batch dominates every gradient it touches
/// — the loss stops being about reading and starts being about that verb.
const WEIGHT: std::ops::RangeInclusive<f32> = 0.25..=4.0;

/// How much each class's mistakes count, by how rare the class is.
fn verb_weights(corpus: &[Sample], classes: usize) -> Vec<f32> {
    let mut count = vec![0usize; classes];
    for sample in corpus.iter().filter(|sample| sample.is_command()) {
        if let Some(seen) = count.get_mut(sample.verb as usize) {
            *seen += 1;
        }
    }

    // The mean is taken over verbs that *appear*, so a verb nobody has written a
    // template for cannot drag the average down and inflate everything else.
    let seen: Vec<usize> = count.iter().copied().filter(|n| *n > 0).collect();
    let mean = seen.iter().sum::<usize>() as f32 / seen.len().max(1) as f32;

    count
        .iter()
        .map(|n| match n {
            // Unseen: weight 1, which costs nothing because no example carries
            // this class. `every_verb_has_a_template` is what keeps it empty.
            0 => 1.0,
            n => (mean / *n as f32).clamp(*WEIGHT.start(), *WEIGHT.end()),
        })
        .collect()
}

/// How much a missed refusal counts, by how rare refusals are.
fn refusal_weight(corpus: &[Sample]) -> f32 {
    let refusals = corpus
        .iter()
        .filter(|sample| !sample.is_command())
        .count()
        .max(1);
    let commands = corpus.len() - refusals;
    (commands as f32 / refusals as f32).clamp(1.0, 64.0)
}

/// One number off the command line.
fn numbered(flag: &str) -> Option<usize> {
    let mut args = std::env::args();
    while let Some(arg) = args.next() {
        if arg == flag {
            return args.next()?.parse().ok();
        }
    }
    None
}

/// Train, reporting the holdout after every pass.
fn run<B: burn::tensor::backend::AutodiffBackend>(
    device: &B::Device,
    register: Register,
    all: &Corpus,
    vocabulary: &Vocabulary,
    epochs: usize,
) {
    let (corpus, holdout, refused) = (&all.learn, &all.holdout, &all.refused);
    let classes = register.classes();
    // **Seeded, and it was not.** The comment below claimed two runs produce the
    // same weights while the *initialisation* was left to chance, so two runs of
    // the same architecture reached 72.5% and 84.7% and there was no way to tell
    // an improvement from a lucky start. In a project whose central claim is
    // bit-identical reproducibility that is the one place it should never have
    // been missing.
    B::seed(device, 0x0B5);
    let mut reader: Reader<B> = ReaderConfig::new(vocabulary.rows())
        .with_classes(classes)
        .with_dropout(register.dropout())
        .init(device);
    let mut optimiser = AdamConfig::new().init();

    // **Both heads are weighted against what the corpus actually holds**, and
    // both numbers are computed rather than written down — a constant here would
    // be a second expression of a fact the corpus already states, which is the
    // defect §19 keeps paying for.
    let verbs = verb_weights(corpus, classes);
    let refusing = refusal_weight(corpus);
    println!(
        "  command loss weighted {refusing:.1}:1 · verb weights {:.2}–{:.2}\n",
        verbs.iter().copied().fold(f32::MAX, f32::min),
        verbs.iter().copied().fold(0.0, f32::max),
    );

    // **Inverse frequency, because template expansion is not usage.** A verb's
    // share of the corpus is the product of its slot cardinalities and the
    // number of ways it was written down, and only the second of those is a
    // fact about language. Without this the head learns the expansion as a
    // prior. Clamped, because a rare verb weighted by raw inverse frequency
    // would swamp every gradient it appears in.
    let verb_loss = CrossEntropyLossConfig::new()
        .with_weights(Some(verbs))
        .init(device);
    // **The refusal head needs the same treatment for the same reason**, and it
    // is where the effect was first measured: at 34:1 a head that always says
    // *"yes, a command"* scores 97.1%, so refusal errors barely register and the
    // boundary drifts wherever the other gradients leave it. That, and not a
    // shared softmax, is what made refusal accuracy swing forty points between
    // adjacent epochs — splitting the heads apart changed nothing, which is how
    // the real cause was found. Class 0 is *asks for nothing*.
    let command_loss = CrossEntropyLossConfig::new()
        .with_weights(Some(vec![refusing, 1.0]))
        .init(device);
    let tag_loss = CrossEntropyLossConfig::new().init(device);

    // **Shuffled by a fixed rule, not by a clock.** Two runs of this file should
    // produce the same weights: the game's whole architecture rests on being
    // able to reproduce a result, and a trainer seeded from the time of day
    // would be the one place that stopped being true.
    let mut order: Vec<usize> = (0..corpus.len()).collect();

    // **The best pass, not the last one.** Holdout accuracy on this corpus
    // swings by thirty points between epochs while the training loss falls
    // steadily — the reader memorises the sentence *shapes* long before it stops
    // improving at reading them. Saving whatever the final epoch happened to
    // hold shipped 24% where an earlier pass had reached 55%.
    let mut best = 0.0f32;
    let mut kept = reader.clone();

    for epoch in 1..=epochs {
        shuffle(&mut order, epoch as u64);
        // Decayed, because a fixed 1e-3 is what makes the swing that large: the
        // steps stay long after there is anything left to cross.
        let rate = 1.0e-3 / (1.0 + 0.08 * (epoch - 1) as f64);
        let mut total = 0.0f32;
        let mut batches = 0usize;

        for chunk in order.chunks(BATCH) {
            // **Commands first, then the sentences that ask for nothing.** The
            // verb head must not be trained on a refusal: it has no right verb,
            // so any target given is noise. Sorting puts the commands in a
            // contiguous prefix, which a slice can take — the alternative is
            // gathering scattered rows, and this costs one partition.
            let mut samples: Vec<Sample> = chunk.iter().map(|at| corpus[*at].clone()).collect();
            samples.sort_by_key(|sample| !sample.is_command());
            let commands = samples.iter().filter(|s| s.is_command()).count();

            let batch = Batch::<B>::of(&samples, device);
            let reading = reader.forward(batch.tokens.clone(), batch.pad.clone());

            let [rows, width, tags] = reading.tags.dims();

            // *Whether*, on every sentence in the batch.
            let mut loss = command_loss.forward(reading.command, batch.commands.clone());
            // *Which*, on the ones that are asking for something.
            if commands > 0 {
                loss = loss
                    + verb_loss.forward(
                        reading.verb.slice([0..commands, 0..classes]),
                        batch.verbs.clone().slice(0..commands),
                    );
            }
            loss = loss
                + tag_loss.forward(
                    reading.tags.reshape([rows * width, tags]),
                    batch.tags.clone().reshape([rows * width]),
                );

            total += loss.clone().into_scalar().to_f32();
            batches += 1;

            let grads = GradientsParams::from_grads(loss.backward(), &reader);
            reader = optimiser.step(rate, reader, grads);
        }

        let valid = reader.valid();
        let read = score(&valid, holdout, &device.clone());
        let (verbs, tags) = (read.verbs, read.tags);
        // On the refusal holdout the binary head *is* the score: how often it
        // agreed there was nothing being asked for.
        let refusals = score(&valid, refused, &device.clone()).commands;

        // **All three, because any two of them was an incomplete criterion and
        // each time it showed.** Keeping on verbs alone missed that
        // slot-to-verb conditioning dropped refusals 91.1% → 82.5%; keeping on
        // verbs and refusals missed that the spell register's *tagging* swings
        // 88–94% between passes, and a spell reading is assembled out of the
        // spans, so a pass that names the right statement and mis-tags one word
        // of it produces `let tool be refer alembic` and counts as a win here.
        //
        // §15 weighs the dead-end rate above the raw resolution rate, so they
        // are averaged rather than one preferred.
        let together = (verbs + tags + refusals) / 3.0;
        let best_yet = together > best;
        if best_yet {
            best = together;
            kept = reader.clone();
        }
        println!(
            "  epoch {epoch:>3}   loss {:>7.4}   verb {verbs:>5.1}%   tags {tags:>5.1}%   refuse {refusals:>5.1}%{}",
            total / batches.max(1) as f32,
            if best_yet { "   <- kept" } else { "" },
        );
    }
    let reader = kept;
    println!("\n  best holdout class, tag and refusal mean: {best:.1}%");

    let weights = register.weights();
    let path = std::path::Path::new(weights);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match reader.save_file(path, &BinFileRecorder::<FullPrecisionSettings>::new()) {
        Ok(()) => println!("\n  weights written to {weights}.bin\n"),
        Err(error) => println!("\n  could not write weights: {error}\n"),
    }
}

/// How much of the holdout the reader gets right.
///
/// Returns the verb rate and the tagging rate, both as percentages. Tagging is
/// counted over real words only — see the module docs.
fn score<B: Backend>(reader: &Reader<B>, holdout: &[Sample], device: &B::Device) -> Scored {
    let mut verbs_right = 0usize;
    let mut tags_right = 0usize;
    let mut tags_seen = 0usize;
    let mut commands_right = 0usize;

    for chunk in holdout.chunks(BATCH) {
        let batch = Batch::<B>::of(chunk, device);
        let reading = reader.forward(batch.tokens.clone(), batch.pad.clone());
        let [rows, width, _] = reading.tags.dims();

        // **Counted on the device, not on the host.** Reading tensors back as
        // `Vec<i32>` and zipping them is where the first version of this metric
        // went wrong: `into_vec` is typed, the type was wrong for `argmax`'s
        // output, and `unwrap_or_default()` turned that error into an empty
        // vector. Tagging accuracy read a flat **0.0%** for twelve epochs and
        // looked like a model that had learned nothing.
        let verbs = reading
            .verb
            .argmax(1)
            .reshape([rows])
            .equal(batch.verbs.clone());
        verbs_right += counted(verbs.int().sum());

        let tags = reading
            .tags
            .argmax(2)
            .reshape([rows, width])
            .equal(batch.tags.clone())
            .int();
        // Real words only. A five-word sentence is 27 rows of padding, and a
        // score counted over all 32 is mostly a measure of predicting nothing.
        let real = batch.real.clone().int();
        tags_right += counted((tags * real.clone()).sum());
        tags_seen += counted(real.sum());

        // *Whether*, from its own head. On a command holdout this counts how
        // often the reader agreed there was something to do; on a refusal
        // holdout, how often it agreed there was not.
        let said = reading
            .command
            .argmax(1)
            .reshape([rows])
            .equal(batch.commands.clone());
        commands_right += counted(said.int().sum());
    }

    Scored {
        verbs: percent(verbs_right, holdout.len()),
        tags: percent(tags_right, tags_seen),
        commands: percent(commands_right, holdout.len()),
    }
}

/// What one pass over a holdout came to.
struct Scored {
    /// How often the verb head named the right command.
    verbs: f32,
    /// How often a real word was tagged right.
    tags: f32,
    /// How often the binary head agreed about whether there was a command.
    commands: f32,
}

/// A summed count, off the device and into a number.
fn counted<B: Backend>(sum: Tensor<B, 1, Int>) -> usize {
    usize::try_from(sum.into_scalar().to_i64()).unwrap_or(0)
}

fn percent(part: usize, whole: usize) -> f32 {
    if whole == 0 {
        return 0.0;
    }
    part as f32 / whole as f32 * 100.0
}

/// A shuffle that depends on the epoch and nothing else.
fn shuffle(order: &mut [usize], seed: u64) {
    let mut state = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
    for at in (1..order.len()).rev() {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let pick = (state >> 33) as usize % (at + 1);
        order.swap(at, pick);
    }
}
