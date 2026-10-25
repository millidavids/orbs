//! What can be open, and keeping it current with the world.

use orbs_shell::{Editor, Menu, Scroll, Tapestry};
use orbs_sim::Sim;

/// The surfaces that can hold the keyboard, in the order they take it.
///
/// Declared here once and `orbs-shell`'s now: the ordering was right, and being
/// right in one frontend was the problem, because the Bevy build carried its own
/// four-term expression of the same rule. The alias stays because a terminal
/// does own the *dispatch*, even though it no longer owns the *ordering*.
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
    /// `typed` returns nothing because no surface can end the session, and the
    /// menu is the one that can. Changing that signature would put an `Option`
    /// on five arms that never fill it.
    pub(crate) leaving: bool,
    /// A tower the menu asked for, waiting for `drive::run` to build it.
    ///
    /// Raised here and acted on there, for `leaving`'s reason and one more:
    /// putting a different `Sim` in front of the player means rebuilding the
    /// whole `Session`, the terminal's answer to the Bevy build's
    /// eighteen-resource reset — a new struct forgets no derived field.
    pub(crate) swapping: Option<orbs_shell::Raising>,
    /// Whether the menu has asked for the manual, waiting for the book.
    ///
    /// A flag for `leaving`'s reason: `menu_took` takes no `Sim` on purpose, and
    /// the book is assembled from one. `Surfaces::open` has both.
    pub(crate) opening_manual: bool,
    /// A driver the options page chose, waiting for the session to honour it.
    ///
    /// Raised here and acted on there, for `swapping`'s reason: the reader
    /// belongs to the `Session`, and a surface reaching into it would be the
    /// shared half reaching for the backend's. The choice is already written
    /// down by then; this only makes it true for the running session.
    pub(crate) driving: Option<orbs_shell::Driver>,
    /// A setting the menu changed, waiting for the session to honour it.
    ///
    /// [`driving`](Self::driving)'s shape, since rows flow menu-ward only:
    /// `Outcome::Set` is honoured now, not at the next launch.
    pub(crate) setting: Option<(String, String)>,
}

impl Surfaces {
    /// Who the next keystroke belongs to.
    ///
    /// The tie-break and its reasoning live with the enum in `orbs-shell`; this
    /// only says what *this* frontend has open. Both builds ask the one
    /// function, so neither can drift into answering it differently.
    pub(crate) const fn owner(&self, scroll: &Scroll) -> Owner {
        Owner::of(self.held_open(scroll))
    }

    /// What this frontend has open, for `Focus::of` and for a crossing's
    /// `Showing::of` alike — one reading, so the keyboard and the screen cannot
    /// disagree about which surface is up.
    pub(crate) const fn held_open(&self, scroll: &Scroll) -> orbs_shell::Open {
        orbs_shell::Open {
            editing: self.editing.is_some(),
            weaving: self.weaving.is_some(),
            walking: self.walking,
            reading: scroll.is_reading(),
            menuing: self.menuing.is_some(),
            reading_manual: self.reading.is_some(),
        }
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
        held: crate::carried::Carried,
        scrivener: &dyn orbs_sim::Scrivener,
    ) {
        if self.editing.is_none()
            && let Some(request) = sim.opening()
        {
            self.editing = Some(Editor::opened(&request, sim, scrivener));
        }
        if self.weaving.is_none() && sim.weaving() {
            let mut screen = Tapestry::default();
            screen.read_from(sim);
            self.weaving = Some(screen);
        }
        if !self.walking && sim.wandering() {
            self.walking = true;
        }
        // `menu`, not `quit`: the two were one word for an iteration (§19).
        // `quit`'s flag is the session's, set only by a *confirmed* one.
        if self.menuing.is_none() && sim.menuing() {
            // The options page marks what is *in effect*, which the session
            // holds — see `Menu::open_at`. `InTower`, because the word is typed
            // at a prompt: the threshold's menu is `Session::new`'s.
            self.menuing = Some(Menu::open_at(
                orbs_shell::Stance::InTower,
                held.driver(),
                crate::carried::settings(&held),
            ));
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
            screen.read_from(sim);
        }
        if self.walking && sim.stacks().is_none() {
            self.walking = false;
        }
        // §3: the menu's secret words are silent while the tower is tense, and
        // F7 can change that with the menu up.
        if let Some(menu) = &mut self.menuing {
            menu.show_register(sim.register());
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
            super::dispatch::save(editor, sim, scrivener);
        }
    }
}
