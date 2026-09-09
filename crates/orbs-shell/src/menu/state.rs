//! What a keystroke does to the menu.
//!
//! The pages, the words they answer to, and what the shell is asked to do about
//! it. Nothing here draws — see [`paint`](super::paint) — and nothing here ends
//! a process.

use std::path::PathBuf;

use orbs_sim::content::Length;

use super::words::{BACK, Word, word};
use crate::save::Slot;

/// The menu, while it is up.
///
/// **A page, a line and a complaint.** Its choices are authored, its state is
/// what is half-typed and which page is showing, and the screen it sits over is
/// the frontend's. `Tapestry` is the precedent and this is smaller: there is no
/// cursor, because there is nothing here to walk — every choice is a word, which
/// is what the rest of the game is.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Menu {
    /// Which page is showing.
    pub(super) page: Page,
    /// What is being typed at it.
    pub(super) command: String,
    /// What it would not do, and why.
    pub(super) complaint: Option<Complaint>,
    /// The towers the orb is keeping, as of the last time [`Word::Saves`] was
    /// asked for. **Read once when the page opens** rather than every frame: a
    /// listing is six file reads, and the directory does not change while a
    /// player is looking at it.
    pub(super) saves: Vec<Slot>,
    /// The lowest slot with no tower in it, or `None` when the orb is full.
    pub(super) free: Option<usize>,
    /// Which driver the options page last read, so it can show what is chosen.
    ///
    /// Read when the page opens, for `saves`' reason: it is a file read, and it
    /// does not change while a player is looking at it.
    pub(super) driver: Driver,
}

/// Which of the menu's four pages is showing.
///
/// **Pages rather than four surfaces**, because they share the line, the
/// complaint and the way out: `back` and Escape step one level, and from the top
/// Escape is `resume`. Four `Focus` variants would be four of everything for one
/// screen a player never sees more than one page of.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Page {
    /// The words.
    #[default]
    Choices,
    /// The towers the orb is keeping, one per row, numbered.
    Saves,
    /// How long a new game should be.
    Lengths,
    /// What the orb does with a line you type at it.
    Options,
}

/// How the orb reads what you type.
///
/// **The player's choice between the two drivers**, which until now was an
/// environment variable and therefore nobody's. §6's mastery arc rests on the
/// player seeing the canonical form echoed back; this decides whether the orb
/// works one out from a sentence, or answers only the words it already knows.
///
/// # The same in both builds
///
/// [`Augury`](Self::Augury) means the same thing in the terminal as in the
/// window: **the terminal frontend is the whole game, not a cut-down one.** It
/// had no reader while the reader needed `wgpu`; inference is `ndarray` and
/// costs 436µs, so the GPU went behind `orbs-augury`'s `train` feature and this
/// setting stopped being a Bevy-only promise.
///
/// It is still a *preference* rather than a claim about the build — a checkout
/// with no trained weights has nothing to consult and falls through to the
/// matcher, which is what a fresh clone does and is exactly right.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Driver {
    /// The orb works out what you meant, then echoes the command it settled on.
    #[default]
    Augury,
    /// The orb answers only the words it knows, and suggests when it cannot.
    Plain,
}

impl Driver {
    /// Both of them, in the order the page offers them.
    pub const ALL: [Self; 2] = [Self::Augury, Self::Plain];

    /// The word a player types for it.
    ///
    /// No two share a first letter, and neither collides with `back` — held by
    /// `the_inner_pages_have_no_collisions_either`.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Augury => "augury",
            Self::Plain => "plain",
        }
    }

    /// The driver a word names, whole.
    #[must_use]
    pub fn named(word: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|driver| driver.word() == word)
    }
}

/// What the menu would like the shell to do.
///
/// **Nothing here ends a process.** The shell decides what putting the orb down
/// *means* — an `AppExit` in the Bevy build, raw mode going back in the
/// terminal — which is the same division `execute::quit` draws and the reason
/// the two answers have nothing in common.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Take the menu down; the tower is as it was.
    Close,
    /// Leave the orb.
    PutDown,
    /// Put the tower in this file in front of the player.
    ///
    /// **A path, not a slot number.** The shell has to write the game it is
    /// leaving before it loads this one, and it must write it to the file it
    /// came from — a number would have to be resolved twice, and the two
    /// resolutions are exactly what a stale `SLOTS` or a changed `ORBS_SAVE`
    /// would make disagree.
    Load(PathBuf),
    /// Begin a tower in this file, at this length.
    Begin {
        /// Where it will be kept.
        path: PathBuf,
        /// How long it is to be.
        length: Length,
    },
    /// Read lines this way from now on.
    ///
    /// **Already written down by the time this is returned.** The menu persists
    /// the choice itself, the way `saves` reads the directory itself; this is
    /// what tells the running frontend to stop consulting a reader *now* rather
    /// than at the next launch.
    Drive(Driver),
}

