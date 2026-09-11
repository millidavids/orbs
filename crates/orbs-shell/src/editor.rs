//! The spell editor: three states, and words rather than punctuation.
//!
//! DESIGN.md §8 has the player keeping their spellbook in their own editor and
//! hot-reloading it. That needs files on disk and a watcher, which is later
//! Phase 1 work; this is the in-game shape of the same thing, and building it
//! first keeps the two aligned — what a save does here is what `:w` in vim will
//! have to do there.
//!
//! # Three states, and you start in the one that cannot lose your work
//!
//! [`Mode::Command`] is where the editor opens. You type **words** — `edit`,
//! `interpret` and `quit`, which is all of them — and they do what they say.
//! [`Mode::Editing`] is where keystrokes go into the spell, and `Esc` comes
//! back.
//!
//! [`Mode::Reading`] is the third and the newest. The file stopped being
//! rewritten when it is saved (§19), which is what a player asked for — and it
//! took away the only place the orb's reading was ever visible. `interpret`
//! is where that went: the buffer, line for line, as the orb hears it.
//!
//! # There is no `save`
//!
//! The buffer writes itself out a beat after the typing stops — see
//! [`Editor::settle`]. A spell can be edited **while it is running** and the orb
//! picks the change up between steps, which makes saving part of the loop rather
//! than a thing you finish with: you fix a line, and watch the next pass take
//! it. A `save` word standing in the middle of that is a chore where the
//! mechanic should be.
//!
//! It follows that there is nothing to discard, and nothing for `quit` to refuse
//! over. Both of those words existed and both are gone.
//!
//! This replaces a single always-insert mode with a `:` command line, which was
//! a worse shape for two reasons. The small one: `:` is punctuation you have to
//! be told about, and the game around it is built on §6's *"you type what you
//! mean and the orb works out what you meant"* — a modal surface that answers to
//! `edit` and `quit` is the same game as the one outside it, while one that
//! answers to `:wq` is a different program wearing its clothes.
//!
//! The large one: **opening in command state means the first keystroke cannot
//! damage anything.** A player who does not yet know what this screen is presses
//! a key and gets told what the words are, rather than silently editing a spell
//! they thought they were reading.
//!
//! # Why the buffer is here and not in `orbs-sim`
//!
//! The same test that put [`Line`](super::Line) here. A keystroke reaches no
//! decision: it never enters `Submissions`, never enters `ParseLog`, and cannot
//! make two runs from one seed diverge. **The save is the decision**, and that
//! goes through `Sim::write_spell` as a single entry carrying the whole buffer.
//!
//! Three things follow that would otherwise have to be worked around:
//!
//! - `orbs-sim` may not name a layout type at all — `tests/boundaries.rs`
//!   enforces it by substring — so a buffer over there could not know its own
//!   pane height, and scrolling would have to live on this side anyway.
//! - `ORBS_DUMP` runs in this crate and already reaches into `Line` for
//!   `ORBS_LINE`, so a frontend-side buffer is *easier* to drive from a dump,
//!   not harder.
//! - There is only one editor. The third reason used to be *"`orbs-tui` is a
//!   stub with nothing to diverge from"*, and it expired the day that stopped
//!   being true — which is why this file is in `orbs-shell` and not in a
//!   frontend. The decision it supported did not change; what changed is that it
//!   now rests on the crate boundary rather than on nobody having tested it.
//!   Both frontends map their own keystrokes onto `orbs_shell::Key` and reach
//!   the buffer through the same `enter`, `escape` and `type_text`.

use orbs_render::arriving;
// The spell language's own indentation rule, folded here as you type and by the
// orb when it writes the file. **One definition, deliberately** — a buffer that
// indented one way and a saved file that indented another would make every save
// look like it had moved the player's work.
use orbs_sim::parser::{INDENT, indent_around};

/// Which state the editor is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// Typing words at the editor. The state it opens in.
    #[default]
    Command,
    /// Typing into the spell.
    Editing,
    /// Reading the orb's reading of it, rather than the spell itself.
    ///
    /// # The one place a *wrong* reading can be seen
    ///
    /// The file is no longer rewritten when it is saved, which is what a player
    /// asked for and is right — but it took away the only surface that ever
    /// showed what the orb had heard. A resolution that **fails** is reported;
    /// one that succeeds *wrongly* — `the shelf` reaching a different shelf, a
    /// near-miss name landing on its neighbour — has nowhere to show at all.
    ///
    /// So `interpret` is a validation step the player asks for: the buffer, line
    /// for line, as the orb reads it. `<esc>` comes back. It changes nothing and
    /// saves nothing, which is what makes it safe to reach for.
    Reading,
}

/// What the player is editing, if anything.
///
/// Not `Eq`: the settle clock is a float. Comparing two editors was never a
/// thing anything did, and `PartialEq` is enough for the tests that ask whether
/// a keystroke changed the buffer.
#[derive(Debug, Clone, PartialEq)]
pub struct Editor {
    /// The spell's filename, extension included.
    name: String,
    /// The domain it is written for, and runs in.
    ///
    /// On the title bar because a spell that does not say where it runs is a
    /// spell you cannot read — and `grind sage` in a file with no room attached
    /// gives no clue why it resolves.
    domain: String,
    /// The buffer. Always at least one line, so there is somewhere to put the
    /// caret — an empty `Vec` would make every accessor a special case.
    lines: Vec<String>,
    /// Which line the caret is on.
    row: usize,
    /// Which **character** of that line the caret sits before.
    ///
    /// Characters, never bytes. The CP437 work fixed this class of bug once
    /// already; an editor indexing by byte panics on the first pasted `é`.
    column: usize,
    /// The first visible line — see [`scroll_to`](Self::scroll_to).
    top: usize,
    /// Whether anything has changed since the last save.
    dirty: bool,
    /// Which state input goes to.
    mode: Mode,
    /// The word being typed at the command line.
    command: String,
    /// What the editor last refused, and why.
    complaint: Option<Complaint>,
    /// Seconds since the last keystroke, or `None` when there is nothing to save.
    ///
    /// See [`settle`](Self::settle).
    quiet_for: Option<f32>,
    /// Which line the running invocation is on, if one is running.
    ///
    /// Pushed in by the shell each frame rather than pulled: the buffer knows
    /// nothing about the sim, which is what keeps it on this side of the
    /// boundary at all.
    running_line: Option<u64>,
    /// How the orb reads the buffer, line for line.
    ///
    /// Pushed in like [`running_line`](Self::running_line) and for the same
    /// reason: what a line *means* is the sim's decision (rule 2), and a buffer
    /// that worked it out for itself would be a second parser.
    ///
    /// Refreshed when the buffer settles rather than every frame — see
    /// `editing::autosave`. One reading per line, so `reading[i]` belongs to
    /// `lines[i]`; a buffer edited since the last refresh may be one line longer
    /// than its reading, which is why every use of it is indexed rather than
    /// zipped.
    reading: Vec<orbs_sim::Reading>,
    /// Whether the scribing guide is showing beside the buffer.
    guiding: bool,
    /// What the guide is showing, refreshed on the keystroke beat.
    ///
    /// **State, not something the painter works out.** Building it reaches
    /// `scene_at`, which rebuilds every recipe, topic and node in the tower — at
    /// 60 Hz that is the correction `offering` and `editing` have each already
    /// paid for once (*"~59 frames in 60 rebuilt a string identical to the one
    /// already on screen"*). It can only change when a key is pressed, so it is
    /// computed when one is.
    guide: crate::Guide,
    /// A Tab cycle in progress, if the last thing pressed was Tab.
    ///
    /// The prompt keeps one of these too, and the rules that walk it are shared
    /// — see [`tabbing`](crate::tabbing). What is *not* shared is the listing:
    /// the prompt reserves a layout row for `Offered`, and the editor has the
    /// guide pane, which is already showing the same candidates as you type.
    cycle: Option<crate::tabbing::Cycle>,
}

