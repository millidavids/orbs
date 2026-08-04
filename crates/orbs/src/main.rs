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
//! F10 leaves the orb.

mod crt;
mod render;
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
        .add_plugins((
            sim::SimPlugin {
                seed: 0x0B5,
                wizard: wizard(),
            },
            render::RenderPlugin,
            crt::CrtPlugin,
            shell::ShellPlugin,
        ))
        .run()
}

/// Who is at the orb.
///
/// §4's framing is *"always inside"* — the player never sees the wizard, because
/// the player **is** the wizard — so the prompt wears their own name.
///
/// | Source | Wins when |
/// |---|---|
/// | `ORBS_WIZARD` | set, and not blank |
/// | `USER` | POSIX — macOS and Linux |
/// | `USERNAME` | Windows, which does not set `USER` |
/// | the orb's own name | none of them do |
///
/// `USERNAME` is not optional politeness: §13 ships Windows through Steam, so
/// leaving it out would mean the platform most players are on always falls back
/// to `orbs $ ` while the two development platforms quietly look right.
///
/// `ORBS_WIZARD` exists because the alternative was renaming a wizard by
/// overriding a system variable, which works by accident rather than by
/// intention. The real answer is a settings screen, which §15 puts in Phase 5
/// alongside the rest of the options; until then this is the switch.
///
/// Read here rather than inside the sim, deliberately: the environment is not
/// deterministic, and although a name feeds nothing but the prompt, reaching for
/// it from inside a world that must replay identically from a seed is a habit
/// worth not starting.
fn wizard() -> Option<String> {
    ["ORBS_WIZARD", "USER", "USERNAME"]
        .into_iter()
        .filter_map(|key| std::env::var(key).ok())
        .find(|name| !name.trim().is_empty())
}
