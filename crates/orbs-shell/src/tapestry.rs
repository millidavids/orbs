//! The weave screen's state: what is being looked at, and where the cursor is.
//!
//! Knows nothing about Bevy and nothing about panes — the wiring is in
//! `weaving.rs` and the drawing is in `loom.rs`, the same three-way split the
//! editor makes for the same reason: what a key *means* here and how a pane is
//! laid out are different concerns that change for different reasons.
//!
//! # Two tracks, two shapes
//!
//! **The Ley Line** is the tower's: stations on total experience, some of them
//! forks with siblings stacked downward. Left/right walks the stations,
//! up/down picks a sibling, and `take` chooses one.
//!
//! **Mastery** is seven lines, one per room, each read left to right. Up/down
//! picks a *room* and left/right walks its stations — so down means a different
//! thing on each track, and the status row says which. Nothing on a mastery
//! line is ever taken: a station is reached by doing its deed, and `take` there
//! is refused in voice.
//!
//! # It opens in command state, and the way in is a word
//!
//! §19 records why the editor does: *"the first keystroke cannot damage
//! anything."* Taking a fork node is irreversible — it closes its fork — so the
//! same rule applies with more force.
//!
//! **`ley` and `mastery` go into a track**, place the aim on its first node and
//! hand the arrows over; `<esc>` comes back. That is `edit` dropping into the
//! editor's buffer. An arrow pressed at the command line does nothing at all, on
//! purpose: a screen where the arrows are sometimes navigation and sometimes
//! nothing, depending on what you last typed, answers differently to the same
//! key — and *"my first key press was being ignored"* is how the version that
//! let them work immediately was reported.
//!
//! # The cursor is an identity, never an index
//!
//! The world ticks while the screen is open. Crossing a threshold opens a fork
//! and changes what is in the list, so an index would silently come to point at
//! a different node — and a view that survives its subject lies, which is the
//! rule §19 extracted from `Editor::reading`. The cursor holds a **mark** and
//! unplaces itself when that mark is no longer on screen.
//!
//! **A mark, not an id.** A mastery station's id already names one thing, but a
//! step's id is what it *grants* and eight stations grant concentration — so an
//! id cursor pointed at all eight, drew all eight aimed, and could not walk past
//! the fourth station. `Node::mark` carries the total with it and `Stop::mark`
//! is the id; both are unique across both tracks, so one cursor serves both.

use orbs_sim::{Line, Node, Standing, Station, Stop};

/// Which track is being looked at.
///
/// This decides where the cursor may go and which view is drawn below the
/// bar; both headings are always drawn, so a player who has not asked for
/// Mastery can still see that it is there.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Track {
    /// The tower's line: steps and forks.
    #[default]
    LeyLine,
    /// The seven rooms' lines.
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Close it.
    Close,
    /// Take this node — the shell hands the id to `Sim::take`.
    ///
    /// **An id, not an index.** The screen's rows are a view over
    /// `tower::ley_line` and could be reordered by content; the id is what the
    /// world stores and what `progression.toml` calls *"a decision, not prose"*.
    Take(String),
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
    /// A step is taken by being passed, so `take` on one is a player asking for
    /// something they are already holding.
    Already(String),
    /// `take` on a mastery station, which is reached by doing and never taken.
    NotAChoice(String),
    /// `take` on a node whose fork has not opened.
    Locked(String, u64),
    /// `take` on a fork that has already given its one.
    Spent(String),
}

