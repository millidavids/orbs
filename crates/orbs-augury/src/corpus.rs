//! What a register is trained on, and what it is measured against.
//!
//! Two registers, one of everything else. The prompt's reader and the spell's
//! are the same architecture over the same vocabulary asking the same question
//! — *which statement is this, and which words are its argument* — so only the
//! answer space and the corpus differ, and that difference is gathered here
//! rather than spread across two trainers that would drift (§19).
//!
//! A refusal is not a class. [`Sample::reject`] marks an example with
//! [`crate::REJECT`], out of range for both heads on purpose, so the trainer
//! never shows it to the class head. The binary head is the only one a refusal
//! trains.

use orbs_sim::content::{CORPUS_CAP, Phrasings, corpus_scene};

use crate::spelling;
use crate::{Sample, VERBS, Vocabulary};

/// Where the prompt reader's weights live, relative to the workspace root.
pub const READER: &str = "crates/orbs-augury/weights/reader";

/// Where the spell reader's weights live, relative to the workspace root.
pub const SCRIBE: &str = "crates/orbs-augury/weights/scribe";

/// Which reader is being built.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Register {
    /// Lines typed at the prompt: which of `Verb::ALL`.
    Verbs,
    /// Lines of a `.spell`: which canonical shape, or *this is a command*.
    Spells,
}

impl Register {
    /// The one the command line asked for.
    #[must_use]
    pub fn asked() -> Self {
        if std::env::args().any(|arg| arg == "--spells") {
            Self::Spells
        } else {
            Self::Verbs
        }
    }

    /// What the class head chooses between.
    #[must_use]
    pub fn classes(self) -> usize {
        match self {
            Self::Verbs => VERBS,
            Self::Spells => spelling::kinds(),
        }
    }

    /// Where this register's weights are written and read.
    #[must_use]
    pub const fn weights(self) -> &'static str {
        match self {
            Self::Verbs => READER,
            Self::Spells => SCRIBE,
        }
    }

    /// How much of the encoding to drop while training.
    ///
    /// A training-time knob only: burn's `Dropout` is inert on a backend with no
    /// autodiff, which is every backend a reader is loaded on.
    ///
    /// Higher for spells, because control flow has nothing to expand over —
    /// five of the eleven shapes take no argument — so [`Corpus::spells`] lifts
    /// them by repeating what they have, and a reader memorises that in a pass
    /// or two.
    #[must_use]
    pub const fn dropout(self) -> f64 {
        match self {
            Self::Verbs => 0.1,
            Self::Spells => 0.2,
        }
    }

    /// What to call it in a report.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Verbs => "the prompt",
            Self::Spells => "spells",
        }
    }

    /// Everything this register learns from and is measured on.
    #[must_use]
    pub fn corpus(self, vocabulary: &Vocabulary) -> Corpus {
        match self {
            Self::Verbs => Corpus::verbs(vocabulary),
            Self::Spells => Corpus::spells(vocabulary),
        }
    }
}

/// One register's four populations.
#[derive(Debug, Clone)]
pub struct Corpus {
    /// What the reader is shown.
    pub learn: Vec<Sample>,
    /// Phrasings taught to nothing, scored as *did it read them*.
    pub holdout: Vec<Sample>,
    /// Sentences taught to nothing, scored as *did it leave them alone*.
    pub refused: Vec<Sample>,
    /// How many of [`learn`](Self::learn) ask for nothing.
    pub refusals: usize,
}

impl Corpus {
    /// The prompt's: every phrasing of every verb, and the sentences that ask
    /// for nothing.
    ///
    /// Capped, because the expansion was a product: `move {reagent} {place}` is
    /// the widest signature in `Verb::ALL` and its cross-product made it 47% of
    /// the corpus — a prior strong enough that `take me over to the lectern`
    /// came back `move lectern`. See `Phrasings::corpus_capped`.
    #[must_use]
    pub fn verbs(vocabulary: &Vocabulary) -> Self {
        let scene = corpus_scene();
        let phrasings = Phrasings::builtin();

        let mut learn: Vec<Sample> = phrasings
            .corpus_capped(&scene, CORPUS_CAP)
            .iter()
            .filter_map(|example| Sample::encode(example, vocabulary))
            .collect();
        // Refusals come from the content file, not a `const`: twenty
        // hand-written negatives against eleven thousand commands left the
        // reader refusing 0.0% of everything. They expand over the same nouns,
        // so a refusal is not learnable as *"a sentence with no game words"*.
        let refusals = rejected(&phrasings.refused(&scene), vocabulary);
        let counted = refusals.len();
        learn.extend(refusals);

        Self {
            learn,
            holdout: phrasings
                .holdout(&scene)
                .iter()
                .filter_map(|example| Sample::encode(example, vocabulary))
                .collect(),
            // Kept apart from the command holdout and scored the opposite way:
            // a refusal here is the right answer, and on the commands a miss.
            refused: rejected(&phrasings.refused_holdout(&scene), vocabulary),
            refusals: counted,
        }
    }

