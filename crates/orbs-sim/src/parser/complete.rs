//! What the player might be about to type.
//!
//! Lives here rather than in a frontend because it needs the vocabulary
//! ([`SYNONYMS`](super::SYNONYMS)) and the live [`Scene`], both of which are the
//! parser's — and because rule 2 says both frontends must be able to offer the
//! same thing. It is also the only part of the prompt's editing behaviour that
//! can be tested without a window.
//!
//! # The shape is `rustyline`'s, for its reasons
//!
//! **A range, not an append.** `rustyline`'s `complete` returns *where the
//! completable word starts*; `reedline`'s `Suggestion` carries the span it
//! replaces. That matters more here than it does there, because §6's parser is
//! deliberately fuzzy — `clarty` resolves to `clarity` — and a completer that can
//! only extend what was typed is useless the moment the player has typed a
//! near-miss, which is the player this game is built for.
//!
//! **One string, not `display` apart from `insert`.** `rustyline` and `reedline`
//! both split them, and the split was taken here on the theory that a place is
//! registered by full path and shown by leaf. It is not: the leaf is what the
//! list shows *and* what goes into the line, because §6's matcher accepts it and
//! the echo shows it back. Two fields that every construction site sets equal are
//! two fields that can drift — `common()` read one and the Tab listing the other
//! — so there is one until something genuinely needs two.
//!
//! # What it does not do
//!
//! No debouncing, no caching, no background thread. `fish` has all three and
//! `zsh` recommends async, because their corpus is a filesystem and a history of
//! tens of thousands of lines. Ours is a few dozen scene nouns, in memory, with
//! no I/O — and rule 8 forbids async here regardless. Taking that machinery would
//! be pure cost, which is worth writing down because the prior art all points at
//! it.

use core::ops::Range;

use orbs_render::char_index;

use super::scene::Scene;
use super::verb::{NounKind, Verb};
use super::vocabulary::SYNONYMS;

/// One thing the player might have meant: what a list shows and what a Tab
/// inserts, which are the same string.
pub type Suggestion = String;

/// Everything that could finish the word under the caret.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Completion {
    /// The byte range of the line this replaces — the partial word.
    pub replaces: Range<usize>,
    /// The candidates, in scene order.
    pub candidates: Vec<Suggestion>,
}

impl Completion {
    /// Whether there is anything to offer.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.candidates.is_empty()
    }

    /// The text every candidate agrees on, past what is already typed.
    ///
    /// GNU readline's `compute_lcd_of_matches`: with several matches, Tab
    /// advances to the longest common prefix and lists only when that adds
    /// nothing. Extending as far as everyone agrees is free progress.
    #[must_use]
    pub fn common(&self) -> String {
        let mut candidates = self.candidates.iter();
        let Some(first) = candidates.next() else {
            return String::new();
        };
        let mut shared: Vec<char> = first.chars().collect();
        for other in candidates {
            let keep = shared
                .iter()
                .zip(other.chars())
                .take_while(|(a, b)| **a == *b)
                .count();
            shared.truncate(keep);
        }
        shared.into_iter().collect()
    }
}

/// What could finish the word ending at `caret`.
///
/// `prompt_open` turns completion off entirely: while §6's numbered prompt is
/// waiting, the orb wants a **digit**, and offering verbs there — or ghosting one
/// — walks the player into a dead end. §15 weighs the dead-end rate above the raw
/// resolution rate.
#[must_use]
pub fn complete(line: &str, caret: usize, scene: &Scene, prompt_open: bool) -> Completion {
    if prompt_open {
        return Completion::default();
    }

    let upto = char_index(line, caret);
    let word = word_start(line, upto);
    let partial = &line[word..upto];
    let replaces = word..upto;

    // The first phrase is a verb; anything later fills a slot of the verb that
    // phrase named.
    let leading = line[..word].trim();
    let candidates = if leading.is_empty() {
        verbs(partial, scene)
    } else {
        match verb_of(leading, scene) {
            Some((verb, filled)) => nouns(verb, filled, partial, scene),
            None => Vec::new(),
        }
    };

    Completion {
        replaces,
        candidates,
    }
}

