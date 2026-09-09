//! One training example, as numbers.
//!
//! # What the reader is asked to predict
//!
//! Two things, from one pass over the sentence:
//!
//! - **which verb**, read off the leading `<cls>` row — a classification over
//!   [`Verb::ALL`] plus a reject class for *"this is not a command"*;
//! - **which words fill which slot**, tagged per word.
//!
//! Everything else — binding `sage` to the sage that is actually on the shelf,
//! deciding whether `grind` means anything in this room — stays with
//! `parser::resolve`, which is deterministic and can explain itself. A reader
//! says what the sentence *meant*; the matcher says what it *refers to*.
//!
//! # Tagged by slot position, not by noun kind
//!
//! `move {reagent} {place}` needs to know which of two spans is the thing and
//! which is the destination, and `mix {reagent} {reagent}` has two spans of the
//! same kind. Kind cannot separate either. **Position can**, and it is also what
//! `Intent` is made of — an ordered list of arguments — so a prediction
//! reconstructs a command by concatenation and nothing has to be inferred back.
//!
//! # Why `<cls>` rather than pooling
//!
//! A mean over the sentence would let a long argument outvote the verb. The
//! leading row attends to everything and belongs to nothing, which is what makes
//! it free to carry *"what kind of sentence is this"*.

use orbs_sim::content::Example;
use orbs_sim::parser::Verb;

use crate::Vocabulary;

/// The longest sentence the reader will look at, `<cls>` included.
///
/// Matched to the parser's own `MAX_WORDS` of 32 rather than chosen: a line the
/// matcher refuses to score is not one a reader should be trained on, and a
/// reader that read further than the matcher would answer for input the game
/// never accepts.
pub const MAX_LEN: usize = 32;

/// The most arguments any verb takes.
///
/// `move <reagent> [source] <destination>` is the widest signature in
/// `Verb::ALL`, and a test holds that nothing has outgrown it.
pub const MAX_SLOTS: usize = 3;

/// The row `<pad>` sits at, for the rest of a short sentence.
const PAD: u32 = 0;

/// What a word is doing in the sentence.
///
/// Ordinary BIO tagging: a slot's first word `Begin`s it and the rest are
/// `Inside` it, so `castle gates` stays one argument rather than two.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tag {
    /// Not part of any argument — a verb word, filler, politeness.
    Outside,
    /// The first word of the argument filling this slot.
    Begin(usize),
    /// A later word of it.
    Inside(usize),
}

impl Tag {
    /// How many distinct tags there are, which is the slot head's width.
    pub const COUNT: usize = 1 + 2 * MAX_SLOTS;

    /// This tag's row in the head.
    ///
    /// # Panics
    ///
    /// If a slot index exceeds [`MAX_SLOTS`], which `the_widest_signature_still_fits`
    /// holds cannot happen for any verb in `Verb::ALL`.
    #[must_use]
    pub fn row(self) -> u32 {
        let slot = |slot: usize| u32::try_from(slot).expect("three slots, not four billion");
        match self {
            Self::Outside => 0,
            Self::Begin(at) => 1 + slot(at) * 2,
            Self::Inside(at) => 2 + slot(at) * 2,
        }
    }

    /// The tag a row names.
    #[must_use]
    pub fn from_row(row: u32) -> Self {
        let slot = |row: u32| usize::try_from(row).unwrap_or(0);
        match row {
            0 => Self::Outside,
            _ if row % 2 == 1 => Self::Begin(slot((row - 1) / 2)),
            _ => Self::Inside(slot((row - 2) / 2)),
        }
    }
}

/// The label meaning *"this is not a command I know"*.
///
/// **A sentinel in a [`Sample`], and no longer a class the verb head predicts.**
/// It used to be the forty-seventh output of one softmax, which put *"is there a
/// command here"* in direct competition with *"which command is it"* for a
/// single distribution's mass — and the two visibly traded against each other,
/// 38.6% refusal in one epoch and 93.2% in the next (§19). *Whether* is its own
/// question now with its own head, and this value only marks a training example
/// as belonging to it.
#[expect(
    clippy::cast_possible_truncation,
    reason = "`Verb::ALL` is a const array of 46; the cast is evaluated at compile time"
)]
pub const REJECT: u32 = Verb::ALL.len() as u32;