    /// The spell's: every phrasing of every control-flow shape, every phrasing
    /// of a *command*, and the lines that must come back untouched.
    ///
    /// The command class is thinned to the size of a shape: `phrasings.toml` is
    /// eleven thousand lines against `spellings.toml`'s few thousand, all of
    /// them the single class *this is a command*. Left whole, the reader learns
    /// that answering *command* is nearly always safe.
    ///
    /// Strided down to twice the mean size of a shape's class, computed from
    /// what the two files hold rather than written down here. Twice, not once,
    /// because *command* really is the commonest line in a spell.
    #[must_use]
    pub fn spells(vocabulary: &Vocabulary) -> Self {
        let scene = corpus_scene();
        let spellings = Phrasings::spellings();
        let phrasings = Phrasings::builtin();
        let command = spelling::command();

        let mut learn: Vec<Sample> = balance(
            spellings
                .corpus_by_entry(&scene, SPELL_CAP)
                .iter()
                .filter_map(|(class, example)| Sample::spelling(example, *class, vocabulary))
                .collect(),
        );

        let share = learn.len() / spelling::shapes().len().max(1) * 2;
        let commands: Vec<Sample> = thin(
            phrasings
                .corpus_capped(&scene, CORPUS_CAP)
                .iter()
                .filter_map(|example| Sample::spelling(example, command, vocabulary))
                .collect(),
            share,
        );
        learn.extend(commands);

        // `phrasings.toml`'s refusals, not thinned: a player who typed one into
        // a spell wants it left alone just as much as at the prompt.
        // `spellings.toml`'s contribute nothing — they are canonical
        // statements, which `unread` drops.
        //
        // A refusal is not a class, so the balance argument above does not
        // reach it. Thinning them to a shape's share cost twenty-five points:
        // 566 negatives against four thousand commands left the refusal holdout
        // at 40% where the prompt register, with all 2,756, reads 80%.
        let mut refusing = spellings.refused(&scene);
        refusing.extend(phrasings.refused(&scene));
        let refusals = rejected(&unread(refusing), vocabulary);
        let counted = refusals.len();
        learn.extend(refusals);

        let mut holdout: Vec<Sample> = spellings
            .holdout_by_entry(&scene)
            .iter()
            .filter_map(|(class, example)| Sample::spelling(example, *class, vocabulary))
            .collect();
        let held = holdout.len();
        holdout.extend(thin(
            phrasings
                .holdout(&scene)
                .iter()
                .filter_map(|example| Sample::spelling(example, command, vocabulary))
                .collect(),
            held,
        ));

        let mut leave = spellings.refused_holdout(&scene);
        leave.extend(thin(phrasings.refused_holdout(&scene), held));

        Self {
            learn,
            holdout,
            refused: rejected(&unread(leave), vocabulary),
            refusals: counted,
        }
    }
}

/// The most times one example may be repeated by [`balance`].
///
/// A ceiling rather than a target: past this the repetition stops being a fix
/// for the imbalance and starts being the only thing the class has to say.
const COPIES: usize = 12;

/// How many examples one spell template may contribute.
///
/// Twice the prompt's `CORPUS_CAP`, for the opposite reason: there the cap
/// fights a cross-product that made one verb 47% of the corpus, and here there
/// is one two-slot shape — `if {place} has {reagent}` — and it is the shape that
/// needs the pairs. At 24 it read 30.3% of its holdout while every one-slot
/// shape read 70–100%. [`balance`] holds the other end up.
const SPELL_CAP: usize = CORPUS_CAP * 2;

/// Bring the thin classes up towards the mean by repeating what they have.
///
/// The imbalance is arithmetic, not authorship: a shape's size is the size of
/// its expansion, and five of the eleven have nothing to expand over, so their
/// twenty authored phrasings *are* their class. `if {place} is idle` has the
/// same phrasings and twenty-five places to say them about, so it arrives forty
/// times larger — a fact about the noun tables, not about the language.
///
/// Repeated rather than reweighted, and both rather than either: the trainer
/// weights a class by rarity, clamped to 4× (§19), and forty-to-one is past
/// what that can answer. Repetition spreads the correction across batches
/// instead of concentrating it in one lurching row.
fn balance(samples: Vec<Sample>) -> Vec<Sample> {
    let mut count: std::collections::BTreeMap<u32, usize> = std::collections::BTreeMap::new();
    for sample in &samples {
        *count.entry(sample.verb).or_default() += 1;
    }
    let mean = samples.len() / count.len().max(1);

    let mut out = Vec::with_capacity(samples.len());
    for sample in samples {
        let held = count.get(&sample.verb).copied().unwrap_or(1).max(1);
        let copies = (mean / held).clamp(1, COPIES);
        out.extend(std::iter::repeat_n(sample, copies));
    }
    out
}

