//! Driving the bench from Bevy's clocks, and asking the tube whether to move.
//!
//! [`Bench`] is pure `f32` arithmetic and knows nothing
//! about an engine. This is the five lines that find the two clocks and the
//! motion switch — the one place the shell reaches the GPU side at all, and
//! therefore the one part of the bench a terminal build cannot use.

use bevy::prelude::*;

use orbs_shell::{Bench, Climb, Panel, Passing};

use crate::crt::CrtSettings;

/// Whether the player has asked for stillness — `settings` → `access` →
/// `motion`.
///
/// Never read alone: [`moving`] puts it with the tube, and either one saying
/// *still* is enough.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Stillness(bool);

impl Stillness {
    /// Stillness asked for, or not.
    pub(crate) const fn asked(still: bool) -> Self {
        Self(still)
    }

    /// Whether the player wants nothing to move.
    pub(crate) const fn still(self) -> bool {
        self.0
    }
}

impl Default for Stillness {
    /// `ORBS_MOTION` first, then the row — read once here, so it reaches the
    /// boot card through `moving_at_launch` too.
    fn default() -> Self {
        Self(!orbs_shell::settings::motion_wanted())
    }
}

/// Whether anything may move: the `motion` row, and every tube on.
///
/// The one combiner, so no two clocks can disagree about whether a player asked
/// for motion. `None` is *nobody has said* — no tube yet and the row allowing
/// it — which `Bench` and `Passing` read as leaving their own switch alone.
///
/// Any tube off is still: two disagreeing is not a situation the game creates,
/// and the quieter reading is the only safe one to pick if it ever does.
pub(crate) fn moving<'a>(
    tubes: impl IntoIterator<Item = &'a CrtSettings>,
    stillness: Option<&Stillness>,
) -> Option<bool> {
    if stillness.is_some_and(|stillness| stillness.still()) {
        return Some(false);
    }
    let mut tubes = tubes.into_iter();
    tubes
        .next()
        .map(|first| first.on && tubes.all(|other| other.on))
}

/// What the motion switch will say once the camera exists, for what is decided
/// before it does — the boot card.
pub(crate) fn moving_at_launch() -> bool {
    crate::crt::seeded().on && !Stillness::default().still()
}

