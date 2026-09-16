//! A reader that matches the authored templates directly.
//!
//! **The baseline a trained reader has to beat**, and the reason it exists is
//! that nobody knew whether one was needed. `content/phrasings.toml` is written
//! either way — it is a model's only training data — so compiling it into a
//! matcher costs a file and answers the question a model would otherwise be
//! built to answer.
//!
//! # What it can and cannot do
//!
//! It reads **reordering and extra words**, which is exactly what a flat
//! synonym list cannot: `take me over to the laboratory` matches `take me to
//! the {place}` because the literals still arrive in order. It cannot read a
//! word it was never given — `pound the sage` is nothing to a grammar built on
//! `smash`, `crush` and `grind up`, and no amount of template-writing closes
//! that in general.
//!
//! That is the whole shape of the comparison. A trained reader's claim is that
//! it generalises *past* the words it was shown; this measures what is left
//! over when nothing does.

use super::{Augur, MAX_READINGS};
use crate::content::Phrasings;
use crate::parser::{Tokens, is_near};

/// One piece of a template.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Piece {
    /// A word that must appear, near enough to be the same word.
    Word(String),
    /// A slot, which takes whatever the player put there.
    Slot(String),
}

/// One template, compiled.
#[derive(Debug, Clone)]
struct Pattern {
    /// What has to match, in order.
    pieces: Vec<Piece>,
    /// The canonical command, still carrying its markers.
    canonical: String,
    /// How many words this pattern insists on, for preferring the specific one.
    weight: usize,
}

/// A reader compiled from `content/phrasings.toml`.
///
/// See the module documentation for what it is for. Built from the `say`
/// templates only — the holdout is taught to nothing, here as everywhere.
#[derive(Debug, Clone)]
pub struct Grammar {
    patterns: Vec<Pattern>,
}

impl Default for Grammar {
    fn default() -> Self {
        Self::builtin()
    }
}

impl Grammar {
    /// Compiled from the templates built into the binary.
    #[must_use]
    pub fn builtin() -> Self {
        Self::from_phrasings(&Phrasings::builtin())
    }

    /// Compiled from `phrasings`.
    ///
    /// **`say` only.** A grammar taught the holdout would match it perfectly and
    /// measure nothing, which is the one way this comparison can lie.
    #[must_use]
    pub fn from_phrasings(phrasings: &Phrasings) -> Self {
        let mut patterns = Vec::new();
        for entry in phrasings.entries() {
            for template in &entry.say {
                let pieces = compile(template);
                let weight = pieces
                    .iter()
                    .filter(|piece| matches!(piece, Piece::Word(_)))
                    .count();
                patterns.push(Pattern {
                    pieces,
                    canonical: entry.canonical.clone(),
                    weight,
                });
            }
        }
        // Most-insistent first, so `turn {reagent} into powder` is tried before
        // a one-word pattern that would also accept the line. Without it the
        // answer depends on the order somebody wrote the file in.
        patterns.sort_by_key(|pattern| core::cmp::Reverse(pattern.weight));
        Self { patterns }
    }

    /// How many templates it holds, for the bench's report.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.patterns.len()
    }

    /// Whether it holds nothing.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }
}

/// Split a template into the words that must match and the slots that need not.
fn compile(template: &str) -> Vec<Piece> {
    template
        .split_whitespace()
        .map(|token| {
            if let Some(name) = token.strip_prefix('{').and_then(|t| t.strip_suffix('}')) {
                return Piece::Slot(name.to_owned());
            }
            // **Filler is kept, and that is the opposite of what the matcher
            // does.** `normalise::strip_filler` drops `in`, `the` and `to`
            // because they carry no evidence about *which noun* an argument
            // names — and for a grammar they are the evidence, because they are
            // what tells one phrasing from another.
            //
            // Dropping them made *"what is in the {place}"* and *"what is
            // {topic}"* the same pattern, so `survey` and `recall` collided and
            // whichever sorted first won. It cost 4,615 misreadings across the
            // corpus: `what is brewing` answered `survey brewing`.
            //
            // Keeping them costs nothing, because a literal may still skip over
            // input it does not match: *"can you take me over to the
            // laboratory"* still reaches *"take me to the {place}"*.
            Piece::Word(token.to_lowercase())
        })
        .collect()
}

/// What one pattern made of a line.
struct Capture {
    /// What each slot took, by name, in the order the pattern names them.
    slots: Vec<(String, String)>,
    /// How many of the pattern's words were found only by being *near* one the
    /// player typed — see [`Grammar::read`] for why that is counted.
    near: usize,
}

