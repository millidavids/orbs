//! Driving the bench from Bevy's clocks, and asking the tube whether to move.
//!
//! [`Bench`] is pure `f32` arithmetic and knows nothing
//! about an engine. This is the five lines that find the two clocks and the
//! motion switch — the one place the shell reaches the GPU side at all, and
//! therefore the one part of the bench a terminal build cannot use.

use bevy::prelude::*;

use orbs_shell::{Bench, Panel};

use crate::crt::CrtSettings;

/// Advance the fire, and stop it when the tube is off.
///
/// The two are one system because they are one decision: `Bench` has no business
/// knowing what a `CrtSettings` is on any frame it is not being driven.
pub(crate) fn advance(
    time: Res<Time>,
    world_tick: Res<Time<Fixed>>,
    tubes: Query<&CrtSettings>,
    panel: Res<Panel>,
    mut bench: ResMut<Bench>,
) {
    // No camera yet, or no CRT component on it: `None`, which leaves whatever
    // `ORBS_FIRE` said. A missing switch is not the same as a switch set to off,
    // and defaulting to "off" here would make the fire invisible for the frames
    // before the camera spawns rather than merely unlit.
    //
    // **A `Query`, not `Option<Single>`.** `Single` yields `None` for *two or
    // more* matches as readily as for none, so a second camera — a future split
    // view, a capture pass — would land in the "no switch on screen" branch and
    // silently leave the motion running for a player who had turned it off. An
    // accessibility switch that a second entity can disconnect is the defect
    // `crt::settings` already records once.
    //
    // Any tube off turns the motion off. Two switches disagreeing is not a
    // situation the game creates today, and the quieter reading is the only safe
    // one to pick if it ever does: §14's requirement is that a player who asks
    // for no motion gets none.
    let mut tubes = tubes.iter();
    let motion = tubes
        .next()
        .map(|first| first.on && tubes.all(|other| other.on));

    // **The same `Time<Fixed>` the sim runs on** (`sim::plugin` installs it at
    // 1 Hz), so the bench cannot disagree with the sim about when a tick lands —
    // which a second, self-counted copy would the moment the clock hitched.
    bench.advance(
        time.delta_secs(),
        world_tick.overstep_fraction(),
        motion,
        &panel,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_second_camera_does_not_disconnect_the_motion_switch() {
        // `Option<Single>` answers `None` for two matches as readily as for
        // none, and "none" is the branch that leaves the motion *running*. So a
        // split view or a capture pass would silently undo F3-to-OFF for a
        // player who had asked for no motion — §14's switch defeated by an
        // entity that has nothing to do with it.
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Bench>()
            .init_resource::<Panel>()
            .add_systems(Update, advance);
        app.world_mut().spawn(CrtSettings::OFF);
        app.world_mut().spawn(CrtSettings::DEFAULT);

        app.update();
        assert!(
            !app.world().resource::<Bench>().grind(true, false).working,
            "a second tube left the meter shimmering with the first turned off",
        );
    }

    #[test]
    fn killing_the_tube_kills_the_fire() {
        // **§14's disableable requirement, wired to a switch a player can
        // reach.** F3 cycles the CRT to `OFF`; a player who does that has said
        // they do not want motion, and a shimmering meter that ignored them
        // would be exactly the defect `crt::settings` documents — an
        // accessibility switch something else can flip back on.
        //
        // Asserted through the *system*, not through `set_enabled`: the risk was
        // never that the flag fails to work, it is that nothing sets it.
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Bench>()
            // `advance` reads the panel to catch the moment the athanor lights.
            // Empty here: this test is about the tube, and an empty panel is the
            // "no athanor on screen" case that leaves the flare alone.
            .init_resource::<Panel>()
            .add_systems(Update, advance);
        let camera = app.world_mut().spawn(CrtSettings::DEFAULT).id();

        app.update();
        assert!(
            app.world().resource::<Bench>().grind(true, false).working,
            "the bench was out with the tube on",
        );

        app.world_mut().entity_mut(camera).insert(CrtSettings::OFF);
        app.update();
        assert!(
            !app.world().resource::<Bench>().grind(true, false).working,
            "F3 to OFF left the meter shimmering",
        );

        // ...and it comes back, so the switch is a switch rather than a latch.
        app.world_mut()
            .entity_mut(camera)
            .insert(CrtSettings::DEFAULT);
        app.update();
        assert!(app.world().resource::<Bench>().grind(true, false).working);
    }
}
