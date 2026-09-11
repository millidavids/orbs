//! The trained reader for `.spell` files, answering as a [`Scrivener`].
//!
//! # The stricter half
//!
//! DESIGN.md §19, *The orb accepts an abbreviation, never a typo, and never a
//! coin flip*: *"At the prompt the player sees the echo; a spell resolves with
//! nobody watching, so it takes the stricter half."* Everything here is that
//! sentence made mechanical. The prompt's reader offers four candidates and lets
//! `parser::resolve` pick one against the room; this offers a line back to the
//! player's own file and nobody will be looking, so a candidate has to survive
//! five checks before it is offered at all — see [`Scribe::reading`].
//!
//! # Nothing that already reads is touched
//!
//! §19 deleted `scribe::canonicalise` because save-time rewriting *"destroyed the
//! player's words whenever it understood only part of one"*. The first defence
//! against arriving back at that is to answer [`None`] for every line the game
//! could already read — `spell::reads_cleanly`, which is a statement that parses,
//! a call, or a block word standing on its own. Those never reach the model, so
//! no weight can change them.
//!
//! **Not `parser::is_literal`**, which was tried and is the prompt's predicate
//! rather than a spell's: it answers *true* for any line whose first word is a
//! spell word, and half the loose phrasings a spell reader exists for begin with
//! one. *"if the alembic is not busy"* is exactly the line this feature is for,
//! and the gate refused it before the model ever saw it.
//!
//! # Two readers, because a spell body is mostly commands
//!
//! `grind sage` is not control flow and the spell head says so — that is what
//! [`spelling::command`] is for. The line then belongs to the *prompt's* reader,
//! which is the one trained on forty-six verbs, so this holds one of those too
//! and hands the line over. A build with no prompt weights simply leaves command
//! lines alone.

use burn::prelude::*;
use orbs_sim::Scrivener;
use orbs_sim::augur::MAX_READINGS;

use crate::decode::decode;
use crate::trained::{SCRIBE_WEIGHTS, WEIGHTS, identity, weights};
use crate::{Reader, Trained, Vocabulary, spelling};

/// The spell reader as a frontend should hold it.
///
/// The CPU backend, for [`Reading`](crate::Reading)'s reason: a batch of one is
/// almost entirely kernel-launch overhead, so the GPU's place is the training
/// run. It matters more here — the editor writes the buffer out after every pause
/// in the typing, and only the lines that *changed* are read again.
pub type Copying = Scribe<burn::backend::NdArray<f32>>;

impl Copying {
    /// Load both registers' weights onto the CPU.
    ///
    /// # Errors
    ///
    /// If this checkout has no trained spell weights — they are a build artefact
    /// of `cargo run -p orbs-augury --example train --features train -- --spells`,
    /// not source, so a fresh clone has none and a caller should carry on
    /// without a scrivener.
    pub fn cpu() -> Result<Self, burn::record::RecorderError> {
        Self::load(burn::backend::ndarray::NdArrayDevice::default())
    }
}

/// A spell reader with weights that mean something.
#[derive(Debug)]
pub struct Scribe<B: Backend> {
    reader: Reader<B>,
    /// The prompt's reader, for the lines that turn out to be commands.
    ///
    /// [`None`] where this checkout has spell weights and no prompt weights,
    /// which is not a state anyone would train on purpose but is one a partial
    /// build reaches. Command lines are then left exactly as written.
    ///
    /// **Its own copy, not the frontend's.** A build that holds both readers
    /// loads the prompt's weights twice — 341k parameters, so about 1.4MB and
    /// one extra file read at start-up. Sharing would mean an `Arc` reaching
    /// across the `Augur`/`Scrivener` seam and a lifetime on this type, which is
    /// a great deal of structure to save a megabyte that is never touched again.
    prompt: Option<Trained<B>>,
    vocabulary: Vocabulary,
    device: B::Device,
    /// Which weights these are, as `Scrivener::identity` asks — both files',
    /// because a retrained prompt reader changes what a command line reads as
    /// just as a retrained spell reader changes the rest.
    identity: u64,
}

impl<B: Backend> Scribe<B> {
    /// Load the weights the trainer wrote for the spell register.
    ///
    /// # Errors
    ///
    /// If no spell weights have been trained yet, or they were written for a
    /// different vocabulary or a different set of shapes — a hard error rather
    /// than a degradation, because a table that has shifted under trained
    /// weights produces confident nonsense rather than an obvious failure.
    pub fn load(device: B::Device) -> Result<Self, burn::record::RecorderError> {
        let vocabulary = Vocabulary::builtin();
        let reader = weights(SCRIBE_WEIGHTS, spelling::kinds(), &vocabulary, &device)?;
        let prompt = Trained::load(device.clone()).ok();
        let weighed: &[&str] = if prompt.is_some() {
            &[SCRIBE_WEIGHTS, WEIGHTS]
        } else {
            &[SCRIBE_WEIGHTS]
        };
        Ok(Self {
            reader,
            prompt,
            vocabulary,
            device,
            identity: identity(weighed),
        })
    }

