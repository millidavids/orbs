//! Which of the five things on screen has the keyboard, and what it does with it.
//!
//! The Bevy build spends four resources, four run conditions and a four-term
//! predicate on this, and its own comment predicts the ceiling: *"a fifth surface
//! refactors this first."* Here it is one enum and a `match`, because a terminal
//! delivers one keystroke at a time to whoever is asking — there is no message
//! queue to leave keys waiting in, and so no need to *run* in order to throw them
//! away.
//!
//! The state machines themselves are not reimplemented: [`Editor`] and
//! [`Tapestry`] are `orbs-shell`'s, and this decides only which a key reaches.

use crossterm::event::KeyCode;
use orbs_shell::{Editor, EditorOutcome, Menu, MenuOutcome, Scroll, Tapestry};
use orbs_sim::Sim;

/// The surfaces that can hold the keyboard, in the order they take it.
///
/// Declared here once and `orbs-shell`'s now: the ordering was right, and being
/// right in one frontend was the problem, because the Bevy build carried its own
/// four-term expression of the same rule. The alias stays because a terminal
/// does own the *dispatch* below, even though it no longer owns the *ordering*.
pub(crate) use orbs_shell::Focus as Owner;

/// Everything that can be open at once, which is at most one of them.
#[derive(Default)]
pub(crate) struct Surfaces {
    /// The spell being edited.
    pub(crate) editing: Option<Editor>,
    /// The progression screen.
    pub(crate) weaving: Option<Tapestry>,
    /// Whether the arrow keys are walking the stacks.
    pub(crate) walking: bool,
    /// The orb's menu, if `quit` has opened it.
    pub(crate) menuing: Option<Menu>,
    /// The manual, if the menu has opened it.
    ///
    /// Over the menu rather than instead of it: `menuing` stays `Some` while
    /// this is, and closing this gives the keyboard back. The one pair in
    /// `Focus` that can really be open together.
    pub(crate) reading: Option<orbs_shell::ManualReader>,
    /// Whether the menu asked for the orb to be put down.
    ///
    /// A flag rather than a return value, unlike every other surface here:
    /// [`typed`](Self::typed) returns nothing because no surface can end the
    /// session, and the menu is the one that can. Changing that signature would
    /// put an `Option` on five arms that never fill it.
    pub(crate) leaving: bool,
    /// A tower the menu asked for, waiting for `drive::run` to build it.
    ///
    /// Raised here and acted on there, for `leaving`'s reason and one more:
    /// putting a different `Sim` in front of the player means rebuilding the
    /// whole `Session`, the terminal's answer to the Bevy build's
    /// eighteen-resource reset — a new struct forgets no derived field.
    pub(crate) swapping: Option<Swap>,
    /// Whether the menu has asked for the manual, waiting for the book.
    ///
    /// A flag for `leaving`'s reason: `menu_took` takes no `Sim` on purpose, and
    /// the book is assembled from one. `Surfaces::open` has both.
    pub(crate) opening_manual: bool,
    /// A driver the options page chose, waiting for `drive::run` to honour it.
    ///
    /// Raised here and acted on there, for `swapping`'s reason: the reader
    /// belongs to the `Session`, and a surface reaching into it would be the
    /// shared half reaching for the backend's. The choice is already written
    /// down by then; this only makes it true for the running session.
    pub(crate) driving: Option<orbs_shell::Driver>,
    /// A setting the menu changed, waiting for `drive::run` to honour it.
    ///
    /// [`driving`](Self::driving)'s shape, which it should have had from the
    /// start: this arm was `Some(MenuOutcome::Set { .. }) => {}` under a comment
    /// claiming the session would pick the value up from the row next frame.
    /// Nothing did — rows flow menu-ward only — so `linear on` wrote the file,
    /// flipped the row, and the pane kept drawing cells, with reopening the menu
    /// flipping the row back. `Outcome::Set`'s doc is the contract that broke:
    /// *"honour it now rather than at the next launch"*.
    pub(crate) setting: Option<(String, String)>,
}

