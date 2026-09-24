//! Reading back through the transcript — `unfurl`, and the keys that page it.
//!
//! Split out of `plugin.rs`, which CLAUDE.md restricts to registration.
//!
//! How far a page moves is `orbs-shell`'s, deliberately: it is what the
//! transcript would fit, measured with the same `RecordView` it is drawn with,
//! so a second measure would page by a different amount than it showed. What
//! lives here is this frontend's `Tower` being unwrapped for it.

use bevy::prelude::*;

use super::Screen;
use crate::sim::Tower;

/// The shared measurement, with this frontend's `Tower` unwrapped.
pub(super) fn page_step(screen: &Screen, tower: &Tower, back: usize) -> usize {
    orbs_shell::page_step(screen, tower.sim(), back)
}

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
    mut scroll: ResMut<orbs_shell::Scroll>,
    screen: Res<Screen>,
) {
    // Peeked first — see `editing::open_requested` for why reaching for `&mut`
    // unconditionally defeats every `resource_changed::<Tower>` guard.
    if !tower.is_unfurling() {
        return;
    }
    tower.unfurling();
    scroll.read();
    let total = tower.sim().scrollback().records().drawn_len();
    let step = page_step(&screen, &tower, scroll.back());
    scroll.page(step, true, total);
}

/// Give the keyboard back to the prompt.
///
/// Escape, the same as the editor: one meaning in every mode, step out of the
/// one you are in. It does not scroll back to the newest output — a player who
/// read back and pressed Escape wants to type, not to lose their place.
pub(super) fn stop_reading(mut scroll: ResMut<orbs_shell::Scroll>) {
    scroll.stop_reading();
}

/// Look further back through the transcript.
pub(super) fn scroll_back(
    mut scroll: ResMut<orbs_shell::Scroll>,
    screen: Res<Screen>,
    tower: Res<Tower>,
) {
    let total = tower.sim().scrollback().records().drawn_len();
    let step = page_step(&screen, &tower, scroll.back());
    scroll.page(step, true, total);
}

/// Come back toward the newest output.
pub(super) fn scroll_forward(
    mut scroll: ResMut<orbs_shell::Scroll>,
    screen: Res<Screen>,
    tower: Res<Tower>,
) {
    // The real count, not a fabricated nought. This passed `0` while the
    // terminal build passes the record count into the same shared `Scroll::page`
    // — two frontends calling one function with different arguments. Inert only
    // because the forward branch ignores `total` today, so the next change to it
    // would corrupt scrolling in this build alone.
    let total = tower.sim().scrollback().records().drawn_len();
    let step = page_step(&screen, &tower, scroll.back());
    scroll.page(step, false, total);
}
