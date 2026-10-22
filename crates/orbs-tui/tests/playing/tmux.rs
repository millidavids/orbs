//! The terminal the games run in: a private tmux server, and the processes
//! it starts.

use std::process::Command;
use std::time::{Duration, Instant};

/// A private tmux socket, so nothing here can see or be seen by the developer's
/// own sessions, their `~/.tmux.conf`, or a `remain-on-exit` that would keep a
/// window open after `quit` and hang the wait for it.
const SOCKET: &str = "orbs-play";

/// Run a tmux command against the private socket.
pub fn tmux(args: &[&str]) -> Command {
    let mut command = Command::new("tmux");
    // `-f /dev/null` is the other half of the private socket: a server started
    // from this test must not read the developer's config.
    command.args(["-L", SOCKET, "-f", "/dev/null"]).args(args);
    command
}

/// Whether tmux is installed at all.
///
/// A missing tmux is not a broken game, so a scenario says so and returns.
/// Printed rather than silent: a suite that quietly tests nothing is worse than
/// a red one.
#[must_use]
pub fn available() -> bool {
    let found = Command::new("tmux")
        .arg("-V")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|status| status.success());
    if !found {
        eprintln!("SKIPPED: tmux is not installed, so the played game cannot be reached");
    }
    found
}

/// The process tmux started in this session.
pub fn pane_pid(session: &str) -> Option<u32> {
    let out = tmux(&["list-panes", "-t", session, "-F", "#{pane_pid}"])
        .output()
        .ok()?;
    String::from_utf8_lossy(&out.stdout).trim().parse().ok()
}

/// Wait for a process to finish, up to a few seconds.
///
/// A `SIGHUP`ed game writes its tower on the way out, so a folder removed under
/// a game still exiting is recreated by it.
pub fn reap(pid: u32) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline && std::path::Path::new(&format!("/proc/{pid}")).exists() {
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// A number nothing else in this process will pick, for session and folder
/// names.
pub fn next_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    NEXT.fetch_add(1, Ordering::Relaxed)
}
