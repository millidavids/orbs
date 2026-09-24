//! The play page and the lengths page: opening a tower, raising one, and
//! setting one aside.
//!
//! Split out of `pages.rs` by page family — the settings pages are in
//! [`setting`](super::setting), the dispatch in [`pages`](super::pages). These
//! three are the only ones that reach the filesystem.

use orbs_sim::content::Length;

use super::pages::abandoning;
use super::state::{Complaint, Menu, Outcome, Page};
use super::words::{ABANDON, NEW};

impl Menu {
    /// Read the listing and show the play page.
    ///
    /// Read here, not in `paint`: a painter would do six file reads sixty times
    /// a second. It opens whether or not there is anything to list — a session
    /// that keeps nothing can still raise a tower.
    pub(super) fn open_play(&mut self) {
        // Said, not implied: with nowhere to keep anything the listing drew
        // *"no towers yet"*, which reads as a first launch and invites a `new`
        // that will not be remembered.
        self.keeps = crate::save::path().is_some();
        self.saves = if self.keeps {
            crate::save::saves()
        } else {
            self.complaint = Some(Complaint::Unkept);
            Vec::new()
        };
        self.page = Page::Play;
    }

    /// A slot number, `new`, or `abandon <slot>`, on the play page.
    ///
    /// `asked` is the slot a previous `abandon` put a question about, if the
    /// line before this one did. See [`Menu::asking`](super::state::Menu).
    pub(super) fn opened(&mut self, typed: &str, asked: Option<usize>) -> Option<Outcome> {
        if NEW.starts_with(typed) {
            self.open_lengths();
            return None;
        }
        if let Some(rest) = abandoning(typed) {
            self.abandon(rest, asked);
            return None;
        }
        let Ok(slot) = typed.parse::<usize>() else {
            self.complaint = Some(Complaint::Unknown(typed.to_owned()));
            return None;
        };
        match self.saves.iter().find(|save| save.slot == slot) {
            // Said, not opened and not called empty: loading it reaches
            // `read_from`, which sets it aside — from a keystroke meaning
            // *open this*.
            Some(save) if !save.is_readable() => {
                self.complaint = Some(Complaint::Unreadable(slot));
                None
            }
            Some(save) => Some(Outcome::Load(save.path.clone())),
            None => {
                self.complaint = Some(Complaint::Empty(slot));
                None
            }
        }
    }

    /// `abandon <slot>` — ask once, then do it.
    ///
    /// The confirmation is a word, not a screen (§19), and lasts exactly one
    /// line. The only destructive word in the menu, so the file is renamed
    /// rather than unlinked ([`save::abandon`](crate::save::abandon)) and a
    /// mistyped slot is recoverable.
    fn abandon(&mut self, rest: &str, asked: Option<usize>) {
        let Ok(slot) = rest.trim().parse::<usize>() else {
            self.complaint = Some(Complaint::Unknown(
                format!("{ABANDON} {rest}").trim().to_owned(),
            ));
            return;
        };
        let Some(save) = self.saves.iter().find(|save| save.slot == slot) else {
            self.complaint = Some(Complaint::Empty(slot));
            return;
        };
        if asked != Some(slot) {
            // Asked about this slot and no other, so `abandon 2` then `abandon
            // 3` is two questions rather than an answer.
            self.asking = Some(slot);
            self.complaint = Some(Complaint::Abandoning(slot));
            return;
        }
        let path = save.path.clone();
        if crate::save::abandon(&path).is_err() {
            self.complaint = Some(Complaint::Unabandoned(slot));
            return;
        }
        // Re-read rather than edited in place: a `Vec` with a slot removed is a
        // second answer to what the directory says.
        self.saves = crate::save::saves();
    }

