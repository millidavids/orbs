//! What may come next here — one answer, three surfaces.
//!
//! The prompt's Tab listing, the prompt's ghost and the spell editor's Tab all
//! ask *given this line and this caret, what could go there?* One function, so
//! none holds a second opinion (§19).
//!
//! The scribing guide asks *what should I teach you* and lists canonical names
//! through [`spell_vocabulary`](crate::spell_vocabulary); this offers all three
//! registers. They share everything that could drift: [`spell_may`],
//! `Scene::offers` and [`SpellWord::shape`].
//!
//! It answers what the grammar permits, never whether a word is any good:
//! offering a word the orb then refuses is §15's dead end.

use core::ops::Range;

use orbs_render::{Lexeme, char_index};

use super::question::State as SpellState;
use super::scene::Scene;
use super::spellword::SpellWord;
use super::verb::{NounKind, Verb};
use super::vocabulary::SYNONYMS;

/// Where the caret is standing.
#[derive(Debug, Clone, Copy)]
pub struct Situation<'a> {
    /// The live vocabulary — for a spell, the domain the file belongs to, which
    /// is not always the room the player is standing in.
    pub scene: &'a Scene,
    /// Whether this line is a line of a spell rather than a command — which
    /// has words of its own and never sees §6's numbered prompt.
    pub spell: bool,
    /// Whether §6's numbered prompt is waiting for a digit, which turns the
    /// answer off: here, so no surface has to remember to.
    pub prompt_open: bool,
    /// The blocks open above this line, innermost last, from [`open_blocks`].
    /// Empty at the prompt.
    pub open: &'a [SpellWord],
    /// The sets `for each` may walk here — `way` in the archive, `socket` and
    /// `sigil` in the lens.
    ///
    /// Not derivable from the [`Scene`]: a set is declared by the fixture
    /// (`tower::groups_at`) and is never a `Noun`. Empty at the prompt.
    pub sets: &'a [String],
}

/// Why something is being offered.
///
/// Not [`Lexeme`]: `is`, `has`, `be` and `each` are grammar, yet `lex` draws
/// them as names, and a completer must not disagree with the painter. So
/// [`kind`](Self::kind) derives the drawing from the reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    /// One of the language's own words — `repeat`, `if`, `end`.
    Word,
    /// A verb the tower answers to.
    Verb,
    /// Something the tower has — a reagent, an instrument, a place.
    Thing,
    /// A word the grammar fixes in this position — `is`, `has`, `be`, `each`.
    Grammar,
    /// One of the three states a place reports, or a spelling of one.
    State,
    /// A reading a place publishes.
    Reading,
}

impl Reason {
    /// How a surface should draw it — the painter's answer, not a second one.
    #[must_use]
    pub const fn kind(self) -> Lexeme {
        match self {
            Self::Word => Lexeme::Control,
            Self::Verb => Lexeme::Verb,
            // A grammar word, a state and a reading are all ordinary words to
            // `lex`, which classifies only what it can see on the line.
            Self::Thing | Self::Grammar | Self::State | Self::Reading => Lexeme::Name,
        }
    }
}

/// One thing that could go where the caret is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expected {
    /// What a list shows and what a Tab inserts, which are the same string.
    pub text: String,
    /// Why it is here — so a surface can group it, head it, or explain it
    /// without asking the grammar a second time.
    pub why: Reason,
    /// What follows it, for a surface with room to say — `<count>`, `place`.
    /// Empty where nothing follows.
    pub shape: &'static str,
}

impl Expected {
    /// How to draw it. See [`Reason::kind`].
    #[must_use]
    pub const fn kind(&self) -> Lexeme {
        self.why.kind()
    }

    /// A candidate with nothing following it, which is most of them.
    fn plain(text: impl Into<String>, why: Reason) -> Self {
        Self {
            text: text.into(),
            why,
            shape: "",
        }
    }
}

/// Everything that could go where the caret is.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Expectation {
    /// The byte range of the line this replaces — the partial word.
    ///
    /// A range rather than an append, because §6's parser is fuzzy and a
    /// near-miss is what this game expects.
    pub replaces: Range<usize>,
    /// The candidates, in a stable order — verbs alphabetical, nouns in scene
    /// order, control words in the language's own order.
    pub expected: Vec<Expected>,
}