/// A tower the menu asked for.
///
/// An enum rather than two optional fields, exactly as the Bevy build's
/// `Raising` is and for the same reason: the threshold made a new tower with
/// nowhere to keep it reachable, and a path beside a length spells four states
/// for three meanings. The fourth — a load with no file — would be a dead
/// branch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Swap {
    /// Open the tower already in this file.
    Load(std::path::PathBuf),
    /// Begin a tower at this length.
    Begin {
        /// Where it will be kept, or [`None`] for a session that keeps nothing
        /// (`ORBS_SAVE=off`) — a playable tower that is not remembered, which is
        /// how every scenario in the played-game suite runs.
        keep: Option<std::path::PathBuf>,
        /// How long it is to be.
        length: orbs_sim::content::Length,
    },
}

impl Surfaces {
    /// Who the next keystroke belongs to.
    ///
    /// The tie-break and its reasoning live with the enum in `orbs-shell`; this
    /// only says what *this* frontend has open. Both builds ask the one
    /// function, so neither can drift into answering it differently.
    pub(crate) const fn owner(&self, scroll: &Scroll) -> Owner {
        Owner::of(orbs_shell::Open {
            editing: self.editing.is_some(),
            weaving: self.weaving.is_some(),
            walking: self.walking,
            reading: scroll.is_reading(),
            menuing: self.menuing.is_some(),
            reading_manual: self.reading.is_some(),
        })
    }

    /// Take whatever the world has offered since the last tick.
    ///
    /// Each is a take-once handshake: the sim raises a flag when a verb asks for
    /// a surface, and reading it clears it. The Bevy build needs a peek-then-take
    /// pair because `&mut` through a `ResMut` stamps a change tick and defeats
    /// its own guards; a loop with no change detection just takes.
    pub(crate) fn open(
        &mut self,
        sim: &mut Sim,
        scroll: &mut Scroll,
        page: usize,
        driver: orbs_shell::Driver,
        linear: bool,
        mode: orbs_render::DisplayMode,
        scrivener: &dyn orbs_sim::Scrivener,
    ) {
        if self.editing.is_none()
            && let Some(request) = sim.opening()
        {
            let mut editor = Editor::open(&request.name, &request.domain, &request.lines);
            // Read before the first keystroke, so a spell opened with a fault in
            // it says so on the way in. The pair to the settle-beat read below.
            editor.set_reading(sim.read_spell_with(
                &request.name,
                &request.domain,
                &request.lines,
                scrivener,
            ));
            // Opened on the vocabulary rather than blank until the first key.
            editor.refresh(sim);
            self.editing = Some(editor);
        }
        if self.weaving.is_none() && sim.weaving() {
            let mut screen = Tapestry::default();
            screen.refresh(
                sim.experience(),
                sim.renown(),
                sim.scale(),
                sim.ley_line(),
                sim.mastery(),
            );
            self.weaving = Some(screen);
        }
        if !self.walking && sim.wandering() {
            self.walking = true;
        }
        // `menu`, not `quit`: the two were one word for an iteration (§19).
        // `quit`'s flag is `drive`'s, set only by a *confirmed* one.
        if self.menuing.is_none() && sim.menuing() {
            // The options page marks what is *in effect*, which the session
            // holds — see `Menu::show_driver`. `InTower`, because the word is
            // typed at a prompt: the threshold's menu is `Session::new`'s.
            let mut menu = Menu::at(orbs_shell::Stance::InTower);
            menu.show_driver(driver);
            menu.show_settings(crate::drive::settings(driver, linear, mode));
            self.menuing = Some(menu);
        }
        // The book, assembled here because this is where the `Sim` is: the menu
        // raised the flag, and `menu_took` has no world to build one from.
        if self.opening_manual {
            self.opening_manual = false;
            self.reading = Some(orbs_shell::ManualReader::of(orbs_shell::manual_book(sim)));
        }
        if !scroll.is_reading() && sim.unfurling() {
            scroll.read();
            // And page back, which this build did not: an `unfurl` that only
            // takes the keyboard shows the screen the player was already
            // looking at, with nothing moving but the border hint.
            let total = sim.scrollback().records().drawn_len();
            scroll.page(page, true, total);
        }
    }

