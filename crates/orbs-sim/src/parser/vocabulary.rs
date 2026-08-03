//! Three registers in, one register out.
//!
//! DESIGN.md §6: every canonical command carries synonyms across **shell**,
//! **arcane**, and **plain English**, all of which resolve, and the echo always
//! shows the arcane form. This is nearly free — the parser is synonym-based
//! already, and man pages are written per *canonical* command, so the synonym
//! layer costs vocabulary entries rather than prose.
//!
//! Phrases are stored pre-split and matched **longest first**, so `go to` beats
//! `go` and the trailing `to` is never mistaken for filler.

use super::verb::Verb;

/// Which dialect an input phrase belongs to.
///
/// Recorded on every resolution, because the Phase 0 gate (§15) needs to know
/// *which* register newcomers actually reach for — that is the number that says
/// whether the three-register bet paid off.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Register {
    /// The canonical arcane form. What the echo teaches.
    Arcane,
    /// Terminal muscle memory — `cd`, `ls`, `grep`.
    Shell,
    /// What a newcomer guesses — `go to`, `what's here`.
    Plain,
}

/// One way of saying one verb.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Synonym {
    /// What it resolves to.
    pub verb: Verb,
    /// Which dialect it belongs to.
    pub register: Register,
    /// The phrase, pre-split into lowercase words.
    pub words: &'static [&'static str],
}

const fn syn(verb: Verb, register: Register, words: &'static [&'static str]) -> Synonym {
    Synonym {
        verb,
        register,
        words,
    }
}

/// Every recognised phrase. §6.1's table, plus each canonical form.
pub const SYNONYMS: &[Synonym] = &[
    // attend — move to a place
    syn(Verb::Attend, Register::Arcane, &["attend"]),
    syn(Verb::Attend, Register::Shell, &["cd"]),
    syn(Verb::Attend, Register::Plain, &["go", "to"]),
    syn(Verb::Attend, Register::Plain, &["go"]),
    syn(Verb::Attend, Register::Plain, &["enter"]),
    // survey — list what is here
    syn(Verb::Survey, Register::Arcane, &["survey"]),
    syn(Verb::Survey, Register::Shell, &["ls"]),
    syn(Verb::Survey, Register::Shell, &["dir"]),
    syn(Verb::Survey, Register::Plain, &["what's", "here"]),
    syn(Verb::Survey, Register::Plain, &["look"]),
    syn(Verb::Survey, Register::Plain, &["list"]),
    // peruse — read a file
    syn(Verb::Peruse, Register::Arcane, &["peruse"]),
    syn(Verb::Peruse, Register::Shell, &["cat"]),
    syn(Verb::Peruse, Register::Shell, &["less"]),
    syn(Verb::Peruse, Register::Plain, &["read"]),
    syn(Verb::Peruse, Register::Plain, &["open"]),
    syn(Verb::Peruse, Register::Plain, &["show"]),
    // sift — filter for matches
    syn(Verb::Sift, Register::Arcane, &["sift"]),
    // No `find`: shell `find` locates files rather than searching their
    // contents, and it reached `bind` at 750.
    syn(Verb::Sift, Register::Shell, &["grep"]),
    syn(Verb::Sift, Register::Plain, &["look", "for"]),
    syn(Verb::Sift, Register::Plain, &["search"]),
    syn(Verb::Sift, Register::Plain, &["filter"]),
    // status — tower overview
    syn(Verb::Status, Register::Arcane, &["status"]),
    syn(Verb::Status, Register::Plain, &["how", "are", "things"]),
    syn(Verb::Status, Register::Plain, &["overview"]),
    // grimoire — the manual
    syn(Verb::Grimoire, Register::Arcane, &["grimoire"]),
    syn(Verb::Grimoire, Register::Shell, &["man"]),
    syn(Verb::Grimoire, Register::Shell, &["help"]),
    syn(Verb::Grimoire, Register::Shell, &["?"]),
    syn(Verb::Grimoire, Register::Plain, &["how", "do", "i"]),
    syn(Verb::Grimoire, Register::Plain, &["explain"]),
    // verify — detect tampering
    syn(Verb::Verify, Register::Arcane, &["verify"]),
    syn(Verb::Verify, Register::Shell, &["check"]),
    syn(Verb::Verify, Register::Plain, &["inspect"]),
    syn(Verb::Verify, Register::Plain, &["audit"]),
    // undo — revert the last command
    syn(Verb::Undo, Register::Arcane, &["undo"]),
    syn(Verb::Undo, Register::Plain, &["take", "it", "back"]),
    syn(Verb::Undo, Register::Plain, &["revert"]),
    // meditate — fast-forward the clock
    syn(Verb::Meditate, Register::Arcane, &["meditate"]),
    syn(Verb::Meditate, Register::Shell, &["wait"]),
    syn(Verb::Meditate, Register::Shell, &["sleep"]),
    syn(Verb::Meditate, Register::Plain, &["rest"]),
    syn(Verb::Meditate, Register::Plain, &["pass"]),
    // decoct — brew a potion
    syn(Verb::Decoct, Register::Arcane, &["decoct"]),
    syn(Verb::Decoct, Register::Plain, &["brew"]),
    syn(Verb::Decoct, Register::Plain, &["make"]),
    syn(Verb::Decoct, Register::Plain, &["mix"]),
    syn(Verb::Decoct, Register::Plain, &["distil"]),
    // siphon — collect a finished potion
    //
    // Was `decant`, which sat two edits from `decoct` (667) while both are
    // core brewing verbs in a Phase 0 domain. Bare `take` is gone with it: it
    // collided with `make` (decoct) at 750 and "take it back" already means undo.
    syn(Verb::Siphon, Register::Arcane, &["siphon"]),
    syn(Verb::Siphon, Register::Plain, &["collect"]),
    syn(Verb::Siphon, Register::Plain, &["decant"]),
    syn(Verb::Siphon, Register::Plain, &["pour"]),
    // purge — destroy waste
    syn(Verb::Purge, Register::Arcane, &["purge"]),
    syn(Verb::Purge, Register::Shell, &["rm"]),
    syn(Verb::Purge, Register::Plain, &["get", "rid", "of"]),
    syn(Verb::Purge, Register::Plain, &["clean"]),
    syn(Verb::Purge, Register::Plain, &["dump"]),
    // divine — research a fragment
    //
    // Was `decipher`: eight characters, and the third member of a `dec-` prefix
    // that `decoct` and `decant` already shared three ways. `decode` is gone
    // with it — it reached `decoct` at 667 and `study`/`translate` cover it.
    syn(Verb::Divine, Register::Arcane, &["divine"]),
    syn(Verb::Divine, Register::Plain, &["decipher"]),
    syn(Verb::Divine, Register::Plain, &["study"]),
    syn(Verb::Divine, Register::Plain, &["translate"]),
    // scribe — author a script
    //
    // Was `inscribe`: same root, same meaning, two characters shorter.
    syn(Verb::Scribe, Register::Arcane, &["scribe"]),
    syn(Verb::Scribe, Register::Shell, &["vi"]),
    syn(Verb::Scribe, Register::Shell, &["edit"]),
    syn(Verb::Scribe, Register::Plain, &["inscribe"]),
    syn(Verb::Scribe, Register::Plain, &["author"]),
    // bind — attach a script to a trigger
    syn(Verb::Bind, Register::Arcane, &["bind"]),
    syn(Verb::Bind, Register::Shell, &["cron"]),
    syn(Verb::Bind, Register::Plain, &["schedule"]),
    syn(Verb::Bind, Register::Plain, &["automate"]),
    // invoke — run a script
    syn(Verb::Invoke, Register::Arcane, &["invoke"]),
    syn(Verb::Invoke, Register::Shell, &["run"]),
    syn(Verb::Invoke, Register::Shell, &["exec"]),
    syn(Verb::Invoke, Register::Shell, &["./"]),
    syn(Verb::Invoke, Register::Plain, &["cast"]),
    syn(Verb::Invoke, Register::Plain, &["do"]),
];

