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
mod sound;

use bevy::prelude::*;
use bevy::window::WindowResolution;
use orbs_shell::wizard;

/// 1920×1080 puts the 960×720 picture at scale 1.5 — a 12×24 glyph — with 240
/// pixels of bar down each side.
///
/// A half step rather than a whole one, priced in §19: the sampler is
/// `mag: Nearest`, so nearest-neighbour duplicates some source columns and not
/// others. The cell is even on both axes, so 8×16 lands on 12×24 and every cell
/// boundary is a whole pixel — a scale that did not divide the cell would put
/// the grid on half-pixels and moiré against the RGB mask (§4).
///
/// It was 1280×720, ×1.0, the pixels the font designer drew, and that is a real
/// loss. It buys a window that fills a modern display rather than a third of it.
///
/// The bars survive the change: 1440×1080 inside 1920×1080 still leaves 240 a
/// side, so the 4:3 letterbox is exercised on every run.
const INITIAL_WINDOW: (u32, u32) = (1920, 1080);

fn main() -> AppExit {
    // Before the App, because the whole value of it is needing none of the App.
    // See `orbs_shell::dump` — the *shared* dump, so `orbs-tui --dump` prints
    // the same text through the same painters. It chooses its own seed.
    //
    // The engine line is this frontend's to supply: the card is an inventory of
    // the machine and only this binary knows Bevy is in it. The settings list
    // is the caller's for the same reason — a dump builds no `App`, so it has
    // no tube to ask, and the frontends' settings differ anyway.
    if orbs_shell::dump(wizard(), boot::engine(), shell::default_settings()) {
        return AppExit::Success;
    }

    // Before anything reads a save, and once. A player whose towers were beside
    // the binary finds them where the orb now keeps them; the originals are
    // copied rather than moved, so an older build still finds its own. It does
    // nothing when `ORBS_SAVE` names a path or says `off`, or when it has
    // already run, so no instrument in the project reaches it.
    //
    // After the dump, deliberately: a capture keeps nothing and should not be
    // what migrates a player's games.
    orbs_shell::migrate_saves();
    // Settings are kept only once a frontend says so, and this is where this one
    // says it. A process that never calls it gets the defaults — every test
    // binary in the workspace, so a `cargo test` never reads or writes the
    // developer's own settings file. See `settings::store::KEPT_AT`.
    orbs_shell::settings::keep();

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
        // themes belong to the cell renderer. Lifted off true black, because an
        // empty screen at exactly #000000 looks like a crashed one.
        .insert_resource(ClearColor(Color::srgb(0.10, 0.06, 0.15)))
        .add_plugins((
            sim::SimPlugin {
                // A game gets a seed of its own unless `ORBS_SEED` names one or
                // `ORBS_CAPTURE` is taking a screenshot — see
                // `orbs_shell::new_game_seed`. A save on disk outranks it.
                seed: orbs_shell::new_game_seed(),
                wizard: wizard(),
                // The real game keeps its tower between sessions. Only this
                // crate's own tests do not — see `SimPlugin::persist`.
                persist: true,
                // The game waits at its menu; `ORBS_THRESHOLD=0` does not. The
                // switch is the harnesses': `scripts/play.sh` and
                // `scripts/tui.sh` are written to reach a *tower*, and a
                // laboratory scenario that typed past a menu first would be
                // testing the door.
                threshold: orbs_shell::Threshold::chosen(orbs_shell::Threshold::Waiting)
                    .is_waiting(),
            },
            render::RenderPlugin,
            crt::CrtPlugin,
            // After the tube in this list only for readability — what actually
            // orders the two passes is the explicit edge in `SightPlugin`.
            sight::SightPlugin,
            shell::ShellPlugin,
            boot::BootPlugin,
            // Last, and it reads rather than writes: every cue is chosen from a
            // record the transcript has already drawn, so the sound is a view
            // over the frame (rule 2) and nothing above it can come to depend
            // on the orb having a voice.
            sound::SoundPlugin,
        ))
        .run()
}

// Who is at the orb, and which seed the world grows from, are
// `orbs_shell::environment`'s — read outside the sim on purpose, and shared so
// that two frontends cannot derive the prompt name by two different rules.
