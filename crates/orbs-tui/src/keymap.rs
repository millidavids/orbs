//! A crossterm keystroke, as the shared tables understand it.
//!
//! Mapping a backend's events onto [`Key`] is the genuinely backend-shaped half;
//! what each surface *does* with one is `orbs-shell`'s.

use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use orbs_shell::Key;

/// One crossterm key, as every shared table understands it.
///
/// A free function, because all four surfaces need it and not only the prompt.
pub(crate) fn as_key(code: KeyCode) -> Option<Key> {
    Some(match code {
        KeyCode::Enter => Key::Enter,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Esc => Key::Escape,
        KeyCode::Left => Key::Left,
        KeyCode::Right => Key::Right,
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        // Shift+Tab is its own code here and plain Tab under winit, which reads
        // the logical key. Dropped, it completed nothing in one build only.
        KeyCode::Tab | KeyCode::BackTab => Key::Tab,
        // See `orbs/src/shell/input.rs`: both mappers were missing these for a
        // whole version while the manual drew `pgdn for more` and PageDown did
        // nothing. `_ => return None` swallows a new variant without a murmur.
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::Char(glyph) => Key::Text(glyph.to_string()),
        _ => return None,
    })
}

/// The longest gap that still counts as one held arrow rather than two presses.
///
/// The fallback where no `KeyEventKind::Repeat` arrives. Sixty catches 30-40 ms
/// auto-repeat and spares `scripts/tui.sh`'s 100 ms presses; a slower repeat
/// rate breaks through.
const HELD_OVER: Duration = Duration::from_millis(60);

/// When a surface last handed the keyboard back, while arrows keep arriving.
///
/// `HeldOver` for a terminal, which sends no release: an unbroken run of
/// arrows out of a handover is a held key, and the first gap past
/// [`HELD_OVER`] is the release.
#[derive(Debug, Default)]
pub(crate) struct HeldOver {
    since: Option<Instant>,
}

impl HeldOver {
    /// A surface had this keystroke, so arrows arriving next may be its.
    pub(crate) fn handed_over(&mut self) {
        self.since = Some(Instant::now());
    }

    /// Whether this key is auto-repeat left over from a surface that has closed.
    ///
    /// Only the arrows are swallowed: they are the keys a surface holds down and
    /// the only ones whose prompt meaning — recall history — silently rewrites
    /// the command line, where a letter arriving this way is visible and can be
    /// deleted.
    pub(crate) fn swallows(&mut self, event: &KeyEvent) -> bool {
        let arrow = matches!(
            event.code,
            KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right
        );
        let Some(since) = self.since else {
            return false;
        };
        if !arrow {
            // A key nobody holds down to walk with. They have let go.
            self.since = None;
            return false;
        }
        // When the terminal says so, believe it: with keyboard-enhancement flags
        // active crossterm reports `Repeat` outright, which is what a key-release
        // event would have given — no timing involved.
        if event.kind == KeyEventKind::Repeat {
            self.since = Some(Instant::now());
            return true;
        }
        let now = Instant::now();
        if now.duration_since(since) <= HELD_OVER {
            self.since = Some(now);
            return true;
        }
        self.since = None;
        false
    }
}

/// The keystroke hiding behind an `Alt+<letter>`, if that is what this is.
///
/// See the note at the top of `Session::typed`. `Some(rest)` means the
/// terminal sent `\x1b` and a byte in one read and the pair should be replayed
/// as Escape, then `rest`. Restricted to [`KeyCode::Char`] because that is the
/// only shape crossterm builds from an escape-prefixed byte; every other Alt
/// chord is CSI-encoded and is one the player really pressed.
pub(crate) fn escape_prefixed(key: &KeyEvent) -> Option<KeyEvent> {
    if !key.modifiers.contains(KeyModifiers::ALT) {
        return None;
    }
    if !matches!(key.code, KeyCode::Char(_)) {
        return None;
    }
    let mut rest = *key;
    rest.modifiers.remove(KeyModifiers::ALT);
    Some(rest)
}

