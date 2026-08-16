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

mod boot;
mod crt;
mod render;
mod shell;
mod sim;

use bevy::prelude::*;
use bevy::window::WindowResolution;

/// 1280×720 puts the 960×720 picture at **exactly** native cell size — scale
/// 1.0, an 8×16 glyph — with 160 pixels of bar down each side.
///
/// Both halves are deliberate (DESIGN.md §4, §19). Native size is the sharpest
/// the game ever is, so it is what a first look should get; and the bars mean
/// the 4:3 letterbox is exercised on every run rather than only when someone
/// thinks to drag the window.
const INITIAL_WINDOW: (u32, u32) = (1280, 720);

/// The seed the game starts from until saves exist.
const SEED: u64 = 0x0B5;

/// The seed, or `ORBS_SEED`'s if it names a number.
///
/// **A See-it affordance, not a setting.** Anything the world *generates* — the
/// archive's stacks first, sabotage and sieges later — is one seed's worth of
/// evidence per run, and one sample cannot show a distribution. Three dumps of
/// the same maze looked like proof that randomising it had failed; they were
/// three copies of one seed.
///
/// Tests sweep seeds directly through `Sim::new` and always could. This is the
/// same reach from outside the binary, so a person can look rather than trust a
/// test — which is the whole of §15's gate.
fn seed() -> u64 {
    std::env::var("ORBS_SEED")
        .ok()
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(SEED)
}

fn main() -> AppExit {
    let seed = seed();
    // Before the App, because the whole value of it is needing none of the App.
    // See `shell::dump`.
    if shell::dump(seed, wizard()) {
        return AppExit::Success;
    }

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
                seed,
                wizard: wizard(),
            },
            render::RenderPlugin,
            crt::CrtPlugin,
            shell::ShellPlugin,
            boot::BootPlugin,
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
