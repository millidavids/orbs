//! Which of the five things on screen has the keyboard, and what it does with it.
//!
//! The Bevy build spends four resources, four run conditions and a four-term
//! predicate on this, and its own comment predicts the ceiling: *"a fifth surface
//! refactors this first."* Here it is one enum and a `match`, because a terminal
//! delivers one keystroke at a time to whoever is asking — there is no message
//! queue to leave keys waiting in, and so no need to *run* in order to throw them
//! away.
//!
//! What is **not** simpler is the state machines themselves, and they are not
//! reimplemented: [`Editor`] and [`Tapestry`] are `orbs-shell`'s, and this
//! decides only which of them a key reaches.

use crossterm::event::KeyCode;
use orbs_shell::{Editor, EditorOutcome, Menu, MenuOutcome, Scroll, Tapestry};
use orbs_sim::Sim;

/// The surfaces that can hold the keyboard, in the order they take it.
///
/// **This was declared here and is `orbs-shell`'s now.** The enum and its
/// ordering were right, and being right in one frontend was the problem: the
/// Bevy build carried its own four-term expression of the same rule until a
/// fifth surface forced the refactor its own comment had promised. The alias is
/// kept because `Owner` is what this file's prose calls it, and because a
/// terminal really does own the *dispatch* below even though it no longer owns
/// the *ordering*.
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
    /// Whether the arrow keys are answering a chant.
    pub(crate) chorusing: bool,
    /// The orb's menu, if `quit` has opened it.
    pub(crate) menuing: Option<Menu>,
    /// Whether the menu asked for the orb to be put down.
    ///
    /// **A flag rather than a return value**, unlike every other surface here:
    /// [`typed`](Self::typed)'s doc says *"nothing is returned because no
    /// surface can end the session"*, and the menu is the one that can. Changing
    /// that signature would put an `Option` on five arms that will never fill
    /// it; `drive` reads this instead, next to the `Quitting` it already reads.
    pub(crate) leaving: bool,
    /// A tower the menu asked for, waiting for `drive::run` to build it.
    ///
    /// **Raised here and acted on there**, for `leaving`'s reason and one more:
    /// putting a different `Sim` in front of the player means rebuilding the
    /// whole `Session`, which is the terminal's answer to the Bevy build's
    /// eighteen-resource reset — every derived field goes back to its default
    /// because it is a *new struct*, and none of them can be forgotten.
    pub(crate) swapping: Option<Swap>,
    /// A driver the options page chose, waiting for `drive::run` to honour it.
    ///
    /// **Raised here and acted on there**, for `swapping`'s reason: the reader
    /// belongs to the `Session` and a surface reaching into it would be the
    /// shared half reaching for the backend's. The choice is already written
    /// down by the time this is set — this only makes it true for the session
    /// already running.
    pub(crate) driving: Option<orbs_shell::Driver>,
}

