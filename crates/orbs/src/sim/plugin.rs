//! Registration for the simulation driver.

use bevy::prelude::*;

use super::driver::{Tower, advance};

/// Owns the simulation and steps it once per world tick.
pub struct SimPlugin {
    /// Master seed. Two runs from the same seed are byte-identical.
    pub seed: u64,
}

impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Tower::new(self.seed))
            // One tick is one real second (DESIGN.md §5.0).
            .insert_resource(Time::<Fixed>::from_hz(1.0))
            .add_systems(FixedUpdate, advance);
    }
}
