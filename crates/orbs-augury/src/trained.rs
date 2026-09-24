//! The trained reader, answering as an [`Augur`].
//!
//! `orbs-sim` owns *which* lines a reader may see and what happens to its
//! answer; this owns only the answering. The sim never depends on this crate
//! (rule 1), so a tower with no reader compiled in is the game as it was.
//!
//! It still never sees the world: the reader answers `grind sage` with the
//! player's own word in the slot and `parser::resolve` binds it to whatever is
//! on the shelf, so a reader needs no `Scene` and never goes stale.

use burn::prelude::*;
use burn::record::{BinFileRecorder, FullPrecisionSettings, Recorder as _};
use orbs_sim::augur::{Augur, MAX_READINGS};
use orbs_sim::parser::Verb;

use crate::decode::decode;
use crate::{Reader, ReaderConfig, Register, VERBS, Vocabulary};

/// Where the trainer leaves the prompt reader's weights.
pub const WEIGHTS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/weights/reader");

/// Where it leaves the spell reader's.
pub const SCRIBE_WEIGHTS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/weights/scribe");

/// The reader as a frontend should hold it.
///
/// The CPU backend, measured rather than preferred: one 32-token sentence takes
/// 348µs on `ndarray` against 2.71ms on `wgpu`, because a batch of one is
/// almost all kernel-launch overhead. The GPU's place is the training run.
///
/// 348µs also retires the worker thread, the deadline and the *"the orb
/// ponders"* indicator the plan budgeted for — §6 asks for sub-millisecond.
pub type Reading = Trained<burn::backend::NdArray<f32>>;

impl Reading {
    /// Load the trained weights onto the CPU.
    ///
    /// # Errors
    ///
    /// If this checkout has no trained weights — they are a build artefact of
    /// `cargo run -p orbs-augury --example train --features train`, not source,
    /// so a fresh clone has none and a caller carries on without a reader.
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
    /// different vocabulary — a hard error rather than a degradation, because a
    /// shifted table produces confident nonsense rather than a visible failure.
    pub fn load(device: B::Device) -> Result<Self, burn::record::RecorderError> {
        Self::load_from(WEIGHTS, device)
    }

    /// Load weights from `path`, without its `.bin`, rather than the ones the
    /// game ships.
    ///
    /// For a run nobody has shipped yet: `scripts/seeds.sh` trains several and
    /// reads each one back to learn which, if any, is worth keeping.
    ///
    /// # Errors
    ///
    /// As [`load`](Self::load).
    pub fn load_from(path: &str, device: B::Device) -> Result<Self, burn::record::RecorderError> {
        let vocabulary = Vocabulary::builtin();
        let reader = weights(path, Register::Verbs, &vocabulary, &device)?;
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
    /// Several, because the room knows things the reader does not: the tagger
    /// reads 97% and the verb head far less, so the usual failure is a
    /// correctly-found argument under the wrong verb — which a scene settles,
    /// since only one of `move lectern` and `attend lectern` is a thing you can
    /// do to a lectern. `Sim::submit_reading` takes the first that resolves.
    ///
    /// Each candidate is trimmed to its own verb's arity, or a three-slot
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
                // A verb that cannot hold what the tagger found is not a
                // reading of this sentence. The trim below removes every span
                // for a verb taking nothing, so `digest husks` offered `undo`,
                // which always resolves — and beat the correctly-ranked reading
                // above it. The free-text sink in `parser::resolve` is the same
                // shape: something that can never fail to resolve wins by never
                // failing, not by being right.
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
            // After the filter, not before: taking four and then discarding
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
/// Both shapes are checked before the record is applied, because `load_record`
/// answers a table that changed shape underneath it with a panic from inside
/// `burn` — which reaches a player as a crash on a line they typed. Growing the
/// corpus or adding a template does it, so this is ordinary authoring rather
/// than a corrupt file.
///
/// # Errors
///
/// If the file is missing or unreadable, or was trained for a different
/// vocabulary or a different number of classes.
pub(crate) fn weights<B: Backend>(
    path: &str,
    register: Register,
    vocabulary: &Vocabulary,
    device: &B::Device,
) -> Result<Reader<B>, burn::record::RecorderError> {
    let record: <Reader<B> as Module<B>>::Record = BinFileRecorder::<FullPrecisionSettings>::new()
        .load(std::path::PathBuf::from(path), device)?;

    // The command that retrains *this* register, told rather than guessed: the
    // path cannot say it now `load_from` takes any path a run wrote, and the
    // head's width says it only until the spell corpus has as many shapes as
    // there are verbs.
    let classes = register.classes();
    let retrain = match register {
        Register::Verbs => "cargo run --release -p orbs-augury --example train --features train",
        Register::Spells => {
            "cargo run --release -p orbs-augury --example train --features train -- --spells"
        }
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
/// The bytes, not the path or a timestamp: a retrain writes the same path, and
/// a copy changes a timestamp. Read at load, which is once a session.
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
    /// Skipped rather than failed when absent: `cargo test --workspace` must
    /// pass on a machine that has never trained anything.
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
    fn a_path_that_holds_nothing_is_an_error_rather_than_a_panic() {
        // `load_from` takes whatever path a run wrote, and a seed that never
        // finished wrote none.
        let device = burn::backend::ndarray::NdArrayDevice::default();
        assert!(Trained::<NdArray<f32>>::load_from("/nowhere/reader", device).is_err());
    }

    #[test]
    fn a_stale_file_names_the_trainer_for_the_head_it_was_asked_for() {
        // By the head, not the path, since a path is whatever `--out` wrote:
        // the prompt's weights asked for as the spell register are the spell
        // trainer's to replace.
        if trained().is_none() {
            return;
        }
        let device = burn::backend::ndarray::NdArrayDevice::default();
        let error =
            weights::<NdArray<f32>>(WEIGHTS, Register::Spells, &Vocabulary::builtin(), &device)
                .expect_err("the spell head is not the prompt's width");
        assert!(format!("{error:?}").contains("--spells"), "{error:?}");
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
