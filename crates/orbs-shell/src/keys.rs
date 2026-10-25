//! The prompt's key table — one of them, for both frontends.
//!
//! *Finding* a keystroke is backend-shaped and stays in each frontend; what it
//! *means* is not, and two copies get it subtly wrong. So each frontend maps
//! its events onto [`Key`] and calls [`apply`].

use orbs_sim::Sim;
use orbs_sim::tower::Way;

use crate::editor::{Editor, Outcome as EditorOutcome};
use crate::manual::Outcome as ManualOutcome;
use crate::menu::{Menu, Outcome as MenuOutcome};
use crate::tapestry::{Outcome as WeaveOutcome, Tapestry};

use crate::line::Line;
use crate::offering::Offered;

/// A keystroke, as the prompt understands it.
///
/// Deliberately small and deliberately not a `KeyCode`: this is the vocabulary
/// of a command line, not of a keyboard. A frontend that cannot produce one of
/// these simply never sends it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    /// Finish the line.
    Enter,
    /// Delete the character before the caret.
    Backspace,
    /// Abandon the line.
    Escape,
    /// Move the caret.
    Left,
    /// Move the caret.
    Right,
    /// Older in the history.
    Up,
    /// Newer in the history.
    Down,
    /// Caret to the start.
    Home,
    /// Caret to the end.
    End,
    /// Complete, or cycle the completion.
    Tab,
    /// A screenful back.
    ///
    /// For the manual, which must take it before the transcript's own paging,
    /// or both page together.
    PageUp,
    /// A screenful on. See [`PageUp`](Self::PageUp).
    PageDown,
    /// Characters to insert.
    ///
    /// Whatever the platform decided the key means, not a `char`: a Windows dead
    /// key yields two characters from one press, and the space bar is text
    /// rather than a named key everywhere. `Line::insert` filters through the
    /// CP437 repertoire, so a control character cannot reach a cell.
    Text(String),
}

/// Apply one keystroke to the prompt.
///
/// Returns the finished line when Enter was pressed, and `None` otherwise. The
/// caller submits it — this function never touches the world, because a
/// keystroke reaching a decision is what §19 says keeps the editor a frontend's
/// and the save the sim's.
pub fn apply(key: &Key, line: &mut Line, offered: &mut Offered, sim: &Sim) -> Option<String> {
    // Any keystroke retires the last Tab listing — it answered a question the
    // player has already moved past. A Tab cycle ends with it for the same
    // reason: the next Tab should start a fresh completion rather than resume
    // one the player has typed past.
    if !matches!(key, Key::Tab) {
        offered.clear();
        line.end_cycle();
    }

    match key {
        Key::Enter => {
            // An answer to §6's prompt is not remembered. Sampled before `take`,
            // since `submit` clears `Choices`.
            let remember = !answering(sim, line.text());
            return Some(line.take(remember));
        }
        Key::Backspace => line.backspace(),
        Key::Escape => line.clear(),
        Key::Left => line.left(),
        Key::Right => line.right(),
        Key::Up => line.earlier(),
        Key::Down => line.later(),
        Key::Home => line.home(),
        Key::End => line.end(),
        Key::Tab => {
            let open = !sim.choices().is_empty();
            // Candidates are transient Frame content, never a record (§13).
            // Assigned even when empty, so a completing Tab retires the listing.
            offered.options = line.tab(sim.scene(), open);
            offered.current = line.cycling();
        }
        Key::Text(text) => line.insert(text),
        // The prompt never sees these: each frontend answers PgUp/PgDn before
        // the surface dispatch so the transcript pages from anywhere. They are
        // in `Key` for the manual.
        Key::PageUp | Key::PageDown => {}
    }
    None
}

