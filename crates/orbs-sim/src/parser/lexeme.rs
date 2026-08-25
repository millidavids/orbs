//! A spell line, split into the parts a reader wants told apart.
//!
//! # Why this is a language fact and not a painter's
//!
//! Which word is a control word, which names a verb, which is a count — those
//! are answers only the language has, and they are the same answers `read` and
//! `analyse` give. A painter that worked them out again would be a second
//! opinion about the grammar, which is the shape §19 records going wrong three
//! times over (the rail's state words, the substitution table, `is_live`).
//!
//! So the classification is here and the **colour** is the frontend's, which is
//! rule 2 exactly: `orbs-shell` turns a [`Lexeme`] into a `Style`, and each
//! frontend turns that into a hue its tube or terminal can draw.
//!
//! # It is enrichment, and that is what makes it allowed
//!
//! §14 forbids colour being the sole carrier of meaning. Nothing here carries
//! any: take every colour away and the file still says what it said, `interpret`
//! still reports what the orb heard, and a screen reader still hears one sentence
//! per line — `sheet` announces the row whole and draws the runs silently. This
//! is comfort, and comfort is allowed to be pretty.
//!
//! # Lexical, deliberately
//!
//! It reads one line at a time and asks nothing of the world. A name is a name
//! whether or not the tower has one right now, which is what lets the editor
//! colour a spell written for a room the player is not standing in — and what
//! stops a spell's appearance changing as its own pipeline fills the shelf.

use orbs_render::Lexeme;

use super::question::State as SpellState;
use super::{Verb, is_filler, spell_word};

/// One run of a line, and what it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lexed {
    /// Byte offset of the first character.
    pub start: usize,
    /// Byte offset one past the last.
    pub end: usize,
    /// What the run is.
    pub kind: Lexeme,
}

/// Split `line` into runs, in order, covering every non-space byte of it.
///
/// Whitespace between runs is not returned: a gap carries nothing and a painter
/// draws it in whatever style it likes.
///
/// **A comment is one run.** `# two laps` is not four words the orb ignored, it
/// is a sentence the player wrote to themselves, and splitting it would colour
/// `laps` as a name.
#[must_use]
pub fn lex(line: &str) -> Vec<Lexed> {
    let trimmed = line.trim_start();
    let indent = line.len() - trimmed.len();
    if trimmed.starts_with('#') {
        return vec![Lexed {
            start: indent,
            end: line.trim_end().len(),
            kind: Lexeme::Comment,
        }];
    }

    // **The first word decides how the rest is read**, exactly as `read` and
    // `analyse` decide it: a line opening with a control word is a control line,
    // one opening with `name()` is a call, and anything else is a command whose
    // first word is its verb.
    let opens_control = spell_word(trimmed).is_some();
    let mut runs = Vec::new();
    for (index, (at, word)) in words(line).enumerate() {
        let kind = classify(word, index, opens_control);
        runs.push(Lexed {
            start: at,
            end: at + word.len(),
            kind,
        });
    }
    runs
}

/// What one word of a line is.
///
/// `first` is whether the *line* opened with a control word, which is what makes
/// `end` in `repeat until the stacks is idle` read as part of the question rather
/// than as a block close — the same lookahead-free rule `read` uses.
fn classify(word: &str, index: usize, opens_control: bool) -> Lexeme {
    if is_call(word) {
        return Lexeme::Call;
    }
    if spell_word(word).is_some() {
        return Lexeme::Control;
    }
    if word.chars().all(|glyph| glyph.is_ascii_digit()) {
        return Lexeme::Number;
    }
    let lower = word.to_lowercase();
    // **Before filler, and `and` is deliberately not in the list.** A grammar
    // word is filler's opposite — filler is what §6 *strips*, this is what the
    // question turns on — so they must not draw alike. `and` is both: it joins
    // two questions *and* sits on the filler list, and a lookahead-free
    // classifier cannot tell `if X and Y` from `move sage and rock-salt`. It
    // stays filler, which is what §6 does with it in a command.
    if is_grammar(&lower) {
        return Lexeme::Grammar;
    }
    if is_filler(&lower) {
        return Lexeme::Filler;
    }
    // A verb only where a verb can stand: the head of a command line, or the
    // head of a control line's body — `if the mortar is idle` has no verb in it,
    // and colouring `is` as one would be a lie about the grammar.
    //
    // **This must stay above the state check**, and `empty` is why: it is a verb
    // (`empty mortar_and_pestle`) *and* a state (`is empty`), the only word that
    // is both. Position is what tells them apart, and it is the same rule the
    // verb check already rests on.
    if index == 0 && !opens_control && names_a_verb(word) {
        return Lexeme::Verb;
    }
    if SpellState::WORDS.contains(&lower.as_str()) {
        return Lexeme::State;
    }
    Lexeme::Name
}

