//! Every word the reader knows, built from the game's own content.
//!
//! # No downloaded tokenizer, and this is where that rule becomes code
//!
//! DESIGN.md §19 (*the augury*): nothing is pulled from the internet — no
//! pretrained weights, no pretrained embeddings, and **no downloaded tokenizer
//! or BPE vocabulary**. So the vocabulary is assembled here from four things
//! this repository already contains:
//!
//! - every word of every synonym in `parser::SYNONYMS`, all three registers
//! - every canonical verb, and every spell word
//! - every substance and material the content tables name
//! - every word the authored templates in `content/phrasings.toml` and
//!   `content/spellings.toml` use
//!
//! That is the whole of the language the game speaks. A word outside it is a
//! word the player invented, and [`Token::Hashed`] is what happens to it.
//!
//! # Why a hash rather than one `<unk>`
//!
//! A single unknown token throws away everything about a word the reader has
//! not seen — and with no pretrained embeddings underneath, unseen words are
//! common rather than exotic. Hashing **character trigrams** keeps the shape:
//! `powdered` and `powder` collide in most of their buckets, so a model can
//! learn that they behave alike without either being in the vocabulary.
//!
//! It is the cheapest substitute for the subword vocabulary a pretrained
//! encoder would have brought, and it exists because the provenance rule means
//! there is no such encoder.

use std::collections::BTreeMap;

use orbs_sim::content::{Materials, Phrasings, Recipes};
use orbs_sim::parser::{SYNONYMS, SpellWord, Verb};

/// How many buckets unknown words are hashed into.
///
/// **Small on purpose.** Each bucket is a row of the embedding table that every
/// unseen word sharing it has to agree on, so a large number is mostly dead
/// weight in a model this size while a small one makes near-neighbours collide,
/// which is the point.
pub const BUCKETS: usize = 64;

/// Reserved rows, before any word of the game's own.
const RESERVED: &[&str] = &["<pad>", "<cls>", "<unk>"];

/// What a word turns into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    /// A word the game knows, at its row in the table.
    Known(u32),
    /// A word it does not, at one of [`BUCKETS`] rows shared by its shape.
    Hashed(u32),
}

impl Token {
    /// The row this token indexes, whichever kind it is.
    #[must_use]
    pub const fn row(self) -> u32 {
        match self {
            Self::Known(row) | Self::Hashed(row) => row,
        }
    }

    /// Whether the game had a word for this.
    #[must_use]
    pub const fn is_known(self) -> bool {
        matches!(self, Self::Known(_))
    }
}

/// Every word the reader knows, and the rows they sit at.
#[derive(Debug, Clone)]
pub struct Vocabulary {
    /// Row order, for a stable table the weights are trained against.
    types: Vec<String>,
    index: BTreeMap<String, u32>,
}

impl Default for Vocabulary {
    fn default() -> Self {
        Self::builtin()
    }
}

