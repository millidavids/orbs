//! Registration for the boot sequence.

use bevy::prelude::*;

use super::drive::{advance, announce, booting};
use super::stage::Boot;

/// The orb waking up (DESIGN.md §4).
pub struct BootPlugin;

impl Plugin for BootPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Boot>()
            .add_systems(Startup, announce)
            .add_systems(Update, advance.run_if(booting));
    }
}