/// Whether `word` is one the question grammar fixes in place.
///
/// **Four of these are [`STOPPERS`](super::question::STOPPERS)**, read from
/// there rather than copied: they are the words `condition` splits a question
/// on, and a second list of them is the drift §19 records over and over. The
/// rest are the ones no existing constant already holds — the comparison
/// spellings, `has no`, and the two words `let` and `for` fix.
///
/// `every_grammar_word_is_one_the_language_reads` is what keeps this honest.
fn is_grammar(lower: &str) -> bool {
    super::question::STOPPERS.contains(&lower)
        || matches!(
            lower,
            // `has no sage`, and `2 or more` / `2 or fewer`.
            "no" | "or"
            // The comparison spellings: `at least 2`, `at most 2`, `more than
            // 1`, `fewer than 3`, `exactly 2`. `at` is on the filler list and
            // stays there.
            | "least" | "most" | "more" | "fewer" | "exactly"
            // `let m be mortar`, and `for each way`.
            | "be" | "each"
        )
}

/// Whether `word` is written as a call — `gathering()`.
fn is_call(word: &str) -> bool {
    word.ends_with("()") && word.len() > 2
}

/// Whether any verb answers to this word.
///
/// **Room-independent, and that is deliberate.** `Scene::offers` scopes a verb to
/// the room its instrument stands in, and a spell is edited from wherever the
/// player happens to be — so asking the scene would make a laboratory spell lose
/// its colour when read from the archive. The *vocabulary* is fixed; where it
/// resolves is not, and only the first is a fact about the text.
fn names_a_verb(word: &str) -> bool {
    let word = word.to_lowercase();
    Verb::ALL.into_iter().any(|verb| verb.canonical() == word)
        || super::single_words().any(|(known, _)| known == word)
}

/// Every word of `line` with its byte offset.
fn words(line: &str) -> impl Iterator<Item = (usize, &str)> {
    line.split_whitespace()
        .map(move |word| (offset_of(line, word), word))
}

