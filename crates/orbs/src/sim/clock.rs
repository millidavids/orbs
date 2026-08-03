//! How wall-clock time becomes ticks.
//!
//! DESIGN.md §5.0: **one tick is one real second while the window is open.**
//! That is a promise about the wall clock, not about frames, so the sim is
//! driven from `FixedUpdate` rather than `Update`.
//!
//! # Why this is frame-rate independent
//!
//! Bevy accumulates elapsed *time* and runs `FixedUpdate` in a catch-up loop —
//! roughly:
//!
//! ```text
//! overstep += Time<Virtual>::delta()
//! while overstep >= timestep { overstep -= timestep; run(FixedMain) }
//! ```
//!
//! So the number of `step()` calls is `elapsed / timestep`, whatever the frame
//! rate. At 120 fps the sim steps once per ~120 frames; at 20 fps, once per ~20.
//! Both advance the world one tick per second. `frame_rate_does_not_change_world_speed`
//! asserts exactly that.
//!
//! # Why the default clamp had to be changed
//!
//! `Time<Virtual>` refuses to report a delta larger than `max_delta`, and
//! **silently discards the excess** — the only trace is a `debug!` that is off in
//! release. Bevy's default is 250 ms, chosen so that a laptop resuming from an
//! hour's suspend does not try to simulate an hour.
//!
//! That default is wrong for this game in one direction and right in the other:
//!
//! - **Ordinary hitches must be caught up.** A shader compile or an asset load
//!   that costs 400 ms would silently cost the tower time it was owed. Ticks are
//!   world time (§5.0) and every drift accumulates against a wall clock the
//!   player can see.
//! - **Genuine absences must *not* be caught up here.** Minimise, suspend, or
//!   quit is what offline progression exists for (§5), and it has its own rules —
//!   it is an unlockable, and by default almost nothing accrues. Silently
//!   replaying an hour of ticks on resume would bypass that design entirely.
//!
//! So the clamp is kept, and set deliberately at [`MAX_CATCH_UP`]: long enough
//! that no realistic frame hitch loses time, short enough that a real absence
//! still falls through to the offline path.

use core::time::Duration;

use bevy::prelude::*;

/// The longest stall the tick loop will catch up on.
///
/// Above this, the gap is treated as an absence and left to offline progression
/// (§5) rather than replayed as ticks.
///
/// Five seconds is far above any plausible frame hitch and far below any
/// plausible absence. Catching up costs one `step()` per second of stall, and a
/// step is microseconds, so even the worst case here is unmeasurable.
pub(crate) const MAX_CATCH_UP: Duration = Duration::from_secs(5);

/// A virtual clock configured for this game.
pub(crate) fn configured() -> Time<Virtual> {
    Time::<Virtual>::from_max_delta(MAX_CATCH_UP)
}

