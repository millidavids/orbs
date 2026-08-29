//! Who the arrow keys belong to while a figure is being sung (DESIGN.md §10).
//!
//! `wandering.rs`'s shape one room over, and the differences are the interesting
//! part.
//!
//! **The board is not this module's**, and neither is the figure. `orbs-shell`
//! draws the picture whenever a chant is running — watching a bound spell sing
//! one is a thing a player does — so all that is decided here is *who a
//! keystroke reaches*.
//!
//! **It does not take the pane.** `wander` hides the transcript because a maze
//! is too big to sit beside one; a figure is 42 columns and already draws
//! beside it. So a player answering syllables still sees what the orb is saying
//! about them, which is the thing `wander` gives up.
//!
//! **And a press does not wait for a tick.** `Sim::sing` is a fourth entry point
//! beside `submit`, `step` and `walk`, for `walk`'s reason with more force: a
//! key that queued would arrive after the beat it was answering, so it would not
//! merely be slow, it would be always wrong.

use bevy::input::ButtonState;
use bevy::input::keyboard::{KeyCode, KeyboardInput};
use bevy::prelude::*;

use crate::sim::Tower;

/// Whether the arrows are answering a chant.
///
/// A `bool` rather than a state machine, exactly as [`Walk`](super::Walk) is:
/// what a figure holds is the sim's, and what a *cursor* would be there is no
/// cursor to hold.
#[derive(Resource, Debug, Default)]
pub(crate) struct Chorus(bool);

impl Chorus {
    /// Whether the arrows are the chant's.
    pub(crate) const fn is_open(&self) -> bool {
        self.0
    }

    /// Take them.
    pub(crate) const fn open(&mut self) {
        self.0 = true;
    }

    /// Give them back.
    pub(crate) const fn close(&mut self) {
        self.0 = false;
    }
}

/// Whether the arrows have the chant.
pub(crate) fn chorusing(chorus: Res<Chorus>) -> bool {
    chorus.is_open()
}

/// Take the keys when `chorus` asks for them.
pub(crate) fn open_requested(mut tower: ResMut<Tower>, mut chorus: ResMut<Chorus>) {
    // **Peeked before it is taken**, exactly as the editor, the weave screen and
    // the maze do: `chorusing` needs `&mut`, and reaching for it stamps
    // `Tower`'s change tick, which would leave this system re-arming its own run
    // condition every frame.
    if !tower.has_chorusing() {
        return;
    }
    if !tower.chorusing() {
        return;
    }
    chorus.open();
}

/// Give the keys back when the figure is gone.
///
/// **The maze's rule, and it matters more here.** A chant ends on its own — it
/// runs out, or it collapses — so a player left holding the arrows over nothing
/// would have a dead prompt and no way to discover why. `wander` closes when the
/// maze closes; this closes when the figure does.
pub(crate) fn close_when_gone(tower: Res<Tower>, mut chorus: ResMut<Chorus>) {
    if chorus.is_open() && tower.sim().figure().is_none() {
        chorus.close();
    }
}

/// Feed keys to the figure.
pub(crate) fn type_into_chant(
    mut keys: MessageReader<KeyboardInput>,
    held: Res<ButtonInput<KeyCode>>,
    quiet: Res<super::input::Quiet>,
    mut chorus: ResMut<Chorus>,
    mut tower: ResMut<Tower>,
) {
    // A held chord is skipped; a *stale* one is not — see `wandering.rs`, which
    // records what reading it the other way costs.
    let stale_chord = super::input::chord_is_stale(quiet.gap());
    let chord = held.any_pressed([
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
    ]);

    for event in keys.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }
        if chord && !stale_chord {
            continue;
        }
        // **Escape first, and it must `break`.** §19: a mode that consumes
        // Escape and stays is a mode you cannot leave.
        if event.key_code == KeyCode::Escape {
            chorus.close();
            break;
        }
        // **`orbs-shell`'s table, which the terminal build also calls.** A build
        // whose Up key meant a different syllable would be two games, so the
        // mapping lives once and neither frontend holds an opinion about it.
        let Some(key) = super::input::pressed(event) else {
            continue;
        };
        let Some(syllable) = orbs_shell::apply_to_chant(&key) else {
            continue;
        };
        tower.sing(syllable);
    }
}