    /// Keep an open surface current with a world that moved under it.
    ///
    /// All three are unconditional while their surface is open, matching the
    /// Bevy build: a threshold crossed while a player looks at the weave should
    /// land while they look, a spell's running line moves under the editor, and
    /// a spell can close the maze from under the player — holding the keyboard
    /// over a pane with no map on it is the worst way that ends.
    pub(crate) fn tick(&mut self, sim: &Sim) {
        if let Some(editor) = &mut self.editing {
            editor.set_running_line(sim.running_line(editor.name()));
        }
        if let Some(screen) = &mut self.weaving {
            screen.refresh(
                sim.experience(),
                sim.renown(),
                sim.scale(),
                sim.ley_line(),
                sim.mastery(),
            );
        }
        if self.walking && sim.stacks().is_none() {
            self.walking = false;
        }
    }

    /// The buffer writes itself out a beat after the typing stops.
    ///
    /// §19: there is no `save` word in the editor's vocabulary. `quit` flushes
    /// and closes, `w` writes and stays — and if the player simply stops typing,
    /// this is what notices.
    pub(crate) fn settle(
        &mut self,
        delta: f32,
        sim: &mut Sim,
        scrivener: &dyn orbs_sim::Scrivener,
    ) {
        let Some(editor) = &mut self.editing else {
            return;
        };
        if editor.settle(delta) {
            // The save reads the buffer again on its way out — see `save`.
            save(editor, sim, scrivener);
        }
    }

    /// Route one keystroke to the surface that owns it.
    ///
    /// Nothing is returned because no surface can end the session *here*: five
    /// of the six hand the keyboard back instead. The menu is the exception and
    /// it raises [`leaving`](Self::leaving) rather than returning, so five arms
    /// are not made to carry an `Option` only one of them can ever fill.
    pub(crate) fn typed(
        &mut self,
        owner: Owner,
        code: KeyCode,
        sim: &mut Sim,
        scroll: &mut Scroll,
        scrivener: &dyn orbs_sim::Scrivener,
    ) {
        match owner {
            // The prompt is the caller's; it needs the shared key table and the
            // line, neither of which belongs to a surface.
            Owner::Prompt => {}
            Owner::Manual => self.manual_took(code),
            Owner::Menu => self.menu_took(code),
            Owner::Editor => self.editing_took(code, sim, scrivener),
            Owner::Weave => self.weaving_took(code, sim),
            Owner::Maze => self.maze_took(code, sim),
            // No `page` any more: `PageUp`/`PageDown` are answered above the
            // surface dispatch, so the only stepping left is the arrows.
            Owner::Reading => read(code, scroll, sim),
        }
    }

    /// One keystroke, to the manual.
    ///
    /// Takes no `Sim` either, for the menu's reason: the book was assembled when
    /// the reader opened and a chapter is text.
    fn manual_took(&mut self, code: KeyCode) {
        let Some(reader) = &mut self.reading else {
            return;
        };
        let Some(key) = crate::drive::as_key(code) else {
            return;
        };
        if orbs_shell::apply_to_manual(&key, reader).is_some() {
            // Closed. The menu it opened over is still there and gets the
            // keyboard back, which is where it came from.
            self.reading = None;
        }
    }

    /// One keystroke, to the orb's menu.
    ///
    /// Takes no `Sim`: nothing the menu does reaches the world. That is the
    /// whole of the difference from the four below it, and it is why the menu
    /// survives a game being swapped out from under it.
    fn menu_took(&mut self, code: KeyCode) {
        let Some(menu) = &mut self.menuing else {
            return;
        };
        let Some(key) = crate::drive::as_key(code) else {
            return;
        };
        match orbs_shell::apply_to_menu(&key, menu) {
            Some(MenuOutcome::Close) => self.menuing = None,
            // Raised, not acted on: putting the terminal back is `drive`'s, and
            // a surface reaching for raw mode leaks the backend into the shared
            // half.
            Some(MenuOutcome::PutDown) => self.leaving = true,
            // ...and the same for a swap, which needs a whole new `Session`.
            Some(MenuOutcome::Load(path)) => self.swapping = Some(Swap::Load(path)),
            Some(MenuOutcome::Begin { path, length }) => {
                self.swapping = Some(Swap::Begin { keep: path, length });
            }
            // Carried to the session, the only thing that can honour it: every
            // setting this build offers is one it holds in the session, and the
            // menu has already written it down. See [`Surfaces::setting`] for
            // what dropping it cost.
            Some(MenuOutcome::Set { setting, value }) => self.setting = Some((setting, value)),
            // Raised here and filled in `Surfaces::open`, because the book is
            // assembled from the `Sim` and this method takes none — nothing the
            // menu does reaches the world, which is what lets it survive a game
            // being swapped out from under it.
            Some(MenuOutcome::OpenManual) => self.opening_manual = true,
            Some(MenuOutcome::Drive(driver)) => self.driving = Some(driver),
            None => {}
        }
    }