/// Apply our settings to whichever virtual clock the app already has.
///
/// Order-independent by construction. Reaching for `resource_mut` during
/// `Plugin::build` panicked outright when `SimPlugin` was added before
/// `TimePlugin` — an ordering requirement nothing documented and no test
/// covered, because `main.rs` and every test happened to get it right. Bevy's
/// `init_resource` will not overwrite a resource that already exists, so
/// inserting a configured clock first is safe whichever plugin lands first.
pub(crate) fn install(app: &mut App) {
    match app.world_mut().get_resource_mut::<Time<Virtual>>() {
        Some(mut time) => time.set_max_delta(MAX_CATCH_UP),
        None => {
            app.insert_resource(configured());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::{SimPlugin, Tower};
    use bevy::time::{TimePlugin, TimeUpdateStrategy};

    /// Run `frames` updates of exactly `frame_time` each, and report world time.
    ///
    /// Bevy's **first** update reports a zero delta — there is no previous
    /// instant to measure from — so a warm-up update runs first and `frames`
    /// really does mean `frames` worth of elapsed time.
    fn ticks_after(frames: u32, frame_time: Duration) -> u64 {
        let mut app = App::new();
        app.add_plugins(TimePlugin)
            .add_plugins(SimPlugin { seed: 1 })
            .insert_resource(TimeUpdateStrategy::ManualDuration(frame_time));

        app.update();
        for _ in 0..frames {
            app.update();
        }
        app.world().resource::<Tower>().tick().get()
    }

    /// Both plugin orders must work; one of them used to panic.
    #[test]
    fn plugin_order_does_not_matter() {
        for label in ["sim first", "time first"] {
            let mut app = App::new();
            if label == "sim first" {
                app.add_plugins(SimPlugin { seed: 1 })
                    .add_plugins(TimePlugin);
            } else {
                app.add_plugins(TimePlugin)
                    .add_plugins(SimPlugin { seed: 1 });
            }
            app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(1)));
            app.update();
            app.update();
            app.update();

            assert_eq!(
                app.world().resource::<Time<Virtual>>().max_delta(),
                MAX_CATCH_UP,
                "{label}: the clock was not configured"
            );
            assert_eq!(
                app.world().resource::<Tower>().tick().get(),
                2,
                "{label}: ticks did not advance"
            );
        }
    }

    /// The question this module exists to answer.
    #[test]
    fn frame_rate_does_not_change_world_speed() {
        // Ten seconds of wall clock at wildly different frame rates. Frame times
        // divide a second exactly, so "ten seconds" is not an approximation.
        let ten_seconds = [
            ("250 fps", 2500, Duration::from_millis(4)),
            ("125 fps", 1250, Duration::from_millis(8)),
            ("50 fps", 500, Duration::from_millis(20)),
            ("25 fps", 250, Duration::from_millis(40)),
            ("10 fps", 100, Duration::from_millis(100)),
            ("5 fps", 50, Duration::from_millis(200)),
        ];

        for (label, frames, frame_time) in ten_seconds {
            let ticks = ticks_after(frames, frame_time);
            assert_eq!(
                ticks, 10,
                "{label} produced {ticks} ticks in ten seconds, not 10"
            );
        }
    }

    #[test]
    fn a_single_slow_frame_is_caught_up_rather_than_lost() {
        // One frame covering three seconds still owes three ticks. Under Bevy's
        // 250 ms default this loses all three.
        assert_eq!(ticks_after(1, Duration::from_secs(3)), 3);
    }

    #[test]
    fn stalls_up_to_the_catch_up_limit_lose_nothing() {
        let ticks = ticks_after(1, MAX_CATCH_UP);
        let expected = MAX_CATCH_UP.as_secs();
        assert_eq!(ticks, expected, "a stall at the limit lost time");
    }

    #[test]
    fn an_absence_is_not_replayed_as_ticks() {
        // An hour's suspend must not silently run 3600 ticks through the live
        // loop; that is offline progression's job (§5), and it deliberately
        // accrues almost nothing.
        let ticks = ticks_after(1, Duration::from_secs(3600));
        assert!(
            ticks <= MAX_CATCH_UP.as_secs(),
            "an hour's absence replayed {ticks} ticks"
        );
    }

    #[test]
    fn sub_tick_frames_accumulate_rather_than_rounding_away() {
        // At 250 fps a frame is a 250th of a tick. If those rounded to zero the
        // world would never move on a fast machine.
        assert_eq!(ticks_after(249, Duration::from_millis(4)), 0);
        assert_eq!(ticks_after(250, Duration::from_millis(4)), 1);
    }

    /// Proof that [`configure`] is load-bearing rather than decoration.
    ///
    /// The same three-second stall, on an app that keeps Bevy's 250 ms default,
    /// loses every tick but one. If a future Bevy changes the default so this
    /// stops being true, this test says so.
    #[test]
    fn bevys_default_clamp_would_lose_the_time_we_now_keep() {
        #[derive(Resource, Default)]
        struct Ticks(u64);

        let mut app = App::new();
        app.add_plugins(TimePlugin)
            .init_resource::<Ticks>()
            .insert_resource(Time::<Fixed>::from_hz(1.0))
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO))
            .add_systems(FixedUpdate, |mut ticks: ResMut<Ticks>| ticks.0 += 1);
        // Deliberately no `configure()` — this app is the counterfactual.
        app.update();

        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(3)));
        app.update();

        let kept = app.world().resource::<Ticks>().0;
        assert!(
            kept < 3,
            "Bevy's default no longer discards the stall; MAX_CATCH_UP may be redundant"
        );
    }

    #[test]
    fn a_variable_frame_rate_still_keeps_wall_clock_time() {
        // Frame times in the real world are not uniform. Ten seconds delivered
        // as a mix of fast and slow frames must still be ten ticks.
        let mut app = App::new();
        app.add_plugins(TimePlugin)
            .add_plugins(SimPlugin { seed: 1 });
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
        app.update();

        let pattern = [4u64, 8, 40, 4, 200, 4, 16, 24, 4, 96];
        let mut elapsed = Duration::ZERO;
        while elapsed < Duration::from_secs(10) {
            for millis in pattern {
                let step = Duration::from_millis(millis);
                app.insert_resource(TimeUpdateStrategy::ManualDuration(step));
                app.update();
                elapsed += step;
            }
        }

        assert_eq!(
            app.world().resource::<Tower>().tick().get(),
            elapsed.as_secs(),
            "variable frame pacing drifted from the wall clock"
        );
    }
}