/// How many commands the verb head chooses between.
///
/// Exactly `Verb::ALL`: no reject, no spare row.
pub const VERBS: usize = Verb::ALL.len();

/// One example, as rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sample {
    /// `<cls>` then the sentence, padded to [`MAX_LEN`].
    pub tokens: Vec<u32>,
    /// How many of those are real, for masking the padding out.
    pub length: usize,
    /// Which verb, or [`REJECT`].
    pub verb: u32,
    /// One tag per row of [`tokens`](Self::tokens).
    pub tags: Vec<u32>,
}

impl Sample {
    /// Whether this example asks for anything.
    ///
    /// What the binary head is trained to answer, and what decides whether the
    /// verb head is shown this example at all — a sentence asking for nothing
    /// has no right verb, so training one into it is teaching noise.
    #[must_use]
    pub const fn is_command(&self) -> bool {
        self.verb != REJECT
    }
}

impl Sample {
    /// Encode a generated example.
    ///
    /// Returns [`None`] when the example cannot be represented — a canonical
    /// naming a verb outside `Verb::ALL`, or a sentence past [`MAX_LEN`]. Both
    /// are dropped rather than truncated: a half-tagged sentence teaches the
    /// reader that an argument stops where the buffer did.
    #[must_use]
    pub fn encode(example: &Example, vocabulary: &Vocabulary) -> Option<Self> {
        let verb = verb_of(&example.canonical)?;

        let words: Vec<(usize, &str)> = example.said.split_whitespace().fold(
            Vec::new(),
            |mut found: Vec<(usize, &str)>, word| {
                // Byte offsets, so a span from the generator can be matched to
                // the word it covers. `split_whitespace` does not give them.
                let from = found
                    .last()
                    .map_or(0, |(at, previous): &(usize, &str)| at + previous.len());
                let at = example.said[from..]
                    .find(word)
                    .map_or(from, |offset| from + offset);
                found.push((at, word));
                found
            },
        );
        if words.len() + 1 > MAX_LEN {
            return None;
        }

        let mut tokens = Vec::with_capacity(MAX_LEN);
        let mut tags = Vec::with_capacity(MAX_LEN);
        tokens.push(vocabulary.token("<cls>").row());
        tags.push(Tag::Outside.row());

        for (at, word) in &words {
            tokens.push(vocabulary.token(word).row());
            tags.push(tag_for(*at, word.len(), example).row());
        }

        let length = tokens.len();
        tokens.resize(MAX_LEN, PAD);
        tags.resize(MAX_LEN, Tag::Outside.row());

        Some(Self {
            tokens,
            length,
            verb,
            tags,
        })
    }

    /// An example of a sentence that is not a command at all.
    ///
    /// **Negatives are half the job.** A reader trained only on commands has
    /// never been shown a sentence it should refuse, and answers one anyway —
    /// which is the dead end §15 weighs heaviest. `<cls>` predicts [`REJECT`]
    /// and every word is [`Tag::Outside`].
    #[must_use]
    pub fn reject(said: &str, vocabulary: &Vocabulary) -> Option<Self> {
        let rows = vocabulary.encode(said);
        if rows.len() > MAX_LEN {
            return None;
        }
        let length = rows.len();
        let mut tokens: Vec<u32> = rows.iter().map(|token| token.row()).collect();
        tokens.resize(MAX_LEN, PAD);

        Some(Self {
            tokens,
            length,
            verb: REJECT,
            tags: vec![Tag::Outside.row(); MAX_LEN],
        })
    }
}

/// The verb a canonical command names.
fn verb_of(canonical: &str) -> Option<u32> {
    let head = canonical.split_whitespace().next()?;
    Verb::ALL
        .iter()
        .position(|verb| verb.canonical() == head)
        .map(|at| u32::try_from(at).unwrap_or(REJECT))
}

