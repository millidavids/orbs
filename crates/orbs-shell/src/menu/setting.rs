//! The settings pages: choosing a category, and changing a row on one.
//!
//! Split out of `pages.rs` with [`playing`](super::playing), by page family —
//! see that module's header for the seam.
//!
//! Nothing here knows what a phosphor is. A [`Row`](crate::settings::Row) is
//! data the frontend assembled from what it has, and this changes the one the
//! player named; `settings/values.rs` carries that argument.

use super::state::{Complaint, Driver, Menu, Outcome, Page};
use crate::settings::Category;

impl Menu {
    /// A category on the settings page.
    pub(super) fn turned(&mut self, typed: &str) -> Option<Outcome> {
        // Only the pages that have something on them, which is how
        // *unsupported* is said: `orbs-tui` has no tube, so it builds no tube
        // rows, so `tube` is not a word there. See `Menu::pages`.
        let Some(page) = Category::named(typed).filter(|page| self.pages().any(|had| had == *page))
        else {
            self.complaint = Some(Complaint::Unknown(typed.to_owned()));
            return None;
        };
        self.page = Page::Setting(page);
        None
    }

    /// A setting on one of the settings pages.
    ///
    /// `crt` cycles it, as `F3` and `F8` do; `crt off` goes straight there
    /// without pressing through the others, which is how a See-it line names a
    /// screen.
    ///
    /// `said` is the line as typed, case and all, for a row that takes free
    /// text: a name is the player's, not a command, and `Morgana` is not
    /// `morgana`.
    pub(super) fn set(&mut self, page: Category, typed: &str, said: &str) -> Option<Outcome> {
        let (word, rest) = typed.split_once(' ').unwrap_or((typed, ""));
        // A whole word wins, then a prefix only one row has — `Row::value_named`'s
        // rule. Taking the first match made a page's order decide which of two
        // rows a letter meant.
        let chosen =
            crate::prefix::unique_prefix(self.rows(page).map(|row| (row.word.as_str(), row)), word)
                .cloned();
        let Some(row) = chosen else {
            self.complaint = Some(Complaint::Unknown(typed.to_owned()));
            return None;
        };
        // A row with no list takes what it is given — the name. The word alone
        // steps it back to its default, as a word alone steps every row.
        if row.values.is_empty() {
            let given = said.split_once(' ').map_or("", |(_, rest)| rest).trim();
            let value = if given.is_empty() {
                String::new()
            } else {
                match crate::settings::checked_name(given) {
                    Ok(name) => name,
                    Err(fault) => {
                        self.complaint = Some(Complaint::Unnamed(fault));
                        return None;
                    }
                }
            };
            return Some(self.settled(page, &row, value));
        }
        // A secret, whole, before any prefix: it is never offered, so a prefix
        // could only ever be a coincidence. Silent in a tense register, as the
        // menu's words are.
        if let Some(secret) = row.secret(rest.trim()).filter(|_| !self.tense).cloned() {
            let outcome = self.settled(page, &row, secret.value);
            self.complaint = secret.remark.map(Complaint::Remark);
            return Some(outcome);
        }
        let chosen = if rest.trim().is_empty() {
            row.next().map(ToOwned::to_owned)
        } else {
            row.value_named(rest.trim()).map(ToOwned::to_owned)
        };
        let Some(value) = chosen else {
            // §6: the page names what it does not know rather than picking one,
            // including an ambiguous prefix — `Row::value_named` took the first
            // match, so `theme m` meant muted violet and never monochrome.
            self.complaint = Some(Complaint::Unknown(typed.to_owned()));
            return None;
        };
        Some(self.settled(page, &row, value))
    }

