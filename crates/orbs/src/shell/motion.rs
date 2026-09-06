//! Driving the bench from Bevy's clocks, and asking the tube whether to move.
//!
//! [`Bench`] is pure `f32` arithmetic and knows nothing
//! about an engine. This is the five lines that find the two clocks and the
//! motion switch — the one place the shell reaches the GPU side at all, and
//! therefore the one part of the bench a terminal build cannot use.

use bevy::prelude::*;

use orbs_shell::{Bench, Panel, Passing};

use crate::crt::CrtSettings;

/// Advance the fire and the crossing, and stop them when the tube is off.
///
/// One system because it is one decision, and the two clocks it drives are the
/// two things §14 makes disableable: neither `Bench` nor `Passing` has any
/// business knowing what a `CrtSettings` is on a frame it is not being driven.
///
/// **The crossing's clock is advanced in `revealing::drive_passing`, not here.**
/// What this hands it is the *switch* — the one thing a terminal cannot supply —
/// and it is handed to both from the same read, so the fire and the crossing can
/// never disagree about whether a player asked for motion.
pub(crate) fn advance(
    time: Res<Time>,
    world_tick: Res<Time<Fixed>>,
    tubes: Query<&CrtSettings>,
    panel: Res<Panel>,
    mut bench: ResMut<Bench>,
    mut passing: ResMut<Passing>,
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
    // The switch only. Zero seconds, because `drive_passing` owns this clock and
    // two systems advancing one clock would run it at twice the rate.
    passing.advance(0.0, motion);
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
            .init_resource::<Passing>()
            .init_resource::<Panel>()
            .add_systems(Update, advance);
        app.world_mut().spawn(CrtSettings::OFF);
        app.world_mut().spawn(CrtSettings::DEFAULT);

        app.update();
        assert!(
            !app.world().resource::<Bench>().grind(true, false).working,
            "a second tube left the meter shimmering with the first turned off",
        );
        // The crossing inherits the hazard because it rides the same read. It
        // was worth asserting separately rather than trusting that: the two
        // clocks are handed the switch from one place *today*, and a future
        // second read is exactly how they would come to disagree.
        assert!(
            !crosses(&mut app),
            "a second tube left screens crossing with the first turned off",
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
            .init_resource::<Passing>()
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

    #[test]
    fn killing_the_tube_stops_the_crossings() {
        // §14 again, for the other clock this system drives — and the
        // `DESIGN.md:9182` trap with it: reduce-motion must **cut**, never freeze
        // a half-drawn screen. Three animations learned that independently.
        //
        // Through the *system*, for `killing_the_tube_kills_the_fire`'s reason:
        // the risk is never that the flag fails to work, it is that nothing sets
        // it.
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Bench>()
            .init_resource::<Passing>()
            .init_resource::<Panel>()
            .add_systems(Update, advance);
        let camera = app.world_mut().spawn(CrtSettings::DEFAULT).id();

        app.update();
        assert!(crosses(&mut app), "screens did not cross with the tube on");

        app.world_mut().entity_mut(camera).insert(CrtSettings::OFF);
        app.update();
        // **Before the probe, which keeps a screen of its own.** `crosses` has to
        // put one there to have something to cross out of, so asking this after
        // it would be reading the probe rather than the system.
        assert!(
            app.world().resource::<Passing>().kept().is_empty(),
            "a kept screen survived the switch — 43 KiB a player asked not to pay",
        );
        assert!(!crosses(&mut app), "F3 to OFF left screens crossing");
    }

    /// Whether a screen change would start a crossing, right now.
    ///
    /// Asked by making one: `Passing` is deliberately opaque about `enabled`,
    /// because the question a caller ever has is *"will this cross"* rather than
    /// *"is a flag set"*.
    fn crosses(app: &mut App) -> bool {
        let mut passing = std::mem::take(&mut *app.world_mut().resource_mut::<Passing>());
        passing.pose_kept(&orbs_render::Frame::new(orbs_render::GridSize::new(4, 2)));
        passing.observe(&showing("laboratory"));
        passing.observe(&showing("forge"));
        let crossing = !passing.is_settled();
        *app.world_mut().resource_mut::<Passing>() = passing;
        crossing
    }

    /// A tower standing in `room`, with no surface open.
    fn showing(room: &str) -> orbs_shell::Showing {
        orbs_shell::Showing::of(
            orbs_shell::Open::default(),
            &Panel {
                room: room.to_owned(),
                ..Panel::default()
            },
            false,
        )
    }
}
