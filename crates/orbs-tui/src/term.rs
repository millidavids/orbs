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
/// If the terminal refuses raw mode or the alternate screen, as with no tty.
/// It unwinds itself, or a failed second step leaves raw mode on.
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
/// A signal leaves the terminal raw by a route no panic hook sees. A flag the
/// loop sees, so [`leave`] runs on the ordinary path.
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

/// A glyph whose width the terminal might disagree with us about: `○`, East
/// Asian Ambiguous and narrowed by `blit`, so a probe that fires has something
/// to do. It was `‼`, which is Neutral, and the fallback was unreachable.
/// `blit::scan` holds both properties.
pub(crate) const PROBE: char = '○';

/// Whether the terminal draws our symbols one column wide.
///
/// Correctness: one wide cell breaks `blit`'s per-cell diff. Measured — print
/// at a known column and ask where the cursor went; a failure to ask is
/// narrow.
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