/// Something the menu would not do.
///
/// §6 forbids a bare error, so every one of these has a sentence in `prose.toml`
/// naming what to do instead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Complaint {
    /// A word the menu does not know.
    Unknown(String),
    /// A slot with no tower in it.
    Empty(usize),
    /// The orb is keeping as many towers as it can.
    Full,
    /// This session keeps no save at all — `ORBS_SAVE=off`.
    ///
    /// **Said rather than hidden.** A dump and the played-game suite both run
    /// this way on purpose, and a `saves` that drew an empty list would say *you
    /// have no towers* to a player who has several.
    Unkept,
}

impl Menu {
    /// Which page is showing.
    #[must_use]
    pub const fn page(&self) -> Page {
        self.page
    }

    /// The towers the orb is keeping, as the saves page last read them.
    #[must_use]
    pub fn saves(&self) -> &[Slot] {
        &self.saves
    }

    /// Which driver is in effect, as the frontend last said.
    #[must_use]
    pub const fn driver(&self) -> Driver {
        self.driver
    }

    /// Tell the menu what is currently in effect.
    ///
    /// **The frontend is the source of truth, not the settings file.** A session
    /// with nowhere to write one still has a driver, and a page that read the
    /// file would show that session the choice it did not make. Called when the
    /// menu opens, which is the only moment the answer can have changed without
    /// this page being the one that changed it.
    pub const fn show_driver(&mut self, driver: Driver) {
        self.driver = driver;
    }

    /// What is typed at it.
    #[must_use]
    pub fn command(&self) -> &str {
        &self.command
    }

    /// What it last refused.
    #[must_use]
    pub const fn complaint(&self) -> Option<&Complaint> {
        self.complaint.as_ref()
    }

    /// Add typed text to the line.
    pub fn type_text(&mut self, text: &str) {
        for ch in text.chars().filter(|ch| !ch.is_control()) {
            self.command.push(ch);
        }
    }

    /// Rub out the last character.
    pub fn backspace(&mut self) {
        self.command.pop();
    }

    /// Run whatever is on the line.
    ///
    /// An empty line does nothing rather than complaining: pressing Enter at a
    /// bare prompt is not a mistake anywhere else in the game and is not one
    /// here.
    pub fn enter(&mut self) -> Option<Outcome> {
        let typed = std::mem::take(&mut self.command);
        let typed = typed.trim().to_lowercase();
        self.complaint = None;
        if typed.is_empty() {
            return None;
        }
        match self.page {
            Page::Choices => self.chose(&typed),
            Page::Saves => self.picked(&typed),
            Page::Lengths => self.measured(&typed),
            Page::Options => self.drove(&typed),
        }
    }

    /// A word on the top page.
    fn chose(&mut self, typed: &str) -> Option<Outcome> {
        match word(typed) {
            Some(Word::Resume) => Some(Outcome::Close),
            Some(Word::Quit) => Some(Outcome::PutDown),
            Some(Word::Saves) => {
                self.open_saves();
                None
            }
            Some(Word::New) => {
                self.open_lengths();
                None
            }
            Some(Word::Options) => {
                self.open_options();
                None
            }
            None => {
                self.complaint = Some(Complaint::Unknown(typed.to_owned()));
                None
            }
        }
    }

    /// Read the listing and show it.
    ///
    /// **Read here, not in `paint`.** A painter that did six file reads would do
    /// them sixty times a second, and the directory does not change while a
    /// player is looking at it.
    fn open_saves(&mut self) {
        if crate::save::path().is_none() {
            self.complaint = Some(Complaint::Unkept);
            return;
        }
        self.saves = crate::save::saves();
        self.page = Page::Saves;
    }

    /// Offer the lengths, if there is anywhere to put a new tower.
    fn open_lengths(&mut self) {
        if crate::save::path().is_none() {
            self.complaint = Some(Complaint::Unkept);
            return;
        }
        self.free = crate::save::free_slot();
        if self.free.is_none() {
            // **Refused here rather than after the length is chosen.** Asking
            // how long a game should be and then saying there is no room for it
            // is the dead end §15 names.
            self.complaint = Some(Complaint::Full);
            return;
        }
        self.page = Page::Lengths;
    }

    /// Show the drivers.
    ///
    /// **No `Unkept` here, unlike the two pages above.** A session that keeps no
    /// save can still be told how to read a line — the choice simply lasts as
    /// long as the session does, which is better than refusing to offer it. And
    /// no file read either: what is *in effect* is [`driver`](Self::driver), set
    /// by the frontend when the menu opened.
    const fn open_options(&mut self) {
        self.page = Page::Options;
    }

