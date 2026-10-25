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
    /// The player is the wizard (§4). Read here, since the environment is not
    /// deterministic.
    pub wizard: Option<String>,
    /// Whether this app keeps a tower between sessions.
    ///
    /// A field, not `ORBS_SAVE=off`, which is process-global under threaded
    /// tests.
    pub persist: bool,
    /// Whether the orb waits at its menu instead of raising a tower.
    ///
    /// Not `persist: false`: this says *no tower yet*, and
    /// [`autosave`](super::persist::autosave) and
    /// [`keep_on_the_way_out`](super::persist::keep_on_the_way_out) stay
    /// registered, gated on [`playing`](crate::shell::playing).
    pub threshold: bool,
}

impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        // The save outranks the seed and the environment both: `ORBS_SEED` and
        // `USER` build a *new* tower only.
        //
        // At the threshold, no save: a scratch world for the menu's prose,
        // never ticked, replaced by `menuing::swap`.
        let waiting = if self.persist && !self.threshold {
            orbs_shell::read_save()
        } else {
            orbs_shell::Opened::New
        };
        // `tower_from` is the one copy both builds start from. A load from the
        // menu is `Raising::raise`, which never raises a fresh tower.
        let tower = Tower::raised(orbs_shell::tower_from(
            waiting,
            self.seed,
            self.wizard.as_deref(),
        ));
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
            // After boot and with a tower, or a wall-clock wait would advance
            // the RNG and change the seed's world. Per fixed step, so no
            // catch-up accrues.
            .add_systems(FixedUpdate, advance.run_if(crate::shell::playing));

        if self.persist {
            app.add_systems(
                FixedUpdate,
                super::persist::autosave
                    .after(advance)
                    .run_if(crate::shell::playing),
            )
            // `Last`, on `AppExit`, the one message every way out writes.
            // `playing` is a second lock beside `Kept`.
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
        // On `Kept`, not a directory, since `ORBS_SAVE` is process-global.
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
