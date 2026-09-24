//! What a keystroke does to the menu.
//!
//! The pages, the words they answer to, and what the shell is asked to do about
//! it. Nothing here draws — see [`paint`](super::paint) — and nothing here ends
//! a process.

use std::path::PathBuf;

use orbs_sim::content::Length;

use super::stance::Stance;
use crate::save::Slot;
use crate::settings::{Category, Row};

/// The menu, while it is up.
///
/// A page, a line and a complaint. Its choices are authored, its state is what
/// is half-typed and which page is showing, and the screen it sits over is the
/// frontend's. Smaller than its precedent `Tapestry`: no cursor, because there
/// is nothing here to walk — every choice is a word, as in the rest of the game.
/// `Default` is written out rather than derived because of `keeps`: a derive
/// gives it `false`, which says *this session keeps nothing* about a session
/// nothing has asked yet. Every other field's default is genuinely its zero
/// value, which is what made the derive look safe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Menu {
    /// Where the menu is standing — over a tower, or in front of none.
    ///
    /// Decides which words the top page offers and whether there is anything to
    /// close it onto. See [`Stance`].
    pub(super) stance: Stance,
    /// Which page is showing.
    pub(super) page: Page,
    /// What is being typed at it.
    pub(super) command: String,
    /// What it would not do, and why.
    pub(super) complaint: Option<Complaint>,
    /// The towers the orb is keeping, as of the last time [`Word::Saves`] was
    /// asked for. Read when the page opens rather than every frame: a listing is
    /// six file reads, and the directory does not change while it is on screen.
    pub(super) saves: Vec<Slot>,
    /// Whether this session keeps towers at all, as of the same read.
    ///
    /// Beside [`saves`](Self::saves) rather than derived from it: *no towers*
    /// and *nowhere to put one* are two different sentences, and an empty `Vec`
    /// cannot tell them apart.
    ///
    /// `true` before the play page has looked, which is why `Menu` cannot take a
    /// derived `Default` — a menu that has not asked has not found out this
    /// session keeps nothing, and claiming so is the more misleading guess.
    pub(super) keeps: bool,
    /// Where a new tower would be kept, decided when the lengths page opened.
    ///
    /// A path rather than a slot number, for [`Outcome::Load`]'s reason: a
    /// number is resolved twice, and a changed `ORBS_SAVE` makes the two
    /// resolutions disagree. `None` means this session keeps nothing — a tower
    /// playable and not remembered, not a refusal. A *full* orb is refused
    /// before this page opens, so `None` here has one meaning.
    pub(super) keep: Option<PathBuf>,
    /// The slot `abandon` has asked about, while the question is open.
    ///
    /// A field rather than a [`Complaint`], because the complaint is cleared at
    /// the top of every line and this has to survive exactly one: `abandon 3`
    /// puts the question, a second `abandon 3` answers it, any other line
    /// answers no. `quit`'s shape (§19), deliberately — this is the only other
    /// word in the game that cannot be undone by typing something else.
    pub(super) asking: Option<usize>,
    /// Which driver the settings page last read, so it can show what is chosen.
    /// Read when the page opens, for `saves`' reason.
    pub(super) driver: Driver,
    /// Every setting this frontend can honour, and what each is set to.
    ///
    /// Handed in by the frontend when the menu opens, `show_driver`'s rule
    /// generalised: the menu does not know what a phosphor theme is, how many
    /// there are, or whether this build has a tube. A page with no rows is not
    /// offered — so `orbs-tui`, which binds five function keys and has no audio,
    /// shows fewer pages rather than controls that do nothing.
    pub(super) rows: Vec<Row>,
}

/// Which of the menu's pages is showing.
///
/// Pages rather than as many surfaces, because they share the line, the
/// complaint and the way out: `back` and Escape step one level, and from the top
/// Escape is `resume`. A `Focus` variant each would be that much of everything
/// for one screen a player sees a page of at a time.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Page {
    /// The words.
    #[default]
    Choices,
    /// Open a tower: the ones the orb is keeping, numbered, and `new` beside
    /// them.
    ///
    /// One page rather than a listing and a *begin a game* beside it. As two
    /// words on the top page — `saves` and `new` — they asked the player to hold
    /// the difference between *open one you have* and *raise one* before
    /// anything had told them there was one.
    Play,
    /// How long a new game should be.
    Lengths,
    /// Which part of the orb to set: the tube, the accommodations, its habits.
    Settings,
    /// One of those pages, with a row per setting and what it is set to.
    ///
    /// The value is on the row and a word cycles it, so there is no third page
    /// under this one. `crt` steps to the next state and `crt off` goes straight
    /// there — what `F3` already does, plus a way to name the state a See-it
    /// line wants.
    Setting(Category),
}