/// The most words any single phrase spans. Bounds the longest-match window.
pub const LONGEST_PHRASE: usize = 3;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn every_verb_has_a_canonical_arcane_entry() {
        // Without this the echo could name a form the parser will not accept
        // back, which would break the mastery arc at the moment it pays off.
        for verb in Verb::ALL {
            let found = SYNONYMS.iter().any(|entry| {
                entry.verb == verb
                    && entry.register == Register::Arcane
                    && entry.words == [verb.canonical()]
            });
            assert!(found, "{} has no canonical entry", verb.canonical());
        }
    }

    #[test]
    fn every_verb_is_reachable_from_plain_english() {
        // §6: plain English serves "newcomers guessing", and the Phase 0 gate is
        // half non-shell users. A verb reachable only from arcane or shell is
        // invisible to them.
        for verb in Verb::ALL {
            assert!(
                SYNONYMS
                    .iter()
                    .any(|entry| entry.verb == verb && entry.register == Register::Plain),
                "{} has no plain-English synonym",
                verb.canonical()
            );
        }
    }

    #[test]
    fn no_phrase_resolves_to_two_different_verbs() {
        // An ambiguous phrase is a scoring tie forever after, so it is worth
        // catching in the table rather than at runtime.
        let mut seen: Vec<(&[&str], Verb)> = Vec::new();
        for entry in SYNONYMS {
            if let Some((phrase, other)) = seen.iter().find(|(words, _)| *words == entry.words) {
                assert_eq!(
                    *other,
                    entry.verb,
                    "{phrase:?} maps to two verbs: {} and {}",
                    other.canonical(),
                    entry.verb.canonical()
                );
            }
            seen.push((entry.words, entry.verb));
        }
    }

    #[test]
    fn phrases_are_lowercase_and_non_empty() {
        for entry in SYNONYMS {
            assert!(!entry.words.is_empty(), "empty phrase");
            for word in entry.words {
                assert!(!word.is_empty(), "empty word in {:?}", entry.words);
                assert_eq!(
                    *word,
                    word.to_lowercase(),
                    "{word:?} is not lowercase; normalisation would never match it"
                );
            }
        }
    }

    #[test]
    fn longest_phrase_bounds_the_table() {
        let longest = SYNONYMS
            .iter()
            .map(|entry| entry.words.len())
            .max()
            .unwrap_or(0);
        assert_eq!(
            longest, LONGEST_PHRASE,
            "LONGEST_PHRASE is stale; the match window would truncate a phrase"
        );
    }

    #[test]
    fn no_synonym_starts_with_a_filler_word() {
        // Leading filler is stripped *before* the verb is matched, so that
        // "please go to the alembic" finds `go to`. That is only safe while no
        // phrase opens on a word the stripper would eat.
        for entry in SYNONYMS {
            let first = entry.words[0];
            assert!(
                !super::super::normalise::is_filler(first),
                "{:?} opens with filler {first:?}, which leading-filler stripping would eat",
                entry.words
            );
        }
    }

    #[test]
    fn the_three_registers_are_all_populated() {
        let registers: BTreeSet<_> = SYNONYMS.iter().map(|entry| entry.register).collect();
        assert_eq!(registers.len(), 3);
    }
}
