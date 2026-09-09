//! Whether the orb *looks a line up* or has to *work it out*.
//!
//! The augury (DESIGN.md §6) reads anything the player did not type literally,
//! and this is the predicate that decides which is which. It is deliberately a
//! **routing** question and not a parsing one: a literal line goes through
//! [`analyse`](super::analyse) exactly as it always has, keeping
//! [`Elsewhere`](super::Resolution::Elsewhere),
//! [`InSpell`](super::Resolution::InSpell),
//! [`Incomplete`](super::Resolution::Incomplete), the domain bonus and the
//! tolerated-collision scoring intact.
//!
//! **It is not a shortcut past the matcher**, and that distinction is the whole
//! reason this file is small. A hash lookup that answered a literal line itself
//! would lose all four of those behaviours — `mix` typed in the archive would
//! stop saying *"there is nothing here to mix with"* — and `scripts/dumps.sh`
//! would stop being a proof that nothing changed.
//!
//! # What counts as literal
//!
//! Four things, and each is a lookup rather than a guess:
//!
//! - **A bare digit**, which answers a numbered prompt. `Sim::submit` checks
//!   this before everything else, and routing one to the augury would make the
//!   prompt rhetorical — it asks a question and then interprets the answer as a
//!   command.
//! - **A tester's door.** Every one is spelled `debug_*`, matched exactly, and
//!   checked before the parser.
//! - **A spell word**, which is exact and case-folded already
//!   ([`spellword::leading`](super::spellword::leading)).
//! - **Nothing at all.** An empty line is not a question anyone needs answered.
//!
//! # Why an exactly-typed verb is *not* on that list
//!
//! It was, for one commit, and it was wrong. **`put` is a plain-register synonym
//! for `dial`** (the lens), `take` and `make` are claimed, and so are `find`,
//! `set`, `hold`, `show` and `open`. Half of ordinary English opens on a word
//! some register has spoken for, so *"the head is a word the game knows"* routes
//! `put the sage in the mortar and grind it` away from the augury and into a
//! lens command with eight words left over.
//!
//! The distinction a router actually needs is **did the parser explain the whole
//! line**, not *did it recognise the first word* — and that cannot be answered
//! from the text. It needs the candidate scores, which is
//! [`Analysis`](super::Analysis)' question rather than this file's.
//!
//! So this predicate is the narrow half: the lines that must never reach the
//! augury *whatever* the world looks like. The verb-exactness half rides on the
//! analysis.
//!
//! # The answer does not depend on the build
//!
//! A `debug_*` line is literal in a release binary too, where the door does not
//! exist and the line is an ordinary miss. Making the predicate build-dependent
//! would route the game a tester plays differently from the game that ships,
//! and `scripts/play.sh` — which runs against a debug build — would stop
//! exercising the path players get.

use super::{normalise, spellword};

/// The prefix every tester's door is spelled with.
///
/// One rule rather than a list of ten, so a door added later is covered without
/// anyone remembering to come here. `Sim::submit` matches the doors themselves
/// exactly; this only has to know that a line *is* one.
const DOOR: &str = "debug_";

/// Whether the orb reads `line` by looking it up rather than by working it out.
///
/// See the module documentation for what counts and why. Pure, allocation-light,
/// and no more expensive than the first stage of the matcher it routes around.
#[must_use]
pub fn is_literal(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return true;
    }

    // A digit answering a numbered prompt. Asked without reference to whether a
    // prompt is *open*: this is a question about the text, and `Sim::submit`
    // owns the question about the world.
    if trimmed.parse::<usize>().is_ok() {
        return true;
    }

    let tokens = normalise::Tokens::split(trimmed);
    let all = tokens.words();
    let Some(first) = all.first() else {
        return true;
    };

    if first.matching.starts_with(DOOR) {
        return true;
    }

    spellword::leading(trimmed).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Verb, vocabulary};

    #[test]
    fn a_verb_is_not_literal_on_its_own_account() {
        // The routing rule that keeps a canonical command away from the model
        // lives on `Analysis`, not here — see the module docs. Pinned because
        // adding the clause back is the obvious-looking mistake, and `put` is
        // what it costs.
        for verb in Verb::ALL {
            assert!(
                !is_literal(verb.canonical()),
                "{} reads as literal on text alone",
                verb.canonical()
            );
        }
    }

    #[test]
    fn a_word_english_shares_with_a_verb_is_not_literal() {
        // `put` is a plain-register `dial` synonym, `take` and `make` are
        // claimed, and a sentence opening on one of them is still a sentence.
        for line in [
            "put the sage in the mortar and grind it",
            "take the husks out of the mortar",
            "make me a potion of clarity",
        ] {
            assert!(!is_literal(line), "{line:?} reads as literal");
        }
    }

    #[test]
    fn a_bare_digit_is_literal() {
        // Otherwise the numbered prompt asks a question and then reads the
        // answer as a command, which is the dead end §15 weighs most.
        for line in ["1", "2", " 3 ", "10"] {
            assert!(is_literal(line), "{line:?} is not literal");
        }
    }

    #[test]
    fn every_door_is_literal_in_either_build() {
        for line in [
            "debug_spawn sage 2",
            "debug_spell brew",
            "debug_take 1",
            "debug_ward",
            "debug_renown 40",
            "debug_siege",
            "debug_learn brewing",
            "debug_reach",
            "debug_swap",
            "debug_course",
        ] {
            assert!(is_literal(line), "{line:?} is not literal");
        }
    }

    #[test]
    fn a_spell_word_is_literal() {
        // They are answered before the matcher and must be answered before the
        // augury for the same reason: `wait for the mortar` typed at the prompt
        // has a right answer, and it is not a command.
        for line in ["wait for the mortar_and_pestle", "repeat 3", "end", "if"] {
            assert!(is_literal(line), "{line:?} is not literal");
        }
    }

    #[test]
    fn nothing_at_all_is_literal() {
        for line in ["", "   ", "\t"] {
            assert!(is_literal(line), "{line:?} is not literal");
        }
    }

    #[test]
    fn the_phrasings_the_augury_exists_for_are_not_literal() {
        for line in [
            "turn the sage into powder",
            "smash the sage",
            "i need powdered sage",
            "what should i do next",
        ] {
            assert!(!is_literal(line), "{line:?} reads as literal");
        }
    }

    #[test]
    fn a_typo_is_not_literal() {
        // A near miss is exactly what the augury is for. If typos routed around
        // it, the model would be handed its own work back one slip at a time —
        // and `clarty` reaching `clarity` is the player this game is built for.
        for line in ["grnid sage", "atend laboratory", "srvey"] {
            assert!(!is_literal(line), "{line:?} reads as literal");
        }
    }

    #[test]
    fn no_spell_word_is_also_a_single_verb_word() {
        // The two exact tables must not overlap, or which one answers first
        // decides what a word means. `spellword` is checked before the matcher
        // in `analyse` and before everything here; this pins that they never
        // have to disagree.
        for word in crate::parser::SpellWord::ALL {
            assert!(
                vocabulary::verb_of_word(word.canonical()).is_none(),
                "{} is both a spell word and a verb word",
                word.canonical()
            );
        }
    }
}
