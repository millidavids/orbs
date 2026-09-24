//! The click under a keystroke.
//!
//! A key click carries no information, which is what makes it legal under rule
//! 2: every key sounds exactly the same, so it says only *a key moved*, which
//! the prompt line says at the same instant.
//!
//! That is a constraint rather than a simplification. A click that differed by
//! key would be a channel — audible letter-by-letter feedback a terminal player
//! does not get — and two clicks is a reason to add a third. One click, forever.
//!
//! Read from `KeyboardInput` rather than from the line changing, because
//! Backspace, Escape and Enter all move a key without lengthening anything, and
//! a keyboard you can hear only when it agrees with you is worse than silence.

use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;

use super::cues::Voice;

/// Click for anything pressed.
pub(super) fn ring_for_keys(
    mut typed: MessageReader<KeyboardInput>,
    mut ringing: MessageWriter<super::RingMessage>,
) {
    for key in typed.read() {
        if key.state != ButtonState::Pressed || key.repeat {
            continue;
        }
        // A modifier on its own is not a keystroke. Shift held to reach a
        // capital clicks twice — once for the Shift, once for the glyph — and
        // Ctrl, Alt and Super click for chords that never reach the prompt. A
        // click says *a key moved in the line you are typing*.
        if modifier(&key.logical_key) {
            continue;
        }
        // Enter is the one key with a sound of its own: a line landing is a
        // different event from a glyph arriving, and it is the only moment the
        // player has committed to anything.
        let voice = if key.logical_key == Key::Enter {
            Voice::Enter
        } else {
            Voice::Key
        };
        ringing.write(super::RingMessage(voice));
    }
}

/// Whether this key is a modifier pressed on its own.
///
/// Listed rather than derived: winit has no *"is this a modifier"* predicate,
/// and a fallthrough that guessed would be the shape this module already warns
/// about.
const fn modifier(key: &Key) -> bool {
    matches!(
        key,
        Key::Shift
            | Key::Control
            | Key::Alt
            | Key::Super
            | Key::Meta
            | Key::CapsLock
            | Key::NumLock
            | Key::ScrollLock
            | Key::Fn
            | Key::FnLock
            | Key::AltGraph
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_key_but_enter_sounds_the_same() {
        // Rule 2 at its narrowest: a click that differed by key would be
        // letter-by-letter feedback the Frame does not carry and the terminal
        // build cannot reproduce. Two sounds, split on committed or not, which
        // is a thing the screen shows.
        let sounds = |key: &Key| {
            if *key == Key::Enter {
                Voice::Enter
            } else {
                Voice::Key
            }
        };
        let glyphs = [
            Key::Character("a".into()),
            Key::Character("Z".into()),
            Key::Character(" ".into()),
            Key::Backspace,
            Key::Escape,
            Key::Tab,
            Key::ArrowUp,
        ];
        for key in &glyphs {
            assert_eq!(
                sounds(key),
                Voice::Key,
                "{key:?} has a sound of its own, which is a channel",
            );
        }
        assert_eq!(sounds(&Key::Enter), Voice::Enter);
    }

    #[test]
    fn a_modifier_on_its_own_is_not_a_keystroke() {
        // Shift held to reach a capital clicked twice — once for the Shift and
        // once for the letter — and Ctrl, Alt and Super clicked for chords that
        // never reach the prompt at all. A click says *a key moved in the line
        // you are typing*, and a modifier does not.
        for key in [
            Key::Shift,
            Key::Control,
            Key::Alt,
            Key::Super,
            Key::Meta,
            Key::CapsLock,
            Key::NumLock,
            Key::ScrollLock,
            Key::Fn,
            Key::FnLock,
            Key::AltGraph,
        ] {
            assert!(modifier(&key), "{key:?} clicks on its own");
        }

        // ...and everything a player actually types still does.
        for key in [
            Key::Character("a".into()),
            Key::Space,
            Key::Enter,
            Key::Backspace,
            Key::Escape,
            Key::Tab,
            Key::ArrowUp,
            Key::PageDown,
        ] {
            assert!(!modifier(&key), "{key:?} stopped making a sound");
        }
    }
}
