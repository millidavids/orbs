//! A keystroke, to whichever surface holds the keyboard.

use crossterm::event::KeyCode;
use orbs_shell::{Editor, EditorOutcome, MenuOutcome, Scroll};
use orbs_sim::Sim;

use super::state::{Owner, Surfaces, Swap};
use crate::keymap::as_key;

impl Surfaces {
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
            Owner::Menu => {
                // Per keystroke, as the Bevy build has it: `F7` can turn the
                // tower tense with the menu up, and a secret answered in that
                // beat punctures it (§3). A tick is too late.
                if let Some(menu) = &mut self.menuing {
                    menu.show_register(sim.register());
                }
                self.menu_took(code);
            }
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
        let Some(key) = as_key(code) else {
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
        let Some(key) = as_key(code) else {
            return;
        };
        match orbs_shell::apply_to_menu(&key, menu) {
            Some(MenuOutcome::Close) => self.menuing = None,
            // Raised, not acted on: putting the terminal back is the loop's, and
            // a surface reaching for raw mode leaks the backend into the shared
            // half.
            Some(MenuOutcome::PutDown) => self.leaving = true,
            // ...and the same for a swap, which needs a whole new `Session`.
            Some(MenuOutcome::Load(path)) => self.swapping = Some(Swap::Load(path)),
            Some(MenuOutcome::Begin { keep, length }) => {
                self.swapping = Some(Swap::Begin { keep, length });
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
        let Some(key) = as_key(code) else {
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
        let Some(key) = as_key(code) else {
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
        let Some(way) = as_key(code).as_ref().and_then(orbs_shell::apply_to_maze) else {
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
pub(super) fn save(editor: &mut Editor, sim: &mut Sim, scrivener: &dyn orbs_sim::Scrivener) {
    let (name, domain, lines) = (
        editor.name().to_owned(),
        editor.domain().to_owned(),
        editor.lines().to_vec(),
    );
    editor.saved();
    sim.write_spell_reading(&name, &lines, scrivener);
    editor.set_reading(sim.read_spell_with(&name, &domain, &lines, scrivener));
}

/// The transcript, while `unfurl` has the keyboard — the shared table, which
/// the window reads too.
fn read(code: KeyCode, scroll: &mut Scroll, sim: &Sim) {
    let Some(key) = as_key(code) else {
        return;
    };
    let total = sim.scrollback().records().drawn_len();
    orbs_shell::apply_to_reading(&key, scroll, total);
}
