//! The orb's menu: the screen `quit` opens (DESIGN.md §15, §19).
//!
//! # `quit` opens it, and that spends no verb
//!
//! `execute::quit`'s own doc already argued the shape: *"`quit` closes the spell
//! editor, `quit` closes the weave screen, and now `quit` closes the orb: the
//! word means **leave the thing you are in**, whichever thing that is."* From a
//! room, the thing you are in is the game — so `quit` steps out of it and puts
//! this in front of you, and the menu's own `quit` steps out of the orb.
//!
//! **Discoverability was already solved.** `quit` is `is_live` in every room and
//! has a manual page; a menu reached by a key nobody can find would be the exact
//! defect `execute::quit` exists to fix, arriving one level up.
//!
//! **The sim's `Quitting` handshake is unchanged.** It still means *the player
//! asked to leave*, and what leaving does was always the frontend's business —
//! that is why the sim never wrote an `AppExit` itself. So this needed no new
//! verb, no new resource and no save migration.
//!
//! # Rule 2, and why it holds no `Sim`
//!
//! Everything drawn is authored prose or something the shell handed in.
//! [`Menu`] holds what is *typed* and what was refused, exactly as [`Tapestry`]
//! does; the frontend owns the rest.
//!
//! [`Tapestry`]: crate::Tapestry

use std::path::PathBuf;

use orbs_render::{Frame, Painter, Pos, Rect, Span, Style};
use orbs_sim::Prose;
use orbs_sim::content::Length;

use crate::save::Slot;

/// The menu, while it is up.
///
/// **A page, a line and a complaint.** Its choices are authored, its state is
/// what is half-typed and which of three pages is showing, and the screen it
/// sits over is the frontend's. `Tapestry` is the precedent and this is smaller:
/// there is no cursor, because there is nothing here to walk — every choice is a
/// word, which is what the rest of the game is.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Menu {
    /// Which page is showing.
    page: Page,
    /// What is being typed at it.
    command: String,
    /// What it would not do, and why.
    complaint: Option<Complaint>,
    /// The towers the orb is keeping, as of the last time [`Word::Saves`] was
    /// asked for. **Read once when the page opens** rather than every frame: a
    /// listing is six file reads, and the directory does not change while a
    /// player is looking at it.
    saves: Vec<Slot>,
    /// The lowest slot with no tower in it, or `None` when the orb is full.
    free: Option<usize>,
}

/// Which of the menu's three pages is showing.
///
/// **Pages rather than three surfaces**, because they share the line, the
/// complaint and the way out: `back` and Escape step one level, and from the top
/// Escape is `resume`. Three `Focus` variants would be three of everything for
/// one screen a player never sees more than one page of.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Page {
    /// The words.
    #[default]
    Choices,
    /// The towers the orb is keeping, one per row, numbered.
    Saves,
    /// How long a new game should be.
    Lengths,
}

/// A word the menu answers to.
///
/// **Prefix-matched, and no two share a first letter** — the rule the editor's
/// and the weave's vocabularies both follow, so `r`, `s`, `n` and `q` all work
/// and the property is [a test](tests::no_two_words_share_a_first_letter) rather
/// than a convention. It holds across the other two pages as well: `back` on
/// both, `short`/`medium`/`long` on one, and a bare number on the other.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Word {
    /// Go back to the tower.
    Resume,
    /// List the towers the orb is keeping.
    Saves,
    /// Begin one.
    New,
    /// Put the orb down.
    Quit,
}

/// How many words the menu offers.
///
/// **A `u16` the array's length is written in terms of**, rather than a cast of
/// `WORDS.len()`: [`MIN_ROWS`] needs the count in cells, and a cast there is one
/// clippy will not take on faith. Adding a word means changing this, and the
/// array literal will not compile until it is.
const COUNT: u16 = 4;

/// Every word, in the order the menu offers them.
pub const WORDS: [(&str, Word); COUNT as usize] = [
    ("resume", Word::Resume),
    ("saves", Word::Saves),
    ("new", Word::New),
    ("quit", Word::Quit),
];

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
            Page::Saves | Page::Lengths => {
                self.page = Page::Choices;
                None
            }
        }
    }
}

/// The word that steps one page back, on both inner pages.
///
/// Not in [`WORDS`], which is the top page's list. It shares no first letter
/// with the lengths (`short`, `medium`, `long`) and cannot collide with a slot
/// number, which is the whole of what it has to avoid.
const BACK: &str = "back";