impl Expectation {
    /// Whether there is anything to offer.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.expected.is_empty()
    }

    /// Just the strings, for a caller that wants nothing else.
    #[must_use]
    pub fn texts(&self) -> Vec<String> {
        self.expected.iter().map(|one| one.text.clone()).collect()
    }

    /// The text every candidate agrees on.
    ///
    /// readline's `compute_lcd_of_matches`. Tab spends it and the ghost draws
    /// it, so one copy keeps the ghost promising only what Tab does.
    #[must_use]
    pub fn common(&self) -> String {
        let mut rest = self.expected.iter();
        let Some(first) = rest.next() else {
            return String::new();
        };
        let mut shared: Vec<char> = first.text.chars().collect();
        for other in rest {
            let keep = shared
                .iter()
                .zip(other.text.chars())
                .take_while(|(a, b)| **a == *b)
                .count();
            shared.truncate(keep);
        }
        shared.into_iter().collect()
    }
}

/// What could go at `caret`.
#[must_use]
pub fn expect(line: &str, caret: usize, at: &Situation<'_>) -> Expectation {
    if at.prompt_open {
        return Expectation::default();
    }

    let upto = char_index(line, caret);
    let word = word_start(line, upto);
    let partial = &line[word..upto];
    let replaces = word..upto;

    // The first phrase is a verb — or, in a spell, one of the language's own
    // words. Anything later fills a slot of whichever it was.
    let leading = line[..word].trim();
    let expected = if leading.is_empty() {
        let mut out = if at.spell {
            control(partial, at.open)
        } else {
            Vec::new()
        };
        out.extend(verbs(partial, at.scene, at.spell));
        out
    } else if let Some(word) = at.spell.then(|| opening(leading)).flatten() {
        // A line opening with one of the language's own words is the
        // language's to finish, not a verb's.
        after(word, leading, partial, at)
    } else {
        match verb_of(leading, at.scene) {
            Some((verb, filled)) => nouns(verb, filled, partial, at.scene),
            None => Vec::new(),
        }
    };

    Expectation { replaces, expected }
}

/// The language's own word this line opens with, if it opens with one.
fn opening(leading: &str) -> Option<SpellWord> {
    let first = leading.split_whitespace().next()?;
    SpellWord::ALL
        .into_iter()
        .find(|word| word.canonical() == first.to_lowercase())
}

/// What the blocks open at this line allow to be written next.
///
/// A stack, not a count: `else` is legal only directly inside an `if`. A bare
/// `else` ends its ladder, so it replaces the `if`; `else if` chains and leaves
/// it.
#[must_use]
pub fn open_blocks(before: &[String]) -> Vec<SpellWord> {
    let mut open = Vec::new();
    for line in before {
        let trimmed = line.trim();
        let Some(word) = opening(trimmed) else {
            continue;
        };
        match word {
            SpellWord::End => {
                open.pop();
            }
            SpellWord::Else if !chains(trimmed) => {
                // The ladder is finished. Keep the frame — an `end` still has to
                // close it — but stop calling it an `if`.
                if open.last() == Some(&SpellWord::If) {
                    open.pop();
                    open.push(SpellWord::Else);
                }
            }
            other if other.opens_block() => open.push(other),
            _ => {}
        }
    }
    open
}

/// Whether an `else` line carries an `if` — `else if <question>`.
fn chains(line: &str) -> bool {
    super::spellword::argument(line)
        .split_whitespace()
        .next()
        .is_some_and(|word| word.eq_ignore_ascii_case(SpellWord::If.canonical()))
}

/// The language's own words that may open a line **here**.
///
/// In the order they are taught, not alphabetical, which puts `else` before
/// `if`. `until` never opens a line, `end` needs something to close, and `else`
/// needs an `if` directly above.
fn control(partial: &str, open: &[SpellWord]) -> Vec<Expected> {
    SpellWord::ALL
        .iter()
        .filter(|word| legal_here(**word, open))
        .filter(|word| word.canonical().starts_with(partial))
        .map(|word| Expected {
            text: word.canonical().to_owned(),
            why: Reason::Word,
            shape: word.shape(),
        })
        .collect()
}

/// Whether `word` may open a line with these blocks open. See [`control`].
fn legal_here(word: SpellWord, open: &[SpellWord]) -> bool {
    match word {
        SpellWord::Until => false,
        SpellWord::End => !open.is_empty(),
        SpellWord::Else => open.last() == Some(&SpellWord::If),
        _ => true,
    }
}

