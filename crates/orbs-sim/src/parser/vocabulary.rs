//! Three registers in, one register out.
//!
//! §6: every canonical command carries shell, arcane and plain-English
//! synonyms, all of which resolve, and the echo shows the arcane form.
//!
//! Phrases are stored pre-split and matched longest first, so `go to` beats `go`
//! and the trailing `to` is never mistaken for filler.
//!
//! The notes beside an entry record why a word was refused: a rename has to
//! re-run those sweeps, and a partial sweep reads exactly like a clean one.

use super::verb::Verb;

/// Which dialect an input phrase belongs to.
///
/// Recorded on every resolution, so §15's gate can see which register
/// newcomers reach for.
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
    // A set phrase, since a leftover `around` is `Incomplete` (§19).
    syn(Verb::Survey, Register::Plain, &["look", "around"]),
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
    // Kept because it reaches `bind` at 750: unclaimed it would resolve there.
    syn(Verb::Sift, Register::Shell, &["find"]),
    syn(Verb::Sift, Register::Plain, &["look", "for"]),
    syn(Verb::Sift, Register::Plain, &["search"]),
    syn(Verb::Sift, Register::Plain, &["filter"]),
    // status — tower overview
    syn(Verb::Status, Register::Arcane, &["status"]),
    syn(Verb::Status, Register::Plain, &["how", "are", "things"]),
    syn(Verb::Status, Register::Plain, &["overview"]),
    // recall — the manual. Not `grimoire`, which names the book you write.
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
    // `menu` and `quit` refuse `leave` and `exit`, one edit from `weave` and
    // `edit`: the typo's prompt would offer *end the session* beside a verb
    // typed all day. A phrase has no collision, so the plain forms are phrases.
    syn(Verb::Menu, Register::Arcane, &["menu"]),
    syn(Verb::Menu, Register::Plain, &["open", "the", "menu"]),
    syn(Verb::Quit, Register::Arcane, &["quit"]),
    syn(Verb::Quit, Register::Shell, &["logout"]),
    syn(Verb::Quit, Register::Plain, &["put", "it", "down"]),
    syn(Verb::Quit, Register::Plain, &["stop", "playing"]),
    // unfurl — read back through the transcript. Not `less` or `read`, which
    // are `peruse`'s, nor `scroll`, whose `scr` reaches `scribe`.
    syn(Verb::Unfurl, Register::Arcane, &["unfurl"]),
    syn(Verb::Unfurl, Register::Shell, &["history"]),
    // Never bare `page`, which fuzzes `purge`.
    syn(Verb::Unfurl, Register::Plain, &["page", "up"]),
    syn(Verb::Unfurl, Register::Plain, &["page", "back"]),
    // meditate — fast-forward the clock. Not `wait`, a spell word (§19).
    syn(Verb::Meditate, Register::Arcane, &["meditate"]),
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
    // empty — turn an instrument out into the store, where `purge` destroys.
    // Not `clear`, one edit from `purge`'s `clean`.
    syn(Verb::Empty, Register::Arcane, &["empty"]),
    syn(Verb::Empty, Register::Plain, &["unload"]),
    // `siphon`'s words, kept claimed (§19) or they scatter onto `purge` and
    // `stop`. Not `take`, one edit from `make`.
    syn(Verb::Empty, Register::Plain, &["collect"]),
    syn(Verb::Empty, Register::Plain, &["decant"]),
    syn(Verb::Empty, Register::Plain, &["pour"]),
    // stop — cancel a working instrument
    syn(Verb::Stop, Register::Arcane, &["stop"]),
    syn(Verb::Stop, Register::Plain, &["cancel"]),
    syn(Verb::Stop, Register::Plain, &["halt"]),
    // Not `damp`, 750 against `purge`'s `dump`.
    syn(Verb::Stop, Register::Plain, &["quench"]),
    // The retired `decoct`'s words (§19), pointed at the recipe so they do not
    // scatter: `make a potion of clarity` hands a newcomer the recipe.
    syn(Verb::Recall, Register::Plain, &["decoct"]),
    syn(Verb::Recall, Register::Plain, &["brew"]),
    syn(Verb::Recall, Register::Plain, &["make"]),
    // §10.1's per-instrument verbs: charge the tool and start it in one line.
    syn(Verb::Grind, Register::Arcane, &["grind"]),
    // Not `pound`, one edit from `pour`.
    syn(Verb::Grind, Register::Plain, &["crush"]),
    // Not `steep`, one edit from `stop`, which throws away a run in flight.
    syn(Verb::Digest, Register::Arcane, &["digest"]),
    syn(Verb::Digest, Register::Plain, &["bathe"]),
    syn(Verb::Mix, Register::Arcane, &["mix"]),
    syn(Verb::Mix, Register::Plain, &["combine"]),
    syn(Verb::Mix, Register::Plain, &["stir"]),
    syn(Verb::Distil, Register::Arcane, &["distil"]),
    syn(Verb::Distil, Register::Plain, &["distill"]),
    // `light` is claimed because it scores 600 against `list`: unclaimed,
    // `light athanor` ran `survey athanor`.
    syn(Verb::Kindle, Register::Arcane, &["kindle"]),
    syn(Verb::Kindle, Register::Plain, &["light"]),
    syn(Verb::Kindle, Register::Plain, &["fire"]),
    // purge — destroy waste
    syn(Verb::Purge, Register::Arcane, &["purge"]),
    syn(Verb::Purge, Register::Shell, &["rm"]),
    syn(Verb::Purge, Register::Plain, &["get", "rid", "of"]),
    syn(Verb::Purge, Register::Plain, &["clean"]),
    syn(Verb::Purge, Register::Plain, &["dump"]),
    // research — read a fragment at the lectern. Every renamed canonical
    // keeps its old word (`the_words_the_naming_pass_replaced_still_resolve`).
    syn(Verb::Research, Register::Arcane, &["research"]),
    syn(Verb::Research, Register::Arcane, &["divine"]),
    syn(Verb::Research, Register::Plain, &["decipher"]),
    syn(Verb::Research, Register::Plain, &["decode"]),
    syn(Verb::Research, Register::Plain, &["study"]),
    syn(Verb::Research, Register::Plain, &["translate"]),
    // scribe — author a script. `write` is claimed: it reaches `wait` at 600.
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
    // weave — what the work has bought. Never `tree`, which means *list this
    // directory* here (§7): a collision of meaning no fuzzy test catches.
    syn(Verb::Weave, Register::Arcane, &["weave"]),
    syn(Verb::Weave, Register::Plain, &["progress"]),
    syn(Verb::Weave, Register::Plain, &["talents"]),
    syn(Verb::Weave, Register::Plain, &["upgrades"]),
    // follow — one cell through a maze. Not `step` (750 against `stop`) or
    // `tread` (800 against `read`).
    syn(Verb::Follow, Register::Arcane, &["follow"]),
    syn(Verb::Follow, Register::Plain, &["walk"]),
    // wander — the arrow keys in the stacks. `thread`, `stride`, `delve`,
    // `trace` and `pace` all collide. `roam` is two edits from `rm`, so plain
    // only.
    syn(Verb::Wander, Register::Arcane, &["wander"]),
    syn(Verb::Wander, Register::Plain, &["roam"]),
    // The lens. Scoped verbs, so a near miss can only land inside the lens.
    // Not `gaze` (600 against `graze`); not `scry`, whose `scr` is `scribe`'s.
    syn(Verb::Probe, Register::Arcane, &["probe"]),
    // One word per `syn`: `words` is one phrase, so `&["spy", "peek", "try"]`
    // would declare `spy peek try` (`multi_word_plain_synonyms_are_pinned`).
    syn(Verb::Probe, Register::Plain, &["spy"]),
    syn(Verb::Probe, Register::Plain, &["peek"]),
    syn(Verb::Probe, Register::Plain, &["try"]),
    syn(Verb::Dial, Register::Arcane, &["dial"]),
    syn(Verb::Dial, Register::Plain, &["put"]),
    syn(Verb::Dial, Register::Shell, &["set"]),
    // The sanctum. `fortify`, `restore`, `mend`, `rally` and `raise` all
    // collide.
    syn(Verb::Muster, Register::Arcane, &["muster"]),
    syn(Verb::Muster, Register::Arcane, &["marshal"]),
    syn(Verb::Muster, Register::Plain, &["repair"]),
    syn(Verb::Muster, Register::Plain, &["renew"]),
    // `heave`, `drag`, `bring` and `fetch` all collide.
    syn(Verb::Haul, Register::Arcane, &["haul"]),
    syn(Verb::Haul, Register::Plain, &["carry"]),
    syn(Verb::Haul, Register::Plain, &["bear"]),
    // The menagerie. Not `evoke` or `intone` (667 against `invoke`), nor
    // `call` (750 against `wall`).
    syn(Verb::Summon, Register::Arcane, &["summon"]),
    syn(Verb::Summon, Register::Arcane, &["conjure"]),
    syn(Verb::Summon, Register::Plain, &["raise"]),
    // `draw`, `sketch`, `pick` and `choose` sit in other verbs' phrasings;
    // `set`, `assign`, `etch`, `carve`, `wire` and `paint` collide.
    syn(Verb::Limn, Register::Arcane, &["limn"]),
    syn(Verb::Limn, Register::Plain, &["fashion"]),
    // The satchel's push. `que` also prefixes `quench`, accepted and pinned in
    // `ambiguous_synonym_prefixes_are_known`; `stow` and `stash` lose worse.
    syn(Verb::Queue, Register::Arcane, &["queue"]),
    syn(Verb::Queue, Register::Plain, &["queue"]),
    // The bailey, which has no shell register, nor do the other new domains.
    // `siege` is not a verb (600 against `sing`, while that was one).
    syn(Verb::Defend, Register::Arcane, &["defend"]),
    syn(Verb::Defend, Register::Arcane, &["engage"]),
    syn(Verb::Defend, Register::Plain, &["guard"]),
    // `entreat`, `beg` and `beseech` collide; `parley` reads as a truce.
    syn(Verb::Petition, Register::Arcane, &["petition"]),
    syn(Verb::Petition, Register::Plain, &["ask"]),
    // Not `send`, 750 against the spell word `end`.
    syn(Verb::Deploy, Register::Arcane, &["deploy"]),
    syn(Verb::Deploy, Register::Plain, &["order"]),
    // Not `drink`, 600 against `grind`: a potion mid-siege would start a brew.
    syn(Verb::Quaff, Register::Arcane, &["quaff"]),
    syn(Verb::Quaff, Register::Plain, &["sip"]),
    // `commit`, `assign` and `stake` collide on prefixes.
    syn(Verb::Pledge, Register::Arcane, &["pledge"]),
    syn(Verb::Pledge, Register::Arcane, &["allot"]),
    syn(Verb::Pledge, Register::Plain, &["apply"]),
    // Not `wait`, a spell word; `stand`, `ready`, `abide`, `yield`, `endure`,
    // `settle`, `pass` and `brace` all collide.
    syn(Verb::Hold, Register::Arcane, &["hold"]),
    syn(Verb::Hold, Register::Plain, &["watch"]),
    // The forge's three, swept against each other too: `imbue` once reached
    // `imbued` at 975.
    syn(Verb::Imbue, Register::Arcane, &["imbue"]),
    syn(Verb::Imbue, Register::Plain, &["enchant"]),
    syn(Verb::Snap, Register::Arcane, &["snap"]),
    syn(Verb::Snap, Register::Plain, &["flip"]),
    // Not `settle` (834 against `mettle`) or `release` (against `relocate`).
    syn(Verb::Anneal, Register::Arcane, &["anneal"]),
    // Not `finish`, whose `fin` reaches `find`.
    syn(Verb::Anneal, Register::Plain, &["tumble"]),
];