/// How long the player must stop typing before the spell is written out.
///
/// **Long enough to be a pause, short enough to be an edit.** A spell that saves
/// mid-word would reload a running invocation onto a half-typed line; one that
/// waits for a command has made saving a chore in a surface whose whole point is
/// that the orb is reading along with you.
const SETTLE: f32 = 0.6;

/// Something the editor would not do.
///
/// §6 forbids a bare error, so this has a sentence in `prose.toml` that says
/// what to do instead.
///
/// **One variant, and that is the point.** It had a second, `Unsaved`, for
/// quitting with work in the buffer — a state the editor can no longer be in now
/// that it writes itself out after a pause. A refusal nothing can reach is a
/// refusal nobody maintains.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Complaint {
    /// A word the editor does not know.
    Unknown(String),
}

/// What the editor wants the shell to do after a key.
///
/// **There is no `Continue`.** Carrying on is `Option::None` at every call site,
/// and a variant meaning the same thing would be a second way to say it that
/// nothing constructs — the sort of arm a `match` grows and then gets wrong.
///
/// There is no bare `Close` either, for the same reason: closing without saving
/// stopped being reachable when the buffer took over saving itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Write the buffer out, and stay open.
    Save,
    /// Write the buffer out and close.
    SaveAndClose,
}

/// The words the command state answers to, and the ones it shows.
///
/// **Whole words, and a prefix of one will do.** §6's forgiveness does not stop
/// at the editor's door: `e`, `ed`, `edi` all mean `edit`, for the same reason
/// `sur` means `survey` outside. Kept unambiguous in their first letter, so no
/// prefix is ever a coin flip — the property
/// `no_two_editor_words_share_a_first_letter` holds them to.
const WORDS: &[(&str, Word)] = &[
    ("edit", Word::Edit),
    ("guide", Word::Guide),
    ("interpret", Word::Interpret),
    ("quit", Word::Quit),
];

/// The vim shorthand, matched **exactly** and never advertised.
///
/// # An easter egg, and why it is a separate table
///
/// [`WORDS`] is the vocabulary: plain English, prefix-matched, printed on the
/// status row because a player who does not know it has nowhere else to look.
/// These are for the hands that have typed `:wq` ten thousand times and will try
/// it here — finding that it works is a small gift, and not finding it is no
/// loss to anyone else.
///
/// **Exact, not prefixed**, which is what keeps the two tables from interfering:
/// a prefix-matched `w` would shadow nothing today and shadow the first future
/// word beginning with it silently. `no_shorthand_disagrees_with_the_word_it_
/// abbreviates` holds the pair together — `q` reaches `quit` by prefix *and*
/// sits here, and the two must never come to mean different things.
const SHORTHAND: &[(&str, Word)] = &[
    ("w", Word::Save),
    ("q", Word::Quit),
    ("wq", Word::SaveAndQuit),
    ("x", Word::SaveAndQuit),
    // `q!` was `discard`, and there is nothing left to discard: the buffer saves
    // itself as you type. It quits, which is what a vim user means by it once
    // "without saving" has stopped being a thing that can happen.
    ("q!", Word::Quit),
];

/// One thing the command state can be told to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Word {
    Edit,
    /// Show or hide the scribing guide — see [`Editor::guiding`].
    Guide,
    /// Show the buffer as the orb reads it — see [`Mode::Reading`].
    Interpret,
    Quit,
    /// Write it out now rather than waiting for the pause.
    ///
    /// **Not a word, only a shorthand.** Saving happens on its own, so `save`
    /// would be a step the player is asked to remember for something the orb is
    /// already doing — and a spell that runs while you edit it makes *when* it
    /// saves a mechanic rather than a chore. `w` and `wq` still work for the
    /// fingers that reach for them, and mean "now, do not wait for the pause".
    Save,
    SaveAndQuit,
}

impl Editor {
    /// Open `name` on `lines`, in command state.
    #[must_use]
    pub fn open(name: &str, domain: &str, lines: &[String]) -> Self {
        Self {
            name: name.to_owned(),
            domain: domain.to_owned(),
            lines: if lines.is_empty() {
                vec![String::new()]
            } else {
                lines.to_vec()
            },
            row: 0,
            column: 0,
            top: 0,
            dirty: false,
            mode: Mode::default(),
            command: String::new(),
            complaint: None,
            quiet_for: None,
            running_line: None,
            reading: Vec::new(),
            cycle: None,
            // **Open, and that is the whole point of the item.** The language
            // grew past the size one person holds in their head, and a guide
            // nobody knows to ask for helps nobody. `guide` closes it for the
            // hands that no longer need it.
            guiding: true,
            // Replaced by the first `refresh` a frontend does; an empty listing
            // is the honest opening value rather than a world read from a
            // constructor that has no world.
            guide: crate::Guide::Vocabulary {
                control: Vec::new(),
                verbs: Vec::new(),
            },
        }
    }

    /// Let `delta` seconds pass, and say whether the spell should be written now.
    ///
    /// # Why the buffer saves itself
    ///
    /// A spell can be edited **while it runs**, and the orb picks up the change
    /// between steps. That makes saving part of the loop rather than a thing you
    /// finish with — you fix a line and watch the next pass take it — and a
    /// `save` word in the middle of that is a chore standing where the mechanic
    /// should be.
    ///
    /// Debounced rather than per-keystroke because a spell that saved mid-word
    /// would reload a running invocation onto a half-typed line.
    pub fn settle(&mut self, delta: f32) -> bool {
        let Some(quiet) = self.quiet_for.as_mut() else {
            return false;
        };
        *quiet += delta;
        if *quiet < SETTLE {
            return false;
        }
        self.quiet_for = None;
        true
    }