/// What the cursor is on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Aimed {
    /// A node on the Ley Line.
    Node(Node),
    /// A station on a room's mastery line.
    Stop {
        /// The room.
        domain: &'static str,
        /// The station.
        stop: Stop,
    },
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
    /// The node or station the cursor is on, by mark. `None` until an arrow is
    /// pressed.
    cursor: Option<String>,
    /// What the last word was refused for.
    complaint: Option<Complaint>,
    /// The tower's total, pushed in.
    experience: u64,
    /// What the bar is measured against: the last station's total.
    scale: u64,
    /// The Ley Line, pushed in.
    ley_line: Vec<Station>,
    /// The seven lines, pushed in.
    mastery: Vec<Line>,
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

    /// The mark the cursor is on, if it has been placed.
    ///
    /// **What a painter compares against**, so a drawn station knows whether it
    /// is the aimed one. `Node::mark` and `Stop::mark` are what produce it.
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

    /// What the bar is measured against.
    #[must_use]
    pub const fn scale(&self) -> u64 {
        self.scale
    }

    /// The Ley Line's stations, in order.
    #[must_use]
    pub fn ley_line(&self) -> &[Station] {
        &self.ley_line
    }

    /// The seven lines, in the rail's order.
    #[must_use]
    pub fn mastery(&self) -> &[Line] {
        &self.mastery
    }

    /// What the cursor is on, if it has been placed.
    #[must_use]
    pub fn aimed(&self) -> Option<Aimed> {
        let mark = self.cursor.as_ref()?;
        if let Some(node) = self
            .ley_line
            .iter()
            .flat_map(|station| &station.nodes)
            .find(|node| &node.mark() == mark)
        {
            return Some(Aimed::Node(node.clone()));
        }
        self.rows().into_iter().find_map(|line| {
            line.stops
                .iter()
                .find(|stop| stop.mark() == mark)
                .map(|stop| Aimed::Stop {
                    domain: line.domain,
                    stop: stop.clone(),
                })
        })
    }

    /// Take a fresh reading of both tracks.
    ///
    /// **Pushed in rather than pulled**, because the screen knows nothing about
    /// the sim — the same direction `Editor::set_reading` runs in. Called every
    /// frame the screen is open, because the world ticks behind it and a
    /// threshold crossed while a player is looking should land while they are
    /// looking.
    pub fn refresh(
        &mut self,
        experience: u64,
        scale: u64,
        ley_line: Vec<Station>,
        mastery: Vec<Line>,
    ) {
        self.experience = experience;
        self.scale = scale;
        self.ley_line = ley_line;
        self.mastery = mastery;
        // **The cursor is checked against the new reading, not carried over.**
        // A fork opening changes what is on screen, and an id that is no longer
        // there would leave the highlight on a row that has moved.
        if self
            .cursor
            .as_ref()
            .is_some_and(|id| !self.walkable().iter().any(|had| had == id))
        {
            self.cursor = None;
        }
    }

    /// The Ley Line as **columns left to right**: a station's nodes are a
    /// column, so a step is one node tall and a fork is as tall as its choice.
    #[must_use]
    pub fn columns(&self) -> Vec<Vec<Node>> {
        self.ley_line
            .iter()
            .map(|station| station.nodes.clone())
            .collect()
    }

    /// The rooms whose lines can be walked: the **open** ones with a station on
    /// them.
    ///
    /// **A shut room's line is not walkable, because it is not drawn.** The loom
    /// paints a room the player cannot enter as an anonymous dotted run and
    /// draws no stations on it — so a cursor there was invisible, and the
    /// details panel then read out the deed, its count and the room it opens.
    /// That is exactly the foreshadowing the boot report, `survey` and the scene
    /// all withhold; the weave must not be the one surface that gives it away.
    fn rows(&self) -> Vec<&Line> {
        self.mastery
            .iter()
            .filter(|line| line.open && !line.stops.is_empty())
            .collect()
    }

    /// Every mark the cursor may sit on, on the track being walked, in reading
    /// order.
    fn walkable(&self) -> Vec<String> {
        match self.track {
            Track::LeyLine => self
                .columns()
                .into_iter()
                .flatten()
                .map(|node| node.mark())
                .collect(),
            Track::Mastery => self
                .rows()
                .into_iter()
                .flat_map(|line| line.stops.iter().map(|stop| stop.mark().to_owned()))
                .collect(),
        }
    }

    /// Where the cursor is on the Ley Line, as `(column, row)`.
    fn at_station(&self) -> Option<(usize, usize)> {
        let mark = self.cursor.as_ref()?;
        self.columns()
            .iter()
            .enumerate()
            .find_map(|(column, rows)| {
                rows.iter()
                    .position(|node| &node.mark() == mark)
                    .map(|row| (column, row))
            })
    }

    /// Where the cursor is on Mastery, as `(line, station)`.
    fn at_stop(&self) -> Option<(usize, usize)> {
        let mark = self.cursor.as_ref()?;
        self.rows().iter().enumerate().find_map(|(row, line)| {
            line.stops
                .iter()
                .position(|stop| stop.mark() == mark)
                .map(|column| (row, column))
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
            Mode::Browsing => self.take(),
        }
    }

    /// Move the cursor by one column (`across`) and one row (`down`).
    ///
    /// **Two axes, two meanings per track.** On the Ley Line, progression runs
    /// rightward and a fork's choice runs downward, so left/right walks the
    /// stations and up/down picks a sibling — which on a step, one node tall,
    /// does nothing at all. On Mastery, down picks a *room* and left/right
    /// walks its line.
    ///
    /// **The first arrow places rather than moves.** Until one is pressed the
    /// cursor is nowhere, which is what keeps `Enter` from committing anything
    /// before the player has aimed — §19's *"the first keystroke cannot damage
    /// anything"*, carried into a screen where the damage is irreversible.
    pub fn step(&mut self, across: i8, down: i8) {
        // **The command line has the keys until a word hands them over**, which
        // is the editor's shape exactly. An arrow pressed at the command line
        // does nothing, on purpose.
        if self.mode != Mode::Browsing {
            return;
        }
        let walkable = self.walkable();
        if walkable.is_empty() {
            return;
        }
        self.complaint = None;

        // **An arrow with nothing aimed at places the aim**, rather than doing
        // nothing. `look` normally places it, so this is unreachable by typing —
        // but `refresh` unplaces the cursor when the node under it stops being
        // drawn, and that can happen while browsing, on a tick the player did not
        // ask for. Returning early there left every arrow inert while the status
        // row still read `arrows move`: a dead end §6 forbids, arrived at without
        // touching the keyboard.
        match self.track {
            Track::LeyLine => {
                let columns = self.columns();
                let Some((column, row)) = self.at_station() else {
                    self.cursor = walkable.into_iter().next();
                    return;
                };
                // **Stops at the ends rather than wrapping.** A track is read
                // left to right, and wrapping would make its start and its
                // finish the same place.
                let column = shift(column, across, columns.len());
                let rows = &columns[column];
                if rows.is_empty() {
                    return;
                }
                // Clamped, not remembered: columns differ in height, and a
                // cursor carrying a row past the end of a shorter one would
                // vanish.
                let row = shift(row.min(rows.len() - 1), down, rows.len());
                self.cursor = Some(rows[row].mark());
            }
            Track::Mastery => {
                let rows = self.rows();
                let Some((row, column)) = self.at_stop() else {
                    self.cursor = walkable.into_iter().next();
                    return;
                };
                let row = shift(row, down, rows.len());
                let stops = &rows[row].stops;
                let column = shift(column.min(stops.len() - 1), across, stops.len());
                self.cursor = Some(stops[column].mark().to_owned());
            }
        }
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
        self.cursor = self.walkable().into_iter().next();
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
            Some(Word::Take) => self.take(),
            Some(Word::Quit) => Some(Outcome::Close),
            None => {
                self.complaint = Some(Complaint::Unknown(typed));
                None
            }
        }
    }

    /// Take what the cursor is on.
    ///
    /// **Every branch is authored**: §6 forbids a bare error, and a screen whose
    /// central verb answered with silence would be worse than one that had no
    /// verb. An open fork node returns [`Outcome::Take`] and the shell hands the
    /// id to the sim, which re-checks every rule before granting.
    fn take(&mut self) -> Option<Outcome> {
        let node = match self.aimed() {
            None => {
                self.complaint = Some(Complaint::Nothing);
                return None;
            }
            // **Reached by doing, never taken.** The one refusal that is about
            // the shape of the track rather than the state of the node.
            Some(Aimed::Stop { stop, .. }) => {
                self.complaint = Some(Complaint::NotAChoice(stop.id));
                return None;
            }
            Some(Aimed::Node(node)) => node,
        };
        self.complaint = match node.standing {
            Standing::Open => {
                return Some(Outcome::Take(node.id));
            }
            // **Held already, which is not the same as empty.** A step is taken
            // by being passed, so this is the one branch that is about
            // something the tower really has.
            Standing::Taken => Some(Complaint::Already(node.id)),
            // `unlocked` rather than a total comparison: only the field stays
            // right when a fork is shut because a sibling took its one choice.
            Standing::Locked if node.unlocked => Some(Complaint::Spent(node.id)),
            Standing::Locked => Some(Complaint::Locked(node.id, node.at)),
        };
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use orbs_sim::{Lane, Walk};

    /// One node, as the sim would report it at 24 experience.
    fn node(id: &str, at: u64, standing: Standing) -> Node {
        Node {
            id: id.to_owned(),
            at,
            standing,
            unlocked: at <= 24,
            lane: Some(Lane::Craft),
        }
    }

    fn step(id: &str, at: u64) -> Station {
        Station {
            at,
            fork: false,
            nodes: vec![Node {
                id: id.to_owned(),
                at,
                standing: if at <= 24 {
                    Standing::Taken
                } else {
                    Standing::Locked
                },
                unlocked: at <= 24,
                lane: None,
            }],
            opens: Vec::new(),
        }
    }

    fn fork(at: u64, nodes: Vec<Node>) -> Station {
        Station {
            at,
            fork: true,
            nodes,
            opens: Vec::new(),
        }
    }

    fn stop(id: &str, walk: Walk) -> Stop {
        Stop {
            id: id.to_owned(),
            walk,
            done: 0,
            needed: 1,
            opens: Vec::new(),
        }
    }

    fn line(domain: &'static str, stops: Vec<Stop>) -> Line {
        Line {
            domain,
            open: true,
            stops,
        }
    }

    /// A room the player cannot enter yet, with a line behind the door.
    fn shut(domain: &'static str, stops: Vec<Stop>) -> Line {
        Line {
            domain,
            open: false,
            stops,
        }
    }

    /// A tapestry with one taken step, one open fork, one locked fork, and two
    /// rooms' lines.
    fn tapestry() -> Tapestry {
        let mut screen = Tapestry::default();
        screen.refresh(
            24,
            56,
            vec![
                step("concentration", 16),
                fork(
                    24,
                    vec![
                        node("steps_1", 24, Standing::Open),
                        node("satchel_1", 24, Standing::Open),
                    ],
                ),
                fork(40, vec![node("steps_2", 40, Standing::Locked)]),
            ],
            vec![
                line(
                    "laboratory",
                    vec![
                        stop("laboratory_1", Walk::Reached),
                        stop("laboratory_2", Walk::Next),
                        stop("laboratory_3", Walk::Later),
                    ],
                ),
                line(
                    "archive",
                    vec![
                        stop("archive_1", Walk::Next),
                        stop("archive_2", Walk::Later),
                    ],
                ),
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
        // **The property the unplaced cursor exists for.** Taking a fork node
        // closes its fork and cannot be undone, so the first keystroke must not
        // be able to reach it.
        let mut screen = tapestry();
        assert_eq!(screen.enter(), None);
        assert_eq!(screen.cursor(), None);
        assert_eq!(screen.complaint(), None, "an empty line complained");
    }

    #[test]
    fn an_arrow_at_the_command_line_does_nothing_at_all() {
        let mut screen = tapestry();
        for (across, down) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            screen.step(across, down);
        }
        assert_eq!(screen.cursor(), None, "an arrow moved before a word did");
        assert_eq!(screen.mode(), Mode::Command);
    }

    #[test]
    fn a_word_goes_into_a_track_and_aims_at_its_first_node() {
        let mut screen = tapestry();
        say(&mut screen, "mastery");
        assert_eq!(screen.mode(), Mode::Browsing);
        assert_eq!(screen.cursor(), Some("laboratory_1"));

        say(&mut screen, "ley");
        assert_eq!(screen.track(), Track::LeyLine);
        assert_eq!(screen.cursor(), Some("16:concentration"));
    }

    #[test]
    fn on_the_ley_line_rightward_is_progress_and_downward_is_the_choice() {
        let mut screen = tapestry();
        say(&mut screen, "ley");
        screen.step(1, 0);
        assert_eq!(
            screen.cursor(),
            Some("24:steps_1"),
            "right did not walk the line"
        );
        screen.step(0, 1);
        assert_eq!(
            screen.cursor(),
            Some("24:satchel_1"),
            "down did not pick a sibling"
        );
        screen.step(1, 0);
        assert_eq!(
            screen.cursor(),
            Some("40:steps_2"),
            "right did not reach the next fork"
        );
        // The second fork is one node tall, so the row clamps rather than the
        // cursor vanishing.
        screen.step(0, 1);
        assert_eq!(screen.cursor(), Some("40:steps_2"));
    }

    #[test]
    fn on_mastery_downward_is_a_room_and_rightward_is_its_line() {
        let mut screen = tapestry();
        say(&mut screen, "mastery");
        screen.step(1, 0);
        assert_eq!(
            screen.cursor(),
            Some("laboratory_2"),
            "right did not walk the line"
        );
        screen.step(0, 1);
        assert_eq!(
            screen.cursor(),
            Some("archive_2"),
            "down did not change room"
        );
        // A shorter line clamps the station rather than losing the cursor.
        screen.step(1, 0);
        screen.step(1, 0);
        assert_eq!(screen.cursor(), Some("archive_2"));
        screen.step(0, -1);
        assert_eq!(screen.cursor(), Some("laboratory_2"));
    }

    #[test]
    fn the_cursor_stops_at_the_ends_rather_than_wrapping() {
        let mut screen = tapestry();
        say(&mut screen, "ley");
        for _ in 0..10 {
            screen.step(1, 0);
        }
        assert_eq!(screen.cursor(), Some("40:steps_2"), "it wrapped");
        for _ in 0..10 {
            screen.step(-1, 0);
        }
        assert_eq!(
            screen.cursor(),
            Some("16:concentration"),
            "it wrapped the other way"
        );
    }

    #[test]
    fn a_cursor_unplaces_when_what_it_pointed_at_goes_away() {
        let mut screen = tapestry();
        say(&mut screen, "mastery");
        assert!(screen.cursor().is_some());

        screen.refresh(0, 56, Vec::new(), Vec::new());
        assert_eq!(screen.cursor(), None, "it pointed at something gone");
    }

    #[test]
    fn an_arrow_recovers_an_aim_the_world_took_away() {
        let mut restored = tapestry();
        say(&mut restored, "ley");
        restored.refresh(0, 56, Vec::new(), Vec::new());
        assert_eq!(restored.cursor(), None);
        assert_eq!(
            restored.mode(),
            Mode::Browsing,
            "it left browsing on its own"
        );

        restored.refresh(24, 56, vec![step("concentration", 16)], Vec::new());
        restored.step(0, 1);
        assert_eq!(
            restored.cursor(),
            Some("16:concentration"),
            "the arrows stayed dead after the aim was taken away",
        );
    }

    #[test]
    fn typing_while_aiming_keeps_the_aim_and_starts_a_word() {
        // **The flow the arrows exist for**: point at a node, then type `take`.
        let mut screen = tapestry();
        say(&mut screen, "ley");
        screen.step(1, 0);
        screen.step(0, 1);
        assert_eq!(screen.cursor(), Some("24:satchel_1"));

        screen.type_text("t");
        assert_eq!(screen.mode(), Mode::Command, "the letter went nowhere");
        assert_eq!(screen.command(), "t");
        assert_eq!(screen.cursor(), Some("24:satchel_1"), "typing lost the aim");

        assert_eq!(
            screen.enter(),
            Some(Outcome::Take("satchel_1".to_owned())),
            "`take` did not act on what was aimed at",
        );
    }

    #[test]
    fn backspace_while_aiming_deletes_nothing() {
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
        let mut screen = tapestry();
        assert_eq!(say(&mut screen, "q"), Some(Outcome::Close));
    }

    #[test]
    fn no_two_words_share_a_first_letter() {
        for (name, expected) in WORDS {
            let first = &name[..1];
            assert_eq!(
                word(first),
                Some(expected),
                "{first:?} does not reach {name}"
            );
        }
    }

    #[test]
    fn an_unknown_word_says_so_rather_than_clearing() {
        let mut screen = tapestry();
        say(&mut screen, "xyzzy");
        assert_eq!(
            screen.complaint(),
            Some(&Complaint::Unknown("xyzzy".to_owned())),
        );
    }

    #[test]
    fn taking_names_what_it_refused_and_why() {
        let mut screen = tapestry();
        say(&mut screen, "take");
        assert_eq!(screen.complaint(), Some(&Complaint::Nothing), "unaimed");

        // **A mastery station is reached, never taken**, and the refusal says
        // so rather than pretending it is locked.
        say(&mut screen, "mastery");
        say(&mut screen, "take");
        assert_eq!(
            screen.complaint(),
            Some(&Complaint::NotAChoice("laboratory_1".to_owned())),
        );

        // **A step the tower has already passed is not empty.**
        let mut screen = tapestry();
        say(&mut screen, "ley");
        say(&mut screen, "take");
        assert_eq!(
            screen.complaint(),
            Some(&Complaint::Already("concentration".to_owned())),
        );

        // An open fork node is taken. Typing `take` stepped back to the
        // command line, so the arrows need the track word again first.
        say(&mut screen, "ley");
        screen.step(1, 0);
        assert_eq!(
            say(&mut screen, "take"),
            Some(Outcome::Take("steps_1".to_owned()))
        );

        // A locked fork names the total it is waiting for.
        let mut screen = tapestry();
        say(&mut screen, "ley");
        for _ in 0..5 {
            screen.step(1, 0);
        }
        say(&mut screen, "take");
        assert_eq!(
            screen.complaint(),
            Some(&Complaint::Locked("steps_2".to_owned(), 40)),
        );
    }

    #[test]
    fn a_spent_fork_says_you_chose_otherwise() {
        let mut screen = Tapestry::default();
        screen.refresh(
            24,
            56,
            vec![fork(
                24,
                vec![
                    node("steps_1", 24, Standing::Taken),
                    node("satchel_1", 24, Standing::Locked),
                ],
            )],
            Vec::new(),
        );
        say(&mut screen, "ley");
        screen.step(0, 1);
        say(&mut screen, "take");
        assert_eq!(
            screen.complaint(),
            Some(&Complaint::Spent("satchel_1".to_owned())),
        );
    }

    #[test]
    fn changing_track_moves_the_aim_onto_the_new_one() {
        let mut screen = tapestry();
        say(&mut screen, "mastery");
        assert_eq!(screen.cursor(), Some("laboratory_1"));
        say(&mut screen, "ley");
        assert_eq!(screen.track(), Track::LeyLine);
        assert_eq!(screen.cursor(), Some("16:concentration"));
    }

    #[test]
    fn two_steps_granting_one_thing_are_two_places_the_cursor_can_be() {
        // Eight stations grant concentration. While the cursor held the *id*,
        // all eight drew aimed at once, the panel read the first one's cost, and
        // `→` from any of them jumped back to the second station — a player
        // could not walk past the fourth. The mark carries the total; see the
        // module header.
        let mut screen = Tapestry::default();
        screen.refresh(
            24,
            96,
            vec![
                step("concentration", 16),
                fork(24, vec![node("steps_1", 24, Standing::Open)]),
                step("concentration", 96),
            ],
            Vec::new(),
        );
        say(&mut screen, "ley");
        assert_eq!(screen.cursor(), Some("16:concentration"));
        screen.step(1, 0);
        screen.step(1, 0);
        assert_eq!(
            screen.cursor(),
            Some("96:concentration"),
            "the line would not walk past the fork",
        );
        let Some(Aimed::Node(node)) = screen.aimed() else {
            panic!("the cursor is on nothing");
        };
        assert_eq!(node.at, 96, "the panel read a different station");
    }

    #[test]
    fn a_room_the_player_cannot_enter_yet_is_not_walked() {
        // The loom draws a shut room as an anonymous dotted run and puts no
        // stations on it, so a cursor there is invisible — and the details panel
        // would then read out the deed, its count and the room it opens, which
        // is the one thing the boot report, `survey` and the scene all withhold.
        let mut screen = Tapestry::default();
        screen.refresh(
            0,
            56,
            Vec::new(),
            vec![
                line("laboratory", vec![stop("laboratory_1", Walk::Next)]),
                shut("archive", vec![stop("archive_1", Walk::Next)]),
            ],
        );
        say(&mut screen, "mastery");
        assert_eq!(screen.cursor(), Some("laboratory_1"));
        screen.step(0, 1);
        assert_eq!(
            screen.cursor(),
            Some("laboratory_1"),
            "the cursor walked into a room the player cannot enter",
        );
        assert!(matches!(
            screen.aimed(),
            Some(Aimed::Stop {
                domain: "laboratory",
                ..
            })
        ));
    }

    #[test]
    fn the_aim_knows_which_track_it_is_on() {
        let mut screen = tapestry();
        say(&mut screen, "mastery");
        assert!(matches!(
            screen.aimed(),
            Some(Aimed::Stop {
                domain: "laboratory",
                ..
            })
        ));
        say(&mut screen, "ley");
        assert!(matches!(screen.aimed(), Some(Aimed::Node(_))));
    }
}
