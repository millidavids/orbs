//! Taking the terminal over, giving it back, and asking it one question.
//!
//! A full-screen raw-mode application in the manner of `htop` or the Debian
//! installer (§13): the terminal is a framebuffer and a keyboard, nothing more.
//! It does not shell out, touch the real filesystem, or interoperate with the
//! host shell — `attend /tower/laboratory` navigates the simulated tower exactly
//! as it does under Bevy.

use std::io::{Write, stdout};

use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use crossterm::{cursor, execute, queue, style};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use orbs_render::GridSize;

/// Enter raw mode and the alternate screen, and arrange to leave them.
///
/// # Errors
///
/// If the terminal refuses raw mode or the alternate screen — a pipe, a `cron`
/// job, a CI runner with no tty. The caller should say so and exit rather than
/// draw into nothing.
///
/// It unwinds itself: the second step can fail on its own, and returning `?`
/// from the middle would leave raw mode on with the caller's `leave()` past the
/// early return — no echo, no line editing, a shell that looks dead.
pub(crate) fn enter() -> std::io::Result<()> {
    install_panic_hook();
    enable_raw_mode()?;
    if let Err(error) = execute!(stdout(), EnterAlternateScreen, cursor::Hide) {
        let _ = disable_raw_mode();
        return Err(error);
    }
    Ok(())
}

/// Put the terminal back exactly as it was found.
///
/// Deliberately ignores its own errors: it runs on the way out of a panic as
/// well as a clean exit, and a failure to restore is not worth replacing the
/// original message with.
pub(crate) fn leave() {
    let _ = execute!(stdout(), cursor::Show, LeaveAlternateScreen);
    let _ = disable_raw_mode();
}

/// Restore the terminal before the default hook prints anything.
///
/// A hook, never an RAII guard: the workspace sets `panic = "abort"` in release,
/// where `set_hook` handlers still run and `Drop` does not — so a guard would
/// leave a released build's terminal raw, looking like a dead shell.
fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        leave();
        previous(info);
    }));
}

/// The grid, which is however big the user made the window.
///
/// Not [`orbs_render::GRID`], which is the Bevy build's fixed 120×45 in a 4:3
/// picture; a terminal is whatever size the user made it.
///
/// # Errors
///
/// If the terminal will not say how big it is.
pub(crate) fn grid() -> std::io::Result<GridSize> {
    let (cols, rows) = crossterm::terminal::size()?;
    Ok(GridSize::new(cols, rows))
}

// A `fits(grid)` here answered half of "can this screen host the game":
// `Screen::is_hostable` asks the authoring floor *and* the scale, this asked
// only the floor, and two predicates for one question let the frontends route
// to the "too small" card by different rules.

/// Ask to be told when the process is being killed, rather than simply dying.
///
/// The exit `install_panic_hook`'s reasoning covers and its code does not: a
/// signal leaves the terminal raw by a route no panic hook sees — `kill`, a
/// dropped ssh connection, Ctrl-\. Raw mode disables ISIG only for Ctrl-C.
///
/// A flag rather than a handler that draws, because a signal handler may do
/// almost nothing safely and the loop is awake thirty times a second: it sees
/// the flag, returns normally, and [`leave`] runs on the ordinary path.
///
/// # Errors
///
/// If the signals cannot be registered.
pub(crate) fn dying() -> std::io::Result<Arc<AtomicBool>> {
    let flag = Arc::new(AtomicBool::new(false));
    for signal in [
        signal_hook::consts::SIGTERM,
        signal_hook::consts::SIGHUP,
        signal_hook::consts::SIGQUIT,
    ] {
        signal_hook::flag::register(signal, Arc::clone(&flag))?;
    }
    Ok(flag)
}

/// A glyph whose width the terminal might disagree with us about.
///
/// `○` (U+25CB), and which glyph this is is the constant's whole job.
///
/// It was `‼` for a version, which is Neutral rather than Ambiguous — one column
/// on every terminal there is — so `symbols_are_narrow` always returned `true`
/// and the entire fallback beneath it was unreachable.
///
/// What genuinely widens under *ambiguous = wide* is the ward's `○ ♂ ♀ ♠`, the
/// two trees' `• · ■` and the maze's `Ω`. `○` is drawn by both `board.rs` and
/// `loom.rs` and is in `blit::narrowed`'s table, so a probe that fires has
/// something to do.
pub(crate) const PROBE: char = '○';

/// Whether the terminal draws our symbols one column wide.
///
/// Correctness, not polish: one double-width cell shifts the rest of the row and
/// invalidates the per-cell diff `blit` relies on, because the shadow buffer and
/// the screen stop agreeing about which column is which. The Bevy build is
/// immune — an 8×16 bitmap atlas is one cell because the renderer says so.
///
/// So it is measured, not assumed: print the glyph at a known column and ask
/// where the cursor ended up. A failure to ask is treated as narrow, the common
/// case.
///
/// # Errors
///
/// If the terminal cannot be written to or will not report its cursor.
pub(crate) fn symbols_are_narrow() -> std::io::Result<bool> {
    let mut out = stdout();
    queue!(out, cursor::MoveTo(0, 0), style::Print(PROBE))?;
    out.flush()?;
    let (col, _) = cursor::position()?;
    // Clear the probe before anything real is drawn over it.
    queue!(
        out,
        cursor::MoveTo(0, 0),
        style::Print(' '),
        cursor::MoveTo(0, 0)
    )?;
    out.flush()?;
    Ok(col <= 1)
}

#[cfg(test)]
mod tests {
    use super::PROBE;

    /// Every glyph this game draws that is East Asian Ambiguous, and so takes
    /// two columns under a CJK locale or *ambiguous = wide*.
    ///
    /// Measured with Python's `unicodedata.east_asian_width`, not recalled.
    /// `‼ ► ☼ ♦ ░ √ → »` are absent because they are Neutral — one column
    /// everywhere; getting that backwards made the probe dead code.
    const AMBIGUOUS: [char; 8] = ['○', '♂', '♀', '♠', '•', '·', '■', 'Ω'];

    /// The probe has to be a glyph whose width can actually differ: a Neutral
    /// one reports `narrow` on every terminal ever built, which is an instrument
    /// wired to a constant rather than a passing test.
    #[test]
    fn the_probe_is_a_glyph_that_can_actually_widen() {
        assert!(
            AMBIGUOUS.contains(&PROBE),
            "{PROBE:?} is not East Asian Ambiguous, so symbols_are_narrow() can \
             never return false and the whole fallback below it is unreachable"
        );
    }
}
