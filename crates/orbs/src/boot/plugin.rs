//! Registration for the boot sequence.

use bevy::prelude::*;

use super::drive::{advance, announce, booting};
use orbs_shell::Boot;

/// The orb waking up (DESIGN.md §4).
pub struct BootPlugin;

impl Plugin for BootPlugin {
    fn build(&self, app: &mut App) {
        // Still or not is decided here, once: nothing reaches the settings or
        // the tube while the card is up, since every key waits on `booted`.
        app.insert_resource(Boot::default().held_still(!crate::shell::moving_at_launch()))
            .add_systems(Startup, announce)
            .add_systems(Update, advance.run_if(booting));
    }
}
