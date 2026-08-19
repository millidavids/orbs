//! The weave screen's state: what is being looked at, and where the cursor is.
//!
//! Knows nothing about Bevy and nothing about panes — the wiring is in
//! `weaving.rs` and the drawing is in `loom.rs`, the same three-way split the
//! editor makes for the same reason: what a key *means* here and how a pane is
//! laid out are different concerns that change for different reasons.
//!
//! # It opens in command state, and the way in is a word
//!
//! §19 records why the editor does: *"the first keystroke cannot damage
//! anything."* Taking a Mastery node is irreversible — it closes its tier — so
//! the same rule applies with more force.
//!
//! **`ley` and `mastery` go into a track**, place the aim on its first node and
//! hand the arrows over; `<esc>` comes back. That is `edit` dropping into the
//! editor's buffer. An arrow pressed at the command line does nothing at all, on
//! purpose: a screen where the arrows are sometimes navigation and sometimes
//! nothing, depending on what you last typed, answers differently to the same
//! key — and *"my first key press was being ignored"* is how the version that
//! let them work immediately was reported.
//!
//! In this version nothing is takeable at all: every authored node is a marker
//! and `take` answers with an authored line. The state machine is built for the
//! choice anyway, because the first real node should change a content file and a
//! prose key rather than the shape of this.
//!
//! # The cursor is an identity, never an index
//!
//! The world ticks while the screen is open. Crossing a threshold opens a tier
//! and changes what is in the list, so an index would silently come to point at
//! a different node — and a view that survives its subject lies, which is the
//! rule §19 extracted from `Editor::reading`. The cursor holds an id and
//! unplaces itself when that id is no longer on screen.

use orbs_sim::{Node, Standing};

/// Which track is being looked at.
///
/// **Both are always drawn.** This decides where the cursor may go and which
/// heading is emphasised, not what is visible — a player who cannot see Mastery
/// until they ask for it cannot discover that it is there.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Track {
    /// The straight path. Nothing here is ever chosen.
    #[default]
    LeyLine,
    /// The branching tree, where a tier gives one of its nodes.
    Mastery,
}

/// Which state the screen is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// Typing words at the screen. The state it opens in.
    #[default]
    Command,
    /// Walking the nodes with the arrows.
    Browsing,
}

/// A word the screen answers to.
///
/// **Prefix-matched, and no two share a first letter** — the same rule the
/// editor's vocabulary follows, so `l`, `m`, `t` and `q` all work and the
/// property is a test rather than a convention.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Word {
    /// Look at the Ley Line.
    Ley,
    /// Look at Mastery.
    Mastery,
    /// Take what the cursor is on.
    Take,
    /// Leave.
    Quit,
}

/// Every word, longest form first for the listing.
pub const WORDS: [(&str, Word); 4] = [
    ("ley", Word::Ley),
    ("mastery", Word::Mastery),
    ("take", Word::Take),
    ("quit", Word::Quit),
];

/// Move `at` by `by` within `len`, stopping at the ends rather than wrapping.
///
/// **A track is read left to right**, so wrapping would make its start and its
/// finish the same place — which on a progression is the one thing the picture
/// must not say.
fn shift(at: usize, by: i8, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    let step = usize::from(by.unsigned_abs());
    if by < 0 {
        at.saturating_sub(step)
    } else {
        (at + step).min(len - 1)
    }
}

/// The word `typed` names, by unambiguous prefix.
fn word(typed: &str) -> Option<Word> {
    WORDS
        .iter()
        .find(|(name, _)| name.starts_with(typed))
        .map(|(_, word)| *word)
}

/// What the screen would like the shell to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Close it.
    Close,
}

/// Something the screen would not do.
///
/// §6 forbids a bare error, so every one of these has a sentence in
/// `prose.toml` naming what to do instead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Complaint {
    /// A word the screen does not know.
    Unknown(String),
    /// `take` with the cursor unplaced.
    Nothing,
    /// `take` on something the tower already has.
    ///
    /// **Not a marker and not a refusal to be sorry about.** A Ley Line step is
    /// taken by being passed, so `take` on one is a player asking for something
    /// they are already holding — and the first version answered *"nothing is
    /// behind it yet"* about the one grant in the game that certainly does
    /// something, one row under a details panel saying `active`.
    Already(String),
    /// `take` on a node nothing is behind yet.
    NothingBehind(String),
    /// `take` on a node whose tier has not opened.
    Locked(String, u64),
    /// `take` on a tier that has already given its one.
    Spent(String),
}