impl Vocabulary {
    /// Assembled from the content compiled into the binary.
    ///
    /// **Sorted, so the row a word sits at does not depend on the order four
    /// tables were walked in.** Weights are trained against these indices; a
    /// vocabulary that reshuffled when a synonym was added would silently
    /// invalidate every one of them, and nothing would say so.
    ///
    /// # Panics
    ///
    /// If the vocabulary exceeds `u32::MAX` rows, which would mean the content
    /// tables had grown by seven orders of magnitude.
    #[must_use]
    pub fn builtin() -> Self {
        let mut words: Vec<String> = Vec::new();

        for synonym in SYNONYMS {
            words.extend(synonym.words.iter().map(|word| (*word).to_owned()));
        }
        words.extend(Verb::ALL.iter().map(|verb| verb.canonical().to_owned()));
        words.extend(
            SpellWord::ALL
                .iter()
                .map(|word| word.canonical().to_owned()),
        );
        words.extend(
            Recipes::builtin()
                .vocabulary()
                .into_iter()
                .map(str::to_owned),
        );
        words.extend(Materials::builtin().names().into_iter().map(str::to_owned));

        // The templates' own words — everything a phrasing says that is not a
        // slot. This is what makes the corpus and the vocabulary agree by
        // construction rather than by anyone remembering to keep them in step.
        //
        // **Both corpora, one table.** The spell register shares this
        // vocabulary — a sentence is a sentence, and `mortar_and_pestle` means
        // the same in a spell as at the prompt — so a word authored in
        // `spellings.toml` and missing here would reach the reader as a hash
        // bucket, which is the treatment reserved for words the game does not
        // have.
        //
        // ⚠ **Left as written, and the holdouts harvested too — both measured
        // and recorded rather than changed.** Folding `times,` into `times`, and
        // leaving out the 77 words only a holdout says (rows nothing trains),
        // are each right on paper; each also changes the table's size, which
        // re-seeds every row, and one retrain cannot tell the fix from the new
        // seed. DESIGN.md §19, *The review*, has the two runs.
        let phrasings = Phrasings::builtin();
        let spellings = Phrasings::spellings();
        for entry in phrasings.entries().iter().chain(spellings.entries()) {
            for line in entry
                .say
                .iter()
                .chain(&entry.holdout)
                .chain(core::iter::once(&entry.canonical))
            {
                words.extend(
                    line.split_whitespace()
                        .filter(|token| !token.starts_with('{'))
                        .map(str::to_lowercase),
                );
            }
        }
        for refusal in phrasings.refusals().iter().chain(spellings.refusals()) {
            for line in refusal.say.iter().chain(&refusal.holdout) {
                words.extend(
                    line.split_whitespace()
                        .filter(|token| !token.starts_with('{'))
                        .map(str::to_lowercase),
                );
            }
        }

        // **A set's name is a word the game has**, unlike a name a spell binds:
        // `for each way` walks a fixture the tower raised, so its word gets a
        // row. The `names` a `{name}` slot expands over are left out on purpose
        // — a player's own name arrives as a hash bucket, and so must the ones
        // the reader learns from.
        words.extend(spellings.groups().iter().map(|group| group.to_lowercase()));

        words.sort();
        words.dedup();

        let types: Vec<String> = RESERVED
            .iter()
            .map(|word| (*word).to_owned())
            .chain(words)
            .collect();
        let index = types
            .iter()
            .enumerate()
            .map(|(row, word)| {
                (
                    word.clone(),
                    u32::try_from(row).expect("a vocabulary of thousands, not billions"),
                )
            })
            .collect();

        Self { types, index }
    }

    /// How many rows the embedding table needs, hashed buckets included.
    #[must_use]
    pub const fn rows(&self) -> usize {
        self.types.len() + BUCKETS
    }

    /// How many words the game actually has a word for.
    #[must_use]
    pub const fn known(&self) -> usize {
        self.types.len()
    }

    /// The row `word` sits at, known or hashed.
    ///
    /// # A word is read without its punctuation or its contraction
    ///
    /// **One word in, one row out**, and the one it gets is the word a reader
    /// should see. *"free."* and *"else,"* were rows of their own — unknown words,
    /// hashed — so a line that differed from a corpus line by a full stop read as
    /// a sentence about something the game had never heard of: punctuation read
    /// 36% of the trials that carry it. Trailing and leading punctuation go;
    /// `'s`, `'re`, `'ll`, `'ve`, `'d` and `'m` go; and a word ending *n't*
    /// becomes `not`, because the negation is what that word is for.
    ///
    /// One row per word, still, because the tagger's rows are the sentence's
    /// words: splitting *"isn't"* into two would move every span after it.
    ///
    /// # The word as the table has it, first
    ///
    /// **Folding first took the brackets off `<cls>`.** Every training sentence
    /// was encoded through `token("<cls>")`, which folded to `cls`, an unknown
    /// word — while refusals, and every line at inference, opened on the real
    /// `<cls>` row. The reader learned that the opening row alone said whether
    /// anything was being asked, scored 98.6% inside the trainer on exactly that,
    /// and refused every line a player typed. A reserved row, and a synonym like
    /// `what's`, is a word with punctuation *in* it; looking the word up as
    /// written before folding it is what keeps both.
    #[must_use]
    pub fn token(&self, word: &str) -> Token {
        if let Some(row) = self.index.get(&word.to_lowercase()) {
            return Token::Known(*row);
        }
        let folded = fold_word(word);
        if let Some(row) = self.index.get(&folded) {
            return Token::Known(*row);
        }
        let base = u32::try_from(self.types.len()).unwrap_or(u32::MAX);
        Token::Hashed(base + shape_of(&folded))
    }