    /// What the running invocation is doing, for the marker.
    #[must_use]
    pub const fn running_line(&self) -> Option<u64> {
        self.running_line
    }

    /// Tell the buffer where a running invocation has reached.
    pub const fn set_running_line(&mut self, line: Option<u64>) {
        self.running_line = line;
    }

    /// Tell the buffer how the orb reads it.
    pub fn set_reading(&mut self, reading: Vec<orbs_sim::Reading>) {
        self.reading = reading;
    }

    /// How the orb reads the line at `index`, if it has read that far.
    #[must_use]
    pub fn reading(&self, index: usize) -> Option<&orbs_sim::Reading> {
        self.reading.get(index)
    }

    /// How many lines the orb cannot read.
    ///
    /// The number on the status row: §14 will not have a mark in a gutter be the
    /// only way to know, and *"one line the orb cannot read"* is the spoken form
    /// of a column of them.
    #[must_use]
    pub fn unread(&self) -> usize {
        self.reading
            .iter()
            .filter(|reading| reading.fault.is_some())
            .count()
    }

    /// The spell being edited.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The domain it runs in.
    #[must_use]
    pub fn domain(&self) -> &str {
        &self.domain
    }

    /// The buffer, for saving and for drawing.
    #[must_use]
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    /// Whether there is unsaved work.
    #[must_use]
    pub const fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Which state input is going to.
    #[must_use]
    pub const fn mode(&self) -> Mode {
        self.mode
    }

    /// The word being typed at the command line.
    #[must_use]
    pub fn command(&self) -> &str {
        &self.command
    }

    /// What the editor last refused.
    #[must_use]
    pub const fn complaint(&self) -> Option<&Complaint> {
        self.complaint.as_ref()
    }

    /// Where the caret is, as (row, column) in characters.
    #[must_use]
    pub const fn caret(&self) -> (usize, usize) {
        (self.row, self.column)
    }

    /// Whether the scribing guide is showing.
    #[must_use]
    pub const fn guiding(&self) -> bool {
        self.guiding
    }

    /// What the guide is showing.
    #[must_use]
    pub const fn guide(&self) -> &crate::Guide {
        &self.guide
    }

    /// Work out what the guide should show, now.
    ///
    /// **Called on the keystroke beat by whoever owns the keyboard**, never from
    /// a painter — see [`Editor::guide`] for the 60 Hz correction this avoids.
    /// Cheap to call redundantly and wrong to call per frame.
    ///
    /// # The caret it asks about is not always the caret on screen
    ///
    /// In [`Mode::Reading`] the pane shows `interpret`'s reading rather than the
    /// buffer, and in [`Mode::Command`] the caret is drawn on the command row
    /// while this one still points into the text. Both would have the guide
    /// describing a word that is not in front of the player, so **the guide only
    /// follows the caret while the buffer has it** — in the other two modes it
    /// falls back to the listing, which is true in every mode.
    pub fn refresh(&mut self, sim: &orbs_sim::Sim) {
        let caret = if self.mode == Mode::Editing {
            (self.row, self.column)
        } else {
            // A position no line has a word at, so `guide` answers with the
            // vocabulary rather than a page about wherever the buffer cursor
            // happens to be resting.
            (usize::MAX, 0)
        };
        self.guide = crate::guide(sim, &self.lines, caret, &self.domain);
    }

    /// Tab: finish the word the caret is on, the way the prompt does.
    ///
    /// # The listing is the guide, and that is why this step is three lines
    ///
    /// The prompt needs `Offered` and a reserved layout row to show what it
    /// found. The editor was going to need the same, and does not: the guide
    /// pane is already showing exactly these candidates, live, as the line is
    /// typed — so a Tab press here has nothing to *say*, only something to
    /// write. Which is the argument for having built the guide first.
    ///
    /// **Editing state only.** In command state the caret is on the editor's own
    /// word — `edit`, `guide`, `interpret`, `quit` — and completing four words
    /// that are listed on screen a row below is help nobody needs.
    pub fn tab(&mut self, sim: &orbs_sim::Sim) {
        if self.mode != Mode::Editing {
            return;
        }
        let Some(line) = self.lines.get(self.row) else {
            return;
        };
        let open = orbs_sim::parser::open_blocks(&self.lines[..self.row]);
        let found = orbs_sim::spell_expect(sim.world(), &self.domain, line, self.column, &open);

        let crate::tabbing::Tabbed::Wrote { replaces, text, .. } =
            crate::tabbing::tab(line, &found, &mut self.cycle)
        else {
            // Nothing to write. The guide is already showing whatever there was.
            return;
        };
        let Some(line) = self.lines.get_mut(self.row) else {
            return;
        };
        line.replace_range(replaces.clone(), &text);
        // **Characters, never bytes** — the caret is a character count and an
        // editor indexing by byte panics on the first pasted `é`.
        let upto = replaces.start.saturating_add(text.len());
        self.column = line
            .get(..upto)
            .map_or(self.column, |before| before.chars().count());
        self.touched();
    }

    /// Abandon any Tab cycle. Anything that is not another Tab ends it.
    pub fn end_cycle(&mut self) {
        self.cycle = None;
    }

    /// The first visible line.
    #[must_use]
    pub const fn top(&self) -> usize {
        self.top
    }

    /// Mark the buffer saved.
    ///
    /// **Disarms the settle clock too.** It cleared only `dirty`, and `settle`
    /// keys off `quiet_for` alone — so an explicit `w` wrote the buffer and then
    /// the debounce wrote it *again* half a second later: two submissions in the
    /// replay stream and two program swaps under a running invocation, for one
    /// save. It was two records in the transcript as well, back when a save said
    /// anything at all — that noise is why saving is silent now (§19).
    pub const fn saved(&mut self) {
        self.dirty = false;
        self.quiet_for = None;
    }

    /// Note that the buffer just changed, restarting the settle clock.
    ///
    /// **The reading goes with it.** `reading[i]` belongs to `lines[i]`, and
    /// inserting or deleting a line shifts every reading below it — so the
    /// danger marks in the gutter paint on the wrong lines, and `interpret`
    /// shows the wrong sentences, until the next settle refreshes them. Half a
    /// second of *no* marks is a pane catching up; half a second of marks
    /// pointing at the wrong line is the pane lying.
    fn touched(&mut self) {
        self.dirty = true;
        self.quiet_for = Some(0.0);
        self.reading.clear();
    }