/// Where `word` sits in `line`.
///
/// **By pointer arithmetic on the slice**, not by searching: `split_whitespace`
/// hands back slices *of* `line`, so their addresses are the answer. Searching
/// for the text would find the first occurrence, and `move sage to sage` has two.
fn offset_of(line: &str, word: &str) -> usize {
    word.as_ptr() as usize - line.as_ptr() as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The kinds of a line, in order.
    fn kinds(line: &str) -> Vec<Lexeme> {
        lex(line).into_iter().map(|run| run.kind).collect()
    }

    /// The text each run covers, so an offset bug cannot hide behind a kind.
    fn spans(line: &str) -> Vec<&str> {
        lex(line)
            .into_iter()
            .map(|run| &line[run.start..run.end])
            .collect()
    }

    #[test]
    fn a_command_is_a_verb_and_then_names() {
        assert_eq!(kinds("grind sage"), [Lexeme::Verb, Lexeme::Name]);
        assert_eq!(spans("grind sage"), ["grind", "sage"]);
    }

    #[test]
    fn filler_recedes_rather_than_vanishing() {
        // §19: the file is the player's and the orb never rewrites it, so a word
        // §6 strips still has to be on screen.
        assert_eq!(
            kinds("empty the mortar_and_pestle"),
            [Lexeme::Verb, Lexeme::Filler, Lexeme::Name],
        );
    }

    #[test]
    fn a_control_line_has_no_verb_in_it() {
        // `is` is part of the question grammar. Colouring the head of a control
        // line as a verb would be a lie about how the line is read.
        //
        // **All five parts differ now**, which is the whole of what the hues
        // were added for: `is` was a `Name` beside `mortar`, and `idle` was a
        // `Name` beside both.
        assert_eq!(
            kinds("if the mortar is idle"),
            [
                Lexeme::Control,
                Lexeme::Filler,
                Lexeme::Name,
                Lexeme::Grammar,
                Lexeme::State
            ],
        );
    }

    #[test]
    fn a_count_is_a_number() {
        assert_eq!(kinds("repeat 2"), [Lexeme::Control, Lexeme::Number]);
        assert_eq!(
            kinds("if the cabinet has 4 fragment"),
            [
                Lexeme::Control,
                Lexeme::Filler,
                Lexeme::Name,
                Lexeme::Grammar,
                Lexeme::Number,
                Lexeme::Name
            ],
        );
    }

    /// `empty` is a verb and a state, and position is what tells them apart.
    ///
    /// **The only word that is both**, which is why the state check sits below
    /// the verb check rather than above it. Above, `empty mortar_and_pestle` —
    /// the commonest line in the game — would draw its verb as a state.
    #[test]
    fn the_one_word_that_is_a_verb_and_a_state_is_read_by_position() {
        assert_eq!(
            kinds("empty mortar_and_pestle"),
            [Lexeme::Verb, Lexeme::Name],
        );
        assert_eq!(
            kinds("if the mortar_and_pestle is empty"),
            [
                Lexeme::Control,
                Lexeme::Filler,
                Lexeme::Name,
                Lexeme::Grammar,
                Lexeme::State
            ],
        );
    }

    /// A grammar word is filler's opposite and must not draw like one.
    ///
    /// Filler is what §6 **strips**; these are what the question turns on. Drawn
    /// dim beside a `the`, `is` and `has` had nothing to tell them apart by,
    /// which is what the hue was added for.
    #[test]
    fn a_grammar_word_is_not_filler() {
        // Every comparison spelling, and the two words `let` and `for` fix.
        for line in [
            "if north has 2 or more marks",
            "if north has at least 2 marks",
            "if north has fewer marks than east",
            "if north has exactly 2 marks",
            "if the cabinet has no fragment",
            "let best be north",
            "for each way",
        ] {
            let found = kinds(line);
            assert!(
                found.contains(&Lexeme::Grammar),
                "{line:?} has no grammar in it: {found:?}",
            );
        }

        // ...and `and` stays filler, because it is both and a lookahead-free
        // classifier cannot tell `if X and Y` from `move sage and rock-salt`.
        assert!(kinds("move sage and rock-salt").contains(&Lexeme::Filler));
    }

    #[test]
    fn a_part_and_its_call_are_told_apart_from_a_verb() {
        // The whole reason `Call` is its own kind: `gathering()` looks like a
        // command and is a name this file defines.
        assert_eq!(kinds("part gathering()"), [Lexeme::Control, Lexeme::Call]);
        assert_eq!(kinds("gathering()"), [Lexeme::Call]);
    }

    #[test]
    fn a_comment_is_one_run_and_not_four_words() {
        assert_eq!(kinds("# two laps"), [Lexeme::Comment]);
        assert_eq!(spans("  # two laps"), ["# two laps"]);
    }

    #[test]
    fn a_blank_line_has_nothing_in_it() {
        assert!(lex("").is_empty());
        assert!(lex("    ").is_empty());
    }

    #[test]
    fn offsets_are_the_words_own_and_not_the_first_match() {
        // `split_whitespace` hands back slices of the line, so the address is the
        // answer. Searching for the text would put both `sage`s at the first one.
        let line = "move sage to sage";
        let runs = lex(line);
        assert_eq!(runs[1].start, 5);
        assert_eq!(runs[3].start, 13);
    }

    #[test]
    fn every_run_lands_inside_the_line_it_came_from() {
        // The property a painter depends on: an offset past the end is a panic on
        // the next redraw, and an editor redraws thirty times a second.
        for line in [
            "grind sage",
            "  repeat until the stacks is idle",
            "part gathering()",
            "# a note",
            "let best be north",
            "for each way",
            "move clarity to arsenal",
        ] {
            for run in lex(line) {
                assert!(run.start <= run.end, "{line:?} has an inverted run");
                assert!(run.end <= line.len(), "{line:?} runs off the end");
                assert!(line.is_char_boundary(run.start), "{line:?} splits a char");
                assert!(line.is_char_boundary(run.end), "{line:?} splits a char");
            }
        }
    }
}