/// Whether this line is answering a numbered prompt rather than phrasing a
/// command.
///
/// Through the sim's own predicate, not a second copy of it. What counts as an
/// answer is §6's rule — a leading `#`, a `1)`, a range would all be changes to
/// it — and the sim already exports the canonical form.
fn answering(sim: &Sim, line: &str) -> bool {
    !sim.choices().is_empty() && orbs_sim::parser::is_answer(line)
}

/// One keystroke, to the spell editor.
///
/// Shared for the module header's reason, as each surface's table is.
pub fn apply_to_editor(
    key: &Key,
    editor: &mut Editor,
    sim: &orbs_sim::Sim,
) -> Option<EditorOutcome> {
    // A Tab cycle ends on anything but Tab. `apply`'s rule, unshared because
    // `Offered` is the prompt's alone.
    if !matches!(key, Key::Tab) {
        editor.end_cycle();
    }
    let outcome = if matches!(key, Key::Tab) {
        editor.tab(sim);
        None
    } else {
        apply_key(key, editor)
    };
    // The keystroke beat, and the only place the guide is worked out. It reaches
    // `scene_at` — every recipe, topic and node — so a painter doing it would
    // run that at 60 Hz; here it runs when something it depends on changed,
    // which `offering` and `editing` each learned once.
    editor.refresh(sim);
    outcome
}

/// One key, applied. See [`apply_to_editor`], which is this plus the refresh.
fn apply_key(key: &Key, editor: &mut Editor) -> Option<EditorOutcome> {
    match key {
        // `or`, not `=`, so a second Enter in one delivery cannot discard a
        // pending `SaveAndClose`.
        Key::Enter => editor.enter(),
        Key::Escape => {
            editor.escape();
            None
        }
        Key::Backspace => {
            editor.backspace();
            None
        }
        Key::Left => {
            editor.left();
            None
        }
        Key::Right => {
            editor.right();
            None
        }
        Key::Up => {
            editor.up();
            None
        }
        Key::Down => {
            editor.down();
            None
        }
        Key::Home => {
            editor.home();
            None
        }
        Key::End => {
            editor.end();
            None
        }
        Key::Text(text) => {
            editor.type_text(text);
            None
        }
        // Taken by `apply_to_editor`, which is the only caller — completion
        // needs the `Sim` this table deliberately does not have.
        //
        // PgUp/PgDn are answered above the surface dispatch in both builds, so
        // the editor never sees them either — see `Key::PageUp`.
        Key::Tab | Key::PageUp | Key::PageDown => None,
    }
}

/// One keystroke, to the orb's menu.
///
/// See [`apply_to_editor`] for why these live here.
///
/// The arrows do nothing, which is the whole difference from [`apply_to_weave`]:
/// there is nothing on this screen to walk, and every choice is a word.
pub fn apply_to_menu(key: &Key, menu: &mut Menu) -> Option<MenuOutcome> {
    match key {
        Key::Enter => menu.enter(),
        Key::Escape => menu.escape(),
        Key::Backspace => {
            menu.backspace();
            None
        }
        Key::Text(text) => {
            menu.type_text(text);
            None
        }
        Key::Up
        | Key::Down
        | Key::Left
        | Key::Right
        | Key::Home
        | Key::End
        | Key::Tab
        | Key::PageUp
        | Key::PageDown => None,
    }
}

/// One keystroke, to the manual.
///
/// The arrows scroll, as in any reader; the line has no history behind them.
pub fn apply_to_manual(key: &Key, reader: &mut crate::manual::Reader) -> Option<ManualOutcome> {
    match key {
        Key::Enter => reader.enter(),
        Key::Escape => reader.escape(),
        Key::Backspace => {
            reader.backspace();
            None
        }
        Key::Text(text) => {
            reader.type_text(text);
            None
        }
        Key::Up => {
            reader.scroll(1, false);
            None
        }
        Key::Down => {
            reader.scroll(1, true);
            None
        }
        Key::PageUp => {
            reader.scroll(reader.page(), false);
            None
        }
        Key::PageDown => {
            reader.scroll(reader.page(), true);
            None
        }
        Key::Home | Key::End | Key::Left | Key::Right | Key::Tab => None,
    }
}

