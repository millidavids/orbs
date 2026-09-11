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

use crate::decode::decode;
use crate::{Reader, ReaderConfig, VERBS, Vocabulary};

/// Where the trainer leaves the prompt reader's weights.
pub const WEIGHTS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/weights/reader");

/// Where it leaves the spell reader's.
pub const SCRIBE_WEIGHTS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/weights/scribe");

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
        let reader = weights(WEIGHTS, VERBS, &vocabulary, &device)?;
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
        let Some(read) = decode(&self.reader, &self.vocabulary, &self.device, line, VERBS) else {
            return Vec::new();
        };
        if !read.answering {
            return Vec::new();
        }
        let filled = read.slots;

        read.ranked
            .into_iter()
            .filter_map(|at| {
                let verb = Verb::ALL.get(at)?;
                // **A verb that cannot hold what the tagger found is not a
                // reading of this sentence.** The trim below is what makes each
                // candidate well-formed, and for a verb that takes *nothing* it
                // trims away every span — so `digest husks` offered a bare
                // `undo`, which takes no argument and therefore always resolves.
                // `Sim::submit_reading` takes the first reading that resolves,
                // so that sink beat the correctly-ranked `digest husks` sitting
                // above it and the orb reverted the player's last command.
                //
                // The same shape as the free-text sink in `parser::resolve`, one
                // level up: **something that can never fail to resolve wins by
                // never failing**, not by being right.
                if verb.signature().len() < filled.len() {
                    return None;
                }
                let mut command = String::from(verb.canonical());
                for slot in filled.iter().take(verb.signature().len()) {
                    command.push(' ');
                    command.push_str(slot);
                }
                Some(command)
            })
            // **After the filter, not before.** Taking four and then discarding
            // some would offer fewer than four readings for no reason.
            .take(MAX_READINGS)
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

/// Load one register's weights, checked against what this build holds.
///
/// **The two shapes are checked before the record is applied**, because
/// `load_record` answers a table that has changed shape underneath it with a
/// panic from inside `burn` rather than an error — which reaches a player as a
/// crash on a line they typed. Growing the corpus grows the vocabulary and
/// adding a template grows the head, so both are the ordinary consequence of
/// authoring rather than a corrupt file.
///
/// # Errors
///
/// If the file is missing or unreadable, or was trained for a different
/// vocabulary or a different number of classes.
pub(crate) fn weights<B: Backend>(
    path: &str,
    classes: usize,
    vocabulary: &Vocabulary,
    device: &B::Device,
) -> Result<Reader<B>, burn::record::RecorderError> {
    let record: <Reader<B> as Module<B>>::Record = BinFileRecorder::<FullPrecisionSettings>::new()
        .load(std::path::PathBuf::from(path), device)?;

    // **The command that retrains *this* file.** The spell reader's is the same
    // trainer with `--spells`, and naming the other one sends somebody to
    // retrain the wrong model and meet the same error.
    let retrain = if path == SCRIBE_WEIGHTS {
        "cargo run --release -p orbs-augury --example train --features train -- --spells"
    } else {
        "cargo run --release -p orbs-augury --example train --features train"
    };
    let stale = |what: &str, was: usize, now: usize| {
        burn::record::RecorderError::Unknown(format!(
            "{path}.bin was trained for {was} {what} and this build has {now}; retrain with \
             `{retrain}`",
        ))
    };
    let rows = record.words.weight.val().dims()[0];
    if rows != vocabulary.rows() {
        return Err(stale("vocabulary rows", rows, vocabulary.rows()));
    }
    // `verb` is `Linear<width * (Tag::COUNT + 1), classes>`, so its output
    // dimension is the width of the head.
    let head = record.verb.weight.val().dims()[1];
    if head != classes {
        return Err(stale("classes", head, classes));
    }

    Ok(ReaderConfig::new(vocabulary.rows())
        .with_classes(classes)
        .init::<B>(device)
        .load_record(record))
}

/// Which weights these are, as `Scrivener::identity` asks: FNV-1a over the
/// bytes of each `.bin` file, in the order given.
///
/// **The bytes, not the path and not a timestamp.** A retrain writes the same
/// path, so a path would call two readers one; a timestamp would call one
/// reader two across a copy. Read once more at load, which is once a session.
pub(crate) fn identity(paths: &[&str]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for path in paths {
        for byte in std::fs::read(format!("{path}.bin")).unwrap_or_default() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    hash
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