    /// A slot number on the saves page.
    fn picked(&mut self, typed: &str) -> Option<Outcome> {
        if typed.starts_with(BACK) || BACK.starts_with(typed) {
            self.page = Page::Choices;
            return None;
        }
        let Ok(slot) = typed.parse::<usize>() else {
            self.complaint = Some(Complaint::Unknown(typed.to_owned()));
            return None;
        };
        match self.saves.iter().find(|save| save.slot == slot) {
            Some(save) => Some(Outcome::Load(save.path.clone())),
            None => {
                self.complaint = Some(Complaint::Empty(slot));
                None
            }
        }
    }

    /// A length on the new-game page.
    fn measured(&mut self, typed: &str) -> Option<Outcome> {
        if typed.starts_with(BACK) || BACK.starts_with(typed) {
            self.page = Page::Choices;
            return None;
        }
        // Prefix-matched like every other word here, so `s`, `m` and `l` work.
        // `Length::named` wants the whole word, and `Baseline` is deliberately
        // not among the ones offered — see `Length::OFFERED`.
        let Some(length) = Length::OFFERED
            .into_iter()
            .find(|length| length.word().starts_with(typed))
        else {
            self.complaint = Some(Complaint::Unknown(typed.to_owned()));
            return None;
        };
        // `free` was filled when the page opened and the page does not open
        // without it, so this is a shape the type system wants rather than a
        // case that happens.
        let (Some(slot), Some(base)) = (self.free, crate::save::path()) else {
            self.complaint = Some(Complaint::Full);
            return None;
        };
        Some(Outcome::Begin {
            path: crate::save::slot_path(&base, slot),
            length,
        })
    }

    /// A driver on the options page.
    fn drove(&mut self, typed: &str) -> Option<Outcome> {
        if typed.starts_with(BACK) || BACK.starts_with(typed) {
            self.page = Page::Choices;
            return None;
        }
        let Some(driver) = Driver::ALL
            .into_iter()
            .find(|driver| driver.word().starts_with(typed))
        else {
            self.complaint = Some(Complaint::Unknown(typed.to_owned()));
            return None;
        };
        // **Written down here, and best-effort.** A session with nowhere to keep
        // a setting still gets to change it for as long as it lasts; refusing
        // the choice because it cannot be remembered would be the dead end §15
        // weighs heaviest, over a preference rather than over a tower.
        crate::settings::set_driver(driver);
        self.driver = driver;
        self.page = Page::Choices;
        Some(Outcome::Drive(driver))
    }