/// Whether this key ends the session, asked while the boot card has the screen.
///
/// Not `Session::typed`, which routes to surfaces, the prompt and the
/// shortcut table, none of which exist yet. §19's removed keypress skip is not
/// being put back — leaving closes the session rather than jumping to the tower
/// — and this exists because raw mode takes away the window manager's way out.
pub(crate) fn leaving(key: &KeyEvent) -> bool {
    if key.code == KeyCode::F(10) {
        return true;
    }
    key.modifiers
        .intersects(KeyModifiers::CONTROL | KeyModifiers::SUPER)
        && matches!(key.code, KeyCode::Char('c' | 'd'))
}

/// Every key the shared tables know reaches them from a terminal.
#[cfg(test)]
mod mapping {
    use super::as_key;
    use crossterm::event::KeyCode;
    use orbs_shell::Key;

    /// The terminal key that should arrive as `key`.
    ///
    /// No `_` arm, on purpose: `PageUp` and `PageDown` were added to `Key` and
    /// missed by both mappers for a version, because a fallthrough cannot fail to
    /// compile. A new variant fails to compile here instead.
    fn code_for(key: &Key) -> KeyCode {
        match key {
            Key::Enter => KeyCode::Enter,
            Key::Backspace => KeyCode::Backspace,
            Key::Escape => KeyCode::Esc,
            Key::Left => KeyCode::Left,
            Key::Right => KeyCode::Right,
            Key::Up => KeyCode::Up,
            Key::Down => KeyCode::Down,
            Key::Home => KeyCode::Home,
            Key::End => KeyCode::End,
            Key::Tab => KeyCode::Tab,
            Key::PageUp => KeyCode::PageUp,
            Key::PageDown => KeyCode::PageDown,
            Key::Text(text) => KeyCode::Char(text.chars().next().unwrap_or(' ')),
        }
    }

    #[test]
    fn every_shell_key_has_a_terminal_key_that_reaches_it() {
        let every = [
            Key::Enter,
            Key::Backspace,
            Key::Escape,
            Key::Left,
            Key::Right,
            Key::Up,
            Key::Down,
            Key::Home,
            Key::End,
            Key::Tab,
            Key::PageUp,
            Key::PageDown,
            Key::Text("q".to_owned()),
        ];
        for key in every {
            assert_eq!(
                as_key(code_for(&key)),
                Some(key.clone()),
                "{key:?} cannot be typed in a terminal",
            );
        }
    }

    #[test]
    fn shift_tab_is_tab() {
        assert_eq!(as_key(KeyCode::BackTab), Some(Key::Tab));
    }
}

/// The two keystrokes a terminal cannot tell apart from one.
///
/// See [`escape_prefixed`]. These need no world at all — the split is a fact
/// about VT input, not about the tower.
#[cfg(test)]
mod escaping {
    use super::escape_prefixed;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    /// Escape and a letter in one read are two keystrokes, not one chord.
    ///
    /// The case that shipped broken: closing the spell editor and typing `quit`
    /// inside a frame put `quit` in the buffer as a line of the spell, because
    /// `\x1bq` reached crossterm in one syscall and came back `Alt+q`.
    #[test]
    fn an_escape_prefixed_letter_splits_back_into_two_keys() {
        let alt_q = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::ALT);
        let rest = escape_prefixed(&alt_q).expect("alt+letter is an escape pair");
        assert_eq!(rest.code, KeyCode::Char('q'));
        assert!(
            !rest.modifiers.contains(KeyModifiers::ALT),
            "the replayed key kept the modifier and would split again"
        );
    }

    /// ...and a chord the player really pressed is left alone.
    ///
    /// `Alt+Left` is word-left in a great many terminals and arrives CSI-encoded
    /// rather than escape-prefixed; turning it into Escape would drop a player
    /// out of the editor — this split's own failure, from the other side.
    #[test]
    fn a_real_alt_chord_is_not_an_escape_pair() {
        for code in [KeyCode::Left, KeyCode::Home, KeyCode::Backspace] {
            let chord = KeyEvent::new(code, KeyModifiers::ALT);
            assert!(
                escape_prefixed(&chord).is_none(),
                "{code:?} with Alt was read as an escape pair"
            );
        }
        let plain = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        assert!(escape_prefixed(&plain).is_none(), "a bare letter split");
    }
}