/// What may follow the language's own word, once the line has opened with it.
///
/// `wait` takes a thing, never a state: the runner scans the record stream for
/// the name, so `wait for the mortar_and_pestle to be idle` waits on a thing
/// called `mortar_and_pestle be idle` and faults. The states belong to `is`.
fn after(word: SpellWord, leading: &str, partial: &str, at: &Situation<'_>) -> Vec<Expected> {
    let scene = at.scene;
    let all: Vec<String> = leading
        .split_whitespace()
        .skip(1)
        .map(str::to_lowercase)
        .collect();
    let rest: Vec<&str> = all
        .iter()
        .map(String::as_str)
        .filter(|word| !super::is_filler(word))
        .collect();

    let offered = match word {
        // A count is a number and completes to nothing, so the only word here is
        // the guard — `repeat until <question>`, the bound a loop should have.
        SpellWord::Repeat => match rest.split_first() {
            None => vec![Expected::plain("until", Reason::Grammar)],
            Some((&"until", asked)) => question(asked, scene),
            Some(_) => Vec::new(),
        },
        SpellWord::If | SpellWord::Until => question(&rest, scene),
        // A count and nothing else: no guard, no second form. `bide` is one
        // number, which is what makes it readable arithmetic on the page.
        SpellWord::Bide => Vec::new(),
        // `else if` chains, and a bare `else` takes nothing.
        SpellWord::Else => match rest.split_first() {
            None => vec![Expected::plain("if", Reason::Word)],
            Some((&"if", asked)) => question(asked, scene),
            Some(_) => Vec::new(),
        },
        // A thing, and never a state. See this function's header.
        SpellWord::Wait => things(scene),
        // `let <name> be <place>` — the name is the player's to coin, so the
        // only thing to offer at that position is nothing.
        SpellWord::Let => match rest.len() {
            0 => Vec::new(),
            1 => vec![Expected::plain("be", Reason::Grammar)],
            _ => places(scene),
        },
        // `pull <name> from <place>` — `let`'s shape. `from` is filler and
        // already dropped, so it is never offered; `let`'s `be` is grammar.
        SpellWord::Pull => match rest.len() {
            0 => Vec::new(),
            _ => places(scene),
        },
        // A call naming a part this file defines, which the scene cannot know.
        SpellWord::Alongside => Vec::new(),
        // The sets the fixture declares, never the room's contents.
        SpellWord::For => match rest.split_first() {
            None => vec![Expected::plain("each", Reason::Grammar)],
            Some((&"each", _)) => at
                .sets
                .iter()
                .map(|set| Expected::plain(set.as_str(), Reason::Thing))
                .collect(),
            Some(_) => Vec::new(),
        },
        // A part is named by whoever writes it, and `end` closes a block.
        SpellWord::Part | SpellWord::End => Vec::new(),
    };

    starting(offered, partial)
}

/// Where a comparative's far side begins, if the line has reached one.
///
/// The opener is found first and its closer after it, or the opening `as` of
/// `as many … as` reads as a closer. Nothing between the two is the number
/// form `more than 1 fragment`, as `question::eat_comparative` decides.
fn far_side_begins(asked: &[&str]) -> Option<usize> {
    let mut at = 0;
    while at < asked.len() {
        let (width, closer) = match asked[at] {
            "more" | "fewer" | "less" => (1, "than"),
            "as" if asked
                .get(at + 1)
                .is_some_and(|word| matches!(*word, "many" | "much")) =>
            {
                (2, "as")
            }
            _ => {
                at += 1;
                continue;
            }
        };
        let after = at + width;
        let found = asked[after..].iter().position(|word| *word == closer)?;
        // Nothing between the two is `more than 1 fragment`, which is a count
        // and not a far side at all.
        return (found > 0).then_some(after + found + 1);
    }
    None
}

