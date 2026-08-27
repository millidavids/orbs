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
mod sight;
mod sim;

use bevy::prelude::*;
use bevy::window::WindowResolution;
use orbs_shell::{seed, wizard};

/// 1920×1080 puts the 960×720 picture at scale **1.5** — a 12×24 glyph — with
/// 240 pixels of bar down each side.
///
/// **A half step rather than a whole one, and §19 already priced it.** The
/// atlas sampler is `mag: Nearest`, so the glyph is not re-rasterised at 1.5 —
/// nearest-neighbour duplicates some source columns and not others, which §19
/// describes as *"a regular 2,1,2,1 alternation, masked by the CRT bloom."*
/// That regularity is what makes it survivable: the cell is even on both axes,
/// so 8×16 lands on 12×24 and every cell boundary is still a whole pixel. A
/// scale that did not divide the cell would put the grid's rules on half-pixels
/// and moiré against the RGB mask, which §4 names as the top legibility hazard.
///
/// It was 1280×720, which is ×1.0 — the sharpest the game ever is, the pixels
/// the font designer actually drew. That is a real loss and worth naming rather
/// than glossing. It is traded for the thing a first look wants: a window that
/// fills a modern display instead of occupying a third of it. §19's *"integer
/// steps or continuous fit?"* entry made the same trade one level down and for
/// the same reason — *"crisp everywhere but fills the window nowhere."*
///
/// **The bars survive the change, and they matter.** 1440×1080 inside 1920×1080
/// still leaves 240 a side, so the 4:3 letterbox is exercised on every run
/// rather than only when someone thinks to drag the window.
const INITIAL_WINDOW: (u32, u32) = (1920, 1080);

fn main() -> AppExit {
    let seed = seed();
    // Before the App, because the whole value of it is needing none of the App.
    // See `orbs_shell::dump` — and note it is the *shared* dump, so
    // `orbs-tui --dump` prints the same text through the same painters.
    //
    // The engine line is the one thing this frontend has to tell it: the card is
    // an inventory of the machine, and only this binary knows Bevy is in it.
    if orbs_shell::dump(seed, wizard(), boot::engine()) {
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
            // After the tube in this list only for readability — what actually
            // orders the two passes is the explicit edge in `SightPlugin`.
            sight::SightPlugin,
            shell::ShellPlugin,
            boot::BootPlugin,
        ))
        .run()
}

// Who is at the orb, and which seed the world grows from, are
// `orbs_shell::environment`'s — read outside the sim on purpose, and shared so
// that two frontends cannot derive the prompt name by two different rules.