/// Every phrase that could be the verb being typed.
///
/// Canonical names *and* synonyms, **including the multi-word ones**: §6's whole
/// claim is that all three registers reach the same command, and a `words.len()
/// == 1` filter here made `go to`, `look for`, `get rid of`, `what's here`,
/// `how do i` and `take it back` invisible to Tab — seven entries of the plain
/// register, silently second class in the one surface that advertises them.
fn verbs(partial: &str, scene: &Scene) -> Vec<Suggestion> {
    let mut out: Vec<Suggestion> = SYNONYMS
        .iter()
        // Tab must not offer a word the parser would refuse. A per-instrument
        // verb out of its domain is exactly that — see `Scene::offers` — and
        // offering it would walk the player into the dead end §15 weighs above
        // the raw resolution rate.
        .filter(|entry| scene.offers(entry.verb))
        .map(|entry| entry.words.join(" "))
        .filter(|phrase| phrase.starts_with(partial))
        .collect();
    out.sort();
    out.dedup();
    out
}

/// The verb a line has already named, and how many argument slots follow it.
///
/// Matched through [`match_phrase`](super::resolve::match_phrase) — the same
/// function `resolve` ranks with — rather than through
/// [`resolve`](super::resolve) itself: a bare `wield ` has no argument yet, so a
/// full resolution comes back `Incomplete` and yields no intent, which is
/// precisely the moment completion is most wanted.
///
/// Going through the real matcher is what keeps Tab and Enter agreeing. A
/// private single-word scan here disagreed twice over: multi-word phrases were
/// invisible, so `look for ` offered places for `Survey` while the line resolves
/// to `sift`, whose slot is free text and must offer **nothing**; and its
/// `max_by_key` tie-break took the *last* maximum where `resolve` applies
/// `named_exactly` → score → `verb_order`, so a tie would offer one verb's
/// arguments and run another's.
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
            // Score, then **the longer phrase**, then table order. The span
            // tie-break is not decoration: `match_phrase` clamps its
            // `PHRASE_BONUS` at `fuzzy::EXACT`, so a typed-perfectly `look for`
            // scores exactly what a typed-perfectly `look` does, and without
            // this the two-word reading loses to whichever came first.
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
fn nouns(verb: Verb, filled: usize, partial: &str, scene: &Scene) -> Vec<Suggestion> {
    let signature = verb.signature();
    let Some(slot) = signature.get(filled).or_else(|| signature.last()) else {
        return Vec::new();
    };

    // Free text completes nothing. A pattern is whatever the player is searching
    // for and a count is a number; offering the scene's nouns for either would
    // be a lie about what the slot accepts.
    if matches!(slot.kind, NounKind::Pattern | NounKind::Count) {
        return Vec::new();
    }

    scene
        .nouns()
        .iter()
        .filter(|noun| noun.kind == slot.kind || slot.kind == NounKind::Any)
        .filter_map(|noun| {
            // A place is shown and typed as its leaf (§7: players say the place,
            // not the path) — through the same helper the echo uses, so Tab
            // inserts exactly the form the echo will show back.
            let leaf = super::intent::leaf(&noun.name);
            leaf.starts_with(partial).then(|| leaf.to_owned())
        })
        .collect()
}

/// The byte index where the word ending at `upto` begins.
fn word_start(line: &str, upto: usize) -> usize {
    line[..upto].rfind(char::is_whitespace).map_or(0, |at| {
        at + line[at..].chars().next().map_or(1, char::len_utf8)
    })
}