/// The word `typed` names, by unambiguous prefix.
fn word(typed: &str) -> Option<Word> {
    WORDS
        .iter()
        .find(|(name, _)| name.starts_with(typed))
        .map(|(_, word)| *word)
}

/// [`crate::save::SLOTS`] as rows.
///
/// Written out rather than cast, and asserted equal in
/// `the_floor_fits_the_tallest_page` — `as` from `usize` is a truncation clippy
/// will not take on faith, and this is the one number the floor below depends
/// on.
const SLOT_ROWS: u16 = 6;

/// Rows the tallest page needs: the saves listing, which is one row per slot,
/// then a blank and the way back.
const TALLEST: u16 = SLOT_ROWS + 2;

/// The smallest pane this can honestly be drawn in.
///
/// Two borders, the lead, a blank, the page, a blank, the line, and a row under
/// it for a complaint. Below this it says the pane is too small rather than
/// drawing a menu with a choice missing from it — the answer the weave already
/// gives, and the same reason: **a choice you cannot see is one you do not
/// have**, and one of these choices is how you get out.
///
/// **Sized for the tallest page rather than the one showing**, so the menu does
/// not fit when you open it and stop fitting when you ask for the listing.
const MIN_ROWS: u16 = 7 + TALLEST;
const MIN_COLS: u16 = 34;

/// The caret's lead-in, and what the typed line is indented by.
const LEAD: &str = "> ";

/// Draw the menu into `pane`.
///
/// **The caret is set after the painter is done**, the way `sheet::paint` hands
/// one back: a `Painter` holds the `Frame`, so the two cannot be reached at
/// once.
pub fn paint(frame: &mut Frame, menu: &Menu, pane: Rect, prose: &Prose) {
    if pane.is_empty() {
        return;
    }
    let caret = {
        let mut painter = frame.painter(pane);
        let area = painter.area();
        painter.border(area, Some(&prose.line("menu_title", &[])), Style::DIM);
        let inner = area.inset(1);
        if inner.is_empty() {
            return;
        }
        if area.rows < MIN_ROWS || area.cols < MIN_COLS {
            painter.paragraph(inner, &Span::new(&prose.line("menu_too_small", &[])));
            None
        } else {
            Some(body(&mut painter, menu, inner, prose))
        }
    };
    // `None` when the pane is too small to type into: a caret is the game's one
    // promise about where a keystroke lands, and drawing it over a message that
    // says there is no room would be a lie about exactly that.
    frame.set_cursor(caret);
}

/// The menu's rows, and where the caret ends up.
fn body(painter: &mut Painter<'_>, menu: &Menu, inner: Rect, prose: &Prose) -> Pos {
    let lead = match menu.page() {
        Page::Choices => "menu_lead",
        Page::Saves => "menu_saves_lead",
        Page::Lengths => "menu_new_lead",
    };
    let mut y = inner.row;
    painter.span(
        Pos::new(inner.col, y),
        &Span::new(&prose.line(lead, &[])).with_style(Style::DIM),
    );
    y = y.saturating_add(2);

    y = match menu.page() {
        Page::Choices => choices(painter, inner, y, prose),
        Page::Saves => listing(painter, menu, inner, y, prose),
        Page::Lengths => lengths(painter, inner, y, prose),
    };
    y = y.saturating_add(1);

    // The line, with a caret. Unlike the weave — which has a browsing mode and
    // draws none — there is exactly one thing to do here and it is typing, so
    // the caret is always honest.
    painter.span(
        Pos::new(inner.col, y),
        &Span::new(&format!("{LEAD}{}", menu.command())),
    );

    if let Some(complaint) = menu.complaint() {
        painter.span(
            Pos::new(inner.col, y.saturating_add(1)),
            &Span::new(&said(complaint, prose)).with_style(Style::DIM),
        );
    }

    let typed = LEAD
        .chars()
        .count()
        .saturating_add(menu.command().chars().count());
    Pos::new(
        inner
            .col
            .saturating_add(u16::try_from(typed).unwrap_or(u16::MAX)),
        y,
    )
}

/// The top page's words.
fn choices(painter: &mut Painter<'_>, inner: Rect, mut y: u16, prose: &Prose) -> u16 {
    for (name, _) in WORDS {
        painter.span(
            Pos::new(inner.col.saturating_add(2), y),
            &Span::new(&prose.line(&format!("menu_word_{name}"), &[])),
        );
        y = y.saturating_add(1);
    }
    y
}

