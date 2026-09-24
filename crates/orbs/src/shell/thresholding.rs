//! The orb's menu, in front of no tower at all.
//!
//! [`Threshold`] is the state and knows nothing about Bevy. This is the wiring,
//! and it is a separate file for the reason `menuing.rs` is.
//!
//! The menu opens when the boot card leaves and not before: every keyed system
//! is gated on [`booted`](crate::boot::booted), so a menu put up during the
//! sequence would be a screen the keyboard could not reach.
//!
//! It also opens after the keys are drained. [`open_at_the_threshold`] is
//! ordered after `type_into_menu` in the input chain, for the reason
//! `menuing.rs` gives: a gated `MessageReader` keeps its cursor, so a surface
//! opening on a frame a reader has not yet run reads whatever `Messages`
//! retains. `ORBS_BOOT=0` makes that the first frame, when a stray keystroke is
//! most likely to still be queued — and the menu's words are destructive, `quit`
//! writing an `AppExit`. §19 records this shape costing a shipped defect.

use bevy::prelude::*;

use orbs_shell::{Boot, Stance, Threshold};

use super::Standing;

/// Run condition: a tower is in front of the player and the world may run.
///
/// `booted` plus one term, replacing `booted` wherever the question is *may the
/// game do something*. The boot card and the threshold are two reasons for the
/// same answer, and every system that cared about the first cares about the
/// second: `F6` at the threshold would write a parse trace of a session that has
/// not happened, which is [`booted`](crate::boot::booted)'s own justification.
///
/// `Option<Res<_>>` for both, so an app built without either plugin behaves as
/// it did before they existed — the headless harnesses build exactly that.
pub(crate) fn playing(boot: Option<Res<Boot>>, threshold: Option<Res<Threshold>>) -> bool {
    boot.is_none_or(|boot| boot.stage().world_runs())
        && threshold.is_none_or(|threshold| threshold.is_playing())
}

/// Run condition: the player has yet to choose a tower.
///
/// Not the complement of [`playing`] — during boot neither is true, which is
/// right: the boot card owns the screen and the menu is not up yet.
pub(crate) fn waiting(boot: Option<Res<Boot>>, threshold: Option<Res<Threshold>>) -> bool {
    boot.is_none_or(|boot| boot.stage().world_runs())
        && threshold.is_some_and(|threshold| threshold.is_waiting())
}

/// Put the menu up once the boot card has gone.
///
/// Runs every frame while [`waiting`] and does nothing once the menu is up, so
/// there is no edge to catch and nothing to arm. Closing it is not possible at
/// this stance ([`Stance::Threshold`]), so the only way past is a tower, and
/// `menuing::swap` is what sets [`Threshold::Playing`] when one arrives.
///
/// A second opening is a defect, and it says so. Re-opening is kept because the
/// alternative is a player stranded at the prompt of a scratch world, but a
/// safety net that silently catches a fall hides what pushed: the close and the
/// re-open land in the same frame, so nothing outside can tell them from the
/// menu never having closed.
///
/// Not hypothetical — a plugin-level test written to hold *"nothing closes the
/// menu at the threshold"* passed with [`Stance::may_close`] deliberately
/// broken, because this caught it. So the net stays and complains once. The
/// property itself is held next door by
/// `the_threshold_has_no_way_to_close_the_menu`, which asks the state machine
/// directly and does fail.
pub(crate) fn open_at_the_threshold(
    mut standing: ResMut<Standing>,
    settable: super::setting::Settable,
    mut opened_before: Local<bool>,
) {
    if standing.is_open() {
        return;
    }
    if *opened_before {
        bevy::log::error!(
            "the threshold's menu was closed and had to be put back up; \
             something reached `MenuOutcome::Close` at `Stance::Threshold`",
        );
    }
    *opened_before = true;
    // The same rows the `menu` verb's opener hands over: one `Settable` asked
    // twice, because a second list would be a settings page that differed
    // depending on which door the player came through.
    standing.open(Stance::Threshold, settable.driver(), settable.rows());
}