/// What may come next inside a question, given the words of it so far.
///
/// `<place> is <state>` or `<place> has [no] [<count>] <thing>`.
fn question(asked: &[&str], scene: &Scene) -> Vec<Expected> {
    // A comparative's far side is its own little question, with its own hinge.
    if let Some(begins) = far_side_begins(asked) {
        let far = &asked[begins..];
        // Straight after the closer: a place, or the one word that scales one.
        if far.is_empty() {
            let mut out = vec![Expected::plain("double", Reason::Grammar)];
            out.extend(places(scene));
            return out;
        }
        // The far side's own `has`, then what it may ask for there.
        if let Some(at) = far.iter().position(|word| *word == "has") {
            return if far.len() == at + 1 {
                let mut out = things(scene);
                out.extend(readings(scene));
                out
            } else {
                vec![Expected::plain("plus", Reason::Grammar)]
            };
        }
        // A place is named and nothing else yet: the two words that may follow.
        return vec![
            Expected::plain("has", Reason::Grammar),
            Expected::plain("plus", Reason::Grammar),
        ];
    }

    let hinge = asked.iter().position(|word| matches!(*word, "is" | "has"));
    let Some(at) = hinge else {
        // No hinge yet: either the place has not been named, or it has and the
        // question word is what is wanted.
        return if asked.is_empty() {
            places(scene)
        } else {
            vec![
                Expected::plain("is", Reason::Grammar),
                Expected::plain("has", Reason::Grammar),
            ]
        };
    };

    match asked[at] {
        "is" => SpellState::WORDS
            .iter()
            .map(|word| Expected::plain(*word, Reason::State))
            .collect(),
        // An optional `no`, a count (which completes to nothing), and a thing
        // or reading — things first, since every scene carries the readings.
        _ => {
            let mut out = Vec::new();
            if asked.len() == at + 1 {
                out.push(Expected::plain("no", Reason::Grammar));
            }
            out.extend(things(scene));
            out.extend(readings(scene));
            out
        }
    }
}

/// The places a question may name.
fn places(scene: &Scene) -> Vec<Expected> {
    of_kind(scene, NounKind::Place, Reason::Thing)
}

/// The readings a place publishes — what `has` asks about.
fn readings(scene: &Scene) -> Vec<Expected> {
    of_kind(scene, NounKind::Sense, Reason::Reading)
}

/// Everything the tower *has* that a spell can name.
///
/// Not a `Command` — every verb synonym is one, and none is a thing — nor a
/// `Sense`, which [`readings`] offers after these.
fn things(scene: &Scene) -> Vec<Expected> {
    scene
        .nouns()
        .iter()
        .filter(|noun| !matches!(noun.kind, NounKind::Sense | NounKind::Command))
        .map(|noun| Expected::plain(super::intent::leaf(&noun.name), Reason::Thing))
        .collect()
}

/// The scene's nouns of one kind, in scene order.
fn of_kind(scene: &Scene, kind: NounKind, why: Reason) -> Vec<Expected> {
    scene
        .nouns()
        .iter()
        .filter(|noun| noun.kind == kind)
        .map(|noun| Expected::plain(super::intent::leaf(&noun.name), why))
        .collect()
}

/// Keep only what the player has already started typing — once, at the end,
/// so a new arm cannot forget it.
fn starting(offered: Vec<Expected>, partial: &str) -> Vec<Expected> {
    if partial.is_empty() {
        return offered;
    }
    offered
        .into_iter()
        .filter(|one| one.text.starts_with(partial))
        .collect()
}

/// Every phrase that could be the verb being typed, multi-word ones included.
///
/// In a spell, two filters: `Scene::offers` asks whether the verb's fixture
/// stands here, and [`spell_may`] whether a spell may issue it at all.
fn verbs(partial: &str, scene: &Scene, spell: bool) -> Vec<Expected> {
    let mut out: Vec<(String, Verb)> = SYNONYMS
        .iter()
        .filter(|entry| scene.offers(entry.verb))
        .filter(|entry| !spell || spell_may(entry.verb))
        .map(|entry| (entry.words.join(" "), entry.verb))
        .filter(|(phrase, _)| phrase.starts_with(partial))
        .collect();
    out.sort();
    out.dedup();
    out.into_iter()
        .map(|(text, verb)| Expected {
            text,
            why: Reason::Verb,
            shape: verb.signature_label(),
        })
        .collect()
}

/// Whether a spell may issue this verb at all — the pair
/// [`spell_vocabulary`](crate::spell_vocabulary) applies too, so Tab never
/// offers a word the guide does not explain.
const fn spell_may(verb: Verb) -> bool {
    // *Nobody built this* and *a spell may not*: both dead ends.
    crate::execute::is_live(verb) && crate::tower::spell::may_issue(verb)
}