/// How the orb reads what you type.
///
/// The player's choice between the two drivers, which until now was an
/// environment variable and therefore nobody's. §6's mastery arc rests on
/// seeing the canonical form echoed back; this decides whether the orb works one
/// out from a sentence, or answers only the words it already knows.
///
/// [`Augury`](Self::Augury) means the same in both builds: the terminal
/// frontend is the whole game, not a cut-down one. It had no reader while the
/// reader needed `wgpu`; inference is `ndarray` and costs 436µs, so the GPU went
/// behind `orbs-augury`'s `train` feature.
///
/// Still a *preference* rather than a claim about the build — a checkout with no
/// trained weights has nothing to consult and falls through to the matcher,
/// which is what a fresh clone does.
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
/// Nothing here ends a process. The shell decides what putting the orb down
/// *means* — an `AppExit` in the Bevy build, raw mode going back in the
/// terminal — the same division `execute::quit` draws.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Take the menu down; the tower is as it was.
    Close,
    /// Leave the orb.
    PutDown,
    /// Put the tower in this file in front of the player.
    ///
    /// A path, not a slot number. The shell writes the game it is leaving before
    /// it loads this one, and to the file that one came from — a number would be
    /// resolved twice, and a stale `SLOTS` or a changed `ORBS_SAVE` is exactly
    /// what makes the two resolutions disagree.
    Load(PathBuf),
    /// Begin a tower in this file, at this length.
    Begin {
        /// Where it will be kept, or [`None`] to keep it nowhere.
        ///
        /// `None` is a playable tower, not a refusal. `ORBS_SAVE=off` is not an
        /// exotic mode — `scripts/dumps.sh` exports it for every capture and the
        /// played-game suite for every scenario — so a threshold that refused to
        /// begin a game without somewhere to keep it would make the build
        /// unreachable under the switch its own instruments run on.
        path: Option<PathBuf>,
        /// How long it is to be.
        length: Length,
    },
    /// Read lines this way from now on.
    ///
    /// Already written down by the time this is returned: the menu persists the
    /// choice itself, as `saves` reads the directory itself. This tells the
    /// running frontend to stop consulting a reader *now* rather than at the
    /// next launch.
    Drive(Driver),
    /// Open the manual.
    ///
    /// Over the menu, not instead of it: the menu stays up behind and gets the
    /// keyboard back when the reader closes, which is why `Focus` puts the
    /// manual *above* it rather than beside it.
    OpenManual,
    /// Set this to that, now.
    ///
    /// Written down by the time this is returned, as [`Drive`] is; this tells
    /// the running frontend to honour it *now* rather than at the next launch.
    /// A player who turns the tube off and cannot see it go off would reasonably
    /// think nothing happened.
    ///
    /// [`Drive`]: Self::Drive
    Set {
        /// The setting's word, which is also its key in the settings file.
        setting: String,
        /// The value it is now, as a word.
        value: String,
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
    /// A slot with a file in it this build cannot read.
    ///
    /// Not [`Empty`](Self::Empty), which it used to answer about a file sitting
    /// right there with somebody's game in it. See [`Slot::held`].
    ///
    /// [`Slot::held`]: crate::Slot::held
    Unreadable(usize),
    /// `abandon` has asked whether this slot really goes.
    ///
    /// The question, not a refusal — it is here because it is drawn where the
    /// refusals are and lasts exactly as long as one. Answering is typing the
    /// same words again; anything else is *no*.
    Abandoning(usize),
    /// A slot this build could not set aside.
    ///
    /// A read-only directory, or a file that has gone since the listing was
    /// read. Said rather than swallowed: the player asked twice and the tower is
    /// still there.
    Unabandoned(usize),
    /// The orb is keeping as many towers as it can.
    Full,
    /// This session keeps no save at all — `ORBS_SAVE=off`.
    ///
    /// Said rather than hidden. A dump and the played-game suite both run this
    /// way on purpose, and a `saves` that drew an empty list would say *you have
    /// no towers* to a player who has several.
    Unkept,
}

impl Default for Menu {
    fn default() -> Self {
        Self::blank()
    }
}

impl Menu {
    /// A menu standing here, with nothing typed at it.
    ///
    /// `Menu::default()` is the in-tower one and stays that way — four frontend
    /// tests and `dump::menued` build it, and all of them mean the menu the
    /// `menu` verb opens.
    #[must_use]
    pub fn at(stance: Stance) -> Self {
        Self {
            stance,
            ..Self::default()
        }
    }

    /// Whether the play page has looked, and what it found.
    ///
    /// Exposed for a test that has not opened the page; the game's answer comes
    /// from `open_play`.
    #[cfg(test)]
    pub(super) const fn keeps(&self) -> bool {
        self.keeps
    }

