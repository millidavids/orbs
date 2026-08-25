//! What a Tab press does to a line — the prompt's and the spell editor's.
//!
//! # Why it is here and not on `Line`
//!
//! It was on [`Line`](crate::Line), which is the prompt's buffer, and the spell
//! editor has a buffer of its own. Giving the editor Tab meant either a second
//! implementation of readline's rules — extend to the longest common prefix,
//! list when that adds nothing, then cycle — or this.
//!
//! A second one would have drifted. The rules are not obvious enough to
//! reconstruct: *"the first Tab lists **without** changing the line"* is bash's
//! default and the opposite of what a fresh implementation reaches for, and the
//! staleness guard inside `advance` exists because a forgotten cancellation
//! otherwise splices a candidate into the middle of a word.
//!
//! # A decision, not a mutation
//!
//! [`tab`] takes the text and hands back what should happen to it. The caller
//! applies it, because the two callers keep their carets differently — `Line`
//! counts characters, the editor keeps a row and a column — and a shared
//! function that tried to move both would need to know about both.

use core::ops::Range;

use orbs_sim::parser::Expectation;

/// Where a run of Tab presses has got to.
///
/// Repeated Tab **cycles** through the candidates rather than re-listing them —
/// readline's `menu-complete`, and what a player expects after the first press
/// says there is more than one answer. Holding the candidate list here rather
/// than recomputing it each press is what makes the cycle stable: the scene can
/// change under a player who is mid-cycle (a tick lands, an instrument finishes)
/// and the list they are walking must not reorder beneath them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cycle {
    /// Byte offset where the completable word begins.
    start: usize,
    /// What currently sits there: the word the player typed until the first
    /// advance, then whichever candidate replaced it.
    ///
    /// Kept so [`advance`] can check the line still says what the cycle last
    /// wrote before overwriting it.
    filled: String,
    /// The candidates, in the order the completer offered them.
    candidates: Vec<String>,
    /// Which one is in the line, or `None` before the first advance.
    ///
    /// The first Tab **lists without changing the line** — bash's default, and
    /// the least surprising thing to do to someone who pressed Tab to ask a
    /// question rather than to make a choice. Choosing starts on the second.
    index: Option<usize>,
}

impl Cycle {
    /// Which candidate is in the line, for a listing to mark.
    #[must_use]
    pub const fn at(&self) -> Option<usize> {
        self.index
    }
}

/// What a Tab press decided.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tabbed {
    /// Nothing to offer. §6 forbids a bare error and this is not even a failed
    /// command, so the answer is silence.
    Nothing,
    /// Put `text` at `replaces`; the caret finishes past it.
    Wrote {
        /// The byte range to replace.
        replaces: Range<usize>,
        /// What goes there.
        text: String,
        /// The candidates, if a cycle is running and a listing should show them.
        candidates: Vec<String>,
    },
    /// Show these and leave the line alone.
    Listed(Vec<String>),
}

/// Tab: extend as far as every candidate agrees, then cycle.
///
/// - **First press** extends to the longest common prefix, which is free
///   progress (GNU readline's `compute_lcd_of_matches`). A lone candidate
///   finishes with a trailing space and there is nothing left to choose.
/// - **When extending adds nothing** — the candidates share no more than what is
///   already typed — it lists them and leaves the line alone. That is bash's
///   default, and the least surprising answer to someone who pressed Tab to ask
///   a question rather than to make a choice.
/// - **Every press after that** puts the next candidate in the line, wrapping.
///   This is readline's `menu-complete`, and it is what makes the list an answer
///   rather than a dead end the player types their way out of.
#[must_use]
pub fn tab(text: &str, found: &Expectation, cycle: &mut Option<Cycle>) -> Tabbed {
    if let Some(next) = advance(text, cycle) {
        return next;
    }

    let candidates = found.texts();
    if candidates.is_empty() {
        *cycle = None;
        return Tabbed::Nothing;
    }
    // **`Expectation`'s, not a local one.** The prompt's ghost draws exactly
    // what this is about to take, so two implementations would have the ghost
    // promising something Tab did not do.
    let common = found.common();
    let typed = text
        .get(found.replaces.clone())
        .unwrap_or_default()
        .to_owned();
    let lone = candidates.len() == 1;

    if common.len() > typed.len() {
        // There is agreement left to spend. Take it and stop — a second Tab
        // re-enters here, finds nothing further shared, and starts cycling.
        let mut insert = common;
        if lone {
            insert.push(' ');
        }
        *cycle = None;
        return Tabbed::Wrote {
            replaces: found.replaces.clone(),
            text: insert,
            candidates: Vec::new(),
        };
    }

    // Nothing left to extend. One candidate means the word is already whole, so
    // finish it; more than one starts the cycle on the first of them.
    if lone {
        *cycle = None;
        let mut insert = candidates[0].clone();
        insert.push(' ');
        return Tabbed::Wrote {
            replaces: found.replaces.clone(),
            text: insert,
            candidates: Vec::new(),
        };
    }

    // Arm the cycle over what is already typed, and list. The line is not
    // touched until the next press.
    *cycle = Some(Cycle {
        start: found.replaces.start,
        filled: typed,
        candidates: candidates.clone(),
        index: None,
    });
    Tabbed::Listed(candidates)
}