/// The weave screen.
#[derive(Debug, Default)]
pub struct Tapestry {
    /// Which track the cursor walks.
    track: Track,
    /// Command or browsing.
    mode: Mode,
    /// The half-typed word.
    command: String,
    /// The node the cursor is on, by id. `None` until an arrow is pressed.
    cursor: Option<String>,
    /// What the last word was refused for.
    complaint: Option<Complaint>,
    /// The tower's total, pushed in.
    experience: u64,
    /// The Ley Line, pushed in.
    ley_line: Vec<Node>,
    /// Mastery's tiers, pushed in.
    mastery: Vec<Vec<Node>>,
}

impl Tapestry {
    /// Which track is being walked.
    #[must_use]
    pub const fn track(&self) -> Track {
        self.track
    }

    /// Command or browsing.
    #[must_use]
    pub const fn mode(&self) -> Mode {
        self.mode
    }

    /// The half-typed word.
    #[must_use]
    pub fn command(&self) -> &str {
        &self.command
    }

    /// The node the cursor is on, if it has been placed.
    #[must_use]
    pub fn cursor(&self) -> Option<&str> {
        self.cursor.as_deref()
    }

    /// What the last word was refused for.
    #[must_use]
    pub const fn complaint(&self) -> Option<&Complaint> {
        self.complaint.as_ref()
    }

    /// The tower's total.
    #[must_use]
    pub const fn experience(&self) -> u64 {
        self.experience
    }

    /// Mastery's tiers, in order.
    #[must_use]
    pub fn mastery(&self) -> &[Vec<Node>] {
        &self.mastery
    }

    /// The Ley Line as columns of one, so both tracks draw through one function.
    ///
    /// A step has no siblings — there is nothing to choose between on the Ley
    /// Line — so its column is one node tall, and that *is* the difference
    /// between the two tracks rather than a special case in the painter.
    #[must_use]
    pub fn ley_line_columns(&self) -> Vec<Vec<Node>> {
        self.ley_line
            .iter()
            .map(|node| vec![node.clone()])
            .collect()
    }

    /// The node the cursor is on, if it has been placed.
    #[must_use]
    pub fn aimed(&self) -> Option<Node> {
        let id = self.cursor.as_ref()?;
        self.walkable().into_iter().find(|node| &node.id == id)
    }

    /// Take a fresh reading of both tracks.
    ///
    /// **Pushed in rather than pulled**, because the screen knows nothing about
    /// the sim — the same direction `Editor::set_reading` runs in. Called every
    /// frame the screen is open, because the world ticks behind it and a
    /// threshold crossed while a player is looking should land while they are
    /// looking.
    pub fn refresh(&mut self, experience: u64, ley_line: Vec<Node>, mastery: Vec<Vec<Node>>) {
        self.experience = experience;
        self.ley_line = ley_line;
        self.mastery = mastery;
        // **The cursor is checked against the new reading, not carried over.**
        // A tier opening changes what is on screen, and an id that is no longer
        // there would leave the highlight on a row that has moved.
        if self
            .cursor
            .as_ref()
            .is_some_and(|id| !self.walkable().iter().any(|node| &node.id == id))
        {
            self.cursor = None;
        }
    }

    /// The track being walked, as **columns left to right**.
    ///
    /// **One shape for both tracks**, which is what makes the cursor's movement
    /// one piece of arithmetic rather than two. A column is a step of the Ley
    /// Line or a tier of Mastery — progression runs rightward in both — and the
    /// rows within a column are the choice, which the Ley Line never has and
    /// Mastery always does. So the Ley Line is columns of one.
    #[must_use]
    pub fn columns(&self) -> Vec<Vec<Node>> {
        match self.track {
            Track::LeyLine => self
                .ley_line
                .iter()
                .map(|node| vec![node.clone()])
                .collect(),
            Track::Mastery => self.mastery.clone(),
        }
    }

    /// Every node the cursor may sit on, in reading order.
    fn walkable(&self) -> Vec<Node> {
        self.columns().into_iter().flatten().collect()
    }

    /// Where the cursor is, as `(column, row)`.
    fn at(&self) -> Option<(usize, usize)> {
        let id = self.cursor.as_ref()?;
        self.columns()
            .iter()
            .enumerate()
            .find_map(|(column, rows)| {
                rows.iter()
                    .position(|node| &node.id == id)
                    .map(|row| (column, row))
            })
    }