    /// What the orb makes of one spell line, or [`None`] to leave it as written.
    ///
    /// # Five checks, and a line survives all of them or none
    ///
    /// 1. **It is not already readable.** A line opening on a spell word, or one
    ///    that parses as a statement, is returned untouched — see the module
    ///    documentation.
    /// 2. **The binary head found something.** Trained on `phrasings.toml`'s
    ///    refusals, the sentences that ask for nothing at all — the spell file's
    ///    are canonical statements check 1 keeps away from it.
    /// 3. **The shape fills honestly.** [`spelling::assemble`] refuses a shape
    ///    with more slots than the tagger found, a count the line does not
    ///    contain, a name the player never wrote, or an `else` or `end` the line
    ///    never says.
    /// 4. **It parses and accounts for the line.** `spell::reads_cleanly` and
    ///    [`spelling::accounts_for`] between them — no join, comparison,
    ///    negation, condition or unit dropped — which is the pair §19's rewriter
    ///    never had.
    /// 5. **It keeps every name.** See [`spelling::names_in`].
    ///
    /// A command line takes the same route through the prompt's reader, with
    /// check 3 replaced by that reader's own arity trim.
    #[must_use]
    pub fn reading(&self, line: &str) -> Option<String> {
        // A blank and a comment are nobody's to read. `Sim::write_spell_reading`
        // keeps the same rule; both, because a reader that answered a comment
        // would turn a line the player commented *out* into one the spell runs,
        // and this is not a rule to hold in only one place.
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return None;
        }
        if orbs_sim::tower::spell::reads_cleanly(trimmed) {
            return None;
        }

        let read = decode(
            &self.reader,
            &self.vocabulary,
            &self.device,
            line,
            spelling::kinds(),
        )?;
        if !read.answering {
            return None;
        }

        // # No reading may drop a name
        //
        // **Measured, not supposed**, and first on commands alone. On `let`'s
        // holdout the head ranked *command* first 299 times in 576 and `let`
        // second, and every one of those `let` readings was built correctly —
        // `let nook be alembic` — while the prompt's reader answered `stop`,
        // `move bailey`, `attend lectern`. So a command that dropped a word the
        // game does not know lost to a statement that kept it.
        //
        // A statement was taken to be unable to drop one, being built from the
        // tagger's spans — and it could, whenever the tagger missed the name:
        // *"name the alembic bertha"* came back `if alembic is empty`. Every
        // reading with room for a name is held to it now, and the best-ranked
        // one that keeps them wins; a shape with no room for one has to be said
        // instead — see `spelling::holds_a_name`. `spelling::names_in` is what
        // separates a name from a typo the reader is entitled to spend.
        let names = spelling::names_in(line, &read.placed, &self.vocabulary);
        read.ranked
            .into_iter()
            .take(MAX_READINGS)
            .enumerate()
            .filter_map(|(rank, class)| {
                let reading = if class == spelling::command() {
                    // **Only as the first choice.** The prompt's reader always
                    // finds *some* verb, so falling through to it after a shape
                    // failed to build is the sink this crate has closed twice
                    // already — something that never fails to resolve winning by
                    // never failing. The head said *this is a `let`*; a `let` it
                    // could not build is a line it cannot read, not a command.
                    (rank == 0).then(|| self.commanded(line)).flatten()
                } else if rank > 0 && !spelling::fills_from_the_line(class) {
                    // **`else` and `end` are the same sink one level down.** A
                    // shape with no slot and no count assembles out of any line
                    // at all, so reaching one by walking down the ranking is
                    // reaching the thing that cannot fail: *"do it for all
                    // columns"* came back `else`. It stands as a first choice and
                    // never as a fallback — and even first, only where the line
                    // says it (`spelling::assemble`).
                    None
                } else {
                    spelling::ordered(class, &read.placed)
                        .and_then(|slots| spelling::assemble(class, line, &slots))
                };
                reading.map(|reading| (class, reading))
            })
            .find(|(class, reading)| {
                !spelling::holds_a_name(*class) || spelling::keeps(reading, &names)
            })
            .map(|(_, reading)| reading)
    }

    /// Why [`reading`](Self::reading) answered as it did — for a report, not for
    /// a player.
    ///
    /// **The instrument this reader was tuned with, kept.** Every fix to the
    /// scrivener in `0.14` came from seeing *which* part failed — the head
    /// choosing *command* on a `let`, the tagger running two words into one slot,
    /// a shape with no slot winning by never failing — and an accuracy figure
    /// says none of that.
    #[must_use]
    pub fn consider(&self, line: &str) -> Considered {
        let gated = |why: &'static str| Considered {
            reading: None,
            gated: Some(why),
            answering: false,
            top: Vec::new(),
            placed: Vec::new(),
        };
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return gated("a blank or a comment");
        }
        if orbs_sim::tower::spell::reads_cleanly(trimmed) {
            return gated("already a statement");
        }
        let Some(read) = decode(
            &self.reader,
            &self.vocabulary,
            &self.device,
            line,
            spelling::kinds(),
        ) else {
            return gated("longer than the reader reads");
        };
        Considered {
            reading: self.reading(line),
            gated: None,
            answering: read.answering,
            top: read
                .ranked
                .iter()
                .take(3)
                .map(|class| spelling::shape_of(*class).unwrap_or("command"))
                .collect(),
            placed: read.placed,
        }
    }

    /// The prompt reader's answer, held to the spell register's standard.
    ///
    /// It offers several and the room is what usually chooses between them; there
    /// is no room here, so the first that accounts for the line wins and a line
    /// none of them account for is left alone.
    fn commanded(&self, line: &str) -> Option<String> {
        let prompt = self.prompt.as_ref()?;
        let readings = prompt.readings(line);
        // **A line the reader counts among its own readings is already one.**
        // Passing over the reading equal to the line and taking the next was a
        // second-best answer to a line that needed none: `kindle charcoal` —
        // canonical, and the prompt reader's own first reading of it — came
        // back `grind charcoal`. Other spacing or capitals are the same line.
        let same = |reading: &String| {
            reading
                .split_whitespace()
                .map(str::to_lowercase)
                .eq(line.split_whitespace().map(str::to_lowercase))
        };
        if readings.iter().any(same) {
            return None;
        }
        readings.into_iter().find(|reading| {
            spelling::accounts_for(line, reading)
                // **A bare verb has to be one the line names.** It takes no
                // argument, so nothing from the line went into it, and the
                // prompt's reader will always find one: *"for every one of the
                // bands"* came back `status`, *"the stairs creak"* `probe`. With
                // an argument, the line's own words are in it.
                && (reading.split_whitespace().nth(1).is_some()
                    || spelling::names_its_verb(line, reading))
        })
    }
}

