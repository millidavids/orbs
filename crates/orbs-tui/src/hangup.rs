//! Noticing that the terminal, or the harness that owned it, has gone away.

use std::time::{Duration, Instant};

/// Whether the terminal this process was given has gone away.
///
/// A leaked process that idles costs nothing; one that spins takes the machine.
/// When the pty master closes the slave reports `POLLHUP`, after which
/// `crossterm::event::poll` answers *ready* for ever and `event::read` loops
/// inside itself on the EOF rather than returning — so the process burns a
/// core. 573 of these once took a 32-core machine to a load average of 581, and
/// `term`'s `SIGHUP` handler cannot reach a child reparented away from a dead
/// terminal.
///
/// Asked of the descriptor, because crossterm has no way to say it, and asked
/// before the read, because the read that would be counted never returns. Zero
/// timeout, so an ordinary frame pays one non-blocking `poll`.
pub(crate) fn hung_up() -> bool {
    gone(&rustix::stdio::stdin())
}

/// How often the watchdog asks whether the terminal is still there.
///
/// Half a second: a leaked process outlives its terminal by that much at most,
/// and a live session pays one non-blocking `poll` twice a second.
const HANGUP_CHECK: Duration = Duration::from_millis(500);

/// Watch for the terminal going away, from a thread the loop cannot block.
///
/// A thread because the loop never gets the chance. [`hung_up`] at the top of
/// each pass is kept, for the tidy exit through `term::leave` while the loop is
/// still turning — but `crossterm::event::read` on a dead pty never returns, so
/// a check there cannot run: measured at 82% CPU with one in place.
///
/// It exits hard, skipping `term::leave`, because everything `leave` does is a
/// write to a terminal just established as gone. The risk is a false positive
/// killing a live session, so `gone` answers only to `HUP`, `ERR` and `NVAL`;
/// both directions are tested.
pub(crate) fn watch_for_hangup() {
    let deadline = lifetime().map(|span| Instant::now() + span);
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(HANGUP_CHECK);
            if hung_up() || deadline.is_some_and(|end| Instant::now() >= end) {
                std::process::exit(0);
            }
        }
    });
}

/// How long this session may live, if something set a limit.
///
/// `ORBS_LIFETIME`, in seconds, absent by default — a real game runs until
/// somebody stops it and nothing here times a player out mid-brew.
///
/// The play harness sets it, closing the half [`watch_for_hangup`] cannot reach:
/// a terminal perfectly alive while the harness that owned it is not. A detached
/// tmux server is nobody's child, so `SIGKILL`ing `cargo test` leaves its
/// sessions drawing at 30fps until something kills the server. An unreadable
/// value means no limit rather than zero, so a typo cannot exit every game half
/// a second after it starts.
fn lifetime() -> Option<Duration> {
    std::env::var("ORBS_LIFETIME")
        .ok()?
        .trim()
        .parse()
        .ok()
        .map(Duration::from_secs)
}

/// Whether the far end of `fd` has closed.
///
/// Split from [`hung_up`] so it can be tested against a pipe: a `poll` that
/// silently never reports `HUP` is a guard that does nothing, and nothing else
/// notices. A pty slave whose master has closed reports `HUP | ERR` — measured,
/// because a pipe and a pty need not agree and it is the pty that matters.
fn gone<Fd: std::os::fd::AsFd>(fd: &Fd) -> bool {
    use rustix::event::{PollFd, PollFlags, Timespec, poll};

    // No flags requested: a hangup is reported whether or not anything was asked
    // for. This asks *is the far end still there*, which crossterm cannot.
    let mut watched = [PollFd::new(fd, PollFlags::empty())];
    // `rustix` rather than `libc`: `unsafe_code` is denied workspace-wide, and a
    // safe wrapper over one `poll` is exactly what that rule is for.
    if poll(&mut watched, Some(&Timespec::default())).is_err() {
        // A `poll` that will not run is itself a terminal that cannot be read.
        return true;
    }
    watched[0]
        .revents()
        .intersects(PollFlags::HUP | PollFlags::ERR | PollFlags::NVAL)
}

#[cfg(test)]
mod tests {
    use super::gone;

    /// A pipe whose writer is still open has not hung up.
    #[test]
    fn a_live_descriptor_is_not_gone() {
        let (reader, writer) = rustix::pipe::pipe().expect("a pipe");
        assert!(
            !gone(&reader),
            "a pipe with its writer open read as hung up"
        );
        drop(writer);
    }

    /// ...and one whose writer has closed has.
    ///
    /// The half that silently does nothing if `poll` is asked wrongly: the
    /// request flags are empty on purpose, and getting that wrong gives a guard
    /// that never fires, which is the state this shipped in.
    #[test]
    fn a_descriptor_whose_far_end_closed_is_gone() {
        let (reader, writer) = rustix::pipe::pipe().expect("a pipe");
        drop(writer);
        assert!(gone(&reader), "a pipe with its writer closed read as live");
    }
}