/// Advance the fire, the crossing and the rolling numbers, and stop them when
/// the tube is off or the player asked for stillness.
///
/// One system because it is one decision, and the clocks it drives are the
/// things §14 makes disableable: none of `Bench`, `Passing` or `Climb` has any
/// business knowing what a `CrtSettings` is on a frame it is not being driven.
///
/// The crossing's clock is advanced in `revealing::drive_passing`, not here.
/// What this hands it is the switch — the one thing a terminal cannot supply —
/// and both get it from the same read, so the fire and the crossing can never
/// disagree about whether a player asked for motion.
pub(crate) fn advance(
    time: Res<Time>,
    world_tick: Res<Time<Fixed>>,
    tubes: Query<&CrtSettings>,
    stillness: Option<Res<Stillness>>,
    panel: Res<Panel>,
    mut bench: ResMut<Bench>,
    mut passing: ResMut<Passing>,
    mut climb: ResMut<Climb>,
) {
    // No camera yet: `None`, which leaves whatever `ORBS_FIRE` said. A missing
    // switch is not a switch set to off, and reading it as off would make the
    // fire invisible for the frames before the camera spawns.
    //
    // A `Query`, not `Option<Single>`: `Single` yields `None` for two or more
    // matches as readily as for none, so a second camera would land in the
    // "no switch" branch and leave the motion running for a player who turned
    // it off. `crt::settings` records that defect once already.
    let motion = moving(tubes.iter(), stillness.as_deref());

    // The same `Time<Fixed>` the sim runs on (`sim::plugin` installs it at
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
    // The numbers ride the same read, after `refresh_panel` like the bench, so
    // a change is seen on the frame it lands.
    climb.advance(time.delta_secs(), motion, &panel);
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
            .init_resource::<Climb>()
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
        // §14's disableable requirement, wired to a switch a player can reach.
        // F3 cycles the CRT to `OFF`; a player who does that has said they do
        // not want motion, and a shimmering meter ignoring them is the defect
        // `crt::settings` documents — a switch something else can flip back on.
        //
        // Asserted through the *system*, not through `set_enabled`: the risk was
        // never that the flag fails to work, it is that nothing sets it.
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Bench>()
            .init_resource::<Passing>()
            .init_resource::<Climb>()
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
        // `DESIGN.md:9182` trap with it: reduce-motion must cut, never freeze a
        // half-drawn screen. Three animations learned that independently.
        //
        // Through the *system*, for `killing_the_tube_kills_the_fire`'s reason:
        // the risk is never that the flag fails to work, it is that nothing sets
        // it.
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Bench>()
            // Permitted rather than read, or an exported `ORBS_PASSAGE=0`
            // fails the control below.
            .insert_resource(Passing::new(true))
            .init_resource::<Climb>()
            .init_resource::<Panel>()
            .add_systems(Update, advance);
        let camera = app.world_mut().spawn(CrtSettings::DEFAULT).id();

        app.update();
        assert!(crosses(&mut app), "screens did not cross with the tube on");

        app.world_mut().entity_mut(camera).insert(CrtSettings::OFF);
        app.update();
        // Before the probe, which keeps a screen of its own: `crosses` has to
        // put one there to have something to cross out of, so asking afterwards
        // would read the probe rather than the system.
        assert!(
            app.world().resource::<Passing>().kept().is_empty(),
            "a kept screen survived the switch — 43 KiB a player asked not to pay",
        );
        assert!(!crosses(&mut app), "F3 to OFF left screens crossing");
    }

    #[test]
    fn killing_the_tube_stops_the_numbers_rolling() {
        // §14 for the third clock: through the system, for
        // `killing_the_tube_kills_the_fire`'s reason.
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Bench>()
            .init_resource::<Passing>()
            .init_resource::<Climb>()
            .init_resource::<Panel>()
            .add_systems(Update, advance);
        let camera = app.world_mut().spawn(CrtSettings::DEFAULT).id();

        assert!(rolls(&mut app), "a gain did not roll with the tube on");
        app.world_mut().entity_mut(camera).insert(CrtSettings::OFF);
        assert!(!rolls(&mut app), "F3 to OFF left the numbers rolling");
    }

    #[test]
    fn asking_for_stillness_stops_everything_with_the_tube_on() {
        // §14's open box: no motion, and the curve kept. Through the systems,
        // for `killing_the_tube_kills_the_fire`'s reason.
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Bench>()
            .init_resource::<Passing>()
            .init_resource::<Climb>()
            .init_resource::<Panel>()
            .init_resource::<crate::render::blink::Blink>()
            .insert_resource(Stillness::asked(true))
            // A fixed step, so a caret that moved is a caret seen moving.
            .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
                core::time::Duration::from_millis(400),
            ))
            .add_systems(Update, (advance, crate::render::blink::tick));
        app.world_mut().spawn(CrtSettings::DEFAULT);

        // 1.2 s is more than a blink: a caret that moved would show.
        for _ in 0..3 {
            app.update();
            assert!(
                app.world()
                    .resource::<crate::render::blink::Blink>()
                    .showing(),
                "the caret blinked for a still player",
            );
        }
        assert!(
            !app.world().resource::<Bench>().grind(true, false).working,
            "the fire moved for a player who asked for stillness",
        );
        assert!(!crosses(&mut app), "screens crossed for a still player");
        assert!(!rolls(&mut app), "numbers rolled for a still player");
    }

    #[test]
    fn the_row_cannot_bring_motion_back_with_the_tube_off() {
        // The state §14 must not reach: motion asked for, the tube saying off.
        let off = [CrtSettings::OFF];
        assert_eq!(moving(&off, Some(&Stillness::asked(false))), Some(false));
        let on = [CrtSettings::DEFAULT];
        assert_eq!(moving(&on, Some(&Stillness::asked(true))), Some(false));
        assert_eq!(moving(&on, Some(&Stillness::asked(false))), Some(true));
        // No tube yet and the row allowing it: nobody has said.
        assert_eq!(moving(&[], Some(&Stillness::asked(false))), None);
        // ...but the row saying still needs no tube to be believed.
        assert_eq!(moving(&[], Some(&Stillness::asked(true))), Some(false));
    }

    /// Whether a gain on the ley gauge would be drawn rolling, right now.
    fn rolls(app: &mut App) -> bool {
        let at = |done| orbs_sim::Toward {
            done,
            span: 100,
            ahead: orbs_sim::Ahead::Tier(100),
        };
        app.world_mut().resource_mut::<Panel>().station = at(0);
        app.update();
        app.world_mut().resource_mut::<Panel>().station = at(90);
        app.update();
        let shown = app
            .world()
            .resource::<Climb>()
            .toward(orbs_shell::Watch::Ley, at(90))
            .shown;
        shown != 90
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