/// The verb a line has already named, and how many argument slots follow it.
///
/// Through [`match_phrase`](super::resolve::match_phrase), the matcher
/// `resolve` ranks with, so Tab and Enter agree; not through `resolve` itself,
/// which calls a bare `wield ` `Incomplete` just when completion is wanted.
fn verb_of(leading: &str, scene: &Scene) -> Option<(Verb, usize)> {
    let split = super::normalise::Tokens::split(leading);
    let all = split.words();
    let words = &all[super::normalise::skip_leading_filler(&all)..];

    let (verb, consumed) = SYNONYMS
        .iter()
        .filter(|entry| scene.offers(entry.verb))
        .filter_map(|entry| {
            super::resolve::match_phrase(entry, words)
                .map(|(score, span)| (score, entry.verb, span))
        })
        .max_by(|a, b| {
            // Score, then the longer phrase — an exact `look for` and `look`
            // score alike — then table order.
            a.0.cmp(&b.0)
                .then_with(|| a.2.cmp(&b.2))
                .then_with(|| verb_order(b.1).cmp(&verb_order(a.1)))
        })
        .map(|(_, verb, span)| (verb, span))?;

    // Filler between the verb and the caret is not a slot: `move sage to ` has
    // filled one, not two, and counting `to` would offer the third slot's nouns
    // for the second.
    let filled = super::normalise::strip_filler(&words[consumed..]).len();
    Some((verb, filled))
}

/// A verb's position in [`Verb::ALL`], mirroring `resolve`'s stable tie-break.
fn verb_order(verb: Verb) -> usize {
    Verb::ALL
        .iter()
        .position(|&other| other == verb)
        .unwrap_or(usize::MAX)
}

/// Every noun that fits the slot the caret is in.
fn nouns(verb: Verb, filled: usize, partial: &str, scene: &Scene) -> Vec<Expected> {
    let signature = verb.signature();
    let Some(slot) = signature.get(filled).or_else(|| signature.last()) else {
        return Vec::new();
    };

    // Free text completes nothing: a pattern is whatever the player is searching
    // for and a count is a number, so the scene's nouns would lie about what the
    // slot accepts.
    if matches!(slot.kind, NounKind::Pattern | NounKind::Count) {
        return Vec::new();
    }

    // `Name` is free text, but the commonest `scribe` reopens a spell, so the
    // spells that exist are offered.
    let offered = if slot.kind == NounKind::Name {
        NounKind::Script
    } else {
        slot.kind
    };

    scene
        .nouns()
        .iter()
        .filter(|noun| offered.accepts(noun.kind))
        .filter_map(|noun| {
            // A place is shown and typed as its leaf (§7: players say the place,
            // not the path) — through the same helper the echo uses, so Tab
            // inserts exactly the form the echo will show back.
            let leaf = super::intent::leaf(&noun.name);
            leaf.starts_with(partial)
                .then(|| Expected::plain(leaf, Reason::Thing))
        })
        .collect()
}

