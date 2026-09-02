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
    /// §4's framing is *"always inside"* — the player never sees the wizard,
    /// because the player **is** the wizard — so their own name at the prompt is
    /// the honest thing to show.
    ///
    /// Read here in the frontend rather than inside the sim, deliberately: the
    /// environment is not deterministic, and although a name feeds nothing but
    /// the prompt, reaching for `USER` from inside a world that must replay
    /// identically from a seed is a habit worth not starting.
    pub wizard: Option<String>,
    /// Whether this app keeps a tower between sessions.
    ///
    /// # Why a field and not the environment
    ///
    /// `ORBS_SAVE=off` exists and would work, but the tests in this crate build
    /// a `SimPlugin` and run it for a few hundred ticks — under `cargo test` the
    /// working directory is the crate root, so they would *read* whatever save
    /// is lying there and *write* one every sixty ticks. Setting an environment
    /// variable to stop that is process-global and `cargo test` runs threads, so
    /// one test's setting is every test's.
    ///
    /// An explicit flag has neither problem and says what it means at the call
    /// site: `main` keeps a tower, a test does not.
    pub persist: bool,
}

impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        // **The save outranks the seed and the environment both.** A world that
        // already exists is the one the player left; `ORBS_SEED` and `USER` are
        // what a *new* tower is built from, and `session::Wizard` says the same
        // thing from the other side.
        let waiting = if self.persist {
            orbs_shell::read_save()
        } else {
            orbs_shell::Opened::New
        };
        let tower = match waiting {
            orbs_shell::Opened::Restored(save) => {
                let mut resumed = Tower::restored(&save);
                // The gap is read here because the sim has no wall clock and §19
                // forbids it acquiring one; the *words* are the sim's, because
                // rule 6 puts prose in content files. Nothing accrues for it —
                // §5 puts offline progression in Phase 11a.
                resumed.say_resumed(orbs_shell::away_for(&save));
                resumed
            }
            orbs_shell::Opened::Unreadable => {
                let mut fresh = Tower::new(self.seed);
                if let Some(wizard) = &self.wizard {
                    fresh.rename(wizard);
                }
                fresh.say_save_unreadable();
                fresh
            }
            orbs_shell::Opened::New => {
                let mut fresh = Tower::new(self.seed);
                if let Some(wizard) = &self.wizard {
                    fresh.rename(wizard);
                }
                fresh
            }
        };

        // One tick is one real second (DESIGN.md §5.0). FixedUpdate, not
        // Update, so world speed is independent of frame rate — see `clock`.
        app.insert_resource(tower)
            .insert_resource(Time::<Fixed>::from_hz(1.0))
            // Gated on the boot sequence being over. Not cosmetic: `tower::drift`
            // rolls once per tick, so ticking through a wall-clock animation
            // would advance the RNG stream by an amount that depends on how long
            // boot took and whether anyone skipped it — the same seed would build
            // a different world. A `run_if` on a `FixedUpdate` system is
            // evaluated per fixed step, so no catch-up burst accrues at the end.
            .add_systems(FixedUpdate, advance.run_if(crate::boot::booted));

        if self.persist {
            app.add_systems(
                FixedUpdate,
                super::persist::autosave
                    .after(advance)
                    .run_if(crate::boot::booted),
            )
            // **`Last`, and reading `AppExit` rather than ordering against
            // `quit`.** Three of the four ways out of this build never touch the
            // `Quitting` flag — `F10` writes an `AppExit` directly, so does the
            // window's close button — so the only place that sees them all is
            // the message every one of them ends by writing.
            .add_systems(Last, super::persist::keep_on_the_way_out);
        }

        // Bevy's virtual clock discards any frame delta beyond max_delta, and
        // its 250 ms default would silently cost the tower time on an ordinary
        // hitch. See `clock` for why this is raised rather than removed, and why
        // it is installed in a way that does not depend on plugin order.
        clock::install(app);

        // Content hot-reload (rule 6). No-op unless `ORBS_CONTENT` names a
        // directory, so a shipped build starts no thread and reads no path.
        // After `insert_resource(tower)`, which it loads into.
        content::install(app);
    }
}
