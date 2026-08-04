//! Registration for the simulation driver.

use bevy::prelude::*;

use super::clock;
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
}

impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        let mut tower = Tower::new(self.seed);
        if let Some(wizard) = &self.wizard {
            tower.rename(wizard);
        }

        // One tick is one real second (DESIGN.md §5.0). FixedUpdate, not
        // Update, so world speed is independent of frame rate — see `clock`.
        app.insert_resource(tower)
            .insert_resource(Time::<Fixed>::from_hz(1.0))
            .add_systems(FixedUpdate, advance);

        // Bevy's virtual clock discards any frame delta beyond max_delta, and
        // its 250 ms default would silently cost the tower time on an ordinary
        // hitch. See `clock` for why this is raised rather than removed, and why
        // it is installed in a way that does not depend on plugin order.
        clock::install(app);
    }
}