impl Register {
    /// The word for this dialect, as the manual and the trace name it.
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
#[must_use]
pub fn synonyms_of(verb: Verb) -> Vec<(Register, String)> {
    let mut out: Vec<(Register, String)> = SYNONYMS
        .iter()
        .filter(|entry| entry.verb == verb)
        .map(|entry| (entry.register, entry.words.join(" ")))
        .collect();
    // Declaration order is the mastery arc's: arcane, shell, plain.
    out.sort();
    out
}

/// Every one-word way of saying any verb, with the verb that claims it.
///
/// The whole typed vocabulary. `tower::scene_at` registers these as
/// [`NounKind::Command`](super::NounKind) and hands them to
/// [`Scene::knowing`](super::Scene::knowing), so a verb's word never fuzzes
/// into a noun. Phrases are absent, or `to` and `it` would be known words.
///
/// An iterator, because the callers run per keystroke and per drawn line.
pub fn single_words() -> impl Iterator<Item = (&'static str, Verb)> {
    SYNONYMS
        .iter()
        .filter(|entry| entry.words.len() == 1)
        .map(|entry| (entry.words[0], entry.verb))
}

/// The verb a single word names, canonical or synonym.
///
/// So `recall walk` reaches `follow`'s page rather than fuzzing into the maze's
/// `wall` reading (§19).
#[must_use]
pub fn verb_of_word(word: &str) -> Option<Verb> {
    single_words()
        .find(|(known, _)| known.eq_ignore_ascii_case(word))
        .map(|(_, verb)| verb)
}

/// The most words any single phrase spans. Bounds the longest-match window.
pub const LONGEST_PHRASE: usize = 3;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn every_verb_has_a_canonical_arcane_entry() {
        // Or the echo could teach a form the parser will not take back.
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
        // §6: a verb only arcane or shell reaches is invisible to a newcomer.
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
        // Leading filler is stripped before the verb is matched, so
        // "please go to the laboratory" finds `go to`.
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
