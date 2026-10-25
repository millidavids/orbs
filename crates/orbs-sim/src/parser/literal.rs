//! Whether the orb *looks a line up* or has to *work it out*.
//!
//! The augury (§6) reads anything the player did not type literally; this
//! decides which is which. It routes rather than parses — a literal line still
//! goes through [`analyse`](super::analyse), keeping `Elsewhere`, `InSpell`,
//! `Incomplete` and the domain bonus.
//!
//! Four things count, each a lookup rather than a guess: a bare digit answering
//! a numbered prompt, a tester's `debug_*` door, a spell word
//! ([`spellword::leading`](super::spellword::leading)), and nothing at all.
//!
//! Not an exactly-typed verb: `put` is `dial`'s, so `put the sage in the mortar
//! and grind it` would route into the lens. Whether the parser explained the
//! whole line is [`Analysis`](super::Analysis)' question; this is the narrow
//! half. Build-independent, so a tester and a player route alike.

use super::{normalise, spellword};

/// The prefix every tester's door is spelled with.
///
/// One rule rather than a list of ten, so a door added later is covered.
/// `Sim::submit` matches the doors exactly; this only has to know a line is one.
const DOOR: &str = "debug_";

/// Whether the orb reads `line` by looking it up rather than by working it out.
///
/// See the module docs for what counts and why.
#[must_use]
pub fn is_literal(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return true;
    }

    // A digit answering a numbered prompt. Whether one is *open* is a question
    // about the world, and `Sim::submit` owns that.
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
        // That rule lives on `Analysis`, not here. Pinned because adding the
        // clause back is the obvious mistake, and `put` is what it costs.
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
        // Answered before the matcher, and before the augury for the same
        // reason: `wait for the mortar` has a right answer that is not a command.
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
        // A near miss is what the augury is for: `clarty` reaching `clarity` is
        // the player this game is built for.
        for line in ["grnid sage", "atend laboratory", "srvey"] {
            assert!(!is_literal(line), "{line:?} reads as literal");
        }
    }

    #[test]
    fn no_spell_word_is_also_a_single_verb_word() {
        // The two exact tables must not overlap, or which one answers first
        // decides what a word means. `spellword` is checked first everywhere.
        for word in crate::parser::SpellWord::ALL {
            assert!(
                vocabulary::verb_of_word(word.canonical()).is_none(),
                "{} is both a spell word and a verb word",
                word.canonical()
            );
        }
    }
}
