//! A scene's steps as keystrokes: the window's own keyboard messages, so every
//! surface takes them through its own handler and none knows the tour exists.

use bevy::ecs::system::SystemParam;
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput, NativeKeyCode};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use orbs_shell::tour::{Named, Seen, Stroke, When};

use super::state::Tour;
use crate::shell::{Editing, Loom, Reading, Standing, Walk};
use crate::sim::Tower;

/// The most a frame advances the tour, so a window uncovered after a minute
/// does not type a burst.
const FRAME: f32 = 0.1;

/// What is up, for a step that waits on it.
#[derive(SystemParam)]
pub(super) struct Up<'w> {
    editing: Res<'w, Editing>,
    loom: Res<'w, Loom>,
    walk: Res<'w, Walk>,
    standing: Res<'w, Standing>,
    reading: Res<'w, Reading>,
    boot: Option<Res<'w, orbs_shell::Boot>>,
    threshold: Option<Res<'w, orbs_shell::Threshold>>,
}

impl Up<'_> {
    /// Whether a line entered now reaches the world: no surface has the keys,
    /// the card has gone, and there is a tower.
    fn prompt(&self) -> bool {
        !self.editing.is_open()
            && !self.loom.is_open()
            && !self.walk.is_open()
            && !self.standing.is_open()
            && !self.reading.is_open()
            && self.holds(When::Live)
            && self
                .threshold
                .as_ref()
                .is_none_or(|threshold| threshold.is_playing())
    }

    fn holds(&self, when: When) -> bool {
        match when {
            When::Editor => self.editing.is_open(),
            When::Weave => self.loom.is_open(),
            When::Maze => self.walk.is_open(),
            When::Menu => self.standing.is_open(),
            When::Manual => self.reading.is_open(),
            When::Live => self.boot.as_ref().is_none_or(|boot| boot.is_live()),
        }
    }
}

/// Press the key the scene's steps want this frame, and release the last one.
///
/// Before `InputSystems`, so `ButtonInput` sees it this frame as it sees a
/// real one.
pub(super) fn press(
    mut tour: ResMut<Tour>,
    real: Res<Time<Real>>,
    tower: Res<Tower>,
    up: Up,
    windows: Query<Entity, With<PrimaryWindow>>,
    mut keys: MessageWriter<KeyboardInput>,
) {
    if let Some(release) = tour.owed.take() {
        keys.write(release);
    }
    if tour.is_yours() {
        return;
    }
    let window = windows.iter().next().unwrap_or(Entity::PLACEHOLDER);
    let seen = Seen {
        tick: tower.sim().tick().get(),
        prompt: up.prompt(),
    };
    let seconds = real.delta_secs().min(FRAME);
    let advance = tour.timeline.advance(seconds, seen, &|when| up.holds(when));
    if let Some(step) = advance.landed {
        let clock = tour.timeline.clock();
        let offsets = tour
            .timeline
            .steps()
            .get(step)
            .map(|step| step.shots.clone())
            .unwrap_or_default();
        if let Some(shots) = tour.shots.as_mut() {
            shots.after(clock, &offsets);
        }
    }
    if let Some(stroke) = advance.press {
        let (down, up) = keystroke(stroke, window);
        keys.write(down);
        tour.owed = Some(up);
    }
}

/// One keystroke as winit would send it: a press, and the release to send next
/// frame.
fn keystroke(stroke: Stroke, window: Entity) -> (KeyboardInput, KeyboardInput) {
    let (key_code, logical_key, text) = match stroke {
        Stroke::Char(glyph) => (code_of(glyph), logical(glyph), Some(glyph.to_string())),
        Stroke::Named(named) => {
            let (code, key) = named_key(named);
            (code, key, None)
        }
    };
    let press = KeyboardInput {
        key_code,
        logical_key: logical_key.clone(),
        state: ButtonState::Pressed,
        text: text.map(Into::into),
        repeat: false,
        window,
    };
    let release = KeyboardInput {
        key_code,
        logical_key,
        state: ButtonState::Released,
        text: None,
        repeat: false,
        window,
    };
    (press, release)
}

/// The logical key a typed character is. The space bar is `Key::Space`, not a
/// character, as winit reports it.
fn logical(glyph: char) -> Key {
    if glyph == ' ' {
        Key::Space
    } else {
        Key::Character(glyph.to_string().into())
    }
}

/// The physical key a character is typed on, on a US layout; anything else is
/// unidentified, which no guard reads.
fn code_of(glyph: char) -> KeyCode {
    const LETTERS: [KeyCode; 26] = [
        KeyCode::KeyA,
        KeyCode::KeyB,
        KeyCode::KeyC,
        KeyCode::KeyD,
        KeyCode::KeyE,
        KeyCode::KeyF,
        KeyCode::KeyG,
        KeyCode::KeyH,
        KeyCode::KeyI,
        KeyCode::KeyJ,
        KeyCode::KeyK,
        KeyCode::KeyL,
        KeyCode::KeyM,
        KeyCode::KeyN,
        KeyCode::KeyO,
        KeyCode::KeyP,
        KeyCode::KeyQ,
        KeyCode::KeyR,
        KeyCode::KeyS,
        KeyCode::KeyT,
        KeyCode::KeyU,
        KeyCode::KeyV,
        KeyCode::KeyW,
        KeyCode::KeyX,
        KeyCode::KeyY,
        KeyCode::KeyZ,
    ];
    const DIGITS: [KeyCode; 10] = [
        KeyCode::Digit0,
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
    ];
    let lower = glyph.to_ascii_lowercase();
    // Base 36 numbers the digits nought to nine and the letters after them.
    let place = lower
        .to_digit(36)
        .and_then(|place| usize::try_from(place).ok());
    match (lower, place) {
        ('0'..='9', Some(place)) => DIGITS[place],
        ('a'..='z', Some(place)) => LETTERS[place - 10],
        (' ', _) => KeyCode::Space,
        ('-' | '_', _) => KeyCode::Minus,
        ('.', _) => KeyCode::Period,
        (',', _) => KeyCode::Comma,
        ('/', _) => KeyCode::Slash,
        _ => KeyCode::Unidentified(NativeKeyCode::Unidentified),
    }
}

/// A named key's physical and logical halves.
const fn named_key(named: Named) -> (KeyCode, Key) {
    match named {
        Named::Enter => (KeyCode::Enter, Key::Enter),
        Named::Escape => (KeyCode::Escape, Key::Escape),
        Named::Tab => (KeyCode::Tab, Key::Tab),
        Named::Backspace => (KeyCode::Backspace, Key::Backspace),
        Named::Up => (KeyCode::ArrowUp, Key::ArrowUp),
        Named::Down => (KeyCode::ArrowDown, Key::ArrowDown),
        Named::Left => (KeyCode::ArrowLeft, Key::ArrowLeft),
        Named::Right => (KeyCode::ArrowRight, Key::ArrowRight),
        Named::Home => (KeyCode::Home, Key::Home),
        Named::End => (KeyCode::End, Key::End),
        Named::PageUp => (KeyCode::PageUp, Key::PageUp),
        Named::PageDown => (KeyCode::PageDown, Key::PageDown),
        Named::F2 => (KeyCode::F2, Key::F2),
        Named::F3 => (KeyCode::F3, Key::F3),
        Named::F5 => (KeyCode::F5, Key::F5),
        Named::F7 => (KeyCode::F7, Key::F7),
        Named::F8 => (KeyCode::F8, Key::F8),
    }
}