impl<B: Backend> Scrivener for Scribe<B> {
    fn read(&self, line: &str) -> Option<String> {
        self.reading(line)
    }

    fn identity(&self) -> u64 {
        self.identity
    }
}

/// What the scrivener made of one line, and the parts it was made from.
#[derive(Debug, Clone)]
pub struct Considered {
    /// The reading, or [`None`] for *left as written*.
    pub reading: Option<String>,
    /// The gate that stopped the line before the model was asked, if one did.
    pub gated: Option<&'static str>,
    /// Whether the binary head found anything here to answer.
    pub answering: bool,
    /// The head's first three choices, best first — a shape, or `command`.
    pub top: Vec<&'static str>,
    /// What the tagger found, slot by slot, empties kept.
    pub placed: Vec<Option<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use burn::backend::NdArray;

    /// The trained spell weights, if this checkout has any.
    ///
    /// **Skipped rather than failed when absent**, for `Trained`'s reason:
    /// weights are a build artefact of a GPU run, and `cargo test --workspace`
    /// must pass on a machine that has never trained anything.
    fn scribe() -> Option<Scribe<NdArray<f32>>> {
        Scribe::load(burn::backend::ndarray::NdArrayDevice::default()).ok()
    }

    #[test]
    fn a_line_the_game_already_reads_is_never_touched() {
        // **Check one, and it needs no weights at all** — which is the point of
        // putting it before the model rather than trusting the model to have
        // learned it. §19 deleted the last thing that rewrote a working line.
        let Some(scribe) = scribe() else {
            println!(
                "no spell weights; run `cargo run --release -p orbs-augury --example train --features train -- --spells`"
            );
            return;
        };
        for line in [
            "if mortar_and_pestle is idle",
            "end",
            "else",
            "bide 10",
            "repeat 3",
            "morning()",
            "",
            "# a note to myself",
        ] {
            assert_eq!(scribe.read(line), None, "{line:?} was rewritten");
        }
    }

    #[test]
    fn a_canonical_command_is_never_read_as_another() {
        // `kindle charcoal` came back `grind charcoal`: the reading equal to the
        // line was skipped and the next one taken. A command line is not gated
        // before the model — `reads_cleanly` is the spell language's question —
        // so this is what stands between a working line and a rewrite.
        let Some(scribe) = scribe() else { return };
        for line in [
            "kindle charcoal",
            "grind sage",
            "distil sage",
            "survey",
            "stop alembic",
        ] {
            let read = scribe.read(line);
            assert!(
                read.is_none() || read.as_deref() == Some(line),
                "{line:?} was rewritten as {read:?}",
            );
        }
    }

    #[test]
    fn it_reads_control_flow_the_matcher_cannot() {
        let Some(scribe) = scribe() else { return };
        for line in [
            "when the mortar_and_pestle is free",
            "wait ten ticks",
            "that is all",
        ] {
            println!("{line:?} -> {:?}", scribe.read(line));
        }
    }
}
