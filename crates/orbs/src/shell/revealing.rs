//! The newest output arriving a character at a time, and the panes moving.
//!
//! Split out of `plugin.rs`, which CLAUDE.md restricts to registration: *"system
//! bodies and helpers go in sibling files."* Two clocks that are both about a
//! *change* having a beginning — text landing, and a pane count moving — so they
//! share a file rather than each getting one.

use bevy::prelude::*;

use super::{PaneTransition, Reveal};
use crate::sim::Tower;
use orbs_shell::{Pace, Panel, Passing, Showing};

/// How fast text types itself — `settings` → `habits` → `pace`.
///
/// Its own resource rather than a field on [`Reveal`], which a swap resets: a
/// new tower is no reason to forget how fast the player reads.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Paced(pub(crate) Pace);

impl Default for Paced {
    fn default() -> Self {
        Self(orbs_shell::settings::pace())
    }
}

/// Let the newest output arrive, a character at a time.
///
/// Reads the scrollback rather than listening for a message: output is produced
/// by the sim on a tick, and the frontend is a *caller* of the sim rather than
/// something it can notify (rule 3). The record count growing is the signal.
pub(super) fn drive_reveal(
    tower: Res<Tower>,
    time: Res<Time>,
    tubes: Query<&crate::crt::CrtSettings>,
    stillness: Option<Res<super::Stillness>>,
    paced: Option<Res<Paced>>,
    mut reveal: ResMut<Reveal>,
) {
    // The shared step the terminal takes too: notice, then type or land whole.
    reveal.drive(
        tower.sim().scrollback().records(),
        time.delta_secs(),
        super::moving(tubes.iter(), stillness.as_deref()),
        paced.map_or_else(Pace::default, |paced| paced.0),
    );
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
pub(super) fn drive_panes(
    time: Res<Time>,
    tubes: Query<&crate::crt::CrtSettings>,
    stillness: Option<Res<super::Stillness>>,
    mut panes: ResMut<PaneTransition>,
) {
    panes.retarget(PANES);
    // When nothing may move a change of count cuts, as a crossing does.
    if super::moving(tubes.iter(), stillness.as_deref()) == Some(false) {
        if !panes.is_settled() {
            *panes = PaneTransition::settled(PANES);
        }
        return;
    }
    panes.advance(time.delta_secs());
}

/// Notice what the pane is showing, and let a crossing run.
///
/// [`drive_panes`]' content half: that moves rectangles on a pane count, this
/// screens on their contents. After `refresh_panel` explicitly, or a crossing
/// starts a frame late. The switch is `motion::advance`'s; `None` is unset.
pub(super) fn drive_passing(
    time: Res<Time>,
    panel: Res<Panel>,
    surfaces: super::input::Surfaces,
    linear: Res<crate::shell::Linear>,
    mut passing: ResMut<Passing>,
) {
    // The tower opening, and it happens once. This system is gated on `booted`,
    // so its first run is the frame the boot card handed over, and
    // `Passing::wake` latches — so every frame after is free and there is no
    // edge for a hitch to miss. `drive` is that, the clock and the look, in
    // the order the terminal takes them too.
    passing.drive(
        time.delta_secs(),
        &Showing::of(surfaces.open(), &panel, linear.showing()),
    );
}

/// Panes the main window holds outside a siege.
///
/// One, since the rail replaced the telemetry pane (§19). `F4` is inert at one
/// pane, recorded rather than fixed (§9, §19); the siege multiplex raises the
/// count, which is why `PaneTransition` survives.
const PANES: u8 = 1;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crt::CrtSettings;
    use crate::shell::Stillness;

    /// Whether a command's answer has all arrived one frame after it lands.
    ///
    /// A frame first: what is on screen before the first look is history, and
    /// lands whole for anyone.
    fn text_lands_whole(still: bool) -> bool {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(Tower::new(1))
            .init_resource::<Reveal>()
            .insert_resource(Stillness::asked(still))
            .add_systems(Update, drive_reveal);
        app.world_mut().spawn(CrtSettings::DEFAULT);
        app.update();
        let mut tower = app.world_mut().resource_mut::<Tower>();
        tower.submit("recall clarity");
        tower.step();
        app.update();
        app.world().resource::<Reveal>().is_settled()
    }

    /// Whether a change of pane count has arrived after one frame.
    fn panes_cut(still: bool) -> bool {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(PaneTransition::settled(PANES + 1))
            .insert_resource(Stillness::asked(still))
            .add_systems(Update, drive_panes);
        app.world_mut().spawn(CrtSettings::DEFAULT);
        app.update();
        app.world().resource::<PaneTransition>().is_settled()
    }

    #[test]
    fn a_still_player_gets_text_whole_and_panes_by_a_cut() {
        // Both halves, so neither can pass by having nothing to animate.
        assert!(
            text_lands_whole(true),
            "text typed itself for a still player"
        );
        assert!(!text_lands_whole(false), "nothing was typing to begin with");
        assert!(panes_cut(true), "the panes moved for a still player");
        assert!(!panes_cut(false), "the panes had nowhere to move");
    }
}