    /// Keep the caret inside a window `rows` tall.
    ///
    /// Called by the painter, which is the only thing that knows how tall the
    /// pane is. Scrolling lives on this side of the boundary for exactly that
    /// reason — see the module docs.
    pub const fn scroll_to(&mut self, rows: usize) {
        if rows == 0 {
            return;
        }
        if self.row < self.top {
            self.top = self.row;
        } else if self.row >= self.top + rows {
            self.top = self.row + 1 - rows;
        }
    }

    /// The characters of the current line.
    fn current(&self) -> &str {
        self.lines.get(self.row).map_or("", String::as_str)
    }

    /// How many characters the current line has.
    fn width(&self) -> usize {
        self.current().chars().count()
    }

    /// The byte offset of character `column` on the current line.
    fn offset(&self, column: usize) -> usize {
        self.current()
            .char_indices()
            .nth(column)
            .map_or_else(|| self.current().len(), |(at, _)| at)
    }

    /// Take one character of typing, wherever the mode sends it.
    ///
    /// # Why the routing lives here rather than in the key handler
    ///
    /// It used to live there, and both the unit tests and `ORBS_DUMP` reached
    /// past it — the tests by driving the command line directly, the dump by
    /// splitting the script itself. So the one decision neither could see was
    /// the one that was wrong. One function, called by the key handler and the
    /// dump alike, is what makes the state machine reachable from a test.
    pub fn type_text(&mut self, text: &str) {
        // Control characters are not text. Winit reports Enter and Tab with a
        // `text` of "\r" and "\t", and letting either through would put a
        // literal control byte in a spell.
        let text: String = text.chars().filter(|c| !c.is_control()).collect();
        if text.is_empty() {
            return;
        }
        match self.mode {
            Mode::Command => self.command.push_str(&text),
            Mode::Editing => self.insert(&text),
            // **A reading takes no text.** It is a view of the buffer, not a
            // second buffer, and a keystroke that appeared to go somewhere here
            // would be a keystroke the player could not find again.
            Mode::Reading => {}
        }
    }

    /// Type `text` into the buffer at the caret.
    fn insert(&mut self, text: &str) {
        let at = self.offset(self.column);
        self.lines[self.row].insert_str(at, text);
        self.column += text.chars().count();
        // **Only for a line that is a control word.** `end` and `else` belong one
        // level out from the body above them, and a player types them *in* that
        // body — so the line moves as the word completes, the way a `}` does in
        // every editor that indents. A command line is left alone: it is already
        // where `Enter` put it, and a line that jumped while you typed ordinary
        // text would be the editor fighting you.
        if orbs_sim::parser::spell_word(&self.lines[self.row]).is_some() {
            self.reindent();
        }
        self.touched();
    }

    /// What `Enter` means here.
    ///
    /// In the buffer it splits the line, indented to the block it lands in. At
    /// the command line it runs the word.
    pub fn enter(&mut self) -> Option<Outcome> {
        match self.mode {
            Mode::Command => self.run_command(),
            Mode::Editing => {
                let at = self.offset(self.column);
                let tail = self.lines[self.row].split_off(at);
                // **Indented only when the split leaves nothing behind**, which
                // is `Enter` at the end of a line — "start the next one", and
                // the block says where that starts. Four spaces per level,
                // eight two blocks deep, which is the whole point.
                //
                // Breaking a line *mid-text* moves the tail exactly as it was.
                // Prepending an indent there, or trimming what the caret split,
                // would stop `Enter` and `Backspace` being each other's inverse
                // — `enter_splits_the_line_and_backspace_joins_it_again` pins
                // that, and it caught this being written the other way round.
                let next = if tail.trim().is_empty() {
                    INDENT.repeat(self.depths(self.row).1)
                } else {
                    tail
                };
                // After an indent, and before text that moved down — in the
                // first case there is nothing on the line yet to sit before.
                self.column = if next.trim().is_empty() {
                    next.chars().count()
                } else {
                    0
                };
                self.lines.insert(self.row + 1, next);
                self.row += 1;
                self.touched();
                None
            }
            // Nothing to run and nothing to split. `<esc>` is the way out, which
            // the status row says.
            Mode::Reading => None,
        }
    }

    /// The depth the line at `index` prints at, and the depth the next starts at.
    ///
    /// Folded from the top of the buffer every time rather than cached. A spell
    /// is tens of lines, and a cache would be a second answer to a question the
    /// orb already answers on save — which is the disagreement this whole shared
    /// rule exists to prevent.
    fn depths(&self, index: usize) -> (usize, usize) {
        let mut depth = 0;
        let mut here = 0;
        for line in self.lines.iter().take(index + 1) {
            (here, depth) = indent_around(line, depth);
        }
        (here, depth)
    }

    /// How many characters `Backspace` should eat to fall back one level, if the
    /// caret is sitting in a line's leading whitespace.
    ///
    /// `None` when it is not, which is the ordinary one-character case. Falls
    /// back to the **previous multiple** of a level rather than a flat four, so
    /// a hand-spaced line lands on the grid instead of being pushed off it.
    fn outdent(&self) -> Option<usize> {
        let width = INDENT.chars().count();
        if self.column == 0 || width == 0 {
            return None;
        }
        if !self
            .current()
            .chars()
            .take(self.column)
            .all(char::is_whitespace)
        {
            return None;
        }
        Some(match self.column % width {
            0 => width,
            over => over,
        })
    }

    /// Put the current line at the depth its block puts it, keeping the caret
    /// where it sits in the text.
    ///
    /// **What makes `end` snap left as you type it.** Without it a player types
    /// `end` at the body's indent, watches it sit one level too deep, and only
    /// finds out it was right when the orb re-indents it on save — which reads
    /// as the save having moved their work.
    fn reindent(&mut self) {
        let wanted = INDENT.repeat(self.depths(self.row).0);
        let line = &self.lines[self.row];
        let body = line.trim_start();
        if line.len() == wanted.len() + body.len() && line.starts_with(&wanted) {
            return;
        }
        // The caret keeps its place *in the text*, not its column: it is sitting
        // in the word that just caused the shift.
        let into = self
            .column
            .saturating_sub(line.chars().count() - body.chars().count());
        let body = body.to_owned();
        self.column = wanted.chars().count() + into;
        self.lines[self.row] = wanted + &body;
    }

