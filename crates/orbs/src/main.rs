//! O.R.B.S. — Bevy frontend. The Steam product.
//!
//! A *caller* of the simulation, never its host: [`sim::SimPlugin`] owns an
//! [`orbs_sim::Sim`] and drives it with `step()` from `FixedUpdate`. Bevy's
//! scheduler never runs sim systems. See CLAUDE.md, architectural rule 3.
//!
//! ```text
//! cargo run -p orbs
//! cargo run -p orbs --release
//! ```
//!
//! Escape leaves the orb.

mod shell;
mod sim;

use bevy::prelude::*;
use bevy::window::WindowResolution;

/// 1280×720 is the smallest window that hosts the 80×22 floor, at a 2× cell
/// (DESIGN.md §4, §9). Starting here means the floor is exercised on every run
/// rather than only when someone thinks to test it.
const INITIAL_WINDOW: (u32, u32) = (1280, 720);

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "O.R.B.S.".to_owned(),
                resolution: WindowResolution::new(INITIAL_WINDOW.0, INITIAL_WINDOW.1),
                ..default()
            }),
            ..default()
        }))
        // Muted violet, the default phosphor (§4). A placeholder — the real
        // themes belong to the cell renderer.
        //
        // Deliberately lifted off true black. Until the cell renderer draws into
        // this window it is an empty screen, and an empty screen that is exactly
        // #000000 is indistinguishable from a crashed one.
        .insert_resource(ClearColor(Color::srgb(0.10, 0.06, 0.15)))
        .add_plugins((sim::SimPlugin { seed: 0x0B5 }, shell::ShellPlugin))
        .run()
}
