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
    syn(Verb::Research, Register::Arcane, &["research"]),
    // **`divine` was the canonical and is kept as a word.** The archive is a
    // room of shelves and readings, and what you do at a lectern is look things
    // up — `research` says that and `divine` says a wizard guessing. Every
    // rename in this table keeps the old spelling working (see
    // `the_words_the_naming_pass_replaced_still_resolve`), because a word the
    // game taught is a word the game owes an answer to.
    syn(Verb::Research, Register::Arcane, &["divine"]),
    syn(Verb::Research, Register::Plain, &["decipher"]),
    syn(Verb::Research, Register::Plain, &["decode"]),
    syn(Verb::Research, Register::Plain, &["study"]),
    syn(Verb::Research, Register::Plain, &["translate"]),
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
    // weave — look at what the work has bought
    //
    // **No shell register, and `tree` in particular is refused.** `status` has
    // none either, so nothing is owed. And in a game whose premise is that the
    // filesystem *is* your duties (§7), `tree` means *list this directory* —
    // a player who types it means `survey`, and it would resolve at 1000 and
    // take over the screen instead. This is the same call the entries above make
    // for `less` and `read`: no fuzzy test catches a collision of *meaning*, so
    // the table has to.
    //
    // `progress` is the plain word a newcomer reaches for, and `talents` and
    // `upgrades` are what they would call the thing after playing anything else.
    // All three verified clean: none scores 600 against any synonym in either
    // direction, and `wea`, `pro`, `tal` and `upg` are unclaimed prefixes.
    syn(Verb::Weave, Register::Arcane, &["weave"]),
    syn(Verb::Weave, Register::Plain, &["progress"]),
    syn(Verb::Weave, Register::Plain, &["talents"]),
    syn(Verb::Weave, Register::Plain, &["upgrades"]),
    // follow — move the archive's reading one cell (§10, `tower::maze`)
    //
    // **Not `step`** (750 against `stop`) and **not `tread`** (800 against
    // `read`, which `peruse` claims). `follow` is 429 against its nearest and
    // shares no three-character prefix with anything — and it is what a player
    // says about a passage.
    syn(Verb::Follow, Register::Arcane, &["follow"]),
    syn(Verb::Follow, Register::Plain, &["walk"]),
    // wander — give the arrow keys the stacks (§10, §19)
    //
    // **The obvious words are all taken or too close.** `enter` is `attend`'s
    // and `walk` is `follow`'s own; `thread` is 667 against `read`, `stride` 667
    // against `scribe`, `delve` 600 against `weave`, `trace` 600 against `twice`
    // — a reading in scope in this very room — and `pace` 750 against `page`.
    //
    // `wander` and `roam` both come in at 500 at worst, over the whole synonym
    // table in both directions, and `wan` and `roa` are unclaimed prefixes.
    // `roam`'s nearest are `read` and `rm`; two edits from the destructive verb
    // in a four-letter word is the shape this vocabulary has twice refused, so
    // it is the *plain* register only and never the word the orb answers in.
    syn(Verb::Wander, Register::Arcane, &["wander"]),
    syn(Verb::Wander, Register::Plain, &["roam"]),
    // The lens (§10). All three are domain-scoped (`Verb::is_operation`), so a
    // near miss here can only ever be a near miss *inside the lens* — which is
    // what makes a three-verb domain affordable at all.
    //
    // **`gaze` is deliberately absent from `scry`'s plain register**: it is 600
    // against `graze`, which nothing owns, and an unclaimed collision is the one
    // §19 records as worse than a claimed one.
    //
    // **`probe` has no shell register**, and neither does `wander`. The obvious
    // words are taken: `open` is `peruse`'s and `run` is `invoke`'s, and this
    // vocabulary's own rule is that an *unclaimed* collision is the dangerous
    // kind — a word owned by two verbs costs a prompt on a typo, which is a
    // price worth paying for nothing here.
    //
    // **`scry` is not a verb, and §10's own word for the domain losing to a
    // three-character prefix is worth the paragraph.** `tests/naming.rs` forbids
    // two canonicals sharing one, having deleted its last exemption on the
    // grounds that an exemption outliving its cause is how a guard stops
    // guarding — and `scr` reaches `scribe`. `probe` opens a reading when none
    // is open, which is `grind`'s move-and-wield idiom one room over.
    syn(Verb::Probe, Register::Arcane, &["probe"]),
    syn(Verb::Probe, Register::Plain, &["spy", "peek", "try"]),
    // `set` is the shell word anyone would reach for; the arcane form is `dial`,
    // which is what a lock has and what a ward is.
    syn(Verb::Dial, Register::Arcane, &["dial"]),
    syn(Verb::Dial, Register::Plain, &["put"]),
    syn(Verb::Dial, Register::Shell, &["set"]),
];

impl Register {
    /// The word for this dialect, as the manual names it.
    ///
    /// It lived privately in `parser::trace`, which exports a column of them for
    /// the balance gate. One table now, because the manual prints the same words
    /// and two copies could disagree about what a register is called.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Arcane => "arcane",
            Self::Shell => "shell",
            Self::Plain => "plain",
        }
    }
}

/// Every way of saying `verb`, in register order, canonical first.
///
/// **An accessor, because there was none.** Three sites filtered the flat table
/// inline and the manual would have been a fourth — and unlike those three, it
/// prints the result, so a phrase joined differently would be visible.
#[must_use]
pub fn synonyms_of(verb: Verb) -> Vec<(Register, String)> {
    let mut out: Vec<(Register, String)> = SYNONYMS
        .iter()
        .filter(|entry| entry.verb == verb)
        .map(|entry| (entry.register, entry.words.join(" ")))
        .collect();
    // `Register` derives `Ord` in declaration order — arcane, shell, plain —
    // which is the order the mastery arc runs in and so the order to read them.
    out.sort();
    out
}

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