    /// Offer the lengths, if there is anywhere to put a new tower.
    ///
    /// A session that keeps nothing can still begin one: this refused with
    /// [`Complaint::Unkept`] when there was no save path, but the tower is
    /// playable, it simply will not be remembered. `ORBS_SAVE=off` is what
    /// `scripts/dumps.sh` and the played-game suite run under, so refusing
    /// `new` made the build unreachable from its own instruments.
    fn open_lengths(&mut self) {
        // Nowhere to keep it and no room for it are different answers, and only
        // the second is a refusal — `free_slot` says `None` to both.
        self.keep = match crate::save::path() {
            None => {
                // Said on the page where the choice is made: `Menu::enter`
                // clears the complaint every line, so the play page's warning
                // was gone by the time a length was picked (§6).
                self.complaint = Some(Complaint::Unkept);
                None
            }
            Some(base) => {
                let Some(slot) = crate::save::free_slot() else {
                    // Refused before the length is chosen: asking how long and
                    // then saying there is no room is §15's dead end.
                    self.complaint = Some(Complaint::Full);
                    return;
                };
                Some(crate::save::slot_path(&base, slot))
            }
        };
        self.page = Page::Lengths;
    }

    /// A length on the new-game page.
    pub(super) fn measured(&mut self, typed: &str) -> Option<Outcome> {
        // Prefix-matched like every other word here, so `s`, `m` and `l` work.
        // `Baseline` is not among the ones offered — see `Length::OFFERED`.
        let Some(length) = Length::OFFERED
            .into_iter()
            .find(|length| length.word().starts_with(typed))
        else {
            self.complaint = Some(Complaint::Unknown(typed.to_owned()));
            return None;
        };
        // Resolved when the page opened, not now: a destination worked out
        // twice is one a changed `ORBS_SAVE` can make disagree with itself.
        Some(Outcome::Begin {
            path: self.keep.clone(),
            length,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::save::Slot;

    /// A menu showing `page`, over a tower, with nothing typed.
    fn on(page: Page) -> Menu {
        Menu {
            page,
            ..Menu::default()
        }
    }

    /// A listed tower in `slot`, readable, at a path nothing will touch.
    fn tower(slot: usize) -> Slot {
        Slot {
            slot,
            path: PathBuf::from(format!("/somewhere/orbs-save-{slot}.toml")),
            held: Some(crate::save::Held {
                wizard: "david".into(),
                length: Length::Long,
                tick: 900,
                experience: 4_200,
                away: None,
            }),
        }
    }

    #[test]
    fn an_empty_slot_is_said_rather_than_opened() {
        // §6: a number with no tower behind it names what to do instead.
        let mut menu = on(Page::Play);
        menu.type_text("4");
        assert_eq!(menu.enter(), None);
        assert_eq!(menu.complaint(), Some(&Complaint::Empty(4)));
    }

    #[test]
    fn a_listed_tower_opens_the_file_it_is_actually_in() {
        // A path, not a slot: the shell writes the game it is leaving before it
        // loads this one, and must write to the file that game came from.
        let mut menu = Menu {
            page: Page::Play,
            saves: vec![tower(2)],
            ..Menu::default()
        };
        menu.type_text("2");
        assert_eq!(
            menu.enter(),
            Some(Outcome::Load(PathBuf::from("/somewhere/orbs-save-2.toml"))),
        );
    }

    #[test]
    fn a_slot_this_build_cannot_read_is_said_and_never_called_empty() {
        // Two false things before this: the slot looked free, and its number
        // answered *"there is no tower in 3"* about a file with somebody's game
        // in it. Opening it is worse still — `read_from` sets an unreadable
        // save aside, so a keystroke meaning *open this* moves the file.
        let mut menu = Menu {
            page: Page::Play,
            saves: vec![Slot {
                slot: 3,
                path: PathBuf::from("/somewhere/orbs-save-3.toml"),
                held: None,
            }],
            ..Menu::default()
        };
        menu.type_text("3");
        assert_eq!(menu.enter(), None, "an unreadable tower was opened");
        assert_eq!(menu.complaint(), Some(&Complaint::Unreadable(3)));
    }

    #[test]
    fn abandon_asks_once_and_any_other_line_answers_no() {
        // `quit`'s shape: the question lasts one line, a second `abandon 2`
        // answers it, and anything else — another slot included — is no.
        let mut menu = Menu {
            page: Page::Play,
            saves: vec![tower(2)],
            ..Menu::default()
        };
        menu.type_text("abandon 2");
        assert_eq!(menu.enter(), None);
        assert_eq!(
            menu.complaint(),
            Some(&Complaint::Abandoning(2)),
            "abandon did not ask",
        );

        // Any other line answers no, and the question is gone.
        menu.type_text("new");
        menu.enter();
        assert_eq!(menu.asking, None, "the question outlived the next line");

        // And asking about a different slot is a second question, not an answer.
        let mut menu = Menu {
            page: Page::Play,
            saves: vec![tower(2), tower(3)],
            ..Menu::default()
        };
        menu.type_text("abandon 2");
        menu.enter();
        menu.type_text("abandon 3");
        menu.enter();
        assert_eq!(
            menu.complaint(),
            Some(&Complaint::Abandoning(3)),
            "a second slot answered the first slot's question",
        );
    }

    #[test]
    fn abandon_is_typed_whole_and_never_by_prefix() {
        // The one word here that is not prefix-matched: a mistyped page costs a
        // step back, a mistyped `abandon` sets a tower aside.
        let mut menu = Menu {
            page: Page::Play,
            saves: vec![tower(2)],
            ..Menu::default()
        };
        for typed in ["a 2", "ab 2", "aband 2"] {
            menu.type_text(typed);
            assert_eq!(menu.enter(), None);
            assert_eq!(
                menu.asking, None,
                "`{typed}` put the question about a tower",
            );
            assert!(
                matches!(menu.complaint(), Some(Complaint::Unknown(_))),
                "`{typed}` was neither answered nor refused",
            );
        }
    }

    #[test]
    fn abandoning_a_slot_with_nothing_in_it_says_so() {
        let mut menu = Menu {
            page: Page::Play,
            saves: vec![tower(2)],
            ..Menu::default()
        };
        menu.type_text("abandon 5");
        assert_eq!(menu.enter(), None);
        assert_eq!(menu.complaint(), Some(&Complaint::Empty(5)));
        assert_eq!(menu.asking, None, "it asked about an empty slot");
    }

    #[test]
    fn new_and_a_slot_number_live_on_the_same_page() {
        // Why the listing and the new game are one page: `n` asks how long, `2`
        // opens the second tower, and neither needs a choice a level earlier.
        let mut menu = on(Page::Play);
        menu.type_text("n");
        assert_eq!(menu.enter(), None);
        assert_eq!(menu.page(), Page::Lengths, "`n` did not reach the lengths");

        // And `new` is not read as a slot, nor `back` as either.
        let mut menu = on(Page::Play);
        menu.type_text("new");
        assert_eq!(menu.enter(), None);
        assert_eq!(menu.page(), Page::Lengths);
    }

    #[test]
    fn a_length_is_chosen_by_prefix_and_baseline_is_not_offered() {
        for (typed, want) in [
            ("s", Length::Short),
            ("m", Length::Medium),
            ("long", Length::Long),
        ] {
            let mut menu = Menu {
                page: Page::Lengths,
                keep: Some(PathBuf::from("/somewhere/orbs-save-3.toml")),
                ..Menu::default()
            };
            menu.type_text(typed);
            let outcome = menu.enter();
            let Some(Outcome::Begin { path, length }) = outcome else {
                panic!("`{typed}` did not begin a game: {outcome:?}");
            };
            assert_eq!(length, want);
            assert_eq!(
                path,
                Some(PathBuf::from("/somewhere/orbs-save-3.toml")),
                "the tower was not begun where the page said it would be kept",
            );
        }

        // `baseline` is the numbers authored first, not a difficulty —
        // `Length::OFFERED` leaves it out and so must this.
        let mut menu = Menu {
            page: Page::Lengths,
            keep: Some(PathBuf::from("/somewhere/orbs-save-3.toml")),
            ..Menu::default()
        };
        menu.type_text("baseline");
        assert_eq!(menu.enter(), None);
        assert_eq!(
            menu.complaint(),
            Some(&Complaint::Unknown("baseline".into())),
        );
    }

    #[test]
    fn a_session_that_keeps_nothing_still_begins_a_tower() {
        // `ORBS_SAVE=off` is not exotic: `scripts/dumps.sh` and the played-game
        // suite both run under it, so a `new` that refused would make the
        // threshold unreachable from the project's own instruments. `keep:
        // None` is what `open_lengths` leaves behind — a playable tower that is
        // not remembered.
        let mut menu = Menu {
            page: Page::Lengths,
            keep: None,
            ..Menu::default()
        };
        menu.type_text("short");
        assert_eq!(
            menu.enter(),
            Some(Outcome::Begin {
                path: None,
                length: Length::Short,
            }),
            "a session with nowhere to save refused to begin a game",
        );
    }
}
