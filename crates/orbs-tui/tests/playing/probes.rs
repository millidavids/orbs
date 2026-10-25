//! A game outlives neither its terminal nor its lifetime.

use std::process::Command;

use crate::play::LIFETIME;
use crate::tmux::available;

/// A game whose terminal is destroyed exits instead of spinning.
///
/// `hangup::watch_for_hangup`, watched on this session's own pane pid, since a
/// global count never reaches zero.
#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_game_whose_terminal_dies_does_not_outlive_it() {
    if !available() {
        return;
    }
    // A fixed name, as `orbs-play` is, so a rerun reuses the socket file
    // rather than leaving one per pid.
    let socket = "orbs-hangup";
    let Some(pid) = probe(socket) else {
        panic!("the probe game never started, so this proves nothing");
    };

    let _ = Command::new("tmux")
        .args(["-L", socket, "kill-server"])
        .status();

    // Generous against `HANGUP_CHECK`'s half second: what is under test is *does
    // it ever exit*, and a loaded machine must not make that a flake.
    for _ in 0..40 {
        if !running(pid) {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
    let _ = Command::new("kill").args(["-9", &pid.to_string()]).status();
    panic!("a game outlived its terminal — it is spinning on a dead pty");
}

/// ...and one whose harness died while its terminal lived exits too, eventually.
///
/// [`LIFETIME`], for a live pty `watch_for_hangup` cannot see: checked wired,
/// and the shipped number asserted.
#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_game_stops_itself_when_its_lifetime_runs_out() {
    if !available() {
        return;
    }
    assert_eq!(
        LIFETIME.as_secs(),
        600,
        "the shipped cap moved; a scenario should never approach it",
    );

    // Five, not one: the probe needs two seconds to come up and be asked its
    // pid, so a one-second lifetime reports a game that never started when what
    // happened was a game that had finished.
    let socket = "orbs-lifetime";
    let Some(pid) = probe_with(socket, &["ORBS_LIFETIME=5"]) else {
        panic!("the probe game never started, so this proves nothing");
    };

    for _ in 0..40 {
        if !running(pid) {
            let _ = Command::new("tmux")
                .args(["-L", socket, "kill-server"])
                .status();
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
    let _ = Command::new("kill").args(["-9", &pid.to_string()]).status();
    let _ = Command::new("tmux")
        .args(["-L", socket, "kill-server"])
        .status();
    panic!("a game with a one-second lifetime was still running ten seconds on");
}

/// Start a game on a server of its own and return the pid tmux gave it.
fn probe(socket: &str) -> Option<u32> {
    probe_with(socket, &[])
}

/// The same, with extra environment for the game.
fn probe_with(socket: &str, env: &[&str]) -> Option<u32> {
    let _ = Command::new("tmux")
        .args(["-L", socket, "kill-server"])
        .status();
    let mut start = Command::new("tmux");
    start
        .args(["-L", socket, "-f", "/dev/null", "new-session", "-d", "-s"])
        .arg("probe")
        .args(["-x", "120", "-y", "45"]);
    for pair in env {
        start.arg("-e").arg(pair);
    }
    // `ORBS_SAVE=off` here as everywhere: a probe must not read or write a tower
    // another scenario is using.
    start.arg("-e").arg("ORBS_SAVE=off").arg(format!(
        // `ORBS_THRESHOLD=0` for the reason the scenarios carry it: the probe
        // asks whether a tower comes up, not whether a menu does.
        "ORBS_BOOT=0 ORBS_THRESHOLD=0 {}",
        env!("CARGO_BIN_EXE_orbs-tui"),
    ));
    start.status().ok()?;
    std::thread::sleep(std::time::Duration::from_secs(2));

    let out = Command::new("tmux")
        .args(["-L", socket, "list-panes", "-F", "#{pane_pid}"])
        .output()
        .ok()?;
    String::from_utf8_lossy(&out.stdout).trim().parse().ok()
}

/// Whether that pid is still alive.
fn running(pid: u32) -> bool {
    std::path::Path::new(&format!("/proc/{pid}")).exists()
}
