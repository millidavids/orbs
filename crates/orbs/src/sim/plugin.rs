//! Registration for the simulation driver.

use bevy::prelude::*;

use super::clock;
use super::driver::{Tower, advance};

/// Owns the simulation and steps it once per world tick.
pub struct SimPlugin {
    /// Master seed. Two runs from the same seed are byte-identical.
    pub seed: u64,
}

impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        // One tick is one real second (DESIGN.md §5.0). FixedUpdate, not
        // Update, so world speed is independent of frame rate — see `clock`.
        app.insert_resource(Tower::new(self.seed))
            .insert_resource(Time::<Fixed>::from_hz(1.0))
            .add_systems(FixedUpdate, advance);

        // Bevy's virtual clock discards any frame delta beyond max_delta, and
        // its 250 ms default would silently cost the tower time on an ordinary
        // hitch. See `clock` for why this is raised rather than removed, and why
        // it is installed in a way that does not depend on plugin order.
        clock::install(app);
    }
}