    fn editing_took(&mut self, code: KeyCode, sim: &mut Sim, scrivener: &dyn orbs_sim::Scrivener) {
        let Some(editor) = &mut self.editing else {
            return;
        };
        // Enter and Escape mean something in both of the editor's states and the
        // editor decides which; branching on the mode here would be a second
        // copy of that machine, in the one place neither the tests nor
        // `ORBS_DUMP` can reach. The table is `orbs-shell`'s for the prompt's
        // reason — this was a second copy, and it had already missed the
        // correction the other carries.
        let Some(key) = crate::drive::as_key(code) else {
            return;
        };
        let outcome = orbs_shell::apply_to_editor(&key, editor, sim);

        match outcome {
            Some(EditorOutcome::Save) => save(editor, sim, scrivener),
            Some(EditorOutcome::SaveAndClose) => {
                save(editor, sim, scrivener);
                self.editing = None;
            }
            None => {}
        }
    }

    /// A key at the weave screen, and what it asked the world for.
    ///
    /// The screen refuses a locked node, a spent fork and a mastery station in
    /// voice before anything reaches here; what does reach here is a fork node
    /// the player chose, handed to the sim the way the Bevy build hands it.
    fn weaving_took(&mut self, code: KeyCode, sim: &mut Sim) {
        let Some(screen) = &mut self.weaving else {
            return;
        };
        let Some(key) = crate::drive::as_key(code) else {
            return;
        };
        match orbs_shell::apply_to_weave(&key, screen) {
            Some(orbs_shell::WeaveOutcome::Close) => self.weaving = None,
            // The Bevy build's line, and it has to be the same one: rule 2 lets a
            // frontend decide how a cell is drawn and nothing else, so both hand
            // the id to the sim and the sim decides.
            Some(orbs_shell::WeaveOutcome::Take(id)) => sim.take(&id),
            None => {}
        }
    }

    fn maze_took(&mut self, code: KeyCode, sim: &mut Sim) {
        if code == KeyCode::Esc {
            self.walking = false;
            return;
        }
        // The arrow-to-`Way` table is `orbs-shell`'s. It was written three
        // times — once per frontend and once more in `dump.rs`.
        let Some(way) = crate::drive::as_key(code)
            .as_ref()
            .and_then(orbs_shell::apply_to_maze)
        else {
            return;
        };
        // `walk`, not `submit` and `step`: a tick per arrow means the player
        // walks as slowly as the world moves, and eight presses advance the
        // tower eight seconds. `Sim::walk` spends no time.
        if !sim.walk(way) {
            self.walking = false;
        }
    }
}

/// One call, carrying the whole buffer — the editor's entire contribution to a
/// replay.
///
/// `Sim::write_spell` records the submission now and queues the write for the
/// next tick, because effects land on tick boundaries.
///
/// And the reading, on the same beat and after the write — on every save, not
/// only the pause's. Refreshed on the settle beat alone, a `w` typed before the
/// pause left the marks a few keystrokes stale and the settle beat then had
/// nothing to save. Reading a buffer parses it and resolves every name against
/// the room, which is why it is not per tick; after the write, so a changed line
/// is read once.
fn save(editor: &mut Editor, sim: &mut Sim, scrivener: &dyn orbs_sim::Scrivener) {
    let (name, domain, lines) = (
        editor.name().to_owned(),
        editor.domain().to_owned(),
        editor.lines().to_vec(),
    );
    editor.saved();
    sim.write_spell_reading(&name, &lines, scrivener);
    editor.set_reading(sim.read_spell_with(&name, &domain, &lines, scrivener));
}