/// Try one pattern against the words of a line.
///
/// Returns what each slot captured and how loosely the words matched, or
/// [`None`].
///
/// # A literal may skip; a slot runs to the next literal
///
/// A literal passes over input it does not match — that is what lets *"can you
/// take me over to the laboratory"* reach *"take me to the {place}"*, and it is
/// the whole reason a grammar beats a synonym list.
///
/// **A slot takes everything up to the next literal, or the rest of the line if
/// there is none**, rather than exactly one word. Taking one word was the first
/// attempt and it scored the grammar at zero: in that same sentence the slot
/// landed on `over` and answered `attend over`. Handing the extra words on is
/// right rather than lax — a reader keeps the player's words and
/// `parser::resolve` picks the name out of them, which is the same division of
/// labour the trained reader will work under.
fn capture(pattern: &Pattern, words: &[&str]) -> Option<Capture> {
    let mut slots: Vec<(String, String)> = Vec::new();
    let mut near = 0usize;
    let mut at = 0usize;

    for (index, piece) in pattern.pieces.iter().enumerate() {
        match piece {
            Piece::Word(wanted) => {
                let found = words[at..].iter().position(|word| is_near(word, wanted))?;
                if words[at + found] != wanted {
                    near += 1;
                }
                at += found + 1;
            }
            Piece::Slot(name) => {
                // A slot is never empty, so the search for what ends it starts
                // one word along.
                if at >= words.len() {
                    return None;
                }
                let next = pattern.pieces[index + 1..]
                    .iter()
                    .find_map(|piece| match piece {
                        Piece::Word(word) => Some(word),
                        Piece::Slot(_) => None,
                    });
                let end = next
                    .and_then(|wanted| {
                        words[at + 1..]
                            .iter()
                            .position(|word| is_near(word, wanted))
                            .map(|found| at + 1 + found)
                    })
                    .unwrap_or(words.len());
                slots.push((name.clone(), words[at..end].join(" ")));
                at = end;
            }
        }
    }
    Some(Capture { slots, near })
}

