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

/// How long the player may be idle before the tower stops keeping time.
///
/// DESIGN.md §5.0: *"a **~60s inactivity grace** so pausing to think is never
/// punished."* Past it the player is not thinking, they are elsewhere, and the
/// tower has no business ageing while nobody is at the orb.
///
/// # What this does and does not buy
///
/// It does **not** make a short pause free — a thirty-second think costs thirty
/// ticks. The sentence immediately after the grace in §5.0 is what makes that
/// acceptable and is easy to read past: *"Drift and decay rates are slow **per
/// tick**; the clock itself is not."* Thirty ticks of drift is nothing. Sixty
/// minutes of it is not, and that is what the grace is for.
///
/// It also makes an idle open window equivalent to a closed one, which §5's
/// offline section requires in as many words — *"drift and aberrations are
/// damped identically whether the window is open and idle or closed"* — and
/// which is what keeps quitting **neutral** rather than optimal.
pub(crate) const GRACE: Duration = Duration::from_secs(60);

/// How long since the player last did anything.
///
/// Real time, not virtual: this is a fact about the person, so nothing that
/// scales or pauses the world may scale or pause it.
#[derive(Resource, Debug, Default)]
pub(crate) struct Idle(Duration);

impl Idle {
    /// Whether somebody is at the orb, and therefore whether the clock runs.
    pub(crate) fn attending(&self) -> bool {
        self.0 < GRACE
    }

    /// Note that the player did something.
    pub(crate) const fn stir(&mut self) {
        self.0 = Duration::ZERO;
    }

    /// Let real time pass.
    pub(crate) fn age(&mut self, delta: Duration) {
        self.0 = self.0.saturating_add(delta);
    }
}

/// Any key means somebody is there.
///
/// **Keystrokes, not submitted lines.** §5.0 requires that *"a fast typist gains
/// nothing over a slow one"*, and charging the clock for the time spent composing
/// a long command is exactly that penalty wearing a different hat.
///
/// `Option<Res<_>>` so `SimPlugin` does not acquire a dependency on `InputPlugin`
/// — the headless clock tests build an app with neither, and a `MessageReader`
/// over an unregistered message panics rather than reading nothing. An app with
/// no input simply has nobody at the orb, which is the right answer anyway.
pub(crate) fn stir(keys: Option<Res<ButtonInput<KeyCode>>>, mut idle: ResMut<Idle>) {
    if keys.is_some_and(|keys| keys.get_just_pressed().next().is_some()) {
        idle.stir();
    }
}

/// Age the idle timer.
///
/// **A pending disambiguation prompt is not activity**, and this is the whole of
/// how that is enforced: nothing but a keystroke calls [`Idle::stir`], so a
/// question left open on screen ages exactly like an empty one. §5.0 calls the
/// alternative out by name.
pub(crate) fn age(time: Res<Time<Real>>, mut idle: ResMut<Idle>) {
    idle.age(time.delta());
}

/// Run condition: the tower's clock is running.
pub(crate) fn attending(idle: Res<Idle>) -> bool {
    idle.attending()
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
            .add_plugins(SimPlugin {
                seed: 1,
                wizard: None,
            })
            .insert_resource(TimeUpdateStrategy::ManualDuration(frame_time));

        app.update();
        for _ in 0..frames {
            app.update();
        }
        app.world().resource::<Tower>().sim().tick().get()
    }

    /// Both plugin orders must work; one of them used to panic.
    #[test]
    fn plugin_order_does_not_matter() {
        for label in ["sim first", "time first"] {
            let mut app = App::new();
            if label == "sim first" {
                app.add_plugins(SimPlugin {
                    seed: 1,
                    wizard: None,
                })
                .add_plugins(TimePlugin);
            } else {
                app.add_plugins(TimePlugin).add_plugins(SimPlugin {
                    seed: 1,
                    wizard: None,
                });
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
                app.world().resource::<Tower>().sim().tick().get(),
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
        app.add_plugins(TimePlugin).add_plugins(SimPlugin {
            seed: 1,
            wizard: None,
        });
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
            app.world().resource::<Tower>().sim().tick().get(),
            elapsed.as_secs(),
            "variable frame pacing drifted from the wall clock"
        );
    }
}

#[cfg(test)]
mod grace {
    use super::*;
    use crate::sim::{SimPlugin, Tower};
    use bevy::input::InputPlugin;
    use bevy::input::keyboard::{Key, KeyboardInput};
    use bevy::time::{TimePlugin, TimeUpdateStrategy};