/// A tower the menu asked for.
#[derive(Debug, Clone)]
pub(crate) struct Swap {
    /// Where it is kept, and where it will be written back.
    pub(crate) path: std::path::PathBuf,
    /// The length a *new* tower is to be, or `None` to load what is at `path`.
    pub(crate) length: Option<orbs_sim::content::Length>,
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
            chorusing: self.chorusing,
            reading: scroll.is_reading(),
            menuing: self.menuing.is_some(),
        })
    }

    /// Take whatever the world has offered since the last tick.
    ///
    /// Each of these is a **take-once handshake**: the sim raises a flag when a
    /// verb asks for a surface, and reading it clears it. The Bevy build needs a
    /// peek-then-take pair because `&mut` through a `ResMut` stamps a change tick
    /// and defeats its own `resource_changed` guards; a loop with no change
    /// detection just takes.
    pub(crate) fn open(
        &mut self,
        sim: &mut Sim,
        scroll: &mut Scroll,
        page: usize,
        driver: orbs_shell::Driver,
    ) {
        if self.editing.is_none()
            && let Some(request) = sim.opening()
        {
            let mut editor = Editor::open(&request.name, &request.domain, &request.lines);
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
        if !self.chorusing && sim.chorusing() {
            self.chorusing = true;
        }
        // **The figure can end without anybody pressing Escape** — it runs out,
        // or it collapses — so the keys have to come back on their own. The maze
        // never does that, which is why this line has no sibling above it.
        if self.chorusing && sim.figure().is_none() {
            self.chorusing = false;
        }
        // **`menu`, not `quit`.** The two were one word for an iteration; §19
        // has why that was superseded. `quit`'s flag is `drive`'s, and it is
        // only ever set by a *confirmed* one.
        if self.menuing.is_none() && sim.menuing() {
            // The options page marks what is *in effect*, which the session
            // holds — see `Menu::show_driver`.
            let mut menu = Menu::default();
            menu.show_driver(driver);
            self.menuing = Some(menu);
        }
        if !scroll.is_reading() && sim.unfurling() {
            scroll.read();
            // **And page back, which this build did not.** `unfurl` that only
            // takes the keyboard shows the player the screen they were already
            // looking at — nothing moves, and the only thing that changes is the
            // border hint. The Bevy build's `start_reading` pages by one screen
            // on open for exactly that reason, so the word does something the
            // moment it is typed.
            let total = sim.scrollback().records().drawn_len();
            scroll.page(page, true, total);
        }
    }

    /// Keep an open surface current with a world that moved under it.
    ///
    /// All three are unconditional while their surface is open, matching the
    /// Bevy build exactly: a threshold crossed while a player is looking at the
    /// weave screen should land while they look, a spell's running line moves
    /// under the editor, and **a spell can close the maze from under the
    /// player** — holding the keyboard over a pane with no map on it is the
    /// worst of the ways that ends.
    pub(crate) fn tick(&mut self, sim: &Sim) {
        if let Some(editor) = &mut self.editing {
            editor.set_running_line(sim.running_line(editor.name()));
            editor.set_reading(sim.read_spell(editor.domain(), editor.lines()));
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
    pub(crate) fn settle(&mut self, delta: f32, sim: &mut Sim) {
        let Some(editor) = &mut self.editing else {
            return;
        };
        if editor.settle(delta) {
            save(editor, sim);
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
    ) {
        match owner {
            // The prompt is the caller's; it needs the shared key table and the
            // line, neither of which belongs to a surface.
            Owner::Prompt => {}
            Owner::Menu => self.menu_took(code),
            Owner::Editor => self.editing_took(code, sim),
            Owner::Weave => self.weaving_took(code, sim),
            Owner::Maze => self.maze_took(code, sim),
            Owner::Chant => self.chant_took(code, sim),
            // **No `page` any more.** `PageUp`/`PageDown` are answered above the
            // surface dispatch now, so the only stepping left in here is the
            // arrows, which move one record.
            Owner::Reading => read(code, scroll, sim),
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
            // **Raised, not acted on.** Putting the terminal back is `drive`'s —
            // raw mode is its to undo, and a surface reaching for it would be
            // the backend leaking into the shared half.
            Some(MenuOutcome::PutDown) => self.leaving = true,
            // ...and the same for a swap, which needs a whole new `Session`.
            Some(MenuOutcome::Load(path)) => {
                self.swapping = Some(Swap { path, length: None });
            }
            Some(MenuOutcome::Begin { path, length }) => {
                self.swapping = Some(Swap {
                    path,
                    length: Some(length),
                });
            }
            Some(MenuOutcome::Drive(driver)) => self.driving = Some(driver),
            None => {}
        }
    }

    fn editing_took(&mut self, code: KeyCode, sim: &mut Sim) {
        let Some(editor) = &mut self.editing else {
            return;
        };
        // **Enter and Escape mean something in both of the editor's states, and
        // the editor decides which.** Branching on the mode here would be a
        // second copy of that state machine, in the one place neither the tests
        // nor `ORBS_DUMP` can reach.
        //
        // And the table itself is `orbs-shell`'s, for the same reason the
        // prompt's is: what a key *means* to an editor is not backend-shaped.
        // This was a second copy, and it had already missed the correction the
        // other one carries — two Enters in a delivery discarding a pending
        // `SaveAndClose`.
        let Some(key) = crate::drive::as_key(code) else {
            return;
        };
        let outcome = orbs_shell::apply_to_editor(&key, editor, sim);

        match outcome {
            Some(EditorOutcome::Save) => save(editor, sim),
            Some(EditorOutcome::SaveAndClose) => {
                save(editor, sim);
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

    /// Answer a syllable on the arrows.
    ///
    /// The maze's shape below, and the same two rules: `orbs-shell` owns the
    /// key-to-syllable table so the two builds cannot disagree, and the press
    /// reaches the world **now** rather than through `submit` — a key that
    /// queued would arrive after the beat it was answering.
    fn chant_took(&mut self, code: KeyCode, sim: &mut Sim) {
        if code == KeyCode::Esc {
            self.chorusing = false;
            return;
        }
        let Some(syllable) = crate::drive::as_key(code)
            .as_ref()
            .and_then(orbs_shell::apply_to_chant)
        else {
            return;
        };
        sim.sing(syllable);
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
        // **`walk`, not `submit` and `step`.** A tick per arrow would mean a
        // player walks as slowly as the world moves, and eight presses would
        // advance the tower eight seconds — a brew finishing while somebody
        // reads a map. `Sim::walk` is the third entry point and spends no time.
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
fn save(editor: &mut Editor, sim: &mut Sim) {
    let (name, lines) = (editor.name().to_owned(), editor.lines().to_vec());
    editor.saved();
    sim.write_spell(&name, &lines);
}

/// The transcript, while `unfurl` has the keyboard.
///
/// **`page` is a record count the painter measured**, not a row count: a record
/// costs *at least* one row, so paging by the pane's rows moves more than a
/// screenful and drops the lines in between.
fn read(code: KeyCode, scroll: &mut Scroll, sim: &Sim) {
    let total = sim.scrollback().records().drawn_len();
    match code {
        KeyCode::Esc => scroll.stop_reading(),
        // **`PageUp`/`PageDown` are deliberately absent**: `drive::typed`
        // answers them above the surface dispatch, ungated, exactly as the Bevy
        // build does — so a second copy here would be two expressions of one
        // rule, which is the shape of half the defects in §19.
        //
        // **The arrows are here because the other build binds them.**
        // `scroll_back`/`scroll_forward` run on ArrowUp/ArrowDown gated on
        // `editing::reading`, and this build shipped without them — so a reader
        // could page but not step, and the one-record nudge that lines two
        // entries up for comparison did not exist.
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
    /// **`chorusing` and `menuing` are not parameters**, and that is deliberate:
    /// the ordering they take part in is `orbs_shell::focus`'s and is tested
    /// there, against every combination. What this file tests is the *dispatch*
    /// — that a key reaches the surface the owner names — which the four below
    /// cover, and which `the_menu_takes_the_keys_and_leaving_is_a_choice_on_it`
    /// covers for the fifth.
    fn surfaces(editing: bool, weaving: bool, walking: bool) -> Surfaces {
        Surfaces {
            editing: editing.then(|| Editor::open("t", "laboratory", &[])),
            weaving: weaving.then(Tapestry::default),
            walking,
            chorusing: false,
            menuing: None,
            leaving: false,
            swapping: None,
            driving: None,
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
        // **The property the whole module exists for, over all sixteen states.**
        // `owner` returns one `Owner` by construction, so what is actually at
        // stake is that the answer is *decided* rather than accidental: for
        // every combination the winner is the first open surface in the
        // documented order, and no later arm can ever steal a key from an
        // earlier one.
        //
        // The header calls a silent tie "the harder bug to find", and it would
        // be: two surfaces both acting on one keystroke is invisible until a
        // player types `:wq` and finds it in their command history.
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
        // **The one surface that can end the session**, and the one whose keys
        // never touch the `Sim` — so this is the whole dispatch, with no world.
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
            surfaces.typed(Owner::Menu, code, &mut sim, &mut scroll);
        }
        surfaces.typed(Owner::Menu, KeyCode::Enter, &mut sim, &mut scroll);
        assert!(surfaces.menuing.is_none(), "resume did not close the menu");
        assert!(!surfaces.leaving, "resume asked to leave the orb");

        // ...and `quit` on the menu is what asks to leave, which `drive` reads.
        surfaces.menuing = Some(Menu::default());
        for code in "quit".chars().map(KeyCode::Char) {
            surfaces.typed(Owner::Menu, code, &mut sim, &mut scroll);
        }
        surfaces.typed(Owner::Menu, KeyCode::Enter, &mut sim, &mut scroll);
        assert!(surfaces.leaving, "quit on the menu did not ask to leave");
    }

    #[test]
    fn a_verb_hands_its_surface_over_once_and_not_twice() {
        // **The take-once handshake, from the frontend's side.** The sim raises
        // a flag and reading it clears it; a second `open` on the same tick must
        // find nothing, or a surface closed by the player reopens under them on
        // the very next frame.
        let mut sim = Sim::new(3);
        sim.submit("attend archive");
        sim.step();
        sim.submit("research");
        sim.step();
        sim.submit("wander");
        sim.step();

        let mut surfaces = Surfaces::default();
        let mut scroll = Scroll::default();
        surfaces.open(&mut sim, &mut scroll, 1, orbs_shell::Driver::default());
        assert_eq!(
            surfaces.owner(&scroll),
            Owner::Maze,
            "wander did not hand the keys over"
        );

        surfaces.walking = false;
        surfaces.open(&mut sim, &mut scroll, 1, orbs_shell::Driver::default());
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
        surfaces.open(&mut sim, &mut scroll, 1, orbs_shell::Driver::default());
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