/// Step a running cycle on, if there is one and the line still matches it.
///
/// The guard is not paranoia. A buffer is edited from several places, and a
/// stale `start` would splice a candidate into the middle of a word. Checking
/// that what sits at the recorded span **is** the candidate that was put there
/// makes a forgotten cancellation harmless — the cycle simply restarts rather
/// than corrupting the line.
fn advance(text: &str, cycle: &mut Option<Cycle>) -> Option<Tabbed> {
    let running = cycle.as_mut()?;
    let span = running.start..running.start.checked_add(running.filled.len())?;
    if text.get(span.clone()) != Some(running.filled.as_str()) {
        *cycle = None;
        return None;
    }

    let next = match running.index {
        None => 0,
        Some(index) => (index + 1) % running.candidates.len(),
    };
    let chosen = running.candidates.get(next)?.clone();
    running.index = Some(next);
    running.filled = chosen.clone();

    Some(Tabbed::Wrote {
        replaces: span,
        text: chosen,
        candidates: running.candidates.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use orbs_sim::parser::{Expected, Reason};

    fn offering(replaces: Range<usize>, words: &[&str]) -> Expectation {
        Expectation {
            replaces,
            expected: words
                .iter()
                .map(|word| Expected {
                    text: (*word).to_owned(),
                    why: Reason::Verb,
                    shape: "",
                })
                .collect(),
        }
    }

    /// One candidate finishes the word and puts a space after it.
    #[test]
    fn a_lone_candidate_is_written_out_whole() {
        let mut cycle = None;
        let found = offering(0..3, &["survey"]);
        let Tabbed::Wrote { replaces, text, .. } = tab("sur", &found, &mut cycle) else {
            panic!("a lone candidate was not written");
        };
        assert_eq!(replaces, 0..3);
        assert_eq!(text, "survey ");
        assert!(cycle.is_none(), "a lone candidate armed a cycle");
    }

    /// Several sharing a prefix spend the agreement first.
    #[test]
    fn agreement_is_spent_before_anything_is_listed() {
        let mut cycle = None;
        let found = offering(0..1, &["grind", "grumble"]);
        let Tabbed::Wrote { text, .. } = tab("g", &found, &mut cycle) else {
            panic!("the shared prefix was not taken");
        };
        assert_eq!(text, "gr", "readline's compute_lcd_of_matches");
        assert!(!text.ends_with(' '), "a choice remains, so no space");
    }

    /// With nothing left to share, the first press lists and the line stands.
    ///
    /// bash's default, and the opposite of what a fresh implementation reaches
    /// for — which is why this lives in one place rather than two.
    #[test]
    fn the_first_press_lists_without_touching_the_line() {
        let mut cycle = None;
        let found = offering(0..2, &["grind", "grumble"]);
        assert_eq!(
            tab("gr", &found, &mut cycle),
            Tabbed::Listed(vec!["grind".to_owned(), "grumble".to_owned()]),
        );
        assert_eq!(
            cycle.as_ref().and_then(Cycle::at),
            None,
            "listing chose one"
        );
    }

    /// ...and every press after that walks the list, wrapping.
    #[test]
    fn later_presses_walk_the_list_and_wrap() {
        let mut cycle = None;
        let found = offering(0..2, &["grind", "grumble"]);
        let mut text = "gr".to_owned();
        let _armed = tab(&text, &found, &mut cycle);

        for expected in ["grind", "grumble", "grind"] {
            let Tabbed::Wrote {
                replaces, text: to, ..
            } = tab(&text, &found, &mut cycle)
            else {
                panic!("the cycle stopped walking");
            };
            text.replace_range(replaces, &to);
            assert_eq!(text, expected);
        }
    }

    /// A cycle whose line has changed under it restarts rather than corrupting.
    #[test]
    fn a_cycle_the_line_no_longer_matches_is_abandoned() {
        let mut cycle = None;
        let found = offering(0..2, &["grind", "grumble"]);
        let _armed = tab("gr", &found, &mut cycle);
        assert!(cycle.is_some());

        // The player typed something else entirely.
        assert!(matches!(tab("xy", &found, &mut cycle), Tabbed::Listed(_)));
    }

    /// Nothing to offer is silence, not an error. §6 forbids a bare error.
    #[test]
    fn nothing_to_offer_says_nothing() {
        let mut cycle = None;
        assert_eq!(tab("zz", &offering(0..2, &[]), &mut cycle), Tabbed::Nothing,);
    }
}
