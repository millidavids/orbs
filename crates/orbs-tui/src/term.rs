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
/// **It unwinds itself.** Taking the terminal is two steps and the second can
/// fail on its own — a tty that allows `tcsetattr` but rejects the alternate
/// screen. Returning `?` straight out of the middle would leave raw mode *on*,
/// and the caller's `leave()` is on the other side of the early return, so
/// nothing would ever put it back: no echo, no line editing, no prompt. The
/// shell would look dead, and the error message explaining why would be the last
/// legible thing on it.
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
/// **A hook, never an RAII guard**, and the distinction is not academic: the
/// workspace sets `panic = "abort"` in release, where `set_hook` handlers still
/// run and `Drop` does not. A guard would leave a released build's terminal in
/// raw mode with the alternate screen up — no echo, no line editing, no prompt —
/// which looks exactly like the shell having died.
fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        leave();
        previous(info);
    }));
}

/// The grid, which is however big the user made the window.
///
/// **Not [`orbs_render::GRID`]**, which is the Bevy build's fixed 120×45 in a
/// 4:3 picture. `orbs-render`'s own viewport module says so: *"a terminal is
/// whatever size the user made it, so `orbs-tui` reports its grid directly."*
///
/// # Errors
///
/// If the terminal will not say how big it is.
pub(crate) fn grid() -> std::io::Result<GridSize> {
    let (cols, rows) = crossterm::terminal::size()?;
    Ok(GridSize::new(cols, rows))
}

// **There was a `fits(grid)` here, and it answered half a question.**
// "Can this screen host the game" is `Screen::is_hostable`, which asks the grid
// against the authoring floor *and* the scale; this asked only the first, at the
// one call site that had already built a `Screen` two lines above it. Two
// predicates for one question meant the two frontends could route to the "too
// small" card by different rules — and the `--dump` parity diff could not catch
// it, because the dump path was asking the whole question all along.

/// Ask to be told when the process is being killed, rather than simply dying.
///
/// **The exit `install_panic_hook`'s reasoning covers and its code does not.**
/// That hook exists because leaving a terminal in raw mode with the alternate
/// screen up "looks exactly like the shell having died" — and a signal reaches
/// that state by a route no hook sees: `kill`, a dropped ssh connection
/// (SIGHUP), a session manager stopping the process, Ctrl-\ delivering SIGQUIT.
/// Raw mode disables ISIG for Ctrl-C, which is why that one is the loop's to
/// answer; it does nothing about a signal sent from outside.
///
/// A flag rather than a handler that draws: a signal handler may do almost
/// nothing safely, and the loop is already awake thirty times a second. It sees
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
/// **`○` (U+25CB), and which glyph this is *is* the constant's whole job.**
///
/// It was `‼` for a version, on the stated grounds that U+203C is East Asian
/// Ambiguous. It is not — it is **Neutral**, and so are `►`, `☼`, `♦` and `░`,
/// every one of which the same comment named as at risk. A Neutral glyph is one
/// column on every terminal there is, so this measured a quantity that cannot
/// vary, `symbols_are_narrow` returned `true` under every configuration, and the
/// entire fallback beneath it was unreachable code with tests passing over it.
///
/// The symbols that genuinely widen under *ambiguous = wide* are the ward's
/// `○ ♂ ♀ ♠`, the two trees' `• · ■`, and the maze's `Ω`. `○` is drawn by
/// `board.rs` and `loom.rs` both, and is in `blit::narrowed`'s table — so a
/// probe that fires is a probe with something to do.
pub(crate) const PROBE: char = '○';

/// Whether the terminal draws our symbols one column wide.
///
/// # Why this is a correctness question and not a polish one
///
/// The Bevy build is immune: it draws an 8×16 bitmap atlas, so a glyph is one
/// cell because the renderer says so. A terminal decides for itself, and a
/// single double-width cell does two things — it shifts the rest of the row, and
/// it invalidates the per-cell diff `blit` relies on, because the shadow buffer
/// and the screen stop agreeing about which column is which.
///
/// So it is **measured, not assumed**: print the glyph at a known column, ask
/// the terminal where the cursor ended up, and believe the answer. A failure to
/// ask is treated as narrow, which is the common case and the one that costs
/// nothing if wrong on a screen nobody is looking at yet.
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

    /// Every glyph this game draws that is **East Asian Ambiguous**, and so
    /// takes two columns under a CJK locale or *ambiguous = wide*.
    ///
    /// Measured with Python's `unicodedata.east_asian_width`, not recalled. The
    /// symbols deliberately **absent** are the ones a reader will expect to see:
    /// `‼ ► ☼ ♦ ░ √ → »` are all Neutral — one column everywhere, nothing to
    /// probe for. Getting that backwards is what made the probe dead code.
    const AMBIGUOUS: [char; 8] = ['○', '♂', '♀', '♠', '•', '·', '■', 'Ω'];

    /// The probe has to be a glyph whose width can actually differ.
    ///
    /// A Neutral probe reports `narrow` on every terminal ever built, which is
    /// not a passing test — it is an instrument wired to a constant.
    #[test]
    fn the_probe_is_a_glyph_that_can_actually_widen() {
        assert!(
            AMBIGUOUS.contains(&PROBE),
            "{PROBE:?} is not East Asian Ambiguous, so symbols_are_narrow() can \
             never return false and the whole fallback below it is unreachable"
        );
    }
}
