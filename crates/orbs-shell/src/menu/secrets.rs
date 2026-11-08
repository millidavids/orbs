//! Words the menu answers and never offers.
//!
//! §3's sysadmin comedy: *discovered, never pushed*, and *never puncturing a
//! tense moment*. The jokes are the prompt's own (`orbs_sim::parser::aside`),
//! one table for both; the menu answers only those, and `moon`. None is listed,
//! completed or matched by prefix — `sudo` is answered alone, never `sudo
//! play` — none changes anything, and all are silent while the tower behind
//! the menu speaks in any register but plain, or is under siege.

use orbs_sim::parser::{AsideKind, aside_alone};

use super::state::{Complaint, Menu};

impl Menu {
    /// Answer `typed` if it is one of the words, and say whether it was.
    pub(super) fn whispered(&mut self, typed: &str) -> bool {
        if self.tense {
            return false;
        }
        if typed == "moon" {
            self.complaint = Some(Complaint::Moon(crate::sky::reading()));
            return true;
        }
        let Some(joke) = aside_alone(typed).filter(|found| found.kind == AsideKind::Joke) else {
            return false;
        };
        self.complaint = Some(Complaint::Remark(joke.key.to_owned()));
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu::state::Outcome;
    use orbs_sim::parser::every_aside;

    /// The prompt's jokes, which the menu answers too.
    fn jokes() -> impl Iterator<Item = (&'static str, &'static str)> {
        every_aside()
            .filter(|(_, found)| found.kind == AsideKind::Joke)
            .map(|(said, found)| (said, found.key))
    }

    #[test]
    fn a_secret_word_is_answered_and_changes_nothing() {
        for (word, key) in jokes() {
            let mut menu = Menu::default();
            menu.type_text(word);
            assert_eq!(menu.enter(), None::<Outcome>, "`{word}` did something");
            assert_eq!(menu.complaint(), Some(&Complaint::Remark(key.to_owned())));
        }
    }

    #[test]
    fn no_secret_is_answered_in_a_tense_register() {
        // §3: comedy never punctures a tense moment. A sabotage tell is one.
        use orbs_render::Presentation;
        for (register, besieged) in [
            (Presentation::Eldritch, false),
            (Presentation::Tampered, false),
            (Presentation::Plain, true),
        ] {
            for word in jokes().map(|(word, _)| word).chain(["moon"]) {
                let mut menu = Menu::default();
                menu.show_tension(register, besieged);
                menu.type_text(word);
                menu.enter();
                assert!(
                    matches!(menu.complaint(), Some(Complaint::Unknown(_))),
                    "`{word}` was answered in {register:?}",
                );
            }
        }
        // And back: the register is asked again, not remembered from opening.
        let mut menu = Menu::default();
        menu.show_tension(Presentation::Eldritch, true);
        menu.show_tension(Presentation::Plain, false);
        menu.type_text("xyzzy");
        menu.enter();
        assert!(matches!(menu.complaint(), Some(Complaint::Remark(_))));
    }

    #[test]
    fn a_secret_is_never_reached_by_a_prefix() {
        // Nor a word with more after it: the prompt's `sudo` takes the line,
        // and the menu has no line to take.
        for typed in ["xyz", "sudo play", "sudo quit"] {
            let mut menu = Menu::default();
            menu.type_text(typed);
            menu.enter();
            assert!(
                matches!(menu.complaint(), Some(Complaint::Unknown(_))),
                "`{typed}` was answered",
            );
        }
    }

    #[test]
    fn a_pointer_is_the_prompts_alone() {
        // `exit` at the menu is `quit`'s business, which the menu already names.
        let mut menu = Menu::default();
        menu.type_text("ps");
        menu.enter();
        assert!(matches!(menu.complaint(), Some(Complaint::Unknown(_))));
    }
}