    /// Where the menu is standing.
    #[must_use]
    pub const fn stance(&self) -> Stance {
        self.stance
    }

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

    /// Whether this session keeps towers at all.
    ///
    /// Not the same question as *"are there any?"*, and the play page needs
    /// both: *no towers yet* is a first launch and invites `new`, while
    /// `Complaint::Unkept` is a session with nowhere to put one. Drawing the
    /// first when the second is true is the untruth the page used to tell — see
    /// `open_play`.
    #[must_use]
    pub const fn keeps_towers(&self) -> bool {
        self.keeps
    }

    /// Which driver is in effect, as the frontend last said.
    #[must_use]
    pub const fn driver(&self) -> Driver {
        self.driver
    }

    /// Tell the menu what this frontend can set, and what each is set to.
    ///
    /// `show_driver`'s rule generalised: the frontend is the source of truth
    /// about its own settings — how many phosphor themes there are, whether it
    /// has a tube, whether anything is playing. A menu that read the settings
    /// *file* would draw a session the choices it did not make, and would offer
    /// the terminal build a CRT.
    pub fn show_settings(&mut self, rows: Vec<Row>) {
        self.rows = rows;
    }

    /// Every setting on `page`, in the order the frontend gave them.
    pub fn rows(&self, page: Category) -> impl Iterator<Item = &Row> {
        self.rows.iter().filter(move |row| row.page == page)
    }

    /// A menu nobody has opened a page on yet. See [`Menu::default`].
    fn blank() -> Self {
        Self {
            stance: Stance::default(),
            page: Page::default(),
            command: String::new(),
            complaint: None,
            saves: Vec::new(),
            keeps: true,
            keep: None,
            asking: None,
            driver: Driver::default(),
            rows: Vec::new(),
        }
    }

    /// Whether a page of settings is on screen.
    ///
    /// For a frontend deciding whether the rows are worth keeping current — see
    /// `shell::setting::follow_the_keys`. A bool rather than exporting `Page`,
    /// which is deliberately not public: a caller outside needs this question,
    /// not the enum.
    #[must_use]
    pub const fn showing_settings(&self) -> bool {
        matches!(self.page, Page::Setting(_))
    }

    /// Every setting, whatever page it is on.
    ///
    /// For a frontend keeping an open page current — see
    /// `shell::setting::follow_the_keys`, and the stale row a function key left
    /// behind before it existed.
    #[must_use]
    pub fn settings(&self) -> &[Row] {
        &self.rows
    }

