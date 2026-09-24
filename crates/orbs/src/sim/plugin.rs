//! Registration for the simulation driver.

use bevy::prelude::*;

use super::clock;
use super::content;
use super::driver::{Tower, advance};

/// Owns the simulation and steps it once per world tick.
pub struct SimPlugin {
    /// Master seed. Two runs from the same seed are byte-identical.
    pub seed: u64,
    /// Who is at the orb, or `None` to leave the default name.
    ///
    /// §4's framing is *"always inside"* — the player is the wizard — so their
    /// own name at the prompt is the honest thing to show.
    ///
    /// Read in the frontend rather than the sim: the environment is not
    /// deterministic, and reaching for `USER` from a world that must replay
    /// from a seed is a habit worth not starting.
    pub wizard: Option<String>,
    /// Whether this app keeps a tower between sessions.
    ///
    /// A field, not `ORBS_SAVE=off`: an environment variable is process-global
    /// and `cargo test` runs threads, so one test's setting is every test's. A
    /// flag says what it means at the call site — `main` keeps a tower, a test
    /// does not.
    pub persist: bool,
    /// Whether the orb waits at its menu instead of raising a tower.
    ///
    /// Not `persist: false`, which decides whether
    /// [`autosave`](super::persist::autosave) and
    /// [`keep_on_the_way_out`](super::persist::keep_on_the_way_out) are
    /// registered at all — a threshold session without them would reach a tower
    /// through the menu and then never save it. `persist` says *this app keeps
    /// towers*; this says *it has not been given one yet*. The systems are
    /// registered either way and gated on [`playing`](crate::shell::playing).
    pub threshold: bool,
}

/// Build the tower a save describes, or a fresh one if there is none.
///
/// Extracted so the menu's loader is not a second copy of the three-way branch
/// over `Restored`, `Unreadable` and `New`.
///
/// `wizard` names a *new* tower's wizard and is ignored by a restored one —
/// `session::Wizard` is explicit that a save outranks the environment.
pub(crate) fn raise(waiting: orbs_shell::Opened, seed: u64, wizard: Option<&str>) -> Tower {
    match waiting {
        orbs_shell::Opened::Restored(save) => {
            let mut resumed = Tower::restored(&save);
            // Read here because the sim has no wall clock and §19 forbids it
            // acquiring one; the *words* are the sim's, by rule 6. Nothing
            // accrues — §5 puts offline progression in Phase 11a.
            resumed.say_resumed(orbs_shell::away_for(&save));
            resumed
        }
        orbs_shell::Opened::Unreadable => {
            let mut fresh = Tower::fresh(seed);
            if let Some(wizard) = wizard {
                fresh.rename(wizard);
            }
            fresh.say_save_unreadable();
            fresh
        }
        orbs_shell::Opened::New => {
            let mut fresh = Tower::fresh(seed);
            if let Some(wizard) = wizard {
                fresh.rename(wizard);
            }
            fresh
        }
    }
}

impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        // The save outranks the seed and the environment both: `ORBS_SEED` and
        // `USER` build a *new* tower only.
        //
        // At the threshold no save is read at all — the player has not asked
        // for a tower, and writing one back sixty ticks later is what the
        // threshold exists to stop. What is built is a scratch world: it
        // supplies the menu's prose, never ticks, and is replaced by whatever
        // `menuing::swap` puts in front of the player.
        let waiting = if self.persist && !self.threshold {
            orbs_shell::read_save()
        } else {
            orbs_shell::Opened::New
        };
        let tower = raise(waiting, self.seed, self.wizard.as_deref());
        // Where this tower is kept, resolved once and carried — see `Kept`: a
        // path read at the moment of writing is a path read after a swap.
        //
        // `None` at the threshold is the structural guarantee: `keep_now` and
        // `keep` both return early without a path, so a scratch world cannot be
        // written at all. `swap` installs the real one with its tower.
        let kept = super::persist::Kept::at(if self.persist && !self.threshold {
            orbs_shell::save_path()
        } else {
            None
        });

        // One tick is one real second (DESIGN.md §5.0). FixedUpdate, not
        // Update, so world speed is independent of frame rate — see `clock`.
        app.insert_resource(tower)
            .insert_resource(super::driver::Readers::from_environment())
            .insert_resource(kept)
            // Inserted here rather than by `ShellPlugin`: this is the plugin
            // that decided whether a tower was raised, and a second place
            // answering the same question is how the two come to disagree.
            .insert_resource(if self.threshold {
                orbs_shell::Threshold::Waiting
            } else {
                orbs_shell::Threshold::Playing
            })
            .insert_resource(Time::<Fixed>::from_hz(1.0))
            // Gated on boot being over and a tower having been chosen. Not
            // cosmetic: `tower::drift` rolls once per tick, so ticking through
            // a wall-clock animation would advance the RNG stream by however
            // long boot took — the same seed, a different world. A menu is the
            // same argument with a longer clock. A `run_if` on `FixedUpdate` is
            // evaluated per fixed step, so no catch-up burst accrues.
            .add_systems(FixedUpdate, advance.run_if(crate::shell::playing));

        if self.persist {
            app.add_systems(
                FixedUpdate,
                super::persist::autosave
                    .after(advance)
                    .run_if(crate::shell::playing),
            )
            // `Last`, reading `AppExit` rather than ordering against `quit`:
            // three of the four ways out never touch the `Quitting` flag, since
            // `F10` and the close button write an `AppExit` directly. The only
            // place that sees them all is the message they all end by writing.
            //
            // The `playing` gate is a second lock, `Kept` being already `None`
            // at the threshold, because this is the system that loses a
            // player's work when it is wrong.
            .add_systems(
                Last,
                super::persist::keep_on_the_way_out.run_if(crate::shell::playing),
            );
        }

        // Bevy's virtual clock discards frame deltas beyond max_delta, and its
        // 250 ms default would cost the tower time on an ordinary hitch. See
        // `clock` for why it is raised rather than removed, and why installing
        // it does not depend on plugin order.
        clock::install(app);

        // Content hot-reload (rule 6). No-op unless `ORBS_CONTENT` names a
        // directory, so a shipped build starts no thread and reads no path.
        // After `insert_resource(tower)`, which it loads into.
        content::install(app);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimePlugin;

    /// A `persist: true` app standing at the threshold — the shipping shape of
    /// the thing under test, because `persist: false` would make the property
    /// true for the wrong reason.
    fn waiting() -> App {
        let mut app = App::new();
        app.add_plugins(TimePlugin).add_plugins(SimPlugin {
            seed: 1,
            wizard: None,
            persist: true,
            threshold: true,
        });
        app
    }

    #[test]
    fn a_tower_nobody_asked_for_is_kept_nowhere() {
        // The property the threshold exists for, and it is structural:
        // `keep_now` and `keep` both return early without a path.
        //
        // Asserted on `Kept` rather than a directory listing because
        // `save_path()` reads `ORBS_SAVE`, which is process-global while
        // `cargo test` runs threads (`save.rs` records why). This asks the
        // question without touching it.
        let app = waiting();
        assert_eq!(
            app.world().resource::<super::super::Kept>().path(),
            None,
            "the threshold resolved somewhere to write a tower nobody played",
        );
    }

    #[test]
    fn the_threshold_reads_no_save_and_says_nothing_about_one() {
        // A save on disk is not opened, set aside or complained about: the
        // player has not asked for a tower. `raise` would otherwise say *"the
        // tower could not be read"* about a file nobody named.
        let app = waiting();
        let tower = app.world().resource::<Tower>();
        assert_eq!(
            tower.sim().snapshot().world.seed,
            1,
            "the threshold restored a save instead of building a scratch world",
        );
    }

    #[test]
    fn the_clock_does_not_run_at_the_threshold() {
        // The boot card's rule with a longer clock: how long somebody reads a
        // menu is wall-clock time, and a world advancing its RNG stream by it
        // would reach a different place from the same seed.
        let mut app = waiting();
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            core::time::Duration::from_secs(5),
        ));
        for _ in 0..8 {
            app.update();
        }
        assert_eq!(
            app.world().resource::<Tower>().sim().tick().get(),
            0,
            "the scratch world ticked while the player was reading a menu",
        );
    }

    #[test]
    fn a_game_that_was_asked_for_runs_and_is_kept() {
        // The complement, so the three above are properties of the *threshold*
        // rather than of `SimPlugin` having stopped working. `persist: false`
        // for `save.rs`'s reason, so what this holds is the clock.
        let mut app = App::new();
        app.add_plugins(TimePlugin).add_plugins(SimPlugin {
            seed: 1,
            wizard: None,
            persist: false,
            threshold: false,
        });
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            core::time::Duration::from_secs(1),
        ));
        assert!(
            app.world().resource::<orbs_shell::Threshold>().is_playing(),
            "a game that was asked for still stood at the threshold",
        );
        for _ in 0..4 {
            app.update();
        }
        assert!(
            app.world().resource::<Tower>().sim().tick().get() > 0,
            "the world did not run for a game that was asked for",
        );
    }
}