/// The byte index where the word ending at `upto` begins.
fn word_start(line: &str, upto: usize) -> usize {
    line[..upto].rfind(char::is_whitespace).map_or(0, |at| {
        at + line[at..].chars().next().map_or(1, char::len_utf8)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tower() -> Scene {
        Scene::new()
            .with(NounKind::Place, "/tower/laboratory")
            .with(NounKind::Place, "/tower/laboratory/mortar_and_pestle")
            .with(NounKind::Reagent, "sage")
            .with(NounKind::Sense, "clarity")
    }

    /// A line of a spell, with nothing open above it.
    fn spelling(line: &str) -> Expectation {
        inside(line, &[])
    }

    /// ...and one written inside the blocks named.
    fn inside(line: &str, open: &[SpellWord]) -> Expectation {
        let scene = tower();
        let sets = [String::from("way")];
        expect(
            line,
            line.chars().count(),
            &Situation {
                scene: &scene,
                spell: true,
                prompt_open: false,
                open,
                sets: &sets,
            },
        )
    }

    fn typing(line: &str) -> Expectation {
        let scene = tower();
        expect(
            line,
            line.chars().count(),
            &Situation {
                scene: &scene,
                spell: false,
                prompt_open: false,
                open: &[],
                sets: &[],
            },
        )
    }

    /// A spell line may open with one of the language's own words; the prompt
    /// cannot run them.
    #[test]
    fn a_spell_offers_the_languages_own_words_and_the_prompt_does_not() {
        let found = spelling("re").texts();
        assert!(found.contains(&"repeat".to_owned()), "{found:?}");

        let prompt = typing("re").texts();
        assert!(
            !prompt.contains(&"repeat".to_owned()),
            "the prompt offered a word it cannot run: {prompt:?}",
        );
    }

    /// ...and the verbs are still there beside them.
    #[test]
    fn a_spell_offers_verbs_as_well_as_words() {
        let found = spelling("su").texts();
        assert!(found.contains(&"survey".to_owned()), "{found:?}");
    }

    /// Control words come first, in the order they are taught.
    #[test]
    fn the_languages_words_lead_and_keep_their_own_order() {
        let found = spelling("").expected;
        let words: Vec<&str> = found
            .iter()
            .take_while(|one| one.why == Reason::Word)
            .map(|one| one.text.as_str())
            .collect();
        assert_eq!(
            words,
            // `SpellWord::ALL`'s order, which the guide shares.
            [
                "wait",
                "repeat",
                "if",
                "let",
                "for",
                "part",
                "bide",
                "pull",
                "alongside",
            ],
            "the language's words reordered or stopped leading",
        );
        assert!(
            found.iter().any(|one| one.why == Reason::Verb),
            "the verbs went missing behind them",
        );
    }

    #[test]
    fn a_word_that_cannot_open_a_line_here_is_not_offered() {
        // `until` is `repeat`'s guard, and alone is `spell_stray_until`.
        for open in [&[][..], &[SpellWord::Repeat][..], &[SpellWord::If][..]] {
            assert!(
                !inside("", open).texts().contains(&"until".to_owned()),
                "`until` was offered as a line opener with {open:?} open",
            );
        }

        // `end` needs something to close.
        assert!(!spelling("").texts().contains(&"end".to_owned()));
        assert!(
            inside("", &[SpellWord::Repeat])
                .texts()
                .contains(&"end".to_owned()),
        );

        // `else` needs an `if` directly above.
        assert!(
            inside("", &[SpellWord::If])
                .texts()
                .contains(&"else".to_owned()),
        );
        assert!(
            !inside("", &[SpellWord::If, SpellWord::Repeat])
                .texts()
                .contains(&"else".to_owned()),
            "`else` was offered inside a `repeat` that is inside the `if`",
        );
    }

    /// The stack is read off the lines above, and `end` pops it.
    #[test]
    fn the_blocks_open_above_a_line_are_counted_from_it() {
        let lines: Vec<String> = ["repeat 3", "if the mortar_and_pestle is idle", "end"]
            .into_iter()
            .map(str::to_owned)
            .collect();
        assert_eq!(open_blocks(&lines), [SpellWord::Repeat]);
        assert_eq!(open_blocks(&lines[..2]), [SpellWord::Repeat, SpellWord::If]);
        assert!(open_blocks(&[]).is_empty());
    }

    /// `is` offers the states and `wait` does not — see `after`.
    #[test]
    fn the_states_belong_to_is_and_not_to_wait() {
        let states = spelling("if the mortar_and_pestle is ").texts();
        assert_eq!(
            states,
            [
                "idle", "free", "still", "working", "busy", "running", "empty", "bare"
            ],
            "the state vocabulary moved",
        );

        let waiting = spelling("wait for the mortar_and_pestle to be ").texts();
        assert!(
            !waiting.iter().any(|word| word == "idle"),
            "`wait` was offered a state it cannot use: {waiting:?}",
        );
    }

    /// The far side is its own little question, and the hinge is the last one.
    #[test]
    fn the_far_side_of_a_comparison_completes_as_its_own_question() {
        // Straight after the closer: somewhere to compare against, or the word
        // that doubles one.
        let far = spelling("if the mortar_and_pestle has fewer sage than ").texts();
        assert!(far.contains(&"double".to_owned()), "{far:?}");
        assert!(
            far.contains(&"mortar_and_pestle".to_owned()),
            "the far side offered no place: {far:?}",
        );

        // A place is named; the trailing space asks what comes next.
        let after =
            spelling("if the mortar_and_pestle has fewer sage than the laboratory ").texts();
        assert_eq!(
            after,
            vec!["has".to_owned(), "plus".to_owned()],
            "the far side offered the wrong continuation",
        );

        // ...and after the far side's own `has`, what may be asked *there*.
        let reading =
            spelling("if the mortar_and_pestle has fewer sage than the laboratory has ").texts();
        assert!(
            reading.contains(&"clarity".to_owned()),
            "the far side's `has` did not offer a reading: {reading:?}",
        );

        // Named, so the only thing left is the operator.
        let operator =
            spelling("if the mortar_and_pestle has fewer sage than the laboratory has clarity ")
                .texts();
        assert_eq!(operator, vec!["plus".to_owned()]);

        // `double` is a prefix and changes none of that.
        let doubled =
            spelling("if the mortar_and_pestle has fewer sage than double the laboratory ").texts();
        assert_eq!(doubled, vec!["has".to_owned(), "plus".to_owned()]);

        // `as many … as` opens with the word that closes it.
        let opening = spelling("if the mortar_and_pestle has as many ").texts();
        assert!(
            opening.contains(&"sage".to_owned()),
            "the opening `as` was read as a closer: {opening:?}",
        );
        assert!(
            !opening.contains(&"plus".to_owned()),
            "the far side answered a line still on the near one: {opening:?}",
        );
        // ...and once it really is closed, the far side answers.
        let closed = spelling("if the mortar_and_pestle has as many sage as ").texts();
        assert!(closed.contains(&"double".to_owned()), "{closed:?}");

        // The number form `more than 2 …` is not a far side.
        let counted = spelling("if the mortar_and_pestle has more than 2 ").texts();
        assert!(
            counted.contains(&"sage".to_owned()),
            "a count was read as a comparison against a place: {counted:?}",
        );
    }

    /// A question is walked one hinge at a time.
    #[test]
    fn a_question_offers_a_place_then_a_hinge_then_an_answer() {
        // Nothing named yet: the places.
        let places = spelling("if ").texts();
        assert!(
            places.contains(&"mortar_and_pestle".to_owned()),
            "{places:?}"
        );
        assert!(
            !places.contains(&"sage".to_owned()),
            "a reagent is not a place to ask about: {places:?}",
        );

        // Named: the two words that hinge a question.
        assert_eq!(spelling("if the mortar_and_pestle ").texts(), ["is", "has"]);

        // ...and a partial narrows it, through the one filter every arm shares.
        assert_eq!(spelling("if the mortar_and_pestle i").texts(), ["is"]);

        // `has` asks about a reading or a thing, and takes an optional `no`.
        let has = spelling("if the mortar_and_pestle has ").texts();
        assert_eq!(has.first().map(String::as_str), Some("no"));
        assert!(has.contains(&"sage".to_owned()), "{has:?}");
    }

    /// `repeat` offers its guard, and `else` chains into another `if`.
    #[test]
    fn a_word_that_takes_another_word_offers_it() {
        assert_eq!(spelling("repeat ").texts(), ["until"]);
        assert_eq!(inside("else ", &[SpellWord::If]).texts(), ["if"]);
        assert_eq!(spelling("for ").texts(), ["each"]);
        assert_eq!(spelling("let mine ").texts(), ["be"]);

        // `repeat until ` is a question like any other.
        let asked = spelling("repeat until ").texts();
        assert!(asked.contains(&"mortar_and_pestle".to_owned()), "{asked:?}");

        // A part is named by whoever writes it; there is nothing to offer.
        assert!(spelling("part ").is_empty());
    }

    /// `for each ` walks a set, and the room's contents are not sets.
    #[test]
    fn for_each_offers_a_set_and_not_the_rooms_contents() {
        assert_eq!(spelling("for each ").texts(), ["way"]);

        let found = spelling("for each ").texts();
        assert!(
            !found.contains(&"mortar_and_pestle".to_owned()),
            "the room's contents were offered as sets: {found:?}",
        );
    }

    /// A verb word is not a thing, so nothing offers one as one.
    #[test]
    fn a_verb_word_is_never_offered_as_a_thing() {
        let scene = Scene::new()
            .with(NounKind::Reagent, "sage")
            .with(NounKind::Command, "grind")
            .with(NounKind::Command, "walk");
        let sets: [String; 0] = [];
        let asked = |line: &str| {
            expect(
                line,
                line.chars().count(),
                &Situation {
                    scene: &scene,
                    spell: true,
                    prompt_open: false,
                    open: &[],
                    sets: &sets,
                },
            )
            .texts()
        };

        assert_eq!(asked("wait "), ["sage"], "a verb word was waitable");
        assert!(
            !asked("if the cabinet has ").contains(&"walk".to_owned()),
            "a verb word was offered as something a place could hold",
        );
    }

    /// A second bare `else` is not offered, and `else if` still chains.
    #[test]
    fn a_ladder_that_has_had_its_else_is_not_offered_another() {
        let ladder = |lines: &[&str]| {
            let owned: Vec<String> = lines.iter().map(|line| (*line).to_owned()).collect();
            let open = open_blocks(&owned);
            inside("", &open).texts().contains(&"else".to_owned())
        };

        assert!(
            ladder(&["if the mortar_and_pestle is idle"]),
            "an open `if`"
        );
        assert!(
            !ladder(&["if the mortar_and_pestle is idle", "else"]),
            "a finished ladder was offered a second `else`",
        );
        // `else if` chains.
        assert!(
            ladder(&[
                "if the mortar_and_pestle is idle",
                "else if the laboratory is idle",
            ]),
            "`else if` was read as the end of its ladder",
        );
        // ...and `end` still closes the frame the `else` left open.
        let closed: Vec<String> = ["if the mortar_and_pestle is idle", "else"]
            .into_iter()
            .map(str::to_owned)
            .collect();
        assert_eq!(open_blocks(&closed).len(), 1, "the frame went missing");
        assert!(
            inside("", &open_blocks(&closed))
                .texts()
                .contains(&"end".to_owned()),
            "nothing was left to close",
        );
    }

    /// A spell is offered only verbs it may issue: `attend` passes
    /// `Scene::offers` and fails `may_issue`.
    #[test]
    fn a_spell_is_not_offered_a_verb_it_may_not_issue() {
        let prompt = typing("att").texts();
        assert!(prompt.contains(&"attend".to_owned()), "{prompt:?}");

        let spell = spelling("att").texts();
        assert!(
            !spell.contains(&"attend".to_owned()),
            "a spell was offered a verb it may not issue: {spell:?}",
        );
    }

    /// Each candidate carries how it is used, from the grammar's own table.
    #[test]
    fn a_candidate_carries_the_shape_that_follows_it() {
        let words = spelling("").expected;
        let repeat = words.iter().find(|one| one.text == "repeat");
        assert_eq!(repeat.map(|one| one.shape), Some("<count>"));

        // `end` takes nothing, and says so with an empty string. Inside a
        // block, since with nothing open it is not offered.
        let closing = inside("", &[SpellWord::Repeat]).expected;
        let end = closing.iter().find(|one| one.text == "end");
        assert_eq!(end.map(|one| one.shape), Some(""));

        // A verb's is its signature label — the same one `help` lists.
        let verb = words.iter().find(|one| one.text == "survey");
        assert_eq!(verb.map(|one| one.shape), Some("place"));
    }

    #[test]
    fn the_common_prefix_is_as_far_as_everyone_agrees() {
        // Two places share `m`... but only one starts with `mo`, so the whole
        // name is common.
        assert_eq!(typing("wield m").common(), "mortar_and_pestle");

        // With nothing typed, the laboratory's places share nothing.
        let both = typing("wield ");
        assert!(both.expected.len() > 1);
        assert_eq!(both.common(), "");

        // Nothing offered agrees on nothing, rather than panicking.
        assert_eq!(typing("xyzzy zz").common(), "");
    }

    /// An argument slot is the same answer in both, because it is the same verb.
    #[test]
    fn an_argument_slot_answers_the_same_in_a_spell_as_at_the_prompt() {
        assert_eq!(spelling("wield mo"), typing("wield mo"));
        assert_eq!(spelling("wield mo").texts(), ["mortar_and_pestle"]);
    }

    /// Each candidate says what it is, so a surface need not ask again.
    #[test]
    fn a_candidate_carries_its_own_kind() {
        let nouns = typing("wield mo").expected;
        assert_eq!(nouns.first().map(Expected::kind), Some(Lexeme::Name));

        let verbs = typing("surv").expected;
        assert_eq!(verbs.first().map(Expected::kind), Some(Lexeme::Verb));

        // Grammar to the guide, an ordinary word to the painter.
        let hinge = spelling("if the mortar_and_pestle ").expected;
        let is = hinge.first().expect("a hinge");
        assert_eq!(is.why, Reason::Grammar);
        assert_eq!(is.kind(), Lexeme::Name);
    }

    /// The numbered prompt closes it, in a spell too.
    #[test]
    fn nothing_is_expected_while_a_numbered_prompt_is_open() {
        let scene = tower();
        for spell in [false, true] {
            let found = expect(
                "sur",
                3,
                &Situation {
                    scene: &scene,
                    spell,
                    prompt_open: true,
                    open: &[],
                    sets: &[],
                },
            );
            assert!(found.is_empty(), "offered something at a numbered prompt");
        }
    }
}