/// Only the lines a spell reader will ever be shown.
///
/// `Scribe` never sees a line `spell::reads_cleanly` accepts, so an
/// already-canonical refusal is input the model will never be given. It also
/// made the trainer's report a fiction — the refusal holdout read 40% while the
/// shipped scrivener left 97% of the same population untouched, so epoch
/// selection kept whichever pass did best at a question nobody asks.
fn unread(lines: Vec<String>) -> Vec<String> {
    lines
        .into_iter()
        .filter(|line| !orbs_sim::tower::spell::reads_cleanly(line))
        .collect()
}

/// Lines the reader is taught to leave alone.
fn rejected(lines: &[String], vocabulary: &Vocabulary) -> Vec<Sample> {
    lines
        .iter()
        .filter_map(|line| Sample::reject(line, vocabulary))
        .collect()
}

/// At most `cap` of `all`, spread across the whole list.
///
/// Strided rather than truncated, for `Phrasings::corpus_capped`'s reason: the
/// first `cap` of an expansion are every phrasing of the first template and none
/// of the rest.
fn thin<T>(all: Vec<T>, cap: usize) -> Vec<T> {
    if cap == 0 || all.len() <= cap {
        return all;
    }
    let stride = all.len().div_ceil(cap);
    all.into_iter().step_by(stride).take(cap).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_spell_corpus_is_not_three_quarters_commands() {
        // The skew this file exists to prevent: every phrasing of every verb
        // belongs to one class of twelve, and left whole it teaches the reader
        // that *command* is nearly always safe. The same defect as `move`
        // owning 47% of the prompt's corpus, one register along.
        let corpus = Corpus::spells(&Vocabulary::builtin());
        let command = u32::try_from(spelling::command()).expect("twelve classes");
        let commands = corpus
            .learn
            .iter()
            .filter(|sample| sample.verb == command)
            .count();
        println!(
            "{} examples, {commands} of them commands, {} refusals",
            corpus.learn.len(),
            corpus.refusals,
        );
        assert!(
            commands * 2 < corpus.learn.len(),
            "{commands} of {} examples are the command class",
            corpus.learn.len(),
        );
        assert!(corpus.refusals > 0, "nothing teaches it to leave a line be");
    }

    #[test]
    fn no_shape_is_forty_times_another() {
        // What `balance` is for. `end` has twenty authored phrasings and
        // nothing to expand over; `if {place} is idle` has twenty-odd and
        // twenty-five places to say them about. Left alone the class head
        // learns the noun tables rather than the language.
        let corpus = Corpus::spells(&Vocabulary::builtin());
        let mut counted: Vec<(String, usize)> = spelling::shapes()
            .iter()
            .enumerate()
            .map(|(at, shape)| {
                let class = u32::try_from(at).expect("twelve classes");
                (
                    shape.clone(),
                    corpus
                        .learn
                        .iter()
                        .filter(|sample| sample.verb == class)
                        .count(),
                )
            })
            .collect();
        counted.sort_by_key(|(_, held)| *held);
        for (shape, held) in &counted {
            println!("  {held:>5}  {shape}");
        }

        let (thinnest, fewest) = counted.first().cloned().expect("shapes exist");
        let (widest, most) = counted.last().cloned().expect("shapes exist");
        assert!(
            most <= fewest * 8,
            "{widest:?} has {most} examples and {thinnest:?} has {fewest}",
        );
    }

    #[test]
    fn every_shape_is_taught_and_every_shape_is_held_back() {
        // A class with no examples cannot be predicted, and one with no holdout
        // cannot be measured — both are silent, and both were real on the prompt
        // side (§19: seventeen of forty-six verbs had no template).
        let corpus = Corpus::spells(&Vocabulary::builtin());
        for (at, shape) in spelling::shapes().iter().enumerate() {
            let class = u32::try_from(at).expect("twelve classes");
            assert!(
                corpus.learn.iter().any(|sample| sample.verb == class),
                "{shape:?} has nothing to learn from",
            );
            assert!(
                corpus.holdout.iter().any(|sample| sample.verb == class),
                "{shape:?} has no holdout",
            );
        }
    }

    #[test]
    fn the_prompt_corpus_is_what_it_always_was() {
        // This module took the corpus construction out of `examples/train.rs`;
        // the numbers must not have moved with it.
        let corpus = Corpus::verbs(&Vocabulary::builtin());
        println!(
            "{} examples, {} of them refusals, {} held back",
            corpus.learn.len(),
            corpus.refusals,
            corpus.holdout.len(),
        );
        assert!(corpus.learn.len() > 5_000);
        assert!(corpus.refusals > 100);
        assert!(!corpus.refused.is_empty());
    }
}
