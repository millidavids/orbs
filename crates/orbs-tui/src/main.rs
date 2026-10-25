//! O.R.B.S. — terminal frontend.
//!
//! Raw mode over a framebuffer and a keyboard; same sim, another rasteriser.
//! Second-class by decision (§13, §19): §15's no-GPU, scriptable target,
//! driven by `tmux send-keys`.
//!
//! ```bash
//! cargo build -p orbs-tui
//! tmux new-session -d -s orbs -x 120 -y 45 target/debug/orbs-tui
//! tmux send-keys -t orbs 'attend laboratory' Enter
//! tmux capture-pane -t orbs -p
//! ```
//!
//! It boots, naming `crossterm` on the card (§4); `ORBS_BOOT=0` skips it.
//! `ORBS_CONTENT` is read at launch and on each swap, with no watcher. No
//! `tracing` subscriber, which would scribble across a raw screen.

mod blit;
mod carried;
mod drive;
mod hangup;
mod keymap;
mod session;
mod surfaces;
mod term;
mod theme;

use std::io::{IsTerminal, stdout};
use std::process::ExitCode;

use carried::default_settings;
use orbs_shell::wizard;

/// The crossterm version this binary is built against.
///
/// Held to `Cargo.toml` by a test, as the Bevy build holds its own pin. Cargo
/// exposes no `env!` for a dependency's version, and guessing one on the POST
/// card would be the lie that card exists to avoid.
const CROSSTERM: &str = "0.29";

/// What this binary is built out of, for the POST card's third line.
///
/// The card is a diegetic inventory of the machine (§4), so it must describe
/// *this* machine: the Bevy build says `bevy 0.20.0` and this one must not.
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

    // The save outranks seed and environment, as in the other build. At the
    // threshold no save is read: a scratch world for the menu's prose, never
    // ticked, until `Session::raise`.
    let opened = if threshold.is_waiting() {
        orbs_shell::Opened::New
    } else {
        orbs_shell::read_save()
    };
    let mut sim = orbs_shell::tower_from(opened, seed, wizard().as_deref());
    // Authored content, if `ORBS_CONTENT` names a directory (rule 6). Read here
    // and on every swap, not watched — a watcher belongs to the frontend that
    // already has one.
    let mut missed = orbs_shell::load(&mut sim);

    term::enter()?;
    let result = drive::run(sim, threshold, engine(), &mut missed);
    term::leave();
    say_missed(missed);
    result
}

/// Say what `ORBS_CONTENT` could not load, once the terminal is given back.
///
/// Printed, since this build has no `tracing` subscriber, and only now, since
/// stderr during play scribbles across a raw screen.
fn say_missed(mut missed: Vec<&'static str>) {
    missed.sort_unstable();
    missed.dedup();
    for file in missed {
        eprintln!(
            "orbs-tui: {} is set but {file} did not load; ran on the built-in text",
            orbs_shell::CONTENT_DIR
        );
    }
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