    /// Type into the command line.
    pub fn type_text(&mut self, text: &str) {
        // Control bytes never reach a word. `Enter` and `Escape` arrive with
        // their own text on some platforms, and letting either through would put
        // a control byte in the buffer the prefix match runs over.
        let text: String = text.chars().filter(|c| !c.is_control()).collect();
        if text.is_empty() {
            return;
        }
        // **A letter always goes somewhere the player can see it.** Browsing
        // swallowing text would be a dead end of exactly the kind §6 forbids —
        // typing `take` while aiming would do nothing, with nothing saying why —
        // so a printable character steps back to the command line and starts the
        // word there. **The cursor is kept**, which is the point: aim with the
        // arrows, then type `take`, and it acts on what you aimed at.
        //
        // Safe, because typing *leaves* browsing: only `Enter` in browsing
        // commits, and a letter can no longer be the keystroke before it.
        self.mode = Mode::Command;
        self.command.push_str(&text);
    }

    /// Delete the character before the caret.
    pub fn backspace(&mut self) {
        match self.mode {
            Mode::Command => {
                self.command.pop();
            }
            Mode::Browsing => {}
        }
    }

    /// `Esc` — step back toward the command line.
    ///
    /// **One meaning in both states**, as it has in the editor: browsing steps
    /// back to the command line, and the command line clears its half-word. It
    /// never closes the screen, so a player who presses it out of habit always
    /// lands somewhere the words are written down.
    pub fn escape(&mut self) {
        self.mode = Mode::Command;
        self.command.clear();
        self.complaint = None;
    }

    /// `Enter` — run the word, or take what the cursor is on.
    pub fn enter(&mut self) -> Option<Outcome> {
        match self.mode {
            Mode::Command => self.run_command(),
            Mode::Browsing => {
                self.take();
                None
            }
        }
    }

    /// Move the cursor by one column (`across`) and one row (`down`).
    ///
    /// **Two axes, because the picture has two.** Progression runs rightward and
    /// a tier's choice runs downward, so left/right walks the track and up/down
    /// picks between siblings — which on the Ley Line, whose columns are one node
    /// tall, does nothing at all. That is the right nothing: there is no choice
    /// on the Ley Line, and an arrow that appeared to pick between steps would be
    /// offering one.
    ///
    /// **The first arrow places rather than moves.** Until one is pressed the
    /// cursor is nowhere, which is what keeps `Enter` from committing anything
    /// before the player has aimed — §19's *"the first keystroke cannot damage
    /// anything"*, carried into a screen where the damage is irreversible.
    pub fn step(&mut self, across: i8, down: i8) {
        // **The command line has the keys until a word hands them over**, which
        // is the editor's shape exactly: `edit` drops into the buffer and `<esc>`
        // comes back, and here `ley` and `mastery` drop into a track. An arrow
        // pressed at the command line does nothing, on purpose — a screen where
        // the arrows are sometimes navigation and sometimes nothing depending on
        // what you last typed is a screen that answers differently to the same
        // key.
        if self.mode != Mode::Browsing {
            return;
        }
        let columns = self.columns();
        if columns.iter().all(Vec::is_empty) {
            return;
        }
        self.complaint = None;

        // **An arrow with nothing aimed at places the aim**, rather than doing
        // nothing. `look` normally places it, so this is unreachable by typing —
        // but `refresh` unplaces the cursor when the node under it stops being
        // drawn, and that can happen while browsing, on a tick the player did not
        // ask for. Returning early there left every arrow inert while the status
        // row still read `arrows move`, with retyping the track word the only way
        // out: a dead end §6 forbids, arrived at without touching the keyboard.
        let Some((column, row)) = self.at() else {
            let first = columns.iter().flatten().next().map(|node| node.id.clone());
            self.cursor = first;
            return;
        };

        // **Stops at the ends rather than wrapping.** A track is read left to
        // right, and wrapping would make its start and its finish the same place
        // — which on a progression is the one thing the picture must not say.
        let column = shift(column, across, columns.len());
        let rows = &columns[column];
        if rows.is_empty() {
            return;
        }
        // Clamped, not remembered: columns may differ in height, and a cursor
        // carrying a row index past the end of a shorter one would vanish.
        let row = shift(row.min(rows.len() - 1), down, rows.len());
        self.cursor = Some(rows[row].id.clone());
    }