/// One keystroke, to the progression screen.
///
/// See [`apply_to_editor`] for why these live here.
pub fn apply_to_weave(key: &Key, screen: &mut Tapestry) -> Option<WeaveOutcome> {
    match key {
        Key::Enter => screen.enter(),
        Key::Escape => {
            screen.escape();
            None
        }
        Key::Backspace => {
            screen.backspace();
            None
        }
        Key::Up => {
            screen.step(0, -1);
            None
        }
        Key::Down => {
            screen.step(0, 1);
            None
        }
        Key::Left => {
            screen.step(-1, 0);
            None
        }
        Key::Right => {
            screen.step(1, 0);
            None
        }
        Key::Text(text) => {
            screen.type_text(text);
            None
        }
        Key::Home | Key::End | Key::Tab | Key::PageUp | Key::PageDown => None,
    }
}

/// One keystroke, to the archive's map — which way the reading walks.
///
/// See [`apply_to_editor`] for why these live here. Both frontends and the dump
/// read it.
#[must_use]
pub const fn apply_to_maze(key: &Key) -> Option<Way> {
    match key {
        Key::Up => Some(Way::North),
        Key::Right => Some(Way::East),
        Key::Down => Some(Way::South),
        Key::Left => Some(Way::West),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter_hands_back_the_line_and_clears_it() {
        let sim = Sim::new(0);
        let mut line = Line::default();
        let mut offered = Offered::default();
        for key in "survey".chars() {
            apply(&Key::Text(key.to_string()), &mut line, &mut offered, &sim);
        }
        assert_eq!(line.text(), "survey");

        let finished = apply(&Key::Enter, &mut line, &mut offered, &sim);
        assert_eq!(finished.as_deref(), Some("survey"));
        assert_eq!(line.text(), "", "the line kept what was submitted");
    }

    #[test]
    fn an_ordinary_key_retires_the_tab_listing_and_tab_does_not() {
        // The rule both frontends have to share: a listing answers a question,
        // and any key that is not another Tab means the player moved past it.
        let sim = Sim::new(0);
        let mut line = Line::default();
        let mut offered = Offered::default();

        apply(&Key::Tab, &mut line, &mut offered, &sim);
        assert!(!offered.is_empty(), "Tab offered nothing to retire");

        apply(&Key::Tab, &mut line, &mut offered, &sim);
        assert!(!offered.is_empty(), "a second Tab retired its own listing");

        apply(&Key::Text("a".to_owned()), &mut line, &mut offered, &sim);
        assert!(offered.is_empty(), "a keystroke left the listing up");
    }

    #[test]
    fn an_answer_to_a_numbered_prompt_is_not_remembered() {
        // §6's rule, and the reason `answering` is sampled before `take`: an
        // answer is not a phrasing, so recall history must not fill with digits.
        let mut sim = Sim::new(0);
        // `purge` alone is ambiguous, which is what opens a numbered prompt.
        sim.submit("purge");
        sim.step();
        assert!(!sim.choices().is_empty(), "no prompt to answer");

        let mut line = Line::default();
        let mut offered = Offered::default();
        apply(&Key::Text("1".to_owned()), &mut line, &mut offered, &sim);
        apply(&Key::Enter, &mut line, &mut offered, &sim);

        apply(&Key::Up, &mut line, &mut offered, &sim);
        assert_ne!(line.text(), "1", "an answer went into the history");
    }

    #[test]
    fn escape_abandons_the_line_without_submitting_it() {
        let sim = Sim::new(0);
        let mut line = Line::default();
        let mut offered = Offered::default();
        apply(&Key::Text("stat".to_owned()), &mut line, &mut offered, &sim);
        assert_eq!(apply(&Key::Escape, &mut line, &mut offered, &sim), None);
        assert_eq!(line.text(), "");
    }
}