    /// An app with a keyboard, so the idle timer has something to be stirred by.
    fn attended() -> App {
        let mut app = App::new();
        app.add_plugins((TimePlugin, InputPlugin))
            .add_plugins(SimPlugin {
                seed: 1,
                wizard: None,
            })
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
        app.update();
        app
    }

    /// Let `seconds` of real time pass, one second per frame, touching nothing.
    fn wait(app: &mut App, seconds: u64) {
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(1)));
        for _ in 0..seconds {
            app.update();
        }
    }

    fn tick(app: &App) -> u64 {
        app.world().resource::<Tower>().sim().tick().get()
    }

    fn press(app: &mut App) {
        app.world_mut().write_message(KeyboardInput {
            key_code: KeyCode::KeyA,
            logical_key: Key::Character("a".into()),
            state: bevy::input::ButtonState::Pressed,
            text: Some("a".into()),
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
    }

    #[test]
    fn a_short_pause_still_costs_its_ticks() {
        // The grace is **not** a free-thinking window, and reading it as one is
        // the easy mistake. §5.0's next sentence is what makes a short pause
        // acceptable: "drift and decay rates are slow *per tick*; the clock
        // itself is not." Thirty ticks of drift is nothing.
        let mut app = attended();
        wait(&mut app, 30);
        assert_eq!(tick(&app), 30, "the tower stopped keeping time too early");
    }

    #[test]
    fn the_tower_stops_keeping_time_once_nobody_is_there() {
        // §5.0's grace. Past it the player is not thinking, they are elsewhere.
        let mut app = attended();
        wait(&mut app, GRACE.as_secs() + 30);
        assert_eq!(
            tick(&app),
            GRACE.as_secs(),
            "the tower kept ageing with nobody at the orb",
        );
    }

    #[test]
    fn an_idle_open_window_ends_up_equivalent_to_a_closed_one() {
        // What the grace buys, and why §5 needs it: "drift and aberrations are
        // damped identically whether the window is open and idle or closed."
        // Without this, leaving the game running is strictly worse than quitting
        // — the inversion §5 spends a section preventing.
        let mut app = attended();
        wait(&mut app, GRACE.as_secs() * 10);
        assert_eq!(tick(&app), GRACE.as_secs());
    }

    #[test]
    fn a_keystroke_brings_the_clock_back() {
        let mut app = attended();
        wait(&mut app, GRACE.as_secs() + 5);
        let asleep = tick(&app);

        press(&mut app);
        wait(&mut app, 10);
        assert!(
            tick(&app) > asleep,
            "the tower never woke up: {asleep} then {}",
            tick(&app),
        );
    }

    #[test]
    fn a_pending_prompt_does_not_hold_the_clock_open() {
        // §5.0 writes this one as a hard rule: the grace timer is "**not** reset
        // by a pending disambiguation prompt — otherwise a player could freeze
        // the tower indefinitely by leaving one open."
        //
        // `decoct` with no argument the world can resolve is exactly that: §6
        // numbers the readings and waits for a digit.
        let mut app = attended();
        // §7 makes a domain's belongings nameable only from inside it, so the
        // parser has nothing to offer readings of until the player is stood in
        // the alembic. One tick to let `attend` run.
        app.world_mut()
            .resource_mut::<Tower>()
            .submit("attend alembic");
        wait(&mut app, 1);
        // The keystroke that would have typed the line, then the line. Pressing
        // first is what makes this the scenario §5.0 describes: the grace runs
        // from the player's last *input*, and the question is whether the prompt
        // it opened keeps renewing it.
        press(&mut app);
        app.update();
        app.world_mut()
            .resource_mut::<Tower>()
            .submit("decoct nonsense");
        app.update();
        assert!(
            !app.world().resource::<Tower>().sim().choices().is_empty(),
            "the test never opened a prompt",
        );

        let typed = tick(&app);
        wait(&mut app, GRACE.as_secs() * 4);
        let aged = tick(&app) - typed;
        assert!(
            aged <= GRACE.as_secs(),
            "an open prompt renewed the grace: {aged} ticks past the keystroke",
        );

        // And it really has stopped, rather than merely slowed.
        let settled = tick(&app);
        wait(&mut app, 30);
        assert_eq!(tick(&app), settled, "the tower never stopped");
    }
}