    /// Escape: one page back, and from the top page back to the tower.
    ///
    /// **From the top it is `resume`**, because the menu is a place you stepped
    /// into rather than a question you must answer; from a page it is `back`,
    /// for the same reason the weave's Escape returns to command mode rather
    /// than closing. Either way it never leaves the orb: the one irreversible
    /// choice here is always typed.
    pub fn escape(&mut self) -> Option<Outcome> {
        self.complaint = None;
        self.command.clear();
        match self.page {
            Page::Choices => Some(Outcome::Close),
            Page::Saves | Page::Lengths | Page::Options => {
                self.page = Page::Choices;
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_word_runs_by_its_shortest_unambiguous_prefix() {
        // Only the two that answer without touching the filesystem: `saves` and
        // `new` read the save directory, and what they do there is
        // `a_page_that_cannot_be_kept_says_so`'s.
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
        // **The two halves of the decision this screen exists to split.** Before
        // it, `quit` at the prompt ended the session outright and there was no
        // way to change your mind; now the word that used to end it opens this,
        // and ending it is a choice made here.
        let mut menu = Menu::default();
        menu.type_text("resume");
        assert_eq!(menu.enter(), Some(Outcome::Close));

        let mut menu = Menu::default();
        menu.type_text("quit");
        assert_eq!(menu.enter(), Some(Outcome::PutDown));
    }

    #[test]
    fn escape_steps_one_page_and_never_leaves_the_orb() {
        // **The one irreversible choice here is always typed.** From an inner
        // page Escape is `back`; from the top it is `resume`; it is never
        // `quit`, however many times it is pressed.
        let mut menu = Menu::default();
        assert_eq!(menu.escape(), Some(Outcome::Close));

        for page in [Page::Saves, Page::Lengths, Page::Options] {
            let mut menu = Menu {
                page,
                ..Menu::default()
            };
            menu.type_text("half a word");
            assert_eq!(menu.escape(), None, "escape left the menu from {page:?}");
            assert_eq!(menu.page(), Page::Choices);
            assert_eq!(menu.command(), "", "escape kept a half-typed line");
            assert_eq!(menu.escape(), Some(Outcome::Close));
        }
    }

    #[test]
    fn back_steps_a_page_by_its_prefix_and_a_slot_number_does_not() {
        for page in [Page::Saves, Page::Lengths, Page::Options] {
            let mut menu = Menu {
                page,
                ..Menu::default()
            };
            menu.type_text("b");
            assert_eq!(menu.enter(), None);
            assert_eq!(menu.page(), Page::Choices, "`b` did not step back");

            let mut menu = Menu {
                page,
                ..Menu::default()
            };
            menu.type_text("back");
            assert_eq!(menu.enter(), None);
            assert_eq!(menu.page(), Page::Choices);
        }
    }

    #[test]
    fn an_empty_slot_is_said_rather_than_opened() {
        // §6: a number with no tower behind it names what to do instead.
        let mut menu = Menu {
            page: Page::Saves,
            ..Menu::default()
        };
        menu.type_text("4");
        assert_eq!(menu.enter(), None);
        assert_eq!(menu.complaint(), Some(&Complaint::Empty(4)));
    }

    #[test]
    fn a_listed_tower_opens_the_file_it_is_actually_in() {
        // **A path, not a slot.** The shell writes the game it is leaving before
        // it loads this one, and it must write to the file that game came from —
        // resolving a number twice is what a changed `ORBS_SAVE` would make
        // disagree.
        let mut menu = Menu {
            page: Page::Saves,
            saves: vec![Slot {
                slot: 2,
                path: PathBuf::from("/somewhere/orbs-save-2.toml"),
                wizard: "david".into(),
                length: Length::Long,
                tick: 900,
                experience: 4_200,
                away: None,
            }],
            ..Menu::default()
        };
        menu.type_text("2");
        assert_eq!(
            menu.enter(),
            Some(Outcome::Load(PathBuf::from("/somewhere/orbs-save-2.toml"))),
        );
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
                free: Some(3),
                ..Menu::default()
            };
            menu.type_text(typed);
            let outcome = menu.enter();
            let Some(Outcome::Begin { length, .. }) = outcome else {
                panic!("`{typed}` did not begin a game: {outcome:?}");
            };
            assert_eq!(length, want);
        }

        // **`baseline` is the numbers we happened to author first, not a
        // difficulty** — `Length::OFFERED` leaves it out and so must this.
        let mut menu = Menu {
            page: Page::Lengths,
            free: Some(3),
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
    fn a_driver_is_chosen_by_prefix_and_steps_back_to_the_top() {
        // **The whole point of the page**, and the shape it shares with the
        // lengths: prefix-matched, and the choice is what closes the page.
        for (typed, want) in [("a", Driver::Augury), ("plain", Driver::Plain)] {
            let mut menu = Menu {
                page: Page::Options,
                ..Menu::default()
            };
            menu.type_text(typed);
            assert_eq!(menu.enter(), Some(Outcome::Drive(want)), "`{typed}`");
            assert_eq!(menu.page(), Page::Choices, "the page stayed open");
            assert_eq!(menu.driver(), want, "the page did not show the new choice");
        }
    }

    #[test]
    fn an_unknown_driver_is_said_rather_than_guessed() {
        // §6 again: the page names what it does not know instead of picking one.
        let mut menu = Menu {
            page: Page::Options,
            ..Menu::default()
        };
        menu.type_text("magic");
        assert_eq!(menu.enter(), None);
        assert_eq!(
            menu.complaint(),
            Some(&Complaint::Unknown("magic".into())),
            "an unknown driver silently chose one",
        );
        assert_eq!(menu.page(), Page::Options, "it left the page anyway");
    }

    #[test]
    fn an_empty_line_is_not_a_mistake_and_an_unknown_word_is_said() {
        let mut menu = Menu::default();
        assert_eq!(menu.enter(), None);
        assert_eq!(menu.complaint(), None, "a bare Enter complained");

        menu.type_text("burn it all down");
        assert_eq!(menu.enter(), None);
        assert_eq!(
            menu.complaint(),
            Some(&Complaint::Unknown("burn it all down".into())),
            "an unknown word said nothing, which §6 forbids",
        );
        assert_eq!(menu.command(), "", "the line kept what it could not run");
    }

    #[test]
    fn typing_and_rubbing_out_reach_the_same_line() {
        let mut menu = Menu::default();
        menu.type_text("quix");
        menu.backspace();
        menu.type_text("t");
        assert_eq!(menu.command(), "quit");
        assert_eq!(menu.enter(), Some(Outcome::PutDown));
    }
}