    /// A line, as rows, with the leading `<cls>` the verb head reads.
    #[must_use]
    pub fn encode(&self, line: &str) -> Vec<Token> {
        core::iter::once(Token::Known(1))
            .chain(line.split_whitespace().map(|word| self.token(word)))
            .collect()
    }

    /// Whether `word` is one slip from a word the table has — a letter
    /// dropped, added or changed, or two side by side swapped.
    ///
    /// **What tells a typo from a name**, with the tagger's help. Both arrive as
    /// hash buckets — *"teh"* and *"bertha"* alike — but a typo is a word the
    /// game has with one thing wrong in it. `spelling::names_in` is the use.
    ///
    /// **Three letters, a swap only.** At three a changed letter makes another
    /// word as often as a typo — *"pip"* is one from *"tip"*, and it is a name —
    /// while two letters swapped is a slip of the hand: *"teh"*. At two almost
    /// everything is one slip from something, so nothing is.
    #[must_use]
    pub fn near(&self, word: &str) -> bool {
        let word: Vec<char> = fold_word(word).chars().collect();
        if word.len() < 3 {
            return false;
        }
        self.types.iter().skip(RESERVED.len()).any(|known| {
            let known: Vec<char> = known.chars().collect();
            if word.len() == 3 {
                swapped(&word, &known)
            } else {
                one_slip(&word, &known)
            }
        })
    }
}

/// Whether `a` and `b` differ by exactly one slip of the hand — see
/// [`Vocabulary::near`].
fn one_slip(a: &[char], b: &[char]) -> bool {
    match a.len().cmp(&b.len()) {
        core::cmp::Ordering::Equal => {
            (0..a.len()).filter(|at| a[*at] != b[*at]).count() == 1 || swapped(a, b)
        }
        core::cmp::Ordering::Less => dropped_one(b, a),
        core::cmp::Ordering::Greater => dropped_one(a, b),
    }
}

/// Whether `a` is `b` with two letters side by side swapped.
fn swapped(a: &[char], b: &[char]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let differ: Vec<usize> = (0..a.len()).filter(|at| a[*at] != b[*at]).collect();
    matches!(differ.as_slice(), [first, second]
        if *second == first + 1 && a[*first] == b[*second] && a[*second] == b[*first])
}

/// Whether `short` is `long` with one letter taken out.
fn dropped_one(long: &[char], short: &[char]) -> bool {
    long.len() == short.len() + 1
        && (0..long.len()).any(|skip| {
            long.iter()
                .enumerate()
                .filter(|(at, _)| *at != skip)
                .map(|(_, letter)| letter)
                .eq(short.iter())
        })
}

/// Endings that say nothing a spell or a command can use.
///
/// Shared with `spelling::unfold`, which reads a line the same way for the
/// checks a reading has to pass — the reader and the checks must agree on what
/// a word is, or a line could be read one way and judged another.
pub(crate) const CONTRACTIONS: [&str; 6] = ["'s", "'re", "'ll", "'ve", "'d", "'m"];

/// A written word as the one word a reader should see.
///
/// See [`Vocabulary::token`]. `_` and `-` survive inside a word, because
/// `mortar_and_pestle` and `rock-salt` are single names with them in.
#[must_use]
pub fn fold_word(word: &str) -> String {
    let word = word
        .replace('\u{2019}', "'")
        .trim_matches(|ch: char| !ch.is_alphanumeric() && ch != '\'')
        .trim_start_matches('\'')
        .to_lowercase();
    if word.ends_with("n't") {
        return "not".to_owned();
    }
    CONTRACTIONS
        .iter()
        .find_map(|ending| word.strip_suffix(ending))
        .unwrap_or(&word)
        .trim_end_matches('\'')
        .to_owned()
}