/// The towers the orb is keeping, one row each.
fn listing(painter: &mut Painter<'_>, menu: &Menu, inner: Rect, mut y: u16, prose: &Prose) -> u16 {
    if menu.saves().is_empty() {
        painter.span(
            Pos::new(inner.col.saturating_add(2), y),
            &Span::new(&prose.line("menu_saves_none", &[])),
        );
        y = y.saturating_add(1);
    }
    for save in menu.saves() {
        painter.span(
            Pos::new(inner.col.saturating_add(2), y),
            &Span::new(&prose.line(
                "menu_saves_row",
                &[
                    ("count", &save.slot.to_string()),
                    ("name", &save.wizard),
                    ("detail", save.length.word()),
                    ("quantity", &save.experience.to_string()),
                ],
            )),
        );
        y = y.saturating_add(1);
    }
    y = y.saturating_add(1);
    painter.span(
        Pos::new(inner.col.saturating_add(2), y),
        &Span::new(&prose.line("menu_word_back", &[])).with_style(Style::DIM),
    );
    y.saturating_add(1)
}

/// How long a new game should be.
fn lengths(painter: &mut Painter<'_>, inner: Rect, mut y: u16, prose: &Prose) -> u16 {
    for length in Length::OFFERED {
        painter.span(
            Pos::new(inner.col.saturating_add(2), y),
            &Span::new(&prose.line(&format!("menu_length_{}", length.word()), &[])),
        );
        y = y.saturating_add(1);
    }
    y = y.saturating_add(1);
    painter.span(
        Pos::new(inner.col.saturating_add(2), y),
        &Span::new(&prose.line("menu_word_back", &[])).with_style(Style::DIM),
    );
    y.saturating_add(1)
}

/// A refusal, as a sentence. §6: never a bare error.
fn said(complaint: &Complaint, prose: &Prose) -> String {
    match complaint {
        Complaint::Unknown(word) => prose.line("menu_unknown", &[("detail", word)]),
        Complaint::Empty(slot) => prose.line("menu_empty", &[("count", &slot.to_string())]),
        Complaint::Full => prose.line("menu_full", &[]),
        Complaint::Unkept => prose.line("menu_unkept", &[]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_two_words_share_a_first_letter() {
        // The prefix rule the editor and the weave both keep, so `r`, `s`, `n`
        // and `q` are all unambiguous.
        let mut firsts: Vec<char> = WORDS
            .iter()
            .filter_map(|(name, _)| name.chars().next())
            .collect();
        firsts.sort_unstable();
        let before = firsts.len();
        firsts.dedup();
        assert_eq!(before, firsts.len(), "two words share a first letter");
    }

    #[test]
    fn the_inner_pages_have_no_collisions_either() {
        // `back` sits on both inner pages beside the lengths and the slot
        // numbers, and it must not prefix-match any of them — `b` has to mean
        // one thing wherever it is typed.
        for length in Length::OFFERED {
            assert!(
                !length.word().starts_with(&BACK[..1]),
                "`b` is ambiguous between back and {}",
                length.word(),
            );
        }
        assert!(BACK.parse::<usize>().is_err(), "back reads as a slot");

        let mut firsts: Vec<char> = Length::OFFERED
            .iter()
            .filter_map(|length| length.word().chars().next())
            .collect();
        firsts.sort_unstable();
        let before = firsts.len();
        firsts.dedup();
        assert_eq!(before, firsts.len(), "two lengths share a first letter");
    }

    #[test]
    fn the_floor_fits_the_tallest_page() {
        // `SLOT_ROWS` is `save::SLOTS` written as a `u16`; if they drift, the
        // listing grows a row the floor did not reserve and the last tower falls
        // off the bottom of a short pane.
        assert_eq!(usize::from(SLOT_ROWS), crate::save::SLOTS);
        const {
            assert!(
                TALLEST >= COUNT,
                "the top page is taller than the floor was sized for",
            );
        }
        let lengths = u16::try_from(Length::OFFERED.len()).expect("three of them");
        assert!(TALLEST >= lengths + 2, "the lengths page does not fit");
    }

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

        for page in [Page::Saves, Page::Lengths] {
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
        let mut menu = Menu {
            page: Page::Saves,
            ..Menu::default()
        };
        menu.type_text("b");
        assert_eq!(menu.enter(), None);
        assert_eq!(menu.page(), Page::Choices);

        let mut menu = Menu {
            page: Page::Lengths,
            ..Menu::default()
        };
        menu.type_text("back");
        assert_eq!(menu.enter(), None);
        assert_eq!(menu.page(), Page::Choices);
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
