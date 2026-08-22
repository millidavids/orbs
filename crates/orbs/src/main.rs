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
use orbs_shell::{seed, wizard};

/// 1280×720 puts the 960×720 picture at **exactly** native cell size — scale
/// 1.0, an 8×16 glyph — with 160 pixels of bar down each side.
///
/// Both halves are deliberate (DESIGN.md §4, §19). Native size is the sharpest
/// the game ever is, so it is what a first look should get; and the bars mean
/// the 4:3 letterbox is exercised on every run rather than only when someone
/// thinks to drag the window.
const INITIAL_WINDOW: (u32, u32) = (1280, 720);

fn main() -> AppExit {
    let seed = seed();
    // Before the App, because the whole value of it is needing none of the App.
    // See `orbs_shell::dump` — and note it is the *shared* dump, so
    // `orbs-tui --dump` prints the same text through the same painters.
    //
    // The engine line is the one thing this frontend has to tell it: the card is
    // an inventory of the machine, and only this binary knows Bevy is in it.
    if orbs_shell::dump(seed, wizard(), &boot::engine()) {
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
                // The real game keeps its tower between sessions. Only this
                // crate's own tests do not — see `SimPlugin::persist`.
                persist: true,
            },
            render::RenderPlugin,
            crt::CrtPlugin,
            shell::ShellPlugin,
            boot::BootPlugin,
        ))
        .run()
}

// Who is at the orb, and which seed the world grows from, are
// `orbs_shell::environment`'s — read outside the sim on purpose, and shared so
// that two frontends cannot derive the prompt name by two different rules.