/// Which bucket a word's *shape* falls in.
///
/// Character trigrams over the padded word, summed — so words sharing most of
/// their trigrams land together more often than not. Deliberately crude: the
/// job is to keep `powdered` near `powder`, not to be a hash function.
fn shape_of(word: &str) -> u32 {
    let padded: Vec<char> = core::iter::once('^')
        .chain(word.chars())
        .chain(core::iter::once('$'))
        .collect();
    let mut total: u32 = 0;
    for window in padded.windows(3) {
        let mut trigram: u32 = 2_166_136_261;
        for character in window {
            trigram ^= *character as u32;
            trigram = trigram.wrapping_mul(16_777_619);
        }
        total = total.wrapping_add(trigram);
    }
    total % u32::try_from(BUCKETS).unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_game_speaks_a_small_language() {
        // A number worth knowing rather than a bound worth enforcing: it sizes
        // the embedding table, and the embedding table is most of the model.
        let vocabulary = Vocabulary::builtin();
        println!(
            "known {} + {BUCKETS} buckets = {} rows",
            vocabulary.known(),
            vocabulary.rows()
        );
        assert!(vocabulary.known() > 200, "suspiciously few words");
        assert!(vocabulary.known() < 5_000, "suspiciously many");
    }

    #[test]
    fn every_canonical_verb_is_a_word_it_knows() {
        // The mastery arc's words cannot be the ones the reader has to guess at.
        let vocabulary = Vocabulary::builtin();
        for verb in Verb::ALL {
            assert!(
                vocabulary.token(verb.canonical()).is_known(),
                "{} is unknown to the reader",
                verb.canonical()
            );
        }
    }

    #[test]
    fn every_word_the_templates_use_is_known() {
        // The corpus and the vocabulary are built from each other, so a template
        // can never introduce a word the reader has no row for.
        let vocabulary = Vocabulary::builtin();
        for entry in Phrasings::builtin().entries() {
            for line in entry.say.iter().chain(&entry.holdout) {
                for word in line.split_whitespace().filter(|w| !w.starts_with('{')) {
                    assert!(
                        vocabulary.token(&word.to_lowercase()).is_known(),
                        "{word:?} is unknown"
                    );
                }
            }
        }
    }

    #[test]
    fn a_word_it_has_never_seen_keeps_its_shape() {
        // The substitute for the subword vocabulary a pretrained encoder would
        // have brought, and the reason it is a hash rather than one `<unk>`.
        let vocabulary = Vocabulary::builtin();
        let invented = vocabulary.token("zzzqqxx");
        assert!(!invented.is_known());
        assert!(invented.row() >= u32::try_from(vocabulary.known()).expect("small"));
        assert!(invented.row() < u32::try_from(vocabulary.rows()).expect("small"));
    }

    #[test]
    fn the_table_is_stable_across_builds() {
        // Weights are trained against these indices. A vocabulary that
        // reshuffled when a synonym was added would invalidate every one of them
        // and nothing would say so.
        let first = Vocabulary::builtin();
        let second = Vocabulary::builtin();
        assert_eq!(first.types, second.types);
        assert_eq!(first.token("grind"), second.token("grind"));
    }

    #[test]
    fn a_word_is_read_without_its_punctuation_or_its_contraction() {
        // A full stop is not a new word, and *"isn't"* is a negation.
        let vocabulary = Vocabulary::builtin();
        assert_eq!(vocabulary.token("free."), vocabulary.token("free"));
        assert_eq!(vocabulary.token("else,"), vocabulary.token("else"));
        assert_eq!(vocabulary.token("Alembic's"), vocabulary.token("alembic"));
        assert_eq!(vocabulary.token("isn't"), vocabulary.token("not"));
        assert!(vocabulary.token("not").is_known());
        // ...and a name with `_` or `-` inside it is still one name.
        assert_eq!(fold_word("rock-salt,"), "rock-salt");
        assert_eq!(fold_word("mortar_and_pestle."), "mortar_and_pestle");
    }

    #[test]
    fn a_typo_is_near_a_word_the_game_has_and_a_name_is_not() {
        let vocabulary = Vocabulary::builtin();
        for typo in ["teh", "grnd", "otherwsie", "tiems", "pasue", "taht"] {
            assert!(vocabulary.near(typo), "{typo:?} is no slip of anything");
        }
        // ...and a short name is not: *pip* is one changed letter from *tip*.
        for name in ["bertha", "zeph", "pip", "cog"] {
            assert!(!vocabulary.near(name), "{name:?} reads as a typo");
        }
    }

    #[test]
    fn a_line_arrives_with_its_cls() {
        let vocabulary = Vocabulary::builtin();
        let encoded = vocabulary.encode("smash the sage");
        assert_eq!(encoded.len(), 4, "three words and a <cls>");
        assert_eq!(encoded[0], Token::Known(1));
    }
}