    /// Go into a track: aim at its first node and hand the arrows over.
    ///
    /// **This is `edit` dropping into the buffer.** A word is the way in, so
    /// there is exactly one state in which an arrow key means anything and the
    /// player got there by saying so. `<esc>` comes back, as it does everywhere.
    ///
    /// It places the cursor rather than leaving it nowhere, because a browse
    /// mode with nothing aimed at would be a mode whose only visible difference
    /// is that the words stopped working.
    fn look(&mut self, track: Track) {
        self.track = track;
        self.mode = Mode::Browsing;
        self.complaint = None;
        self.cursor = self
            .columns()
            .into_iter()
            .flatten()
            .next()
            .map(|node| node.id);
    }

    /// Run whatever word is at the command line.
    fn run_command(&mut self) -> Option<Outcome> {
        let typed = std::mem::take(&mut self.command);
        let typed = typed.trim().to_lowercase();
        self.complaint = None;
        if typed.is_empty() {
            return None;
        }
        match word(&typed) {
            Some(Word::Ley) => {
                self.look(Track::LeyLine);
                None
            }
            Some(Word::Mastery) => {
                self.look(Track::Mastery);
                None
            }
            Some(Word::Take) => {
                self.take();
                None
            }
            Some(Word::Quit) => Some(Outcome::Close),
            None => {
                self.complaint = Some(Complaint::Unknown(typed));
                None
            }
        }
    }

