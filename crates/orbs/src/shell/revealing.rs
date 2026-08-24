//! The newest output arriving a character at a time, and the panes moving.
//!
//! Split out of `plugin.rs`, which CLAUDE.md restricts to registration: *"system
//! bodies and helpers go in sibling files."* Two clocks that are both about a
//! *change* having a beginning — text landing, and a pane count moving — so they
//! share a file rather than each getting one.

use bevy::prelude::*;

use super::{PaneTransition, Reveal};
use crate::sim::Tower;

/// Let the newest output arrive, a character at a time.
///
/// Reads the scrollback rather than listening for a message: output is produced
/// by the sim on a tick, and the frontend is a *caller* of the sim rather than
/// something it can notify (rule 3). The record count growing is the signal.
pub(super) fn drive_reveal(tower: Res<Tower>, time: Res<Time>, mut reveal: ResMut<Reveal>) {
    let records = tower.sim().scrollback().records();
    let cells = tail_cells(records, reveal.settled_len());
    reveal.observe(records.drawn_len(), cells);
    reveal.advance(time.delta_secs());
}

/// Characters in the records added since `from`.
///
/// Measured through `to_line`, which is what the view draws, so the budget and
/// the drawing agree about what a character is.
fn tail_cells(records: &orbs_render::Records, from: usize) -> u16 {
    let counted: usize = records
        .iter()
        .skip(from)
        .map(|record| record.to_line().chars().count())
        .sum();
    u16::try_from(counted).unwrap_or(u16::MAX)
}

/// Any keystroke puts the whole of the output on screen at once.
///
/// §9's parity rule in practice: waiting must never be something a player is
/// made to do, or the animation stops being flavour and starts being a cost that
/// an accessibility setting could buy its way out of.
pub(super) fn finish_reveal(mut reveal: ResMut<Reveal>) {
    reveal.finish();
}

/// Aim the pane transition at what the current grid asks for, and let it move.
///
/// The pane count was derived inside `paint` every frame, which is why a pane
/// used to appear between one frame and the next. It is decided here instead so
/// that a *change* in it is something with a beginning.
pub(super) fn drive_panes(time: Res<Time>, mut panes: ResMut<PaneTransition>) {
    panes.retarget(PANES);
    panes.advance(time.delta_secs());
}

/// Panes the main window holds outside a siege.
///
/// **One, since the tower rail replaced the telemetry pane** (§19, Phase 2). It
/// was two, and the second held nine developer readings; the rail carries five
/// of them in sixteen columns down the right, and the session pane gets the rest
/// — **102 columns of body against the 58 it had**, which is what a transcript
/// beside an instrument panel and a maze map actually wants.
///
/// **`F4` is visibly inert at one pane, and that is recorded rather than fixed.**
/// §9 makes the focus mode a setting the player may change at any time and §19
/// fixes the switch on `F4`; reassigning it to toggle the rail would re-litigate
/// both. With one pane the two tilings are identical, so the key changes nothing
/// until multiplexing returns the second pane in Phase 9a — at which point it
/// reclaims its job with no code to change.
///
/// The decision the old constant encoded is not gone: `ORBS_DUMP` can still be
/// handed a grid below the floor, and `dump.rs` still asks. §9 caps the count at
/// four, and the siege multiplex is what raises it — which is why
/// `PaneTransition` survives with nothing left to animate here.
const PANES: u8 = 1;
