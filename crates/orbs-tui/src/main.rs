//! O.R.B.S. — terminal frontend.
//!
//! A full-screen raw-mode application. It does not shell out, touch the real
//! filesystem, or interoperate with the host shell — the terminal is a
//! framebuffer and a keyboard. Same sim, same commands, same simulated tower;
//! only the rasteriser differs.
//!
//! Second-class by decision, not by neglect (DESIGN.md §13, §19). The Bevy
//! build is the product and never waits for this one. This one is for §15's
//! charter: *"a no-GPU, no-window, instant-startup, trivially scriptable
//! target"* — the scripting being `tmux send-keys`, which makes every screen
//! something a person or a script can drive and read back.
//!
//! ```bash
//! cargo build -p orbs-tui
//! tmux new-session -d -s orbs -x 120 -y 45 target/debug/orbs-tui
//! tmux send-keys -t orbs 'attend laboratory' Enter
//! tmux capture-pane -t orbs -p
//! ```
//!
//! It boots, and `ORBS_BOOT=0` stops it. §4's sequence runs here too, naming
//! `crossterm` where the other build names `bevy` — the card is a diegetic
//! inventory of the machine, and this is a different machine. Skipping it for
//! *"instant-startup"* (§15) was really about the development loop, which
//! `ORBS_BOOT=0` already answers. The clock and the card are `orbs-shell`'s;
//! here is the frame that advances one and the engine line that fills the other.
//!
//! No content watcher: `ORBS_CONTENT` is read once at startup. A watcher is
//! `notify` and a background thread, and it belongs to the frontend that already
//! has one. Edit `prose.toml` and restart.
//!
//! No `tracing` subscriber either. In raw mode a subscriber writing to stderr
//! would scribble across the screen at the moment a writer is looking at their
//! own edit; with none, the macro compiles to nothing. The `--dump` path owns
//! the terminal in the ordinary way and prints its own warning instead.

mod blit;
mod drive;
mod surfaces;
mod term;
mod theme;

use std::io::{IsTerminal, stdout};
use std::process::ExitCode;

use drive::default_settings;
use orbs_shell::wizard;
use orbs_sim::Sim;

/// The crossterm version this binary is built against.
///
/// Held to `Cargo.toml` by a test, as the Bevy build holds its own pin. Cargo
/// exposes no `env!` for a dependency's version, and guessing one on the POST
/// card would be the lie that card exists to avoid.
const CROSSTERM: &str = "0.29";

/// What this binary is built out of, for the POST card's third line.
///
/// The card is a diegetic inventory of the machine (§4), so it must describe
/// *this* machine: the Bevy build says `bevy 0.19.0` and this one must not.
fn engine() -> String {
    format!("crossterm {CROSSTERM}")
}

fn main() -> ExitCode {
    // `--dump "attend laboratory; grind sage"` prints one frame as text and
    // exits, through the same painters the Bevy build's `ORBS_DUMP` uses: two
    // binaries, one screen. A dump chooses its own seed — the instrument's,
    // `orbs_shell::seed` — so nothing here can hand it a game's.
    let mut args = std::env::args().skip(1);
    if let Some(flag) = args.next() {
        if flag == "--dump" {
            let script = args.next().unwrap_or_else(|| "1".to_owned());
            orbs_shell::dump_script(wizard(), &engine(), default_settings(), &script);
            return ExitCode::SUCCESS;
        }
        eprintln!("usage: orbs-tui [--dump <commands>]");
        return ExitCode::FAILURE;
    }
    // ...and `ORBS_DUMP` works here too, so a See-it line written for one
    // frontend runs against the other unchanged.
    if orbs_shell::dump(wizard(), &engine(), default_settings()) {
        return ExitCode::SUCCESS;
    }

    if !stdout().is_terminal() {
        eprintln!("orbs-tui needs a terminal. Try `--dump` for one frame as text.");
        return ExitCode::FAILURE;
    }

    // Before anything reads a save, and once — the same call the other build
    // makes, with the same guards. See `orbs`'s `main`.
    orbs_shell::migrate_saves();
    // The same opt-in the other build makes. See `orbs`'s `main`.
    orbs_shell::settings::keep();

    // A game gets a seed of its own unless `ORBS_SEED` names one — which the
    // play harness always does, so its scenarios stay the worlds they were
    // written against. A save on disk outranks it inside `play`.
    match play(orbs_shell::new_game_seed()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            // `play` restores the terminal before returning, so this lands on a
            // screen that can show it.
            eprintln!("orbs-tui: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Take the terminal, play, and give it back.
fn play(seed: u64) -> std::io::Result<()> {
    // The game waits at its menu; `ORBS_THRESHOLD=0` does not. The switch is the
    // harnesses': `scripts/play.sh` drives this binary under `tmux` to reach a
    // *tower*, and a laboratory scenario that typed its way past a menu first
    // would be testing the door.
    let threshold = orbs_shell::Threshold::chosen(orbs_shell::Threshold::Waiting);

    // The save outranks the seed and the environment both, as in the other
    // build. `session::Wizard` goes the other way round: a name is world state,
    // so it is read from the machine only when there is no world yet.
    //
    // At the threshold no save is read at all — the player has not asked for a
    // tower, and putting one in front of them (then writing it back a minute
    // later) is what the threshold exists to stop. This scratch world supplies
    // the prose the menu is drawn from and never ticks; a chosen tower arrives
    // through `drive::run`'s swap.
    let opened = if threshold.is_waiting() {
        orbs_shell::Opened::New
    } else {
        orbs_shell::read_save()
    };
    let mut sim = match opened {
        orbs_shell::Opened::Restored(save) => {
            let mut resumed = Sim::restored(&save);
            resumed.say_resumed(orbs_shell::away_for(&save));
            resumed
        }
        orbs_shell::Opened::Unreadable => {
            let mut fresh = orbs_shell::fresh(seed, true, orbs_sim::content::Length::Medium);
            if let Some(name) = wizard() {
                fresh.rename(&name);
            }
            fresh.say_save_unreadable();
            fresh
        }
        orbs_shell::Opened::New => {
            let mut fresh = orbs_shell::fresh(seed, true, orbs_sim::content::Length::Medium);
            if let Some(name) = wizard() {
                fresh.rename(&name);
            }
            fresh
        }
    };
    // Authored content, if `ORBS_CONTENT` names a directory (rule 6). Read once,
    // not watched — a watcher belongs to the frontend that already has one.
    // The complaint goes to `tracing`, which this build subscribes to; the dump
    // prints its own line.
    for file in orbs_shell::load(&mut sim) {
        tracing::warn!("content: {file} did not load; running on the built-in text");
    }

    term::enter()?;
    let result = drive::run(sim, threshold, engine());
    term::leave();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_crossterm_version_is_the_one_the_manifest_asks_for() {
        // A boot report that could go stale is a lie the player reads first.
        // The same test the Bevy build keeps beside its own pin.
        let manifest = include_str!("../Cargo.toml");
        assert!(
            manifest.contains(&format!("crossterm = \"{CROSSTERM}\"")),
            "the card says crossterm {CROSSTERM}, which Cargo.toml does not ask for",
        );
    }

    #[test]
    fn the_engine_line_is_drawable() {
        // The repertoire is the intersection of what both frontends can draw, so
        // a line only this one can render would break the boundary going out.
        for glyph in engine().chars() {
            assert!(
                orbs_render::is_renderable(glyph),
                "{glyph:?} in the engine line cannot be drawn"
            );
        }
    }
}
