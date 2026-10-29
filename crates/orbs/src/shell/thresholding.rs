//! The orb's menu, in front of no tower at all.
//!
//! The wiring for [`Threshold`]. The menu opens once the card leaves, since
//! keys are gated on [`booted`](crate::boot::booted), and after
//! `type_into_menu` drains the keys, as `menuing.rs` gives (§19).

use bevy::prelude::*;

use orbs_shell::{Boot, Stance, Threshold};

use super::Standing;

/// Run condition: a tower is in front of the player and the world may run.
///
/// [`booted`](crate::boot::booted) plus a tower, wherever the question is
/// *may the game do something*. `Option<Res<_>>`, for the headless harnesses.
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

/// Whether this tower's threshold menu has been put up already.
///
/// A resource on `shell_resources!`, not a `Local`, so a new tower starts
/// unopened: the tour puts the threshold up again in a later scene.
#[derive(Resource, Debug, Default)]
pub(crate) struct Opened(bool);

#[cfg(test)]
impl Opened {
    /// Whether the menu has been put up for this tower.
    pub(crate) const fn is_set(&self) -> bool {
        self.0
    }
}

/// Put the menu up once the boot card has gone.
///
/// Every frame while [`waiting`], so there is no edge to catch. Nothing closes
/// it at [`Stance::Threshold`]; `menuing::swap` sets [`Threshold::Playing`].
/// A second opening is a defect the net catches and complains of once; the
/// rule is held by `the_threshold_has_no_way_to_close_the_menu`.
pub(crate) fn open_at_the_threshold(
    mut standing: ResMut<Standing>,
    settable: super::setting::Settable,
    mut opened: ResMut<Opened>,
) {
    if standing.is_open() {
        return;
    }
    if opened.0 {
        bevy::log::error!(
            "the threshold's menu was closed and had to be put back up; \
             something reached `MenuOutcome::Close` at `Stance::Threshold`",
        );
    }
    opened.0 = true;
    // The same rows the `menu` verb's opener hands over: one `Settable` asked
    // twice, because a second list would be a settings page that differed
    // depending on which door the player came through.
    standing.open_menu(Stance::Threshold, settable.driver(), settable.rows());
}
