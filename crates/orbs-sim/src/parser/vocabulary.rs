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
    syn(Verb::Sift, Register::Shell, &["grep"]),
    // `find` is kept despite reaching `bind` at 750 — *because* it does. An
    // unclaimed `find` resolves to `bind`, so dropping it does not remove the
    // collision, it converts a prompt into a wrong command. Shell `find` locates
    // files rather than searching contents, but this game has no file-finding
    // verb, so search is the only thing a player can mean.
    syn(Verb::Sift, Register::Shell, &["find"]),
    syn(Verb::Sift, Register::Plain, &["look", "for"]),
    syn(Verb::Sift, Register::Plain, &["search"]),
    syn(Verb::Sift, Register::Plain, &["filter"]),
    // status — tower overview
    syn(Verb::Status, Register::Arcane, &["status"]),
    syn(Verb::Status, Register::Plain, &["how", "are", "things"]),
    syn(Verb::Status, Register::Plain, &["overview"]),
    // recall — the manual. **`grimoire` is deliberately not here**: the word
    // names `/grimoire`, the book of spells the player writes, and one word
    // cannot be both the reference you read and the book you write in. Released
    // rather than re-pointed, because §6.1's "a released word does not stop
    // resolving" rule is about *shipped* vocabulary and nothing has shipped.
    syn(Verb::Recall, Register::Arcane, &["recall"]),
    syn(Verb::Recall, Register::Shell, &["man"]),
    syn(Verb::Recall, Register::Shell, &["help"]),
    syn(Verb::Recall, Register::Shell, &["?"]),
    syn(Verb::Recall, Register::Plain, &["how", "do", "i"]),
    syn(Verb::Recall, Register::Plain, &["explain"]),
    // verify — detect tampering
    syn(Verb::Verify, Register::Arcane, &["verify"]),
    syn(Verb::Verify, Register::Shell, &["check"]),
    syn(Verb::Verify, Register::Plain, &["inspect"]),
    syn(Verb::Verify, Register::Plain, &["audit"]),
    // undo — revert the last command
    syn(Verb::Undo, Register::Arcane, &["undo"]),
    syn(Verb::Undo, Register::Plain, &["take", "it", "back"]),
    syn(Verb::Undo, Register::Plain, &["revert"]),
    // unfurl — read back through what the orb has said.
    //
    // **No `less` and no `read`**: both are `peruse`'s, and `peruse` reads a
    // *file* while this reads the transcript. Two words for two different
    // surfaces is the collision the naming pass exists to prevent, and the
    // player who types `less` means the log they can name.
    //
    // **And no `scroll` either.** It was the obvious plain word and `scr` then
    // reached `scribe` *and* `unfurl` — caught by
    // `ambiguous_synonym_prefixes_are_known` before it shipped, which is the
    // whole reason that test pins a set rather than counting one.
    //
    // `page up` is better than the word it replaced anyway: it is what a player
    // would say, and it happens to name the key that has always done this.
    syn(Verb::Unfurl, Register::Arcane, &["unfurl"]),
    syn(Verb::Unfurl, Register::Shell, &["history"]),
    //
    // **The phrases only, never bare `page`.** On its own it fuzzy-matches
    // `purge` — and a collision between *read back* and *destroy what is in
    // this* is not one to tolerate, whichever way round it resolves. `page up`
    // and `page back` are what a player says anyway, and neither is near
    // anything.
    syn(Verb::Unfurl, Register::Plain, &["page", "up"]),
    syn(Verb::Unfurl, Register::Plain, &["page", "back"]),
    // meditate — fast-forward the clock
    syn(Verb::Meditate, Register::Arcane, &["meditate"]),
    // **`wait` left this list for the spell vocabulary** (§19). It was
    // `meditate`'s shell synonym and is now §8's smallest control structure, and
    // one word cannot be both: `wait for the mortar` at the prompt has to mean
    // the same thing it means in a spell, or the editor teaches a line that
    // destroys something when typed.
    //
    // `sleep` is the shell word that carries the sense here — `wait` was always
    // the weaker of the two for *"let time pass"*, and `sleep 30` is the one a
    // shell native reaches for anyway.
    syn(Verb::Meditate, Register::Shell, &["sleep"]),
    syn(Verb::Meditate, Register::Plain, &["rest"]),
    syn(Verb::Meditate, Register::Plain, &["pass"]),
    // move — carry a reagent between places (§10.1)
    syn(Verb::Move, Register::Arcane, &["move"]),
    syn(Verb::Move, Register::Shell, &["mv"]),
    syn(Verb::Move, Register::Plain, &["transfer"]),
    syn(Verb::Move, Register::Plain, &["transport"]),
    syn(Verb::Move, Register::Plain, &["relocate"]),
    // wield — set an instrument working
    syn(Verb::Wield, Register::Arcane, &["wield"]),
    syn(Verb::Wield, Register::Plain, &["use"]),
    syn(Verb::Wield, Register::Plain, &["begin"]),
    // empty — turn an instrument out into the store
    //
    // The counterpart of `purge`, and the distinction is the whole of §10.1's
    // byproduct rule: `purge` destroys what you did not mean to make, `empty`
    // *keeps* it. Husks are the mortar's leavings and the water bath's input, so
    // the loop that throws them away is the loop that never finds route B.
    //
    // Not `clear`: it is one edit from `clean`, which `purge` claims, and
    // confusing "put this somewhere safe" with "destroy it" is the one collision
    // this domain can least afford — the same reasoning that kept `damp` out.
    syn(Verb::Empty, Register::Arcane, &["empty"]),
    syn(Verb::Empty, Register::Plain, &["unload"]),
    // **`siphon`'s words, inherited** (§19). §6.1's rule is that a released word
    // does not stop resolving — it resolves to whatever it is nearest — so
    // leaving `collect`, `decant` and `pour` unclaimed would scatter them across
    // `purge` and `stop`, which are the two verbs in this room a mistake costs
    // most. `empty` is the honest heir: it is what taking things out of a tool
    // is called now.
    //
    // `take` is deliberately **not** inherited. It sat one edit from `make`
    // (`recall`) and was only safe while it belonged to a verb with a `Place`
    // signature; on `empty` it would be the same collision with none of the
    // separation.
    syn(Verb::Empty, Register::Plain, &["collect"]),
    syn(Verb::Empty, Register::Plain, &["decant"]),
    syn(Verb::Empty, Register::Plain, &["pour"]),
    // stop — cancel a working instrument
    syn(Verb::Stop, Register::Arcane, &["stop"]),
    syn(Verb::Stop, Register::Plain, &["cancel"]),
    syn(Verb::Stop, Register::Plain, &["halt"]),
    // Not `damp`: it scored 750 against `dump` (purge), and confusing "stop the
    // athanor" with "destroy what is in it" is the one collision this domain
    // cannot afford. `quench` is the better word anyway.
    syn(Verb::Stop, Register::Plain, &["quench"]),
    // The retired brewing verb (§19). `decoct` no longer exists as a command:
    // brewing is §10.1's pipeline, and the only single line that makes a potion
    // is a spell the player wrote.
    //
    // Every word it owned stays **claimed**, pointed at the recipe. The Phase 0
    // naming pass established why: a released word does not stop resolving, it
    // resolves to whatever it is nearest — an unclaimed `decant` landed on
    // `decoct`. Releasing these five would scatter them across `divine`,
    // `siphon` and `meditate` silently.
    //
    // It also answers the newcomer's sentence usefully. §15 chose brewing to
    // gate the parser because *"a shell-naive tester immediately understands
    // 'make a potion'"*, and `make a potion of clarity` -> `recall clarity`
    // hands them the recipe, which is the tutorial entry point.
    // `mix` and `distil` **left** for §10.1's per-instrument verbs below. They
    // are not lost to the manual: `mix` and `distil` take a `Reagent` while
    // `recall` takes a `Topic`, and the two never resolve to the same reading
    // because the *scene* decides. `distil clarity` finds no reagent called
    // `clarity` — it is a recipe output, a Topic — so the manual wins; `distil
    // clarified-draught` finds the reagent on the bench, so the alembic wins.
    // The tutorial sentence survives on `brew`, `make` and `decoct`.
    syn(Verb::Recall, Register::Plain, &["decoct"]),
    syn(Verb::Recall, Register::Plain, &["brew"]),
    syn(Verb::Recall, Register::Plain, &["make"]),
    // §10.1's per-instrument verbs: charge the tool and start it in one line.
    //
    // Four commands a stage — `move`, `wield`, `siphon`, `purge` — is the loop
    // as first built, and the two in the middle are the ones a player types
    // most. Naming the *operation* rather than the tool collapses the first two
    // and reads as the domain's own language: you grind sage, you do not move
    // sage into a mortar and then operate the mortar.
    //
    // `wield` stays. It is the general form, it is what a script writes when the
    // instrument is the variable, and it is the only way to work a tool a verb
    // has not been coined for.
    syn(Verb::Grind, Register::Arcane, &["grind"]),
    syn(Verb::Grind, Register::Plain, &["crush"]),
    // `pound` is the other word for a mortar and sits one edit from `pour`,
    // which collects a finished potion. Left unclaimed it resolves *to* `pour`
    // at 800, which is wrong but harmless — an empty instrument refuses. Claimed
    // for the mortar it would make `pour` a coin flip in both directions, and
    // `pour` is the verb that ends a stage. Not worth one more synonym.
    //
    // `digest` is the alchemical term for gentle heating in a water bath, and it
    // is three edits from anything else here. `steep` is what a player reaches
    // for and is one edit from **both** `sleep` and `stop` — and `stop` cancels a
    // run in flight. A typo that throws away six minutes of brewing is not a
    // trade for a synonym nobody needs.
    syn(Verb::Digest, Register::Arcane, &["digest"]),
    syn(Verb::Digest, Register::Plain, &["bathe"]),
    syn(Verb::Mix, Register::Arcane, &["mix"]),
    syn(Verb::Mix, Register::Plain, &["combine"]),
    syn(Verb::Mix, Register::Plain, &["stir"]),
    syn(Verb::Distil, Register::Arcane, &["distil"]),
    syn(Verb::Distil, Register::Plain, &["distill"]),
    // The athanor's own verb. It is the odd one of the five: lighting a fire is
    // not a run, so it takes no Focus slot and produces nothing — but it charges
    // and starts exactly like the others, which is the whole reason it belongs
    // here rather than under `wield`. `kindle charcoal` is `move charcoal to
    // athanor` and `wield athanor`; bare `kindle` relights what was banked,
    // which is the last line of every script loop.
    //
    // `light` was left out of the first naming pass *because* it scores 600
    // against `list` (survey). That was the wrong lesson from the right number:
    // an unclaimed word does not stop resolving, it resolves to whatever it is
    // nearest — so `light athanor` silently ran `survey athanor`, showed an
    // empty instrument, and read as "the fuel is gone and it will not relight".
    // Claimed, an exact `light` scores 1000 and beats the fuzzy `list` outright,
    // and it is the most natural English there is for lighting a fire.
    syn(Verb::Kindle, Register::Arcane, &["kindle"]),
    syn(Verb::Kindle, Register::Plain, &["light"]),
    syn(Verb::Kindle, Register::Plain, &["fire"]),
    // siphon — collect a finished potion
    //
    // Was `decant`, which sat two edits from `decoct` (667) while both are core
    // brewing verbs in a Phase 0 domain. `decant` and `take` are both kept: an
    // unclaimed `decant` resolves to `decoct` and an unclaimed `take` reaches it
    // through `make` (750), so releasing either would brew when the player meant
    // to collect. Claimed, they cost a prompt on a typo instead.
    //
    // "take it back" still reaches undo: three words beat one on longest match.
    // purge — destroy waste
    syn(Verb::Purge, Register::Arcane, &["purge"]),
    syn(Verb::Purge, Register::Shell, &["rm"]),
    syn(Verb::Purge, Register::Plain, &["get", "rid", "of"]),
    syn(Verb::Purge, Register::Plain, &["clean"]),
    syn(Verb::Purge, Register::Plain, &["dump"]),
    // divine — research a fragment
    //
    // Was `decipher`: eight characters, and the third member of a `dec-` prefix.
    // `decipher` and `decode` are both kept — `decode` reaches `decoct` at 667,
    // so releasing it would make "decode this fragment" brew a potion.
    syn(Verb::Divine, Register::Arcane, &["divine"]),
    syn(Verb::Divine, Register::Plain, &["decipher"]),
    syn(Verb::Divine, Register::Plain, &["decode"]),
    syn(Verb::Divine, Register::Plain, &["study"]),
    syn(Verb::Divine, Register::Plain, &["translate"]),
    // scribe — author a script
    //
    // Was `inscribe`: same root, same meaning, two characters shorter. `write`
    // reaches `wait` (meditate) at exactly 600, so it stays claimed here.
    syn(Verb::Scribe, Register::Arcane, &["scribe"]),
    syn(Verb::Scribe, Register::Shell, &["vi"]),
    syn(Verb::Scribe, Register::Shell, &["edit"]),
    syn(Verb::Scribe, Register::Plain, &["inscribe"]),
    syn(Verb::Scribe, Register::Plain, &["write"]),
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
        // "please go to the laboratory" finds `go to`. That is only safe while no
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
