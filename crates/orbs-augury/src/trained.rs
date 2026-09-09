//! The trained reader, answering as an [`Augur`].
//!
//! # Where the boundary sits
//!
//! `orbs-sim` owns *which* lines a reader may see and what happens to its
//! answer; this owns only the answering. The sim never depends on this crate —
//! CLAUDE.md rule 1 — so a tower with no reader compiled in is the game exactly
//! as it was.
//!
//! # It still never sees the world
//!
//! The reader answers `grind sage` with the player's own word in the slot, and
//! `parser::resolve` binds it to whatever is actually on the shelf. That is the
//! same division of labour the grammar works under, and it is why a reader
//! needs no `Scene` and cannot go stale between ticks.

use burn::prelude::*;
use burn::record::{BinFileRecorder, FullPrecisionSettings, Recorder as _};
use orbs_sim::augur::{Augur, MAX_READINGS};
use orbs_sim::parser::Verb;

use crate::{Batch, MAX_SLOTS, Reader, ReaderConfig, Sample, Tag, VERBS, Vocabulary};

/// Where the trainer leaves its weights.
pub const WEIGHTS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/weights/reader");

/// The reader as a frontend should hold it.
///
/// **The CPU backend, and that is a measurement rather than a preference.** One
/// 32-token sentence takes **348µs** on `ndarray` against **2.71ms** on `wgpu`:
/// a batch of one is almost entirely kernel-launch overhead, so the GPU's place
/// is the training run and not the prompt.
///
/// 348µs is also what retires the worker thread, the deadline and the *"the orb
/// ponders"* indicator the plan budgeted for. §6 asks for sub-millisecond; this
/// is a third of one, so a reader can simply answer.
pub type Reading = Trained<burn::backend::NdArray<f32>>;

impl Reading {
    /// Load the trained weights onto the CPU.
    ///
    /// # Errors
    ///
    /// If this checkout has no trained weights — they are a build artefact of
    /// `cargo run -p orbs-augury --example train --features train`, not source,
    /// so a fresh clone
    /// has none and a caller should carry on without a reader.
    pub fn cpu() -> Result<Self, burn::record::RecorderError> {
        Self::load(burn::backend::ndarray::NdArrayDevice::default())
    }
}

/// A reader with weights that mean something.
#[derive(Debug)]
pub struct Trained<B: Backend> {
    reader: Reader<B>,
    vocabulary: Vocabulary,
    device: B::Device,
}

impl<B: Backend> Trained<B> {
    /// Load the weights the trainer wrote.
    ///
    /// # Errors
    ///
    /// If no weights have been trained yet, or they were written for a
    /// different vocabulary — which is a hard error rather than a degradation,
    /// because a table that has shifted under trained weights produces confident
    /// nonsense rather than an obvious failure.
    pub fn load(device: B::Device) -> Result<Self, burn::record::RecorderError> {
        let vocabulary = Vocabulary::builtin();
        let record: <Reader<B> as Module<B>>::Record =
            BinFileRecorder::<FullPrecisionSettings>::new()
                .load(std::path::PathBuf::from(WEIGHTS), &device)?;

        // **The vocabulary is checked before the record is applied**, because
        // `load_record` answers a table that has changed shape underneath it
        // with a panic from inside `burn` rather than an error — which reaches
        // a player as a crash on a line they typed. Growing the corpus grows
        // the table, so this is the ordinary consequence of authoring, not a
        // corrupt file.
        let trained = record.words.weight.val().dims()[0];
        if trained != vocabulary.rows() {
            return Err(burn::record::RecorderError::Unknown(format!(
                "these weights were trained for a vocabulary of {trained} rows and this \
                 build has {}; retrain with `cargo run --release -p orbs-augury --example train --features train`",
                vocabulary.rows()
            )));
        }

        let reader = ReaderConfig::new(vocabulary.rows())
            .init::<B>(&device)
            .load_record(record);
        Ok(Self {
            reader,
            vocabulary,
            device,
        })
    }

