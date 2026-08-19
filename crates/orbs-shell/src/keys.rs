//! The prompt's key table — one of them, for both frontends.
//!
//! # Why this is shared and the routing around it is not
//!
//! Everything about *finding* a keystroke is backend-shaped and unshareable:
//! winit has a press/release model, key repeat, held modifiers and a focus
//! notion, and a terminal has none of those — it hands over discrete events
//! carrying their own modifiers. `orbs`'s `HeldOver`, `Quiet` and `chord_is_stale`
//! exist entirely to make sense of the first, and would be a workaround for a
//! problem the second does not have.
//!
//! What a keystroke *means* is not backend-shaped at all. Enter submits and
//! remembers unless the line is answering a numbered prompt; Tab completes and
//! keeps its cycle; every other key retires the listing. Those are the rules a
//! second implementation would get subtly wrong, and the way you would find out
//! is a player typing `:wq` and finding it in their command history.
//!
//! So each frontend maps its own events onto [`Key`] and calls [`apply`].

use orbs_sim::Sim;

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
    /// Characters to insert.
    ///
    /// **Whatever the platform decided the key means**, not a `char` — a Windows
    /// dead key yields two characters from one press, and the space bar is text
    /// rather than a named key on every backend. `Line::insert` filters this
    /// through the CP437 repertoire, so a control character cannot reach a cell.
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
            // A bare digit answering §6's numbered prompt is an answer, not a
            // phrasing, so it is not remembered. **Sampled before `take`**, and
            // the caller must submit before asking the world anything else:
            // `submit` clears `Choices`, so an `answering` read afterwards is
            // always false and every answer would land in the history.
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
            // Candidates are **transient Frame content**, not a record. There is
            // deliberately no `scrollback_mut` (§13): a frontend writing into the
            // log makes a session that `(seed, submissions)` cannot replay, and a
            // Tab press is not a submission.
            //
            // Assigned even when empty, so a Tab that *completes* a word retires
            // the listing that asked which word it was.
            offered.options = line.tab(sim.scene(), open);
            offered.current = line.cycling();
        }
        Key::Text(text) => line.insert(text),
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
