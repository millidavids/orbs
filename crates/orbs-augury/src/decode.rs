//! One line in, one forward pass, and the three answers that come back out.
//!
//! # Shared, because the two registers ask the same question
//!
//! *Is anything being asked for; which of the classes is it; which words are its
//! argument.* The prompt's reader turns that into a command and the spell's into
//! a statement, but the tensor work between the sentence and those three answers
//! is identical — and a second copy of it is how the two would come to disagree
//! about, say, which row a tag belongs to. §19 records that shape repeatedly.

use burn::prelude::*;

use crate::{Batch, MAX_SLOTS, Reader, Sample, Tag, Vocabulary};

/// What one pass over one line concluded.
#[derive(Debug, Clone)]
pub struct Decoded {
    /// Whether the binary head found anything here to answer.
    ///
    /// **Read this first.** The class head always names its best guess — it has
    /// no way not to — so on a sentence that asks for nothing that guess means
    /// nothing.
    pub answering: bool,
    /// Class rows, best first: the whole ranking rather than the argmax.
    pub ranked: Vec<usize>,
    /// The words filling each slot the tagger marked, in slot order.
    ///
    /// Empty slots are dropped rather than kept as blanks, so this is *what was
    /// found*, and a caller compares its length against what its own answer
    /// takes.
    pub slots: Vec<String>,
    /// The same words **indexed by slot, with the empties kept**.
    ///
    /// What the spell register reads, because it numbers slots by kind — a place
    /// is slot 0 and a name slot 2 whether or not a reagent sits between them —
    /// and compacting would move a name into the reagent's place. See
    /// `spelling::slot_for`.
    pub placed: Vec<Option<String>>,
}

/// Read one line with `reader`, over `classes` answers.
///
/// [`None`] when the line cannot be encoded at all — past `MAX_LEN`, or empty.
#[must_use]
pub fn decode<B: Backend>(
    reader: &Reader<B>,
    vocabulary: &Vocabulary,
    device: &B::Device,
    line: &str,
    classes: usize,
) -> Option<Decoded> {
    let sample = Sample::reject(line, vocabulary)?;
    let batch = Batch::<B>::of(std::slice::from_ref(&sample), device);
    let reading = reader.forward(batch.tokens, batch.pad);

    // **Whether, before which.** Row 1 is *"there is something here"*.
    let answering = reading
        .command
        .argmax(1)
        .reshape([1])
        .into_scalar()
        .to_i64()
        == 1;

    let [rows, width, _] = reading.tags.dims();
    let tags: Vec<i64> = reading
        .tags
        .argmax(2)
        .reshape([rows * width])
        .into_data()
        .convert::<i64>()
        .into_vec()
        .unwrap_or_default();

    // Words in the order the sentence had them, gathered per slot. The sample's
    // rows are `<cls>` then one per word, so the tag at row `n + 1` belongs to
    // word `n`.
    // Each as an argument holds it — see `slot_word`.
    let mut slots: Vec<Vec<String>> = vec![Vec::new(); MAX_SLOTS];
    for (at, word) in line.split_whitespace().enumerate() {
        let Some(row) = tags.get(at + 1) else { break };
        match Tag::from_row(u32::try_from(*row).unwrap_or(0)) {
            Tag::Outside => {}
            Tag::Begin(slot) | Tag::Inside(slot) => {
                let word = slot_word(word);
                if let Some(found) = slots.get_mut(slot)
                    && !word.is_empty()
                {
                    found.push(word);
                }
            }
        }
    }

    // The class head's whole ranking, not just its argmax. A batch of one, so
    // this is a few dozen floats and an ordinary sort.
    let scores: Vec<f32> = reading
        .verb
        .reshape([classes])
        .into_data()
        .convert::<f32>()
        .into_vec()
        .unwrap_or_default();
    if scores.len() != classes {
        // The one shape this can take is a silent `into_vec` type mismatch,
        // which has cost this crate a day once already. Answering nothing falls
        // through to whatever the caller does without a reader, rather than
        // answering with row zero.
        return None;
    }
    let mut ranked: Vec<usize> = (0..classes).collect();
    ranked.sort_by(|a, b| scores[*b].total_cmp(&scores[*a]));

    Some(Decoded {
        answering,
        ranked,
        placed: slots
            .iter()
            .map(|slot| (!slot.is_empty()).then(|| slot.join(" ")))
            .collect(),
        slots: slots
            .iter()
            .filter(|slot| !slot.is_empty())
            .map(|slot| slot.join(" "))
            .collect(),
    })
}

/// A tagged word as an argument holds it: without the sentence's punctuation
/// on its ends, or a trailing `'s`.
///
/// **The tagger marks words, and the word it marked had the comma on.** *"grind
/// the sage, please"* read as `grind sage,`. This sheds what the parser sheds
/// before matching — `.,!;:?` — with the quotes and brackets around a word, and
/// **keeps `-` and `_` at the end** as well as inside. `rock-salt` and
/// `mortar_and_pestle` are names with them in, and `sage-` is sabotage: the
/// sigil `tower::claimed` appends precisely so the resolver will not
/// fold the lie back, which a reader must not do either — `fold_word` would.
///
/// Case is left alone. A slot can be a spell's name, and a file is found by the
/// name the player gave it.
fn slot_word(word: &str) -> String {
    let word = word.replace('\u{2019}', "'");
    let trimmed = word
        .trim_start_matches(|ch: char| !ch.is_alphanumeric())
        .trim_end_matches(|ch: char| !ch.is_alphanumeric() && !matches!(ch, '-' | '_'));
    let cut = crate::vocabulary::CONTRACTIONS.iter().find_map(|ending| {
        let at = trimmed.len().checked_sub(ending.len())?;
        trimmed
            .get(at..)
            .filter(|tail| tail.eq_ignore_ascii_case(ending))
            .map(|_| at)
    });
    cut.map_or(trimmed, |at| &trimmed[..at]).to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_slot_keeps_its_word_and_sheds_the_sentence() {
        assert_eq!(slot_word("sage,"), "sage");
        assert_eq!(slot_word("(sage)."), "sage");
        assert_eq!(slot_word("\"sage\""), "sage");
        assert_eq!(slot_word("mortar's"), "mortar");
        assert_eq!(slot_word("mortar\u{2019}s"), "mortar");
        assert_eq!(slot_word("rock-salt,"), "rock-salt");
        assert_eq!(slot_word("mortar_and_pestle."), "mortar_and_pestle");
        assert_eq!(
            slot_word("Morning"),
            "Morning",
            "a spell's name lost its case"
        );
        assert_eq!(slot_word("..."), "");
    }

    #[test]
    fn the_sabotage_sigil_is_never_folded_back() {
        // `grind sage-` is the enemy's lie, and it has to run as one. A reader
        // that trimmed the sigil would repair a sabotaged spell the moment
        // anything read it again — which, after a load, is the next save.
        assert_eq!(slot_word(&orbs_sim::tower::claimed("sage")), "sage-");
    }
}