    /// Delete the character before the caret, joining lines at column zero.
    ///
    /// **In the indent it deletes a whole level**, which is the other half of
    /// auto-indent rather than a nicety: `Enter` inside a block leaves the caret
    /// four columns in, and without this getting back out costs the four
    /// keypresses the indent just saved.
    pub fn backspace(&mut self) {
        match self.mode {
            Mode::Command => {
                self.command.pop();
                return;
            }
            // **A reading deletes nothing**, the same rule `type_text` and
            // `enter` already follow: it is a view of the buffer, not a second
            // one. This fell through to the buffer, so a player who opened
            // `interpret` to check the orb's reading and tapped Backspace out of
            // habit deleted a character from a spell they could not see change —
            // and the settle clock wrote the damage out a beat later.
            Mode::Reading => return,
            Mode::Editing => {}
        }
        if let Some(back) = self.outdent() {
            let from = self.offset(self.column - back);
            let to = self.offset(self.column);
            self.lines[self.row].replace_range(from..to, "");
            self.column -= back;
        } else if self.column > 0 {
            let at = self.offset(self.column - 1);
            self.lines[self.row].remove(at);
            self.column -= 1;
        } else if self.row > 0 {
            let line = self.lines.remove(self.row);
            self.row -= 1;
            self.column = self.width();
            self.lines[self.row].push_str(&line);
        } else {
            return;
        }
        self.touched();
    }

    /// `Esc` — leave the buffer for the command line.
    ///
    /// **One meaning, in both states.** In the buffer it steps back to the
    /// command line; at the command line it clears whatever half-word is there.
    /// It never leaves the editor and never discards anything, so a player who
    /// presses it out of habit — or out of not knowing what else to press —
    /// always lands somewhere the words are written down.
    pub fn escape(&mut self) {
        self.mode = Mode::Command;
        self.command.clear();
        self.complaint = None;
    }

    /// Run whatever word is at the command line.
    ///
    /// Unknown words complain rather than doing nothing: §6 forbids a bare
    /// error, and a command line that silently clears is indistinguishable from
    /// one that worked.
    fn run_command(&mut self) -> Option<Outcome> {
        let typed = std::mem::take(&mut self.command);
        let typed = typed.trim().to_lowercase();
        self.complaint = None;
        if typed.is_empty() {
            return None;
        }

        match word(&typed) {
            Some(Word::Edit) => {
                self.mode = Mode::Editing;
                None
            }
            Some(Word::Interpret) => {
                self.mode = Mode::Reading;
                None
            }
            // **A toggle rather than two words.** `guide`/`hide` would be two
            // things to learn for one piece of state a player can see the answer
            // to: the pane is either there or it is not.
            Some(Word::Guide) => {
                self.guiding = !self.guiding;
                None
            }
            Some(Word::Save) => Some(Outcome::Save),
            // **`quit` never refuses now.** It refused on unsaved work, and
            // there is no such state: the buffer writes itself out after a
            // pause, and quitting flushes whatever the pause has not caught yet.
            Some(Word::Quit | Word::SaveAndQuit) => Some(Outcome::SaveAndClose),
            None => {
                self.complaint = Some(Complaint::Unknown(typed));
                None
            }
        }
    }

    /// Move the caret one character left, wrapping to the line above.
    pub fn left(&mut self) {
        if self.column > 0 {
            self.column -= 1;
        } else if self.row > 0 {
            self.row -= 1;
            self.column = self.width();
        }
    }

    /// Move the caret one character right, wrapping to the line below.
    pub fn right(&mut self) {
        if self.column < self.width() {
            self.column += 1;
        } else if self.row + 1 < self.lines.len() {
            self.row += 1;
            self.column = 0;
        }
    }

    /// Move the caret up a line, keeping it inside that line.
    pub fn up(&mut self) {
        if self.row > 0 {
            self.row -= 1;
            self.column = self.column.min(self.width());
        }
    }

    /// Move the caret down a line, keeping it inside that line.
    pub fn down(&mut self) {
        if self.row + 1 < self.lines.len() {
            self.row += 1;
            self.column = self.column.min(self.width());
        }
    }

    /// Caret to the start of the line.
    pub const fn home(&mut self) {
        self.column = 0;
    }

    /// Caret to the end of the line.
    pub fn end(&mut self) {
        self.column = self.width();
    }

    /// The status line's right-hand half: where the caret is.
    #[must_use]
    pub fn position(&self) -> String {
        format!("{}:{}", self.row + 1, self.column + 1)
    }

    /// A line as it is drawn, clipped to `cells`.
    #[must_use]
    pub fn visible(&self, row: usize, cells: u16) -> &str {
        // Through `arriving`, which cuts on a **character** boundary — the same
        // helper the reveal and the POST card use, for the same reason: a byte
        // slice can split `é` into something that is not a `str` and panic.
        self.lines
            .get(row)
            .map_or("", |line| arriving(line, u32::from(cells)))
    }

    /// How many lines the buffer has.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.lines.len()
    }

    /// Whether the buffer has no lines at all.
    ///
    /// **Never true through the editor's own doors**, and that is the point of
    /// saying so rather than leaving it to be assumed: `Editor::open` seeds an
    /// empty spell with one blank line, and `backspace` at the head of the first
    /// line joins nothing. A zero here would mean the buffer was built some
    /// other way, and every `visible` and `position` call reasons about a
    /// caret row that exists.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