    /// The pages that have anything on them.
    ///
    /// A page with no rows is not offered, which is how *unsupported* is
    /// expressed: no mask and no dimmed row, because a control that does nothing
    /// is the dead affordance §15 weighs heaviest.
    pub fn pages(&self) -> impl Iterator<Item = Category> + '_ {
        Category::ALL
            .into_iter()
            .filter(|page| self.rows(*page).next().is_some())
    }

    /// Tell the menu what is currently in effect.
    ///
    /// The frontend is the source of truth, not the settings file: a session
    /// with nowhere to write one still has a driver, and a page that read the
    /// file would show it the choice it did not make. Called when the menu
    /// opens, the only moment the answer can have changed without this page
    /// changing it.
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
            // An empty line is still a line, and §19's rule is that any line but
            // the same one answers *no*. Returning before `answered` left the
            // question armed with its text rubbed off: a bare Enter cleared
            // `menu_abandoning` from the pane while `asking` kept pointing at
            // the slot.
            self.asking = None;
            return None;
        }
        // What a word *means* is [`pages`](super::pages)'; only the shape of the
        // line is here. See that module for why.
        self.answered(&typed)
    }

    /// Escape: one page back, and from the top page back to the tower.
    ///
    /// From the top it is `resume`, because the menu is a place you stepped into
    /// rather than a question you must answer; from a page it is `back`, for the
    /// reason the weave's Escape returns to command mode rather than closing.
    /// Either way it never leaves the orb: the one irreversible choice here is
    /// always typed.
    ///
    /// At the threshold there is no top to leave from, and [`Stance::may_close`]
    /// gates it. Closing hands the keyboard back to the prompt — of a scratch
    /// world that never ticks and is never kept, with no way back to the menu
    /// and no way out of the orb. So Escape on the top page does nothing there,
    /// and `resume` is not a word the menu answers to at all: `Word::offered`
    /// filters it out of the vocabulary, not merely out of the listing.
    pub fn escape(&mut self) -> Option<Outcome> {
        self.complaint = None;
        self.command.clear();
        // Escape answers an open `abandon` question with *no*, like any other
        // line. It clears the question's *text* either way, so leaving the latch
        // armed meant the page said nothing was being asked while `asking` still
        // pointed at a slot — harmless today, since `answered` takes it on the
        // next line, and the kind of state that stops being harmless when
        // something else reads it. `enter` does the same for an empty line.
        self.asking = None;
        match self.page {
            Page::Choices if self.stance.may_close() => Some(Outcome::Close),
            // Nothing behind it. Not a complaint either: Escape at a screen with
            // no way back is a question the player has not asked, and answering
            // it with a refusal would be noise on every stray keystroke.
            Page::Choices => None,
            // A settings page steps back to `settings`, not to the top: Escape
            // means *one level*, which is what `back` means everywhere else.
            Page::Setting(_) => {
                self.page = Page::Settings;
                None
            }
            // The second page that is two deep: `new` moved down under `play` at
            // `0.16.1` and this arm did not follow, so Escape stepped past the
            // tower listing. `pages::answered` has the same table for `back` and
            // the two must agree — `escape_and_back_step_to_the_same_page`.
            Page::Lengths => {
                self.page = Page::Play;
                None
            }
            Page::Play | Page::Settings => {
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
    fn escape_steps_one_page_and_never_leaves_the_orb() {
        // The one irreversible choice here is always typed. From an inner page
        // Escape is `back`, from the top `resume`, never `quit`.
        let mut menu = Menu::default();
        assert_eq!(menu.escape(), Some(Outcome::Close));

        // Each page steps to its own parent, not to the top. This asserted
        // `Page::Choices` for everything and so could not see `new` moving down
        // a level under `Play`.
        for (page, parent) in [
            (Page::Play, Page::Choices),
            (Page::Lengths, Page::Play),
            (Page::Settings, Page::Choices),
        ] {
            let mut menu = Menu {
                page,
                ..Menu::default()
            };
            menu.type_text("half a word");
            assert_eq!(menu.escape(), None, "escape left the menu from {page:?}");
            assert_eq!(menu.page(), parent, "escape from {page:?} overshot");
            assert_eq!(menu.command(), "", "escape kept a half-typed line");
        }
    }

    #[test]
    fn escape_and_back_step_to_the_same_page() {
        // Two expressions of one rule — `escape` here and the `back` table in
        // `pages::answered` — and they both went to the top page for a version
        // while `Lengths`'s parent was `Play`.
        for page in [Page::Play, Page::Lengths, Page::Settings] {
            let mut escaped = Menu {
                page,
                ..Menu::default()
            };
            escaped.escape();

            let mut backed = Menu {
                page,
                ..Menu::default()
            };
            backed.type_text("back");
            backed.enter();

            assert_eq!(
                escaped.page(),
                backed.page(),
                "escape and `back` disagree about where {page:?} steps to",
            );
        }
    }

    #[test]
    fn the_threshold_has_no_way_to_close_the_menu() {
        // The property this stance exists for: closing hands the keyboard back
        // to the prompt, and at the threshold that prompt belongs to a scratch
        // world that never ticks and is never kept — no way back to the menu, no
        // way out of the orb. Escape, from every page, however many times.
        let mut menu = Menu::at(Stance::Threshold);
        for _ in 0..3 {
            assert_eq!(menu.escape(), None, "escape closed the threshold");
            assert_eq!(menu.page(), Page::Choices);
        }
        // Escape steps to the page's *parent* — `Lengths` to `Play`, the rest to
        // the top — and the threshold answers nothing from the top, so pressed
        // enough times every page reaches it and stays there.
        for page in [Page::Play, Page::Lengths, Page::Settings] {
            let mut menu = Menu {
                page,
                ..Menu::at(Stance::Threshold)
            };
            for _ in 0..4 {
                assert_eq!(menu.escape(), None, "escape closed the threshold");
            }
            assert_eq!(menu.page(), Page::Choices, "escape did not step back");
        }

        // And `resume`, whole or by prefix, which the listing does not draw but
        // the line would otherwise still answer.
        for typed in ["r", "res", "resume"] {
            let mut menu = Menu::at(Stance::Threshold);
            menu.type_text(typed);
            assert_eq!(menu.enter(), None, "`{typed}` closed the threshold");
            assert_eq!(
                menu.complaint(),
                Some(&Complaint::Unknown(typed.into())),
                "`{typed}` was neither answered nor refused",
            );
        }

        // The same menu over a tower does close, which makes the above a
        // property of the stance rather than of the code path.
        let mut menu = Menu::at(Stance::InTower);
        assert_eq!(menu.escape(), Some(Outcome::Close));
    }

    #[test]
    fn quit_still_leaves_the_orb_from_the_threshold() {
        // The stance takes away stepping back into a tower, not putting the orb
        // down.
        let mut menu = Menu::at(Stance::Threshold);
        menu.type_text("quit");
        assert_eq!(menu.enter(), Some(Outcome::PutDown));
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
