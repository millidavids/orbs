//! O.R.B.S. — terminal frontend.
//!
//! A full-screen raw-mode application. It does not shell out, touch the real
//! filesystem, or interoperate with the host shell — the terminal is a
//! framebuffer and a keyboard. Same sim, same commands, same simulated tower;
//! only the rasteriser differs.
//!
//! **Second-class by decision, not by neglect** (DESIGN.md §13, §19). The Bevy
//! build is the product and never waits for this one. What this is for is
//! §15's charter: *"a no-GPU, no-window, instant-startup, trivially scriptable
//! target"* — and the scripting is `tmux send-keys`, which makes every screen in
//! the game something a person or a script can drive and read back.
//!
//! ```bash
//! cargo build -p orbs-tui
//! tmux new-session -d -s orbs -x 120 -y 45 target/debug/orbs-tui
//! tmux send-keys -t orbs 'attend laboratory' Enter
//! tmux capture-pane -t orbs -p
//! ```
//!
//! # It boots, and `ORBS_BOOT=0` is how you stop it
//!
//! §4's sequence runs here too: a beat of dark, then the card drawing its own
//! border while the logo prints itself and the dependencies report in — naming
//! `crossterm` where the other build names `bevy`, because the card is a
//! *diegetic inventory of the machine* and this is a different machine.
//!
//! **It was skipped here for a version**, on the argument that a dev tool whose
//! charter is *"instant-startup"* (§15) must not begin by making you wait. That
//! was the wrong trade to make silently: the sequence is the game's opening
//! image, and a second frontend that quietly omits it is not the same game with
//! a different rasteriser. The argument was really about the **development**
//! loop, and `ORBS_BOOT=0` already answered that — on both sides, for the same
//! reason.
//!
//! The clock and the card are `orbs-shell`'s; what lives here is the frame that
//! advances one and the engine line that fills the other.
//!
//! # One thing it does not do, and that is on purpose
//!
//! **No content watcher.** `ORBS_CONTENT` is read once at startup rather than
//! watched: a watcher is `notify` and a background thread, and it belongs to the
//! frontend that already has one. Edit `prose.toml` and restart — which costs
//! nothing here, which is the point.
//!
//! # It installs no `tracing` subscriber, and that is deliberate
//!
//! `orbs_shell::read` warns through `tracing` when authored content will not
//! parse. In raw mode a subscriber writing to stderr would scribble across the
//! screen at the exact moment a writer is looking at their own edit. With no
//! subscriber the macro compiles to nothing; the `--dump` path, which owns the
//! terminal in the ordinary way, prints its own warning to stderr instead.

mod blit;
mod drive;
mod surfaces;
mod term;
mod theme;

use std::io::{IsTerminal, stdout};
use std::process::ExitCode;

use orbs_shell::{seed, wizard};
use orbs_sim::Sim;

/// The crossterm version this binary is built against.
///
/// A const held to `Cargo.toml` by a test, exactly as the Bevy build holds its
/// own pin. Cargo exposes no `env!` for a dependency's version, and inventing
/// one on the POST card would be the lie that card exists to avoid.
const CROSSTERM: &str = "0.29";

/// What this binary is built out of, for the POST card's third line.
///
/// The card is a diegetic inventory of the machine (§4), so it must describe
/// *this* machine: the Bevy build says `bevy 0.19.0` and this one must not.
fn engine() -> String {
    format!("crossterm {CROSSTERM}")
}

fn main() -> ExitCode {
    let seed = seed();

    // `--dump "attend laboratory; grind sage"` prints one frame as text and
    // exits, through the same painters the Bevy build's `ORBS_DUMP` uses. That
    // agreement is the boundary proof: two binaries, one screen.
    let mut args = std::env::args().skip(1);
    if let Some(flag) = args.next() {
        if flag == "--dump" {
            let script = args.next().unwrap_or_else(|| "1".to_owned());
            orbs_shell::dump_script(seed, wizard(), &engine(), &script);
            return ExitCode::SUCCESS;
        }
        eprintln!("usage: orbs-tui [--dump <commands>]");
        return ExitCode::FAILURE;
    }
    // ...and `ORBS_DUMP` works here too, so a See-it line written for one
    // frontend runs against the other unchanged.
    if orbs_shell::dump(seed, wizard(), &engine()) {
        return ExitCode::SUCCESS;
    }

    if !stdout().is_terminal() {
        eprintln!("orbs-tui needs a terminal. Try `--dump` for one frame as text.");
        return ExitCode::FAILURE;
    }

    match play(seed) {
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
    // **The save outranks the seed and the environment both**, as it does in the
    // other build. `session::Wizard` puts it the other way round: a name is world
    // state, so it is read from the machine only when there is no world yet.
    let mut sim = match orbs_shell::read_save() {
        orbs_shell::Opened::Restored(save) => {
            let mut resumed = Sim::restored(&save);
            resumed.say_resumed(orbs_shell::away_for(&save));
            resumed
        }
        orbs_shell::Opened::Unreadable => {
            let mut fresh = Sim::new(seed);
            if let Some(name) = wizard() {
                fresh.rename(&name);
            }
            fresh.say_save_unreadable();
            fresh
        }
        orbs_shell::Opened::New => {
            let mut fresh = Sim::new(seed);
            if let Some(name) = wizard() {
                fresh.rename(&name);
            }
            fresh
        }
    };
    // Authored content, if `ORBS_CONTENT` names a directory (rule 6). No
    // watcher: that is `notify` and a background thread, and it belongs to the
    // frontend that already has one. A terminal build reads the file once.
    if std::env::var_os(orbs_shell::CONTENT_DIR).is_some()
        && let Some(prose) = orbs_shell::load()
    {
        sim.set_prose(prose);
    }

    term::enter()?;
    let result = drive::run(sim, engine());
    term::leave();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_crossterm_version_is_the_one_the_manifest_asks_for() {
        // The POST card names what the machine is made of, and `tower/boot.rs`
        // exists because "a boot report that could go stale would be a lie the
        // player reads first". This is that rule one screen earlier — and it is
        // the same test the Bevy build keeps beside its own pin.
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
