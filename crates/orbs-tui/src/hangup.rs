//! Noticing that the terminal, or the harness that owned it, has gone away.

use std::time::{Duration, Instant};

/// Whether the terminal this process was given has gone away.
///
/// After `POLLHUP`, crossterm's read spins on the EOF for ever and burns a
/// core, which `SIGHUP` cannot stop in a reparented child. Asked of the
/// descriptor before the read, with a zero timeout.
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
/// A thread, since a read on a dead pty never returns to the loop's
/// [`hung_up`]. It exits hard, `leave` being writes to a dead terminal; `gone`
/// answers only to `HUP`, `ERR` and `NVAL`, tested both ways.
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
/// `ORBS_LIFETIME`, in seconds, absent by default. The play harness sets it,
/// for a live terminal whose owner died, which [`watch_for_hangup`] cannot
/// see. Unreadable means no limit, not zero.
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