    /// Write `value` down for `row`, show it, and say what the frontend should
    /// honour.
    fn settled(&mut self, page: Category, row: &crate::settings::Row, value: String) -> Outcome {
        // Best-effort: a session with nowhere to keep a setting still gets to
        // change it for as long as it lasts, rather than hitting §15's dead end
        // over a preference.
        crate::settings::set(&row.key, &value);
        // The row the player is looking at says what is true, now, without
        // waiting for the frontend to hand the menu a fresh set.
        if let Some(showing) = self
            .rows
            .iter_mut()
            .find(|showing| showing.page == page && showing.word == row.word)
        {
            showing.value.clone_from(&value);
        }

        // A setting like the others, and an outcome of its own: *stop
        // consulting a reader now* is not something a generic `Set` can say
        // without every frontend growing a string comparison.
        if row.key == crate::settings::DRIVER
            && let Some(driver) = Driver::named(&value)
        {
            self.driver = driver;
            return Outcome::Drive(driver);
        }
        Outcome::Set {
            setting: row.key.clone(),
            value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Row;

    /// A menu on the habits page, with the driver row on it.
    fn reading() -> Menu {
        Menu {
            page: Page::Setting(Category::Habits),
            rows: vec![
                Row::new(
                    Category::Habits,
                    "reading",
                    Driver::Augury.word(),
                    &Driver::ALL.map(Driver::word),
                )
                .keyed(crate::settings::DRIVER),
            ],
            ..Menu::default()
        }
    }

    #[test]
    fn the_driver_is_a_setting_and_still_answers_with_its_own_outcome() {
        // A row like the others, and an outcome unlike them: *stop consulting a
        // reader now* needs its own variant, and both builds act on it.
        let mut menu = reading();
        menu.type_text("reading plain");
        assert_eq!(menu.enter(), Some(Outcome::Drive(Driver::Plain)));
        assert_eq!(
            menu.driver(),
            Driver::Plain,
            "the page did not show the new choice",
        );

        // ...and the word alone steps it, like every other setting.
        let mut menu = reading();
        menu.type_text("reading");
        assert_eq!(menu.enter(), Some(Outcome::Drive(Driver::Plain)));
    }

    #[test]
    fn the_driver_is_kept_under_the_name_the_old_file_used() {
        // `reading` to a player, `driver` in the file, so a settings file
        // written by `0.13.17` still reads. `Row::key` exists for this and this
        // is the one row that uses it.
        let menu = reading();
        let row = &menu.rows[0];
        assert_eq!(row.word, "reading", "the word a player types changed");
        assert_eq!(
            row.key,
            crate::settings::DRIVER,
            "the key in the settings file changed, which orphans every file",
        );
    }

    /// A menu with two settings on two pages, and nothing typed.
    fn settable() -> Menu {
        Menu {
            page: Page::Settings,
            rows: vec![
                Row::new(
                    Category::Tube,
                    "crt",
                    "default",
                    &["default", "peak", "off"],
                ),
                Row::new(Category::Habits, "focus", "wide", &["deep", "wide"]),
            ],
            ..Menu::default()
        }
    }

    #[test]
    fn a_page_with_nothing_on_it_is_not_offered() {
        // How *unsupported* is said — no mask, no dimmed row. `orbs-tui` has no
        // tube, so it builds no tube rows, so `tube` is not a word there; a
        // control that does nothing is §15's dead affordance.
        let mut menu = settable();
        assert_eq!(
            menu.pages().collect::<Vec<_>>(),
            [Category::Tube, Category::Habits],
            "a page with no rows was offered",
        );

        menu.type_text("access");
        assert_eq!(menu.enter(), None);
        assert_eq!(
            menu.complaint(),
            Some(&Complaint::Unknown("access".into())),
            "a page with nothing on it answered to its word",
        );
        assert_eq!(menu.page(), Page::Settings, "it opened an empty page");

        // ...and the two that do have rows open.
        for (typed, want) in [("t", Category::Tube), ("h", Category::Habits)] {
            let mut menu = settable();
            menu.type_text(typed);
            assert_eq!(menu.enter(), None);
            assert_eq!(menu.page(), Page::Setting(want), "`{typed}`");
        }
    }

    #[test]
    fn a_word_steps_a_setting_and_a_word_with_a_value_names_one() {
        // `crt` is what `F3` does; `crt off` is what a See-it line wants. Both
        // reach the same place, so neither needs a page of its own.
        let mut menu = Menu {
            page: Page::Setting(Category::Tube),
            ..settable()
        };
        menu.type_text("crt");
        assert_eq!(
            menu.enter(),
            Some(Outcome::Set {
                setting: "crt".into(),
                value: "peak".into(),
            }),
            "the word did not step the setting",
        );

        let mut menu = Menu {
            page: Page::Setting(Category::Tube),
            ..settable()
        };
        menu.type_text("crt off");
        assert_eq!(
            menu.enter(),
            Some(Outcome::Set {
                setting: "crt".into(),
                value: "off".into(),
            }),
        );
    }

    /// A habits page with the name on it, the free-text row.
    fn naming() -> Menu {
        Menu {
            page: Page::Setting(Category::Habits),
            rows: vec![Row::new(Category::Habits, "name", "wizard", &[])],
            ..Menu::default()
        }
    }

    #[test]
    fn a_name_keeps_its_case_and_the_word_alone_puts_it_back() {
        let mut menu = naming();
        menu.type_text("name Morgana");
        assert_eq!(
            menu.enter(),
            Some(Outcome::Set {
                setting: "name".into(),
                value: "Morgana".into(),
            }),
            "the menu lowered a name, or refused it",
        );

        menu.type_text("name");
        assert_eq!(
            menu.enter(),
            Some(Outcome::Set {
                setting: "name".into(),
                value: String::new(),
            }),
            "the word alone did not step the name back to its default",
        );
    }

    #[test]
    fn a_name_the_prompt_cannot_carry_is_refused_and_said_why() {
        for (typed, fault) in [
            ("name Morgan le Fay", crate::settings::NameFault::Spaced),
            ("name Morgana✨", crate::settings::NameFault::Undrawable(8)),
        ] {
            let mut menu = naming();
            menu.type_text(typed);
            assert_eq!(menu.enter(), None, "`{typed}` was taken");
            assert_eq!(menu.complaint(), Some(&Complaint::Unnamed(fault)));
        }
    }

    #[test]
    fn a_rows_secret_is_silent_in_the_eldritch_register() {
        let menu = || Menu {
            page: Page::Setting(Category::Sound),
            rows: vec![
                Row::new(Category::Sound, "voice", "full", &["off", "full"]).secretly(
                    "eleven",
                    "full",
                    Some("menu_secret_eleven"),
                ),
            ],
            ..Menu::default()
        };
        let mut calm = menu();
        calm.type_text("voice eleven");
        assert!(calm.enter().is_some(), "the secret was not answered");

        // §3: comedy never punctures a tense moment.
        let mut eldritch = menu();
        eldritch.show_register(orbs_render::Presentation::Eldritch);
        eldritch.type_text("voice eleven");
        assert_eq!(eldritch.enter(), None, "the secret set something");
        assert!(matches!(eldritch.complaint(), Some(Complaint::Unknown(_))));
    }

    #[test]
    fn a_letter_two_rows_share_names_neither() {
        // `g` was `glow` because `glow` came first on the page, and `grille`
        // could not be reached by its letter at all.
        let mut menu = Menu {
            page: Page::Setting(Category::Picture),
            rows: vec![
                Row::new(Category::Picture, "glow", "full", &["off", "full"]),
                Row::new(Category::Picture, "grille", "full", &["off", "full"]),
            ],
            ..Menu::default()
        };
        menu.type_text("g off");
        assert_eq!(menu.enter(), None, "an ambiguous letter set something");
        assert!(matches!(menu.complaint(), Some(Complaint::Unknown(_))));

        menu.type_text("gr off");
        assert_eq!(
            menu.enter(),
            Some(Outcome::Set {
                setting: "grille".into(),
                value: "off".into(),
            }),
        );
    }

    #[test]
    fn the_row_a_player_is_looking_at_says_what_is_true_now() {
        // Without waiting for the frontend to hand over a fresh set. A page
        // still saying `default` after you turned the tube off is §15's dead
        // end in miniature.
        let mut menu = Menu {
            page: Page::Setting(Category::Tube),
            ..settable()
        };
        menu.type_text("crt off");
        menu.enter();
        assert_eq!(
            menu.rows(Category::Tube)
                .next()
                .map(|row| row.value.as_str()),
            Some("off"),
            "the row did not catch up with the choice",
        );
    }

    #[test]
    fn a_setting_or_a_value_the_page_does_not_have_is_said() {
        // §6 twice over: the page names what it does not know rather than
        // picking something.
        for typed in ["zorb", "crt zorb"] {
            let mut menu = Menu {
                page: Page::Setting(Category::Tube),
                ..settable()
            };
            menu.type_text(typed);
            assert_eq!(menu.enter(), None, "`{typed}` set something");
            assert_eq!(
                menu.complaint(),
                Some(&Complaint::Unknown(typed.into())),
                "`{typed}` was neither answered nor refused",
            );
        }
    }

    #[test]
    fn back_from_a_setting_steps_to_the_settings_and_not_to_the_top() {
        // The one page in the menu that is two deep. `back` means *one level*
        // everywhere else here and must mean it here too.
        for typed in ["b", "back"] {
            let mut menu = Menu {
                page: Page::Setting(Category::Tube),
                ..settable()
            };
            menu.type_text(typed);
            assert_eq!(menu.enter(), None);
            assert_eq!(menu.page(), Page::Settings, "`{typed}` went too far");
        }
        // ...and Escape agrees with it.
        let mut menu = Menu {
            page: Page::Setting(Category::Tube),
            ..settable()
        };
        assert_eq!(menu.escape(), None);
        assert_eq!(menu.page(), Page::Settings);
    }

    #[test]
    fn an_unknown_driver_is_said_rather_than_guessed() {
        // §6 again: the page names what it does not know instead of picking one.
        let mut menu = reading();
        menu.type_text("reading magic");
        assert_eq!(menu.enter(), None);
        assert_eq!(
            menu.complaint(),
            Some(&Complaint::Unknown("reading magic".into())),
            "an unknown driver silently chose one",
        );
        assert_eq!(
            menu.page(),
            Page::Setting(Category::Habits),
            "it left the page anyway",
        );
    }
}
