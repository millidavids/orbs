//! Where a finished line goes: the dispatch, and the top page.
//!
//! [`state`](super::state) is the menu's shape — what is typed, which page is
//! showing, what it last refused. This is the half that *answers*: it takes the
//! line, deals with `back`, and hands the rest to whichever page is up.
//!
//! It moved out twice. `state.rs` reached 757 lines against CLAUDE.md's ~300, so
//! the struct went one way and what a word *means* the other; then this reached
//! 884, so the pages moved out again — [`playing`](super::playing) for `play`
//! and the lengths, [`setting`](super::setting) for the settings pages. The seam
//! is page family, which is how a player meets them.
//!
//! Every page answers `back` the same way, and [`is_back`] in
//! [`Menu::answered`] is the one expression of it. Three pages had the same four
//! lines pasted at the top of their handler, which is the shape §19 records
//! drifting. The `back` table lives here too, because a page's parent is the
//! routing's business — and `Lengths` moving under `Play` was missed precisely
//! because the table had a catch-all arm.

use super::state::{Complaint, Menu, Outcome, Page};
use super::words::{ABANDON, BACK, Word, word};

/// Whether `typed` is `back`, by prefix.
///
/// Either direction: `bac` is a prefix of `back`, and a player who typed the
/// whole word plus something is still asking for it. The vocabulary's rule,
/// applied to the one word that is not in it.
fn is_back(typed: &str) -> bool {
    typed.starts_with(BACK) || BACK.starts_with(typed)
}

/// The slot `abandon` names, if that is what was typed.
///
/// Not prefix-matched, unlike every other word here, and that is the point: it
/// is the one destructive word in the menu, and `a` reaching it would put the
/// question on a mistyped keystroke. The player who means it can spare the
/// letters.
pub(super) fn abandoning(typed: &str) -> Option<&str> {
    typed.strip_prefix(ABANDON)
}

impl Menu {
    /// Run `typed` against whichever page is showing.
    pub(super) fn answered(&mut self, typed: &str) -> Option<Outcome> {
        // Taken, not read: `abandon` asks once and any other line answers *no*,
        // so the question must be gone by the time anything else looks and only
        // the very next line may see it. `execute::quit` does the same with
        // `Quitting::never_mind`.
        let asked = self.asking.take();
        // `back` first and once, so no inner page has to remember it. One level,
        // which is why the settings pages name their parent rather than the top.
        if !matches!(self.page, Page::Choices) && is_back(typed) {
            // Every inner page names its parent, and `Lengths` was the one that
            // did not: `_ => Page::Choices` caught it, so `play` → `new` →
            // `back` stepped two levels and threw the player past the listing
            // they came through. `menu_word_back` says *"the way you came"*.
            self.page = match self.page {
                Page::Setting(_) => Page::Settings,
                Page::Lengths => Page::Play,
                _ => Page::Choices,
            };
            return None;
        }
        match self.page {
            Page::Choices => self.chose(typed),
            Page::Play => self.opened(typed, asked),
            Page::Lengths => self.measured(typed),
            Page::Settings => self.turned(typed),
            Page::Setting(page) => self.set(page, typed),
        }
    }

    /// A word on the top page.
    fn chose(&mut self, typed: &str) -> Option<Outcome> {
        match word(typed, self.stance) {
            Some(Word::Resume) => Some(Outcome::Close),
            Some(Word::Quit) => Some(Outcome::PutDown),
            Some(Word::Play) => {
                self.open_play();
                None
            }
            Some(Word::Manual) => Some(Outcome::OpenManual),
            Some(Word::Settings) => {
                // No `Unkept` here, unlike the page above: a session that keeps
                // no save can still be told how to read a line, and the choice
                // lasting only as long as the session beats refusing it. No file
                // read either — what is in effect is `Menu::driver`, set by the
                // frontend when the menu opened.
                self.page = Page::Settings;
                None
            }
            None => {
                self.complaint = Some(Complaint::Unknown(typed.to_owned()));
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A menu showing `page`, over a tower, with nothing typed.
    fn on(page: Page) -> Menu {
        Menu {
            page,
            ..Menu::default()
        }
    }

    #[test]
    fn a_word_runs_by_its_shortest_unambiguous_prefix() {
        // Only the two that answer without touching the filesystem: `play` reads
        // the save directory, and what it does there is the play page's tests'.
        for (name, want) in [("resume", Outcome::Close), ("quit", Outcome::PutDown)] {
            let mut menu = Menu::default();
            menu.type_text(&name[..1]);
            assert_eq!(
                menu.enter(),
                Some(want),
                "`{}` did not reach {name}",
                &name[..1],
            );
        }
    }

    #[test]
    fn the_menu_quits_the_orb_and_resume_goes_back() {
        // The two halves of the decision this screen exists to split: `quit` at
        // the prompt used to end the session outright with no way to change your
        // mind. Now that word opens this, and ending it is a choice made here.
        let mut menu = Menu::default();
        menu.type_text("resume");
        assert_eq!(menu.enter(), Some(Outcome::Close));

        let mut menu = Menu::default();
        menu.type_text("quit");
        assert_eq!(menu.enter(), Some(Outcome::PutDown));
    }

    #[test]
    fn back_steps_a_page_by_its_prefix_and_a_slot_number_does_not() {
        // One expression of it, in `answered`, rather than four lines per
        // handler. Swept over every page so a page added without the rule fails
        // here rather than in a player's hands.
        //
        // To its parent, not the top: `Lengths` sits under `Play`, and this
        // asserted `Choices` everywhere, so it could not see `new` moving down a
        // level and `back` from the lengths overshooting the tower listing.
        for (page, parent) in [
            (Page::Play, Page::Choices),
            (Page::Lengths, Page::Play),
            (Page::Settings, Page::Choices),
        ] {
            for typed in ["b", "ba", "back"] {
                let mut menu = on(page);
                menu.type_text(typed);
                assert_eq!(menu.enter(), None);
                assert_eq!(menu.page(), parent, "`{typed}` did not step back");
            }
        }
    }

    #[test]
    fn a_menu_that_has_not_looked_does_not_claim_the_orb_keeps_nothing() {
        // The field that forced `Default` to be written out: a derive gave
        // `keeps` the zero value, which asserts *this session keeps no towers*
        // about a session nothing has asked. The play page finds out; until
        // then the listing must not draw the wrong one of two sentences.
        for stance in [
            super::super::Stance::InTower,
            super::super::Stance::Threshold,
        ] {
            assert!(
                Menu::at(stance).keeps(),
                "a fresh {stance:?} menu claims the orb keeps nothing",
            );
        }
    }
}