    /// Take what the cursor is on — which today is always a refusal.
    ///
    /// **Every branch is authored**, including the one that cannot be reached
    /// yet: §6 forbids a bare error, and a screen whose central verb answered
    /// with silence would be worse than one that had no verb. The first real
    /// node replaces `NothingBehind` with a grant and leaves the rest standing.
    fn take(&mut self) {
        let Some(node) = self.aimed() else {
            self.complaint = Some(Complaint::Nothing);
            return;
        };
        self.complaint = Some(match node.standing {
            // **Held already, which is not the same as empty.** A Ley Line step
            // is taken by being passed, so this is the one branch that is about
            // something the tower really has.
            Standing::Taken => Complaint::Already(node.id),
            // Open, and nothing is behind any of them yet.
            Standing::Open => Complaint::NothingBehind(node.id),
            // `unlocked` rather than a total comparison: the two are the same
            // arithmetic today, and only the field stays right when a tier is
            // shut because a sibling took its one choice.
            Standing::Locked if node.unlocked => Complaint::Spent(node.id),
            Standing::Locked => Complaint::Locked(node.id, node.at),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One node, as the sim would report it at 24 experience.
    fn node(id: &str, at: u64, standing: Standing) -> Node {
        Node {
            id: id.to_owned(),
            at,
            standing,
            unlocked: at <= 24,
        }
    }

    /// A tapestry with one taken ley-line step and one open tier.
    fn tapestry() -> Tapestry {
        let mut screen = Tapestry::default();
        screen.refresh(
            24,
            vec![node("concentration", 16, Standing::Taken)],
            vec![
                vec![
                    node("tbi_a", 24, Standing::Open),
                    node("tbi_b", 24, Standing::Open),
                ],
                vec![node("tbi_c", 40, Standing::Locked)],
            ],
        );
        screen
    }

    /// Type a word and press Enter.
    fn say(screen: &mut Tapestry, text: &str) -> Option<Outcome> {
        screen.type_text(text);
        screen.enter()
    }

    #[test]
    fn it_opens_on_the_ley_line_in_command_state_with_no_cursor() {
        let screen = tapestry();
        assert_eq!(screen.mode(), Mode::Command);
        assert_eq!(screen.track(), Track::LeyLine);
        assert_eq!(screen.cursor(), None, "something was already pointed at");
    }

    #[test]
    fn enter_before_any_arrow_takes_nothing() {
        // **The property the unplaced cursor exists for.** Taking a Mastery node
        // closes its tier and cannot be undone, so the first keystroke must not
        // be able to reach it — §19's rule for the editor, where the damage was
        // only a lost line.
        let mut screen = tapestry();
        assert_eq!(screen.enter(), None);
        assert_eq!(screen.cursor(), None);
        assert_eq!(screen.complaint(), None, "an empty line complained");
    }

    #[test]
    fn an_arrow_at_the_command_line_does_nothing_at_all() {
        // **The way in is a word**, as `edit` is for the editor's buffer. This is
        // also the bug it was reported as: the arrows appeared to do nothing on a
        // screen that had just opened, because the screen had the keyboard and
        // the command line was where the keys were going.
        let mut screen = tapestry();
        for (across, down) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            screen.step(across, down);
        }
        assert_eq!(screen.cursor(), None, "an arrow moved before a word did");
        assert_eq!(screen.mode(), Mode::Command);
    }

    #[test]
    fn a_word_goes_into_a_track_and_aims_at_its_first_node() {
        // A browse mode with nothing aimed at would be a mode whose only visible
        // difference is that the words stopped working.
        let mut screen = tapestry();
        say(&mut screen, "mastery");
        assert_eq!(screen.mode(), Mode::Browsing);
        assert_eq!(screen.cursor(), Some("tbi_a"));

        say(&mut screen, "ley");
        assert_eq!(screen.track(), Track::LeyLine);
        assert_eq!(screen.cursor(), Some("concentration"));
    }

    #[test]
    fn rightward_is_progress_and_downward_is_the_choice() {
        // **The two axes are two meanings**, which is the whole of why the
        // picture is a grid: a tier runs down and the track runs right.
        let mut screen = tapestry();
        say(&mut screen, "mastery");

        screen.step(0, 1);
        assert_eq!(
            screen.cursor(),
            Some("tbi_b"),
            "down did not pick a sibling"
        );
        screen.step(1, 0);
        assert_eq!(
            screen.cursor(),
            Some("tbi_c"),
            "right did not walk the track"
        );

        // The second tier is one node tall, so the row clamps rather than the
        // cursor vanishing — columns may differ in height.
        screen.step(0, 1);
        assert_eq!(screen.cursor(), Some("tbi_c"));
    }

    #[test]
    fn the_cursor_stops_at_the_ends_rather_than_wrapping() {
        // A track is read left to right, and wrapping would make its start and
        // its finish the same place — on a progression, the one thing the
        // picture must not say.
        let mut screen = tapestry();
        say(&mut screen, "mastery");
        for _ in 0..10 {
            screen.step(1, 0);
        }
        assert_eq!(screen.cursor(), Some("tbi_c"), "it wrapped");
        for _ in 0..10 {
            screen.step(-1, 0);
        }
        assert_eq!(screen.cursor(), Some("tbi_a"), "it wrapped the other way");
    }

    #[test]
    fn a_cursor_unplaces_when_what_it_pointed_at_goes_away() {
        // The world ticks behind the screen, so what is on it changes while a
        // player is looking. An id that is no longer drawn would leave the mark
        // on a node that has moved — a view outliving its subject.
        let mut screen = tapestry();
        say(&mut screen, "mastery");
        assert!(screen.cursor().is_some());

        screen.refresh(0, Vec::new(), Vec::new());
        assert_eq!(screen.cursor(), None, "it pointed at something gone");
    }

    #[test]
    fn an_arrow_recovers_an_aim_the_world_took_away() {
        // **The dead end that needed no keystroke to reach.** `refresh` unplaces
        // the cursor when the node under it stops being drawn, and that happens
        // on a tick the player did not ask for — after which every arrow was
        // inert while the status row still read `arrows move`, and retyping the
        // track word was the only way out.
        let mut screen = tapestry();
        say(&mut screen, "mastery");
        screen.refresh(0, Vec::new(), Vec::new());
        assert_eq!(screen.cursor(), None);
        assert_eq!(screen.mode(), Mode::Browsing, "it left browsing on its own");

        // The world comes back — a tier is still open — and an arrow works again.
        let mut restored = tapestry();
        say(&mut restored, "mastery");
        restored.refresh(0, Vec::new(), Vec::new());
        restored.refresh(
            24,
            vec![node("concentration", 16, Standing::Taken)],
            vec![vec![node("tbi_a", 24, Standing::Open)]],
        );
        restored.step(0, 1);
        assert_eq!(
            restored.cursor(),
            Some("tbi_a"),
            "the arrows stayed dead after the aim was taken away",
        );
    }

    #[test]
    fn typing_while_aiming_keeps_the_aim_and_starts_a_word() {
        // **The flow the arrows exist for**: point at a node, then type `take`.
        // Swallowing the letters would have been a dead end — nothing typed,
        // nothing said — and clearing the cursor would have made the arrows
        // useless for the one word that needs them.
        let mut screen = tapestry();
        say(&mut screen, "mastery");
        screen.step(0, 1);
        assert_eq!(screen.cursor(), Some("tbi_b"));

        screen.type_text("t");
        assert_eq!(screen.mode(), Mode::Command, "the letter went nowhere");
        assert_eq!(screen.command(), "t");
        assert_eq!(screen.cursor(), Some("tbi_b"), "typing lost the aim");

        screen.enter();
        assert_eq!(
            screen.complaint(),
            Some(&Complaint::NothingBehind("tbi_b".to_owned())),
            "`take` did not act on what was aimed at",
        );
    }

    #[test]
    fn backspace_while_aiming_deletes_nothing() {
        // The editor's `Backspace`-in-`Reading` defect (§19), refused in advance:
        // every mode is matched rather than treated as "not `Command`".
        let mut screen = tapestry();
        say(&mut screen, "mastery");
        screen.backspace();
        assert_eq!(screen.command(), "");
        assert_eq!(screen.mode(), Mode::Browsing, "it left the aim");
    }

    #[test]
    fn escape_steps_back_and_never_closes() {
        let mut screen = tapestry();
        say(&mut screen, "mastery");
        screen.escape();
        assert_eq!(screen.mode(), Mode::Command);

        screen.type_text("qui");
        screen.escape();
        assert_eq!(screen.command(), "", "the half-word survived");
        assert_eq!(screen.enter(), None, "escape closed the screen");
    }

    #[test]
    fn quit_is_the_only_way_out() {
        let mut screen = tapestry();
        assert_eq!(say(&mut screen, "quit"), Some(Outcome::Close));
        // ...and any unambiguous prefix reaches it, as it does in the editor.
        let mut screen = tapestry();
        assert_eq!(say(&mut screen, "q"), Some(Outcome::Close));
    }

    #[test]
    fn no_two_words_share_a_first_letter() {
        // What makes single-letter prefixes unambiguous, and the reason this is
        // a property rather than four hand-written cases.
        for (name, expected) in WORDS {
            let first = &name[..1];
            assert_eq!(
                word(first),
                Some(expected),
                "{first:?} does not reach {name}",
            );
        }
    }

    #[test]
    fn an_unknown_word_says_so_rather_than_clearing() {
        // §6 forbids a bare error, and a command line that silently empties is
        // indistinguishable from one that worked.
        let mut screen = tapestry();
        say(&mut screen, "xyzzy");
        assert_eq!(
            screen.complaint(),
            Some(&Complaint::Unknown("xyzzy".to_owned())),
        );
    }

    #[test]
    fn taking_names_what_it_refused_and_why() {
        // Nothing is takeable yet, so every branch is a refusal — and each one
        // has to say something different, because "no" without a reason is the
        // dead end §15 weighs above the raw resolution rate.
        let mut screen = tapestry();
        say(&mut screen, "take");
        assert_eq!(screen.complaint(), Some(&Complaint::Nothing), "unaimed");

        say(&mut screen, "mastery");
        say(&mut screen, "take");
        assert_eq!(
            screen.complaint(),
            Some(&Complaint::NothingBehind("tbi_a".to_owned())),
            "an open node did not say it was a marker",
        );

        // **A step the tower has already passed is not a marker.** This answered
        // `NothingBehind` — calling the game's one real grant empty, one row
        // under a details panel reading `active`. The screen contradicted itself
        // about the only thing the Ley Line delivers.
        let mut screen = tapestry();
        say(&mut screen, "ley");
        say(&mut screen, "take");
        assert_eq!(
            screen.complaint(),
            Some(&Complaint::Already("concentration".to_owned())),
        );

        // A locked tier names the total it is waiting for.
        let mut screen = tapestry();
        say(&mut screen, "mastery");
        for _ in 0..5 {
            screen.step(1, 0);
        }
        say(&mut screen, "take");
        assert_eq!(
            screen.complaint(),
            Some(&Complaint::Locked("tbi_c".to_owned(), 40)),
        );
    }

    #[test]
    fn changing_track_moves_the_aim_onto_the_new_one() {
        // A cursor left on a node of a track nobody is looking at is a mark the
        // player cannot see and `take` would still act on.
        let mut screen = tapestry();
        say(&mut screen, "mastery");
        assert_eq!(screen.cursor(), Some("tbi_a"));
        say(&mut screen, "ley");
        assert_eq!(screen.track(), Track::LeyLine);
        assert_eq!(screen.cursor(), Some("concentration"));
    }
}
