//! Reading back through the transcript — `unfurl`, and the keys that page it.
//!
//! How far a page moves is `orbs-shell`'s, deliberately: it is what the
//! transcript would fit, measured with the same `RecordView` it is drawn with,
//! so a second measure would page by a different amount than it showed.

use bevy::prelude::*;
use orbs_shell::{Key, Scroll};

use super::Screen;
use crate::sim::Tower;

/// The keys that move through the transcript while reading, as the table both
/// builds read (`orbs_shell::apply_to_reading`) names them.
///
/// Up and Down are a record, Home and End an end. Escape leaves, as it leaves
/// the editor: one meaning in every mode. It does not scroll back to the newest
/// output — a player who read back and pressed Escape wants to type, not to
/// lose their place.
const READING_KEYS: [(KeyCode, Key); 5] = [
    (KeyCode::ArrowUp, Key::Up),
    (KeyCode::ArrowDown, Key::Down),
    (KeyCode::Home, Key::Home),
    (KeyCode::End, Key::End),
    (KeyCode::Escape, Key::Escape),
];

/// Hand the transcript the keyboard when `unfurl` has asked for it.
///
/// The sim owns the *decision* and the frontend owns the scroll, exactly as it
/// does for the editor — see `execute::unfurl`. `Sim::unfurling` takes rather
/// than reads, so this fires once per `unfurl` rather than every frame.
///
/// It pages back on the way in: entering a reading mode showing the screen you
/// were already looking at leaves the player pressing a key to find out whether
/// the word did anything.
pub(super) fn start_reading(
    mut tower: ResMut<Tower>,
    mut scroll: ResMut<Scroll>,
    screen: Res<Screen>,
) {
    // Peeked first — see `editing::open_requested` for why reaching for `&mut`
    // unconditionally defeats every `resource_changed::<Tower>` guard.
    if !tower.is_unfurling() {
        return;
    }
    tower.unfurling();
    scroll.read();
    page(&mut scroll, &screen, &tower, true);
}

/// Move through the transcript while reading.
///
/// `ButtonInput` rather than `KeyboardInput`: a gated message reader keeps its
/// cursor and re-reads the key that closed the manual on the frame it next
/// runs (`plugin.rs` records that shipping once for the menu).
pub(super) fn read_keys(
    pressed: Res<ButtonInput<KeyCode>>,
    mut scroll: ResMut<Scroll>,
    tower: Res<Tower>,
) {
    let total = tower.sim().scrollback().records().drawn_len();
    for (code, key) in &READING_KEYS {
        if pressed.just_pressed(*code) {
            orbs_shell::apply_to_reading(key, &mut scroll, total);
        }
    }
}

/// Whether a key [`read_keys`] answers was pressed this frame.
pub(super) fn reading_key(pressed: Res<ButtonInput<KeyCode>>) -> bool {
    pressed.any_just_pressed(READING_KEYS.map(|(code, _)| code))
}

/// Look further back through the transcript.
pub(super) fn scroll_back(mut scroll: ResMut<Scroll>, screen: Res<Screen>, tower: Res<Tower>) {
    page(&mut scroll, &screen, &tower, true);
}

/// Come back toward the newest output.
pub(super) fn scroll_forward(mut scroll: ResMut<Scroll>, screen: Res<Screen>, tower: Res<Tower>) {
    page(&mut scroll, &screen, &tower, false);
}

/// A screenful older, or newer.
///
/// The real record count both ways, as the terminal build passes it. The
/// forward branch once had `0`, inert only because `Scroll::page` ignores
/// `total` going forward today.
fn page(scroll: &mut Scroll, screen: &Screen, tower: &Tower, older: bool) {
    let total = tower.sim().scrollback().records().drawn_len();
    let step = orbs_shell::page_step(screen, tower.sim(), scroll.back());
    scroll.page(step, older, total);
}
