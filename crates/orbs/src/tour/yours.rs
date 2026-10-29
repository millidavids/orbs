//! A person's key: `F11` moves through the tour, anything else takes the scene.
//!
//! Read from `WindowEvent`, which only winit writes, so the keys the tour
//! presses itself are never mistaken for a person's.

use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::window::WindowEvent;

use super::state::Tour;

/// Watch for a person at the keyboard.
///
/// A press only, never a repeat or a release — `F11`'s release lands after the
/// next scene has begun — and never a modifier alone, which winit also sends
/// when the window takes the focus back.
pub(super) fn watch(
    mut events: MessageReader<WindowEvent>,
    mut tour: ResMut<Tour>,
    mut shifted: Local<bool>,
) {
    for event in events.read() {
        let WindowEvent::KeyboardInput(key) = event else {
            continue;
        };
        if matches!(key.key_code, KeyCode::ShiftLeft | KeyCode::ShiftRight) {
            *shifted = key.state == ButtonState::Pressed;
            continue;
        }
        if key.state != ButtonState::Pressed || key.repeat || is_modifier(key.key_code) {
            continue;
        }
        if key.key_code == KeyCode::F11 {
            if *shifted {
                tour.back();
            } else {
                tour.forward();
            }
            continue;
        }
        tour.take_over();
    }
}

/// A key that changes others rather than meaning anything alone.
const fn is_modifier(code: KeyCode) -> bool {
    matches!(
        code,
        KeyCode::ControlLeft
            | KeyCode::ControlRight
            | KeyCode::AltLeft
            | KeyCode::AltRight
            | KeyCode::SuperLeft
            | KeyCode::SuperRight
            | KeyCode::CapsLock
            | KeyCode::NumLock
            | KeyCode::Fn
            | KeyCode::FnLock
    )
}