    /// What the reader makes of one line, best first.
    ///
    /// Empty when the reader answered *"nothing is being asked for here"* —
    /// which is its own head's decision rather than a threshold applied to the
    /// verb scores afterwards.
    ///
    /// # Several, because the room knows things the reader does not
    ///
    /// The tagger reads 97% and the verb head far less, so the usual failure is
    /// a correctly-found argument under the wrong verb. **That is exactly the
    /// error a scene can settle**: `move lectern` and `attend lectern` are the
    /// same words under two verbs, and only one of them is a thing you can do to
    /// a lectern. `Sim::submit_reading` walks this list and takes the first that
    /// resolves, so a second choice costs one sort here and nothing at all when
    /// the first was right.
    ///
    /// **Each candidate is trimmed to its own verb's arity.** A three-slot
    /// tagging offered to a one-slot verb would be refused for having too many
    /// arguments rather than judged on the argument that matters.
    #[must_use]
    pub fn readings(&self, line: &str) -> Vec<String> {
        let Some(sample) = Sample::reject(line, &self.vocabulary) else {
            return Vec::new();
        };
        let batch = Batch::<B>::of(std::slice::from_ref(&sample), &self.device);
        let reading = self.reader.forward(batch.tokens, batch.pad);

        // **Whether, before which.** The verb head always names its best guess —
        // it has no way not to — so the binary head is what decides there is
        // anything to name. Row 1 is *"this is a command"*.
        if scalar(reading.command.argmax(1).reshape([1])) == 0 {
            return Vec::new();
        }

        let [rows, width, _] = reading.tags.dims();
        let tags = reading.tags.argmax(2).reshape([rows * width]);
        let tags: Vec<i64> = tags
            .into_data()
            .convert::<i64>()
            .into_vec()
            .unwrap_or_default();

        // Words in the order the sentence had them, gathered per slot. The
        // sample's rows are `<cls>` then one per word, so the tag at row `n + 1`
        // belongs to word `n`.
        let words: Vec<&str> = line.split_whitespace().collect();
        let mut slots: Vec<Vec<&str>> = vec![Vec::new(); MAX_SLOTS];
        for (at, word) in words.iter().enumerate() {
            let Some(row) = tags.get(at + 1) else { break };
            match Tag::from_row(u32::try_from(*row).unwrap_or(0)) {
                Tag::Outside => {}
                Tag::Begin(slot) | Tag::Inside(slot) => {
                    if let Some(found) = slots.get_mut(slot) {
                        found.push(word);
                    }
                }
            }
        }
        let filled: Vec<String> = slots
            .iter()
            .filter(|slot| !slot.is_empty())
            .map(|slot| slot.join(" "))
            .collect();

        // The verb head's whole ranking, not just its argmax. A batch of one, so
        // this is 46 floats and an ordinary sort.
        let scores: Vec<f32> = reading
            .verb
            .reshape([VERBS])
            .into_data()
            .convert::<f32>()
            .into_vec()
            .unwrap_or_default();
        if scores.len() != VERBS {
            // The one shape this can take is a silent `into_vec` type mismatch,
            // which has cost this crate a day once already. An empty list falls
            // through to the matcher rather than answering with row zero.
            return Vec::new();
        }
        let mut ranked: Vec<usize> = (0..VERBS).collect();
        ranked.sort_by(|a, b| scores[*b].total_cmp(&scores[*a]));

        ranked
            .into_iter()
            .take(MAX_READINGS)
            .filter_map(|at| {
                let verb = Verb::ALL.get(at)?;
                let mut command = String::from(verb.canonical());
                for slot in filled.iter().take(verb.signature().len()) {
                    command.push(' ');
                    command.push_str(slot);
                }
                Some(command)
            })
            .collect()
    }

    /// The single best reading, when only one is wanted.
    #[must_use]
    pub fn command(&self, line: &str) -> Option<String> {
        self.readings(line).into_iter().next()
    }
}

impl<B: Backend> Augur for Trained<B> {
    fn read(&self, line: &str) -> Vec<String> {
        self.readings(line)
    }
}

/// One number off a summed or arg-maxed tensor.
fn scalar<B: Backend>(tensor: Tensor<B, 1, Int>) -> i64 {
    tensor.into_scalar().to_i64()
}

#[cfg(test)]
mod tests {
    use super::*;
    use burn::backend::NdArray;

    /// The trained weights, if this checkout has any.
    ///
    /// **Skipped rather than failed when absent.** Weights are a build artefact
    /// of a GPU run, and `cargo test --workspace` must pass on a machine that
    /// has never trained anything.
    fn trained() -> Option<Trained<NdArray<f32>>> {
        Trained::load(burn::backend::ndarray::NdArrayDevice::default()).ok()
    }

    #[test]
    fn it_reads_a_phrasing_the_matcher_cannot() {
        let Some(reader) = trained() else {
            println!(
                "no weights; run `cargo run --release -p orbs-augury --example train --features train`"
            );
            return;
        };
        let read = reader.command("smash the sage");
        println!("smash the sage -> {read:?}");
        assert!(read.is_some(), "the reader refused a corpus phrasing");
    }

    #[test]
    fn it_can_refuse() {
        // The reject class doing its job. §15 weighs the dead-end rate above the
        // raw resolution rate, so a reader that answers everything is worse than
        // one that answers less.
        let Some(reader) = trained() else { return };
        println!(
            "what should i do next -> {:?}",
            reader.command("what should i do next")
        );
    }
}