impl Augur for Grammar {
    fn read(&self, line: &str) -> Vec<String> {
        // Whole, filler included — see `compile`. The pattern keeps its function
        // words, so the input has to keep them too or they could never match.
        //
        // **Folded by the parser's own rule**, rather than lowercased and split.
        // `fold` sheds the trailing punctuation the matcher sheds, and without
        // it an exactly-typed `powder.` was a *near* miss of `powder`: the
        // tie-break below applied backwards, with a template that matched every
        // word exactly losing to one that did not.
        let tokens = Tokens::split(line);
        let words: Vec<&str> = tokens.words().iter().map(|word| word.matching).collect();
        if words.is_empty() {
            return Vec::new();
        }

        // **Every template that matches, most insistent first.** A grammar
        // cannot tell `run {script}` from `run the {place}` — nothing in the
        // sentence says which `night_watch` is — so it offers both and lets the
        // caller keep whichever resolves against the actual room.
        let mut found: Vec<(usize, usize, String)> = Vec::new();
        for pattern in &self.patterns {
            // **Stop when nothing left can reach the list.** These are in weight
            // order, so once `MAX_READINGS` distinct readings are held at a
            // higher weight, no lighter pattern can displace one — and scanning
            // all 2,900 templates, fuzzily, against every line the player types
            // is work whose answer is thrown away.
            if found
                .get(MAX_READINGS - 1)
                .is_some_and(|(weight, _, _)| pattern.weight < *weight)
            {
                break;
            }
            let Some(Capture { slots, near }) = capture(pattern, &words) else {
                continue;
            };
            // **Substituted by name, not by position**, because a phrasing may
            // name the slots in a different order from the command. *"load the
            // {place} with {reagent}"* means `move {reagent} {place}`, and
            // filling markers in the order they were captured put the room in
            // the reagent's slot and the reagent in the room's — a swapped
            // command that resolved to something real and wrong. The grammar
            // reading its own templates back at 94.7% is what found it.
            let mut out = pattern.canonical.clone();
            for (name, value) in slots {
                let marker = format!("{{{name}}}");
                if let Some(open) = out.find(&marker) {
                    out.replace_range(open..open + marker.len(), &value);
                }
            }
            // A canonical that still has a marker in it was not filled, and
            // handing `grind {reagent}` to the parser would have it look for a
            // reagent called `{reagent}`.
            if out.contains('{') {
                continue;
            }
            // Several templates of one entry reach the same command — `smash the
            // sage` and `crush the sage` are both `grind sage`. Held once, at
            // its best match, so a second copy neither spends a reading slot nor
            // makes the count above wrong.
            match found.iter_mut().find(|(_, _, held)| *held == out) {
                Some(held) if near < held.1 => held.1 = near,
                Some(_) => {}
                None => found.push((pattern.weight, near, out)),
            }
        }

        // **Among the equally insistent, the one that matched exactly.** A near
        // word is there for the player's typing — `smsah the sage` — and was
        // never meant to let one template claim another's word. It did: `mill
        // the {reagent}` and `still the {reagent}` insist on the same two words,
        // `still` is near enough to `mill`, and `grind` is written above
        // `distil` — so `still the amber` answered `grind amber`, and the file's
        // order decided it, which is the thing the weight sort exists to stop.
        // `burn` found `churn`, `purge` found `merge`, `stir` found `still`:
        // 2,407 of the corpus's 2,795 outranked readings were this one tie.
        //
        // A tie on both still falls to the file's order, and it is stable on
        // purpose: two templates that match a line equally well are a question
        // only the room can answer, which is why the caller is offered both.
        found.sort_by_key(|(weight, near, _)| (core::cmp::Reverse(*weight), *near));
        found
            .into_iter()
            .take(MAX_READINGS)
            .map(|(_, _, out)| out)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grammar(toml: &str) -> Grammar {
        Grammar::from_phrasings(&Phrasings::parse(toml).expect("parses"))
    }

    const GRIND: &str = r#"
        [[entry]]
        canonical = "grind {reagent}"
        say = ["smash the {reagent}", "turn the {reagent} into powder"]
        holdout = ["pound the {reagent}"]
    "#;

    #[test]
    fn it_reads_a_template_it_was_given() {
        assert_eq!(
            grammar(GRIND)
                .read("smash the sage")
                .first()
                .map(String::as_str),
            Some("grind sage")
        );
    }

    #[test]
    fn it_keeps_the_players_word_rather_than_resolving_it() {
        // A reader never sees the world — binding `sag` to `sage` is the
        // matcher's, and doing it here would need a `Scene` that could go stale.
        assert_eq!(
            grammar(GRIND)
                .read("smash the sag")
                .first()
                .map(String::as_str),
            Some("grind sag")
        );
    }

    #[test]
    fn it_reads_extra_words_and_reordering() {
        // The thing a flat synonym list cannot do, which is why a grammar is
        // worth measuring before a model is built.
        assert_eq!(
            grammar(GRIND)
                .read("could you turn the sage into powder for me")
                .first()
                .map(String::as_str),
            Some("grind sage")
        );
    }

    #[test]
    fn it_abstains_from_a_word_it_was_never_given() {
        // **The ceiling, stated as a test.** No amount of template-writing
        // closes this in general, and it is exactly what a trained reader claims
        // to do better.
        assert!(grammar(GRIND).read("pound the sage").is_empty());
    }

    #[test]
    fn it_abstains_rather_than_answering_with_an_unfilled_slot() {
        // `grind {reagent}` handed to the parser would look for a reagent
        // called `{reagent}`.
        let read = grammar(GRIND).read("smash the");
        assert!(read.is_empty(), "answered {read:?}");
    }

    #[test]
    fn it_offers_both_readings_of_a_sentence_it_cannot_tell_apart() {
        // **The reason the seam returns a list.** Nothing in `run night_watch`
        // says whether `night_watch` is a script or a place, and a reader that
        // cannot see the world has no way to find out. It offers both; the
        // caller keeps whichever resolves.
        //
        // **Both templates are word-for-word identical here, deliberately.**
        // `run {script}` against `run the {place}` used to be the fixture and
        // stopped being ambiguous once `compile` kept filler: `the` is evidence
        // now, so those two are different phrasings and the grammar tells them
        // apart on its own. What is left genuinely undecidable is the case where
        // the words really are the same and only the *kind* of the noun differs.
        let grammar = grammar(
            r#"
            [[entry]]
            canonical = "invoke {script}"
            say = ["run {script}"]
            holdout = ["get {script} going"]

            [[entry]]
            canonical = "wield {place}"
            say = ["run {place}"]
            holdout = ["set {place} off"]
            "#,
        );
        let readings = grammar.read("run night_watch");
        assert!(
            readings.contains(&"invoke night_watch".to_owned())
                && readings.contains(&"wield night_watch".to_owned()),
            "offered only {readings:?}"
        );
    }

    #[test]
    fn it_never_offers_the_same_command_twice() {
        // `smash the sage` and `crush the sage` are both `grind sage`; a second
        // copy spends a reading slot on nothing.
        let grammar = grammar(
            r#"
            [[entry]]
            canonical = "grind {reagent}"
            say = ["smash the {reagent}", "smash up the {reagent}"]
            holdout = ["pound the {reagent}"]
            "#,
        );
        let readings = grammar.read("smash up the sage");
        assert_eq!(readings.iter().filter(|r| *r == "grind sage").count(), 1);
    }

    #[test]
    fn it_offers_no_more_than_the_caller_will_try() {
        // Past a handful, offering more is not offering better.
        assert!(Grammar::builtin().read("start the laboratory").len() <= MAX_READINGS);
    }

    #[test]
    fn a_more_insistent_pattern_is_tried_first() {
        // Otherwise the answer depends on the order the file was written in.
        let grammar = grammar(
            r#"
            [[entry]]
            canonical = "grind {reagent}"
            say = ["work the {reagent}", "work the {reagent} down into powder"]
            holdout = ["pound the {reagent}"]
            "#,
        );
        assert!(grammar.patterns[0].weight >= grammar.patterns[1].weight);
    }

    #[test]
    fn an_exact_word_outranks_a_near_one() {
        // `still` is near enough to `mill` to match it, and `grind` is written
        // first — so before this, the file's order read `still the sage` as
        // grinding it.
        let grammar = grammar(
            r#"
            [[entry]]
            canonical = "grind {reagent}"
            say = ["mill the {reagent}"]
            holdout = ["pound the {reagent}"]

            [[entry]]
            canonical = "distil {reagent}"
            say = ["still the {reagent}"]
            holdout = ["boil the {reagent} away"]
            "#,
        );
        assert_eq!(
            grammar.read("still the sage").first().map(String::as_str),
            Some("distil sage")
        );
        // ...and a typo still lands, which is what nearness is for.
        assert_eq!(
            grammar.read("mil the sage").first().map(String::as_str),
            Some("grind sage")
        );
    }

    #[test]
    fn the_scan_stops_early_without_missing_a_better_match() {
        // `read` stops once `MAX_READINGS` readings are held at a weight nothing
        // left can beat — the 2,900 fuzzy template matches a line used to cost
        // are what that saves. **A pattern of the same weight is still tried**,
        // which is what keeps the exact one below five decoys from being missed.
        let grammar = grammar(
            r#"
            [[entry]]
            canonical = "mix {reagent}"
            say = ["smesh the {reagent}"]

            [[entry]]
            canonical = "distil {reagent}"
            say = ["smush the {reagent}"]

            [[entry]]
            canonical = "digest {reagent}"
            say = ["smosh the {reagent}"]

            [[entry]]
            canonical = "kindle {reagent}"
            say = ["smish the {reagent}"]

            [[entry]]
            canonical = "purge {reagent}"
            say = ["smath the {reagent}"]

            [[entry]]
            canonical = "grind {reagent}"
            say = ["smash the {reagent}"]
            "#,
        );
        let readings = grammar.read("smash the sage");
        assert_eq!(
            readings.first().map(String::as_str),
            Some("grind sage"),
            "the exact template was scanned past: {readings:?}",
        );
        assert!(readings.len() <= MAX_READINGS);
    }

    #[test]
    fn a_word_typed_exactly_is_exact_though_it_carries_a_stop() {
        // The tie-break above reads the *folded* word, as the matcher does.
        // Without that, `powder.` was a near miss of `powder` — so the template
        // that matched every word exactly scored as loosely as the one that did
        // not, and the file's order broke the tie, which is the thing the
        // ordering exists to stop.
        let grammar = grammar(
            r#"
            [[entry]]
            canonical = "mix {reagent}"
            say = ["turn the {reagent} into powdur"]
            holdout = ["fold in the {reagent}"]

            [[entry]]
            canonical = "grind {reagent}"
            say = ["turn the {reagent} into powder"]
            holdout = ["pound the {reagent}"]
            "#,
        );
        assert_eq!(
            grammar
                .read("turn the sage into powder.")
                .first()
                .map(String::as_str),
            Some("grind sage"),
        );
    }

    #[test]
    fn the_builtin_templates_compile_and_read_themselves() {
        // A template the grammar cannot read back is one that will never fire,
        // and it would show in the bench as a command that misses everything.
        let grammar = Grammar::builtin();
        assert!(!grammar.is_empty());
        assert!(
            grammar
                .read("smash the sage")
                .contains(&"grind sage".to_owned()),
            "the builtin templates cannot read their own phrasing back"
        );
    }
}