/// Which slot, if any, the word at `at` belongs to.
fn tag_for(at: usize, len: usize, example: &Example) -> Tag {
    for (slot, span) in example.spans.iter().enumerate().take(MAX_SLOTS) {
        if at >= span.at.start && at + len <= span.at.end {
            return if at == span.at.start {
                Tag::Begin(slot)
            } else {
                Tag::Inside(slot)
            };
        }
    }
    Tag::Outside
}

#[cfg(test)]
mod tests {
    use super::*;
    use orbs_sim::content::{Phrasings, corpus_scene};

    fn one(said: &str) -> Sample {
        let phrasings = Phrasings::builtin();
        let scene = corpus_scene();
        let example = phrasings
            .corpus(&scene)
            .into_iter()
            .find(|example| example.said == said)
            .unwrap_or_else(|| panic!("{said:?} is not in the corpus"));
        Sample::encode(&example, &Vocabulary::builtin()).expect("encodes")
    }

    #[test]
    fn the_widest_signature_still_fits() {
        // `MAX_SLOTS` is a fact about `Verb::ALL`, not a guess. A verb that grew
        // a fourth argument would silently lose it in tagging.
        let widest = Verb::ALL
            .iter()
            .map(|verb| verb.signature().len())
            .max()
            .expect("verbs exist");
        assert!(widest <= MAX_SLOTS, "a verb takes {widest} arguments");
    }

    #[test]
    fn a_sentence_opens_with_cls_and_ends_in_padding() {
        let sample = one("smash the sage");
        assert_eq!(sample.tokens.len(), MAX_LEN);
        assert_eq!(sample.tags.len(), MAX_LEN);
        assert_eq!(sample.length, 4, "<cls> and three words");
        assert!(sample.tokens[sample.length..].iter().all(|row| *row == PAD));
    }

    #[test]
    fn the_argument_is_tagged_and_the_rest_is_not() {
        // `smash the sage` -> `grind sage`: only `sage` fills a slot.
        let sample = one("smash the sage");
        let tags: Vec<Tag> = sample.tags[..sample.length]
            .iter()
            .map(|row| Tag::from_row(*row))
            .collect();
        assert_eq!(
            tags,
            vec![Tag::Outside, Tag::Outside, Tag::Outside, Tag::Begin(0)]
        );
    }

    #[test]
    fn the_verb_is_the_head_of_the_canonical_form() {
        let sample = one("smash the sage");
        assert_eq!(
            Verb::ALL[sample.verb as usize],
            Verb::Grind,
            "the reader would learn the wrong command"
        );
    }

    #[test]
    fn a_refusal_names_no_verb_and_tags_nothing() {
        let sample =
            Sample::reject("what should i do next", &Vocabulary::builtin()).expect("encodes");
        assert_eq!(sample.verb, REJECT);
        assert!(sample.tags.iter().all(|row| *row == Tag::Outside.row()));
    }

    #[test]
    fn every_tag_round_trips_through_its_row() {
        // The heads read rows back; a tag that did not survive the trip would
        // silently retag a whole corpus.
        for slot in 0..MAX_SLOTS {
            for tag in [Tag::Begin(slot), Tag::Inside(slot)] {
                assert_eq!(Tag::from_row(tag.row()), tag);
            }
        }
        assert_eq!(Tag::from_row(Tag::Outside.row()), Tag::Outside);
        assert!(Tag::COUNT > Tag::Inside(MAX_SLOTS - 1).row() as usize);
    }

    #[test]
    fn the_whole_corpus_encodes() {
        // **The number that says whether the reader can be trained at all**, and
        // a silent drop here would shrink the corpus without saying so.
        let vocabulary = Vocabulary::builtin();
        let scene = corpus_scene();
        let corpus = Phrasings::builtin().corpus(&scene);
        let encoded = corpus
            .iter()
            .filter_map(|example| Sample::encode(example, &vocabulary))
            .count();
        println!("encoded {encoded} of {} corpus examples", corpus.len());
        assert!(
            encoded * 100 >= corpus.len() * 99,
            "only {encoded} of {} examples encode",
            corpus.len()
        );
    }
}
