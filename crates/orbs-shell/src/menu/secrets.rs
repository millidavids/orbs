//! Words the menu answers and never offers.
//!
//! §3's sysadmin comedy: *discovered, never pushed*, and *never puncturing a
//! tense moment*. None of them is listed, completed or matched by prefix, and
//! none changes anything — each is a line said back. All are silent while the
//! tower behind the menu speaks in any register but plain.

use super::state::{Complaint, Menu};

/// The words, whole, and the prose key each is answered with.
const SAID: [(&str, &str); 6] = [
    // Colossal Cave's magic word, which does exactly what it did there.
    ("xyzzy", "menu_secret_xyzzy"),
    ("plugh", "menu_secret_plugh"),
    ("hello sailor", "menu_secret_hello_sailor"),
    ("sudo", "menu_secret_sudo"),
    ("make me a sandwich", "menu_secret_sandwich"),
    ("sudo make me a sandwich", "menu_secret_sudo_sandwich"),
];

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
        let Some((_, key)) = SAID.iter().find(|(word, _)| *word == typed) else {
            return false;
        };
        self.complaint = Some(Complaint::Remark((*key).to_owned()));
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu::state::Outcome;

    #[test]
    fn a_secret_word_is_answered_and_changes_nothing() {
        for (word, key) in SAID {
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
        for register in [Presentation::Eldritch, Presentation::Tampered] {
            for word in SAID.map(|(word, _)| word).into_iter().chain(["moon"]) {
                let mut menu = Menu::default();
                menu.show_register(register);
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
        menu.show_register(Presentation::Eldritch);
        menu.show_register(Presentation::Plain);
        menu.type_text("xyzzy");
        menu.enter();
        assert!(matches!(menu.complaint(), Some(Complaint::Remark(_))));
    }

    #[test]
    fn a_secret_is_never_reached_by_a_prefix() {
        let mut menu = Menu::default();
        menu.type_text("xyz");
        menu.enter();
        assert!(matches!(menu.complaint(), Some(Complaint::Unknown(_))));
    }
}