/// The word `typed` names.
///
/// **Shorthand first, exactly; then the vocabulary, by prefix.** That order is
/// what lets `wq` mean save-and-quit while `w` alone means save — a single
/// prefix pass over one merged table would resolve `w` and `wq` by whichever
/// happened to be listed first, which is an ordering nobody would think to
/// check.
fn word(typed: &str) -> Option<Word> {
    SHORTHAND
        .iter()
        .find(|(name, _)| *name == typed)
        .or_else(|| WORDS.iter().find(|(name, _)| name.starts_with(typed)))
        .map(|(_, word)| *word)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn editor() -> Editor {
        Editor::open(
            "morning.spell",
            "laboratory",
            &["attend laboratory".to_owned()],
        )
    }

    /// Type a whole word and press Enter.
    fn say(editor: &mut Editor, text: &str) -> Option<Outcome> {
        for character in text.chars() {
            editor.type_text(&character.to_string());
        }
        editor.enter()
    }

    #[test]
    fn the_editor_opens_in_command_state() {
        // **The first keystroke must not be able to damage a spell.** A player
        // who does not yet know what this screen is should be told what the
        // words are, not silently start editing something they thought they were
        // reading.
        let mut editor = editor();
        assert_eq!(editor.mode(), Mode::Command);

        editor.type_text("x");
        assert_eq!(editor.lines()[0], "attend laboratory", "the spell changed");
        assert!(!editor.is_dirty());
        assert_eq!(editor.command(), "x");
    }

    #[test]
    fn edit_drops_into_the_buffer_and_escape_comes_back() {
        let mut editor = editor();
        assert_eq!(say(&mut editor, "edit"), None);
        assert_eq!(editor.mode(), Mode::Editing);

        editor.end();
        editor.type_text("!");
        assert_eq!(editor.lines()[0], "attend laboratory!");
        assert!(editor.is_dirty());

        editor.escape();
        assert_eq!(editor.mode(), Mode::Command);
        editor.type_text("x");
        assert_eq!(
            editor.lines()[0],
            "attend laboratory!",
            "typing after escape reached the buffer",
        );
    }

    #[test]
    fn the_three_words_do_what_they_say() {
        let mut editor = editor();
        assert_eq!(say(&mut editor, "quit"), Some(Outcome::SaveAndClose));
        assert_eq!(say(&mut editor, "edit"), None);
        assert_eq!(editor.mode(), Mode::Editing);
        editor.escape();
        assert_eq!(say(&mut editor, "interpret"), None);
        assert_eq!(editor.mode(), Mode::Reading);
    }

    #[test]
    fn a_reading_is_a_view_and_never_a_second_buffer() {
        // **The failure this exists to catch is silent.** A keystroke that
        // appeared to land in the reading would be a keystroke the player cannot
        // find again — the buffer is what gets saved, and it would not be in it.
        let mut editor = editor();
        say(&mut editor, "edit");
        editor.type_text("grind sage");
        editor.escape();
        let before = editor.lines().to_vec();
        // A clean baseline, so `is_dirty` below is about the reading rather than
        // about the real edit above it.
        editor.saved();

        say(&mut editor, "interpret");
        editor.type_text("this must go nowhere");
        assert_eq!(editor.enter(), None, "the reading ran something");
        assert_eq!(editor.lines(), before, "a keystroke reached the buffer");

        // **Backspace too**, which fell through to the buffer while `type_text`
        // and `enter` did not: a player who opened `interpret` to check the orb's
        // reading and tapped it out of habit deleted a character from a spell
        // they could not see change, and the settle clock wrote the damage out.
        // Twice, because the first would only reach the indent.
        editor.backspace();
        editor.backspace();
        assert_eq!(editor.lines(), before, "backspace edited the buffer");
        assert!(!editor.is_dirty(), "a reading marked the buffer as touched");

        // ...and `esc` is the way out, as the status row says.
        editor.escape();
        assert_eq!(editor.mode(), Mode::Command);
    }

    #[test]
    fn editing_drops_the_reading_rather_than_pointing_it_at_the_wrong_lines() {
        // `reading[i]` belongs to `lines[i]`, and inserting a line shifts every
        // reading below it. Half a second of *no* marks is a pane catching up;
        // half a second of marks on the wrong lines is the pane lying, and the
        // player is looking straight at it while they type.
        let mut editor = editor();
        say(&mut editor, "edit");
        editor.type_text("grind sage");
        editor.set_reading(vec![reading(1, "grind sage", Some("spell_missing"))]);
        assert_eq!(editor.unread(), 1);

        editor.enter();
        editor.type_text("empty mortar_and_pestle");
        assert_eq!(
            editor.unread(),
            0,
            "a stale reading survived the line it described moving",
        );
        assert!(editor.reading(0).is_none(), "the marks outlived the edit");
    }

    #[test]
    fn the_count_of_unread_lines_is_the_readings_and_nothing_else() {
        // The number the status row says, and the one §14 needs because a mark
        // on a line is otherwise carried by colour alone.
        let mut editor = editor();
        assert_eq!(editor.unread(), 0, "an unread buffer reported faults");

        editor.set_reading(vec![
            reading(1, "grind sage", None),
            reading(2, "xyzzy plugh", Some("spell_missing")),
            reading(3, "if mortr is idle", Some("spell_nowhere")),
        ]);
        assert_eq!(editor.unread(), 2);
        assert_eq!(
            editor.reading(0).map(|read| read.heard.as_str()),
            Some("grind sage")
        );
        assert!(editor.reading(9).is_none(), "it read past the end");
    }

    /// One line of a reading, as the sim would hand it over.
    fn reading(line: usize, heard: &str, fault: Option<&'static str>) -> orbs_sim::Reading {
        orbs_sim::Reading {
            line,
            heard: heard.to_owned(),
            // Nothing read these fixtures, so the line is the player's own.
            was: None,
            fault: fault.map(|key| orbs_sim::Fault { key, detail: None }),
        }
    }

    #[test]
    fn any_unambiguous_prefix_will_do() {
        // §6's forgiveness does not stop at the editor's door. `sur` means
        // `survey` outside it; `e` means `edit` inside.
        for (typed, expected) in [
            ("e", None),
            ("ed", None),
            ("q", Some(Outcome::SaveAndClose)),
            ("qui", Some(Outcome::SaveAndClose)),
        ] {
            let mut editor = editor();
            assert_eq!(say(&mut editor, typed), expected, "{typed:?}");
        }
    }

    #[test]
    fn the_vim_shorthand_is_there_for_the_hands_that_know_it() {
        // An easter egg, not vocabulary: never printed on the status row, and
        // nothing else in the editor mentions it.
        for (typed, expected) in [
            ("w", Some(Outcome::Save)),
            ("wq", Some(Outcome::SaveAndClose)),
            ("x", Some(Outcome::SaveAndClose)),
            ("q", Some(Outcome::SaveAndClose)),
            // Kept for the fingers that reach for it, and it can no longer mean
            // "without saving" — there is no unsaved state to leave behind.
            ("q!", Some(Outcome::SaveAndClose)),
        ] {
            let mut editor = editor();
            assert_eq!(say(&mut editor, typed), expected, "{typed:?}");
        }
    }

    #[test]
    fn no_shorthand_disagrees_with_the_word_it_abbreviates() {
        // **The invariant holding two tables together.** `q` is both a shorthand
        // and a prefix of `quit`; they must never come to mean different things,
        // or the same keystroke would do one thing today and another after
        // someone reorders a list.
        //
        // Checked through `word` rather than by comparing the tables, because
        // `word` is what the editor actually calls — a test that agreed with the
        // tables and disagreed with the lookup would prove nothing.
        for (typed, shorthand) in SHORTHAND {
            let Some(by_prefix) = WORDS
                .iter()
                .find(|(name, _)| name.starts_with(typed))
                .map(|(_, word)| *word)
            else {
                continue;
            };
            assert_eq!(
                word(typed),
                Some(*shorthand),
                "{typed:?} resolves to something other than its shorthand",
            );
            assert_eq!(
                by_prefix, *shorthand,
                "{typed:?} means one thing as a shorthand and another as a prefix",
            );
        }
    }

    #[test]
    fn a_longer_shorthand_is_not_shadowed_by_a_shorter_one() {
        // `w` and `wq` differ by one character and mean different things. A
        // single prefix pass over one merged table would resolve them by
        // whichever was listed first — an ordering nobody would think to check,
        // and a save-and-quit that silently only saved.
        let mut editor = editor();
        assert_eq!(say(&mut editor, "w"), Some(Outcome::Save));
        assert_eq!(say(&mut editor, "wq"), Some(Outcome::SaveAndClose));
    }

    #[test]
    fn the_status_row_names_every_word_and_never_the_easter_egg() {
        // **Read out of `prose.toml`, not out of a literal here.** A copy of the
        // row in the test is a copy that goes stale silently: this test held the
        // string `"edit  save  quit  discard"` for two words that no longer
        // exist, and passed the whole time.
        //
        // Two properties, and they pull opposite ways — which is why they are
        // one test. The row is the entire interface in command state, so a word
        // missing from it is a word with nowhere to be discovered; and the
        // shorthand stops being an easter egg the moment it is printed.
        let shown = orbs_sim::Prose::builtin().line("editor_words", &[]);
        let listed: Vec<&str> = shown.split_whitespace().collect();

        for (name, _) in WORDS {
            assert!(listed.contains(name), "{name:?} is not on the status row");
        }
        for (typed, _) in SHORTHAND {
            assert!(!listed.contains(typed), "{typed:?} is on the status row");
        }
    }

    #[test]
    fn no_two_editor_words_share_a_first_letter() {
        // What makes single-letter prefixes safe rather than a coin flip. The
        // naming pass holds the game's verbs to the same rule; this is that rule
        // applied to the four words inside the editor.
        let mut initials: Vec<char> = WORDS
            .iter()
            .filter_map(|(name, _)| name.chars().next())
            .collect();
        let count = initials.len();
        initials.sort_unstable();
        initials.dedup();
        assert_eq!(initials.len(), count, "two editor words start alike");
    }

    #[test]
    fn typing_settles_into_a_save_and_a_keystroke_pushes_it_back() {
        // **The whole point of the debounce.** A buffer that saved on every
        // keystroke would hand a running invocation a half-typed line to reload
        // from, and one that never saved on its own would put a chore in the
        // middle of the loop this surface exists for.
        let mut editor = editor();
        say(&mut editor, "edit");

        assert!(
            !editor.settle(10.0),
            "an untouched buffer asked to be saved"
        );

        editor.type_text("x");
        assert!(!editor.settle(SETTLE / 2.0), "saved mid-word");

        // The keystroke that lands inside the pause restarts it, rather than
        // topping up a clock that was already most of the way to firing.
        editor.type_text("y");
        assert!(
            !editor.settle(SETTLE / 2.0),
            "the keystroke did not push it back"
        );

        assert!(editor.settle(SETTLE), "the pause never produced a save");
        editor.saved();

        // And it fires **once** — a settle that stayed armed would rewrite the
        // spell every frame, reloading a running invocation each time.
        assert!(!editor.settle(10.0), "settled twice on one edit");
    }

    #[test]
    fn quitting_writes_the_buffer_out_rather_than_refusing() {
        // `quit` used to refuse on unsaved work, with `discard` as the way out.
        // Neither exists now: there is no reachable unsaved state to refuse for,
        // and quitting flushes whatever the pause has not caught yet. The
        // failure this guards is a `quit` that closes on the last few keystrokes
        // typed before it.
        let mut editor = editor();
        say(&mut editor, "edit");
        editor.type_text("x");
        editor.escape();

        assert!(editor.is_dirty());
        assert_eq!(say(&mut editor, "quit"), Some(Outcome::SaveAndClose));
    }

    #[test]
    fn the_running_line_is_said_as_well_as_marked() {
        // §14: a `»` in a gutter column is exactly the kind of visual-only fact
        // that is forbidden, so the title says it too — and `Painter::border`
        // pushes a title to the speech stream. This holds the sentence's
        // existence; `sheet.rs` holds where it is drawn.
        let mut editor = editor();
        assert_eq!(editor.running_line(), None);
        editor.set_running_line(Some(2));
        assert_eq!(editor.running_line(), Some(2));

        let said = orbs_sim::Prose::builtin().line("editor_at_line", &[("count", "2")]);
        assert!(said.contains('2'), "the line number is not in the sentence");
        assert_ne!(
            said, "editor_at_line",
            "no prose is authored for the marker"
        );
    }

    #[test]
    fn an_unknown_word_complains_rather_than_doing_nothing() {
        let mut editor = editor();
        assert_eq!(say(&mut editor, "frobnicate"), None);
        assert_eq!(
            editor.complaint(),
            Some(&Complaint::Unknown("frobnicate".to_owned())),
        );
    }

    #[test]
    fn escape_never_leaves_the_editor_and_never_discards() {
        // **One meaning in both states**, so a player pressing it out of habit —
        // or out of not knowing what else to press — always lands where the
        // words are written down.
        let mut editor = editor();
        say(&mut editor, "edit");
        editor.type_text("x");
        editor.escape();
        assert_eq!(editor.mode(), Mode::Command);
        assert!(editor.is_dirty(), "escape discarded the edit");

        editor.type_text("qui");
        editor.escape();
        assert_eq!(editor.command(), "", "escape left a half-word behind");
        assert_eq!(editor.mode(), Mode::Command);
    }

    #[test]
    fn an_empty_spell_still_has_somewhere_to_put_the_caret() {
        let editor = Editor::open("new.spell", "laboratory", &[]);
        assert_eq!(editor.len(), 1);
        assert_eq!(editor.caret(), (0, 0));
    }

    #[test]
    fn the_caret_counts_characters_rather_than_bytes() {
        // The bug class the CP437 work already fixed once. A byte-indexed editor
        // panics the first time someone pastes a non-ASCII character.
        let mut editor = Editor::open("x.spell", "laboratory", &["héllo".to_owned()]);
        say(&mut editor, "edit");
        editor.end();
        assert_eq!(editor.caret(), (0, 5), "5 characters, not 6 bytes");
        editor.backspace();
        assert_eq!(editor.lines()[0], "héll");
        editor.home();
        editor.right();
        editor.right();
        editor.type_text("X");
        assert_eq!(editor.lines()[0], "héXll");
    }

    #[test]
    fn control_characters_never_reach_the_buffer() {
        // Winit reports Enter as a `text` of "\r" and Tab as "\t". Either in a
        // spell would be a control byte in a file a player reads back.
        let mut editor = editor();
        say(&mut editor, "edit");
        editor.type_text("\r");
        editor.type_text("\t");
        assert_eq!(editor.lines()[0], "attend laboratory");
        assert!(
            !editor.is_dirty(),
            "a filtered keystroke dirtied the buffer"
        );
    }

    /// Type a whole spell into an empty buffer, `Enter` between lines.
    ///
    /// **No spaces typed anywhere**, which is the property under test: whatever
    /// indentation comes out was the editor's doing.
    fn typed(script: &[&str]) -> Editor {
        let mut editor = Editor::open("x.spell", "laboratory", &[]);
        say(&mut editor, "edit");
        for (at, line) in script.iter().enumerate() {
            if at > 0 {
                editor.enter();
            }
            editor.type_text(line);
        }
        editor
    }

    #[test]
    fn a_block_indents_its_body_as_you_type_it() {
        // The complaint this answers: four spacebar presses per line inside a
        // block, eight two blocks deep, on every line of every spell.
        let editor = typed(&[
            "repeat 2",
            "kindle charcoal",
            "if the mortar is idle",
            "grind sage",
            "end",
            "empty mortar_and_pestle",
            "end",
        ]);
        assert_eq!(
            editor.lines(),
            [
                "repeat 2",
                "    kindle charcoal",
                "    if the mortar is idle",
                "        grind sage",
                "    end",
                "    empty mortar_and_pestle",
                "end",
            ],
        );
    }

    #[test]
    fn end_and_else_step_back_out_as_the_word_completes() {
        // A player types `end` *in* the body, so the line has to move when the
        // word lands — the way a `}` does in every editor that indents. Without
        // it the line sits one level too deep until the orb re-indents it on
        // save, which reads as the save having moved their work.
        let mut editor = typed(&["repeat", "kindle charcoal"]);
        editor.enter();
        assert_eq!(editor.caret(), (2, 4), "the new line did not open indented");

        editor.type_text("en");
        assert_eq!(editor.lines()[2], "    en", "a half-typed word jumped");
        editor.type_text("d");
        assert_eq!(editor.lines()[2], "end", "`end` did not step back out");
        assert_eq!(editor.caret(), (2, 3), "the caret left the word it was in");

        // ...and `else` hinges: out for itself, in again for what follows.
        let editor = typed(&[
            "if the mortar is idle",
            "grind sage",
            "else",
            "kindle charcoal",
        ]);
        assert_eq!(
            editor.lines(),
            [
                "if the mortar is idle",
                "    grind sage",
                "else",
                "    kindle charcoal",
            ],
        );
    }

    #[test]
    fn backspace_in_the_indent_falls_back_a_whole_level() {
        // The other half of auto-indent rather than a nicety. `Enter` inside a
        // block leaves the caret four columns in, and without this getting out
        // costs the four keypresses the indent just saved.
        let mut editor = typed(&["repeat", "if the mortar is idle"]);
        editor.enter();
        assert_eq!(editor.caret(), (2, 8), "two blocks deep is eight columns");

        editor.backspace();
        assert_eq!(editor.caret(), (2, 4));
        editor.backspace();
        assert_eq!(editor.caret(), (2, 0));
        assert_eq!(editor.lines()[2], "");

        // At column zero it joins lines again, exactly as it always did.
        editor.backspace();
        assert_eq!(editor.lines().len(), 2, "backspace stopped joining lines");
    }

    #[test]
    fn backspace_lands_a_hand_spaced_line_on_the_grid() {
        // A line spaced by hand to a column that is not a multiple of a level
        // falls back **to** the grid rather than being pushed off it — one press
        // gets you somewhere the next press behaves predictably from.
        let mut editor = Editor::open("x.spell", "laboratory", &["      grind sage".to_owned()]);
        say(&mut editor, "edit");
        editor.home();
        for _ in 0..6 {
            editor.right();
        }
        editor.backspace();
        assert_eq!(editor.caret(), (0, 4), "did not fall back to a level");
        editor.backspace();
        assert_eq!(editor.caret(), (0, 0));
    }

    #[test]
    fn what_the_editor_indents_is_what_the_orb_writes_down() {
        // **The invariant the shared rule exists for.** The buffer indents as
        // you type and the orb re-indents when it saves; if those two disagreed,
        // every save would look like it had moved your work.
        //
        // Checked through `parser::indent_around` — the function both fold —
        // rather than by comparing two hand-written expectations, which would
        // agree with each other and with neither implementation.
        let editor = typed(&[
            "repeat 2",
            "kindle charcoal",
            "if the mortar is idle",
            "grind sage",
            "else",
            "empty mortar_and_pestle",
            "end",
            "end",
        ]);
        let mut depth = 0;
        for (at, line) in editor.lines().iter().enumerate() {
            let here;
            (here, depth) = orbs_sim::parser::indent_around(line, depth);
            assert_eq!(
                line.chars().take_while(|c| *c == ' ').count(),
                here * orbs_sim::parser::INDENT.len(),
                "line {} sits where the orb would not put it: {line:?}",
                at + 1,
            );
        }
    }

    #[test]
    fn enter_splits_the_line_and_backspace_joins_it_again() {
        let mut editor = editor();
        say(&mut editor, "edit");
        editor.home();
        for _ in 0..6 {
            editor.right();
        }
        editor.enter();
        assert_eq!(editor.lines(), ["attend", " laboratory"]);
        assert_eq!(editor.caret(), (1, 0));

        editor.backspace();
        assert_eq!(editor.lines(), ["attend laboratory"]);
        assert_eq!(editor.caret(), (0, 6));
    }

    #[test]
    fn the_caret_wraps_between_lines_rather_than_stopping() {
        let mut editor = Editor::open("x.spell", "laboratory", &["ab".to_owned(), "cd".to_owned()]);
        say(&mut editor, "edit");
        editor.end();
        editor.right();
        assert_eq!(editor.caret(), (1, 0), "right at end of line should wrap");
        editor.left();
        assert_eq!(editor.caret(), (0, 2), "left at column 0 should wrap back");
    }

    #[test]
    fn moving_up_into_a_shorter_line_keeps_the_caret_inside_it() {
        let mut editor = Editor::open(
            "x.spell",
            "laboratory",
            &["ab".to_owned(), "cdefgh".to_owned()],
        );
        say(&mut editor, "edit");
        editor.down();
        editor.end();
        editor.up();
        assert_eq!(editor.caret(), (0, 2), "the caret escaped the shorter line");
    }

    #[test]
    fn scrolling_follows_the_caret_in_both_directions() {
        let lines: Vec<String> = (0..20).map(|n| format!("line {n}")).collect();
        let mut editor = Editor::open("long.spell", "laboratory", &lines);
        say(&mut editor, "edit");

        for _ in 0..12 {
            editor.down();
        }
        editor.scroll_to(5);
        assert!(
            editor.top() <= 12 && 12 < editor.top() + 5,
            "caret off-screen"
        );

        for _ in 0..12 {
            editor.up();
        }
        editor.scroll_to(5);
        assert_eq!(
            editor.top(),
            0,
            "scrolling back up did not follow the caret"
        );
    }
}