/// The transcript, while `unfurl` has the keyboard.
///
/// `page` is a record count the painter measured, not a row count: a record
/// costs at least one row, so paging by the pane's rows moves more than a
/// screenful and drops the lines in between.
fn read(code: KeyCode, scroll: &mut Scroll, sim: &Sim) {
    let total = sim.scrollback().records().drawn_len();
    match code {
        KeyCode::Esc => scroll.stop_reading(),
        // `PageUp`/`PageDown` are deliberately absent: `drive::typed` answers
        // them above the surface dispatch, so a copy here would be two
        // expressions of one rule. The arrows are here because the other build
        // binds them — this one shipped without, so a reader could page but not
        // step, and the one-record nudge did not exist.
        KeyCode::Up => scroll.page(1, true, total),
        KeyCode::Down => scroll.page(1, false, total),
        KeyCode::Home => scroll.page(total, true, total),
        KeyCode::End => scroll.rewind(),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::{Owner, Surfaces};
    use crossterm::event::KeyCode;
    use orbs_shell::{Editor, Menu, Scroll, Tapestry};
    use orbs_sim::Sim;

    /// Build the flags directly, so a state no verb can reach is still asked
    /// about. The point of the sweep below is the *ties*, and a tie is by
    /// definition a state the game is not supposed to be able to produce.
    ///
    /// `menuing` is not a parameter, deliberately: the ordering is
    /// `orbs_shell::focus`'s and is tested there against every combination. What
    /// this file tests is the *dispatch* — that a key reaches the surface the
    /// owner names.
    fn surfaces(editing: bool, weaving: bool, walking: bool) -> Surfaces {
        Surfaces {
            editing: editing.then(|| Editor::open("t", "laboratory", &[])),
            weaving: weaving.then(Tapestry::default),
            walking,
            menuing: None,
            reading: None,
            opening_manual: false,
            leaving: false,
            swapping: None,
            driving: None,
            setting: None,
        }
    }

    fn scroll(reading: bool) -> Scroll {
        let mut scroll = Scroll::default();
        if reading {
            scroll.read();
        }
        scroll
    }

    #[test]
    fn nothing_open_is_the_prompt() {
        assert_eq!(
            surfaces(false, false, false).owner(&scroll(false)),
            Owner::Prompt
        );
    }

    #[test]
    fn every_surface_is_reachable_on_its_own() {
        // A precedence chain hides its own last arm if an earlier one is always
        // true, and the arm that goes missing is the one added most recently.
        for (editing, weaving, walking, reading, want) in [
            (true, false, false, false, Owner::Editor),
            (false, true, false, false, Owner::Weave),
            (false, false, true, false, Owner::Maze),
            (false, false, false, true, Owner::Reading),
        ] {
            assert_eq!(
                surfaces(editing, weaving, walking).owner(&scroll(reading)),
                want,
                "{want:?} cannot be reached even when it is the only thing open",
            );
        }
    }

    #[test]
    fn exactly_one_surface_holds_the_keyboard_in_every_state() {
        // The property the module exists for, over all sixteen states. `owner`
        // returns one `Owner` by construction, so what is at stake is the answer
        // being *decided* rather than accidental: the winner is always the first
        // open surface in the documented order. A silent tie is the harder bug —
        // two surfaces acting on one keystroke is invisible until a player types
        // `:wq` and finds it in their command history.
        for bits in 0..16u8 {
            let (editing, weaving, walking, reading) =
                (bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
            let want = if editing {
                Owner::Editor
            } else if weaving {
                Owner::Weave
            } else if walking {
                Owner::Maze
            } else if reading {
                Owner::Reading
            } else {
                Owner::Prompt
            };
            assert_eq!(
                surfaces(editing, weaving, walking).owner(&scroll(reading)),
                want,
                "editing={editing} weaving={weaving} walking={walking} reading={reading}",
            );
        }
    }

    #[test]
    fn the_menu_takes_the_keys_and_leaving_is_a_choice_on_it() {
        // The one surface that can end the session, and the one whose keys never
        // touch the `Sim` — so this is the whole dispatch, with no world.
        let mut surfaces = surfaces(false, false, false);
        surfaces.menuing = Some(Menu::default());
        assert_eq!(
            surfaces.owner(&scroll(false)),
            Owner::Menu,
            "the menu did not take the keyboard from the prompt",
        );

        // It wins over everything, which is the tie `Focus::of` documents: the
        // way out is what a player meant.
        let mut over = self::surfaces(true, true, true);
        over.menuing = Some(Menu::default());
        assert_eq!(over.owner(&scroll(true)), Owner::Menu);

        // `resume` gives the keyboard back and leaves nothing behind.
        let mut scroll = scroll(false);
        let mut sim = Sim::new(1);
        for code in "resume".chars().map(KeyCode::Char) {
            surfaces.typed(
                Owner::Menu,
                code,
                &mut sim,
                &mut scroll,
                &orbs_sim::Verbatim,
            );
        }
        surfaces.typed(
            Owner::Menu,
            KeyCode::Enter,
            &mut sim,
            &mut scroll,
            &orbs_sim::Verbatim,
        );
        assert!(surfaces.menuing.is_none(), "resume did not close the menu");
        assert!(!surfaces.leaving, "resume asked to leave the orb");

        // ...and `quit` on the menu is what asks to leave, which `drive` reads.
        surfaces.menuing = Some(Menu::default());
        for code in "quit".chars().map(KeyCode::Char) {
            surfaces.typed(
                Owner::Menu,
                code,
                &mut sim,
                &mut scroll,
                &orbs_sim::Verbatim,
            );
        }
        surfaces.typed(
            Owner::Menu,
            KeyCode::Enter,
            &mut sim,
            &mut scroll,
            &orbs_sim::Verbatim,
        );
        assert!(surfaces.leaving, "quit on the menu did not ask to leave");
    }

    #[test]
    fn a_verb_hands_its_surface_over_once_and_not_twice() {
        // The take-once handshake from the frontend's side: the sim raises a
        // flag and reading it clears it, so a second `open` on the same tick
        // must find nothing or a closed surface reopens under the player.
        let mut sim = Sim::new(3);
        sim.submit("attend archive");
        sim.step();
        sim.submit("research");
        sim.step();
        sim.submit("wander");
        sim.step();

        let mut surfaces = Surfaces::default();
        let mut scroll = Scroll::default();
        surfaces.open(
            &mut sim,
            &mut scroll,
            1,
            orbs_shell::Driver::default(),
            false,
            orbs_render::DisplayMode::Wide,
            &orbs_sim::Verbatim,
        );
        assert_eq!(
            surfaces.owner(&scroll),
            Owner::Maze,
            "wander did not hand the keys over"
        );

        surfaces.walking = false;
        surfaces.open(
            &mut sim,
            &mut scroll,
            1,
            orbs_shell::Driver::default(),
            false,
            orbs_render::DisplayMode::Wide,
            &orbs_sim::Verbatim,
        );
        assert_eq!(
            surfaces.owner(&scroll),
            Owner::Prompt,
            "the wander request was still pending after being taken, so the maze reopened",
        );
    }

    #[test]
    fn a_maze_that_closes_takes_the_keyboard_with_it() {
        // §19's worst ending: holding the arrows over a pane with no map on it.
        // A spell can `stop lectern`, and the player is left walking nothing.
        let mut sim = Sim::new(3);
        for line in ["attend archive", "research", "wander"] {
            sim.submit(line);
            sim.step();
        }
        let mut surfaces = Surfaces::default();
        let mut scroll = Scroll::default();
        surfaces.open(
            &mut sim,
            &mut scroll,
            1,
            orbs_shell::Driver::default(),
            false,
            orbs_render::DisplayMode::Wide,
            &orbs_sim::Verbatim,
        );
        assert_eq!(surfaces.owner(&scroll), Owner::Maze);

        sim.submit("stop stacks");
        sim.step();
        surfaces.tick(&sim);
        assert_eq!(
            surfaces.owner(&scroll),
            Owner::Prompt,
            "the stacks closed and the arrows still belonged to the maze",
        );
    }
}