/// Whether a line is only a number — an answer to §6's numbered prompt.
#[must_use]
pub fn is_answer(line: &str) -> bool {
    line.trim().parse::<usize>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tower() -> Scene {
        Scene::new()
            .with(NounKind::Place, "/tower/laboratory")
            .with(NounKind::Place, "/tower/laboratory/mortar_and_pestle")
            .with(NounKind::Place, "/tower/laboratory/balneum_mariae")
            .with(NounKind::Place, "/tower/archive")
            .with(NounKind::Reagent, "sage")
            .with(NounKind::Reagent, "rock-salt")
            .with(NounKind::File, "feed.log")
    }

    fn inserts(line: &str) -> Vec<String> {
        let caret = line.chars().count();
        complete(line, caret, &tower(), false).candidates
    }

    #[test]
    fn the_first_word_completes_to_verbs_in_every_register() {
        // §6 claims all three registers reach the same command, so completing
        // only the arcane form would teach that the other two are lesser.
        let found = inserts("sur");
        assert!(found.contains(&"survey".to_owned()), "{found:?}");

        let shell = inserts("l");
        assert!(shell.contains(&"ls".to_owned()), "{shell:?}");
        assert!(shell.contains(&"look".to_owned()), "{shell:?}");
    }

    #[test]
    fn a_later_word_completes_to_the_nouns_that_slot_accepts() {
        // `wield` takes a place; `move` takes a reagent first. Offering the
        // wrong kind would suggest a command that cannot resolve.
        let places = inserts("wield mo");
        assert_eq!(places, ["mortar_and_pestle"]);

        let reagents = inserts("move sa");
        assert_eq!(reagents, ["sage"]);
    }

    #[test]
    fn a_place_completes_to_its_leaf_not_its_path() {
        // The scene holds `/tower/laboratory/mortar_and_pestle`; inserting that
        // would be valid, unreadable, and not what anyone typed.
        assert_eq!(inserts("wield mortar"), ["mortar_and_pestle"]);
    }

    #[test]
    fn free_text_slots_complete_nothing() {
        // A pattern is whatever the player is searching for and a count is a
        // number. Offering scene nouns for either lies about the slot.
        assert!(inserts("sift ma").is_empty());
        assert!(inserts("meditate 3").is_empty());
    }

    #[test]
    fn the_replaced_range_is_the_partial_word_only() {
        // A range rather than an append, so a candidate can *correct* a
        // near-miss rather than only extend a prefix — which is what §6's fuzzy
        // matching means for a completer.
        let completion = complete("wield mo", 8, &tower(), false);
        assert_eq!(completion.replaces, 6..8);
        assert_eq!(&"wield mo"[completion.replaces], "mo");
    }

    #[test]
    fn the_common_prefix_is_as_far_as_everyone_agrees() {
        // readline's `compute_lcd_of_matches`. Two places share `m`... but only
        // one starts with `mo`, so the whole name is common.
        let completion = complete("wield m", 7, &tower(), false);
        assert_eq!(completion.common(), "mortar_and_pestle");

        // With nothing typed, the two laboratory instruments share nothing.
        let both = complete("wield ", 6, &tower(), false);
        assert!(both.candidates.len() > 1);
        assert_eq!(both.common(), "");
    }

    #[test]
    fn nothing_is_offered_while_a_numbered_prompt_is_open() {
        // The orb wants a digit. Offering verbs — or ghosting one — walks the
        // player into a dead end, which §15 weighs above the resolution rate.
        let completion = complete("sur", 3, &tower(), true);
        assert!(completion.is_empty());
    }

    #[test]
    fn a_multi_word_verb_completes_its_own_slot() {
        // `look for` is `sift`, whose first slot is free text — so it must offer
        // nothing. A single-word scan read the head `look` as `survey` and
        // offered places: Tab suggesting arguments for a verb Enter would not
        // run. §6's whole claim is that the plain register is not second class.
        assert!(
            inserts("look for ").is_empty(),
            "{:?}",
            inserts("look for ")
        );

        // ...and the register is reachable from an empty line at all.
        let plain = inserts("go");
        assert!(plain.contains(&"go to".to_owned()), "{plain:?}");
    }

    #[test]
    fn filler_between_the_verb_and_the_caret_is_not_a_slot() {
        // `move sage to ` has filled one slot, not two. Counting `to` would
        // offer the destination's nouns for the source.
        assert_eq!(inserts("move sage to mo"), ["mortar_and_pestle"]);
    }

    #[test]
    fn an_unknown_verb_offers_nothing_for_its_arguments() {
        assert!(inserts("xyzzy pl").is_empty());
    }

    #[test]
    fn a_digit_is_recognised_as_an_answer() {
        assert!(is_answer("2"));
        assert!(is_answer("  10 "));
        assert!(!is_answer("survey"));
        assert!(!is_answer(""));
    }
}
