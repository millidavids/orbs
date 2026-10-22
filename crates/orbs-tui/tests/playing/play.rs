//! A game, played through a real terminal, with the screen read back.
//!
//! Assertions anchor on the *last block*, never on the screen. The transcript
//! keeps its history, so waiting for a substring anywhere is wrong twice over,
//! both measured:
//!
//! - **Stale matches.** The clarity chain empties the mortar twice, six steps
//!   apart; the second wait returned in 1 ms against the first one's line.
//! - **Saturation.** Counting occurrences deadlocks: the pane holds 8 command
//!   blocks at 120×45, so from the ninth repeat each new line pushes an old one
//!   off, and the tail over `Records::drawn` runs *backwards*.
//!
//! So a match is scoped by position: everything since the last `wizard $`
//! header. It is self-pacing too — the result lands next tick.
//!
//! The parsing depends on the screen's shape:
//!
//! ```text
//! │wizard $ grind sage                    │mp bm fr al at │   ← history, inside
//! │→ grind sage                           │███████████████│
//! │√ sage: dispensary to mortar_and_pestle │███████████████│
//! └───────────────────────────────────────┴───────────────┘
//!  wizard $ grind sa                                          ← live, outside
//! ```
//!
//! The rail shares every row, so a row is split on `│` and the transcript is
//! the first cell. The live prompt is drawn *outside* the border and is not
//! part of any block — it holds a half-typed line and the ghost completion.
//!
//! Typing at a game is `driving`; reading it back is `reading`.

use std::time::{Duration, Instant};

use crate::launch::{Launch, Switch};
use crate::tmux::{reap, tmux};

pub use crate::tmux::available;

/// The prompt name, pinned. It falls back to `$USER` otherwise, which would make
/// every block header machine-dependent.
pub const WIZARD: &str = "wizard";

/// How long any wait may take before it is called a failure.
///
/// Generous on purpose: a timeout should mean *broken*, never *busy*.
pub(crate) const PATIENCE: Duration = Duration::from_secs(20);

/// How often the screen is re-read while waiting. A capture costs ~1.1 ms, so
/// polling is close to free and the resolution is worth more.
pub(crate) const POLL: Duration = Duration::from_millis(10);

/// A seed whose first two hundred ticks are free of ambient sabotage.
///
/// `substitution` fires from the seeded `Threat` stream, so the schedule is
/// fixed in tick space and can be *chosen away*. Only a siege poisons a log
/// (§19); the shelf is all the calm layer touches.
pub const QUIET: u64 = 11;

/// The game's own grid, so a capture is directly comparable with `ORBS_DUMP`.
pub const GRID: (u16, u16) = (120, 45);

/// How long a game this harness starts may live, whatever happens to the harness.
///
/// Ten minutes: a backstop for a killed run, not a budget for a slow scenario.
pub const LIFETIME: std::time::Duration = std::time::Duration::from_secs(600);

/// A running game.
pub struct Game {
    pub(crate) session: String,
    pub(crate) cols: u16,
    pub(crate) rows: u16,
    pub(crate) dir: std::path::PathBuf,
    /// The game's process, when something must outlive it — see `Launch::spawn`.
    pub(crate) pid: Option<u32>,
}

impl Drop for Game {
    fn drop(&mut self) {
        // Best effort: a failed kill must not replace a panicking test's own
        // assertion message.
        let _ = tmux(&["kill-session", "-t", &self.session]).status();
        if let Some(pid) = self.pid {
            reap(pid);
        }
        // And the scratch directory — one per game, 600 in `/tmp` before anybody
        // noticed. Here rather than in the script so an ordinary run leaves
        // nothing; `play.sh` sweeps what a killed run could not.
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

impl Game {
    /// How a game is started, from the scenarios' defaults.
    pub fn launch() -> Launch {
        Launch::default()
    }

    /// Start a game at the standard grid on the quiet seed.
    #[must_use]
    pub fn start() -> Self {
        Self::launch().settled()
    }

    /// Start a game on a chosen seed — for the scenarios that want a particular
    /// maze, ward or scroll draw.
    #[must_use]
    pub fn seeded(seed: u64) -> Self {
        Self::launch().seed(seed).settled()
    }

    /// Start a game at a chosen size, for the layout scenarios.
    #[must_use]
    pub fn sized(cols: u16, rows: u16) -> Self {
        Self::launch().sized(cols, rows).settled()
    }

    /// Start a game **with §4's boot sequence**, which every other scenario skips.
    ///
    /// No wait for a prompt: the point is the time before there is one.
    #[must_use]
    pub fn booting() -> Self {
        // Moving, since the live card is the point; still holds it at its end.
        Self::launch().booting().motion(Switch::Always).spawn()
    }

    /// Start a game **at the orb's menu**, which every other scenario skips.
    ///
    /// No wait for a prompt: there is not one.
    #[must_use]
    pub fn at_the_threshold() -> Self {
        Self::launch().at_the_threshold().spawn()
    }

    /// Start a game at the orb's menu **with the installation on**, so `new`
    /// reaches the practice tower rather than a tower.
    #[must_use]
    pub fn installing() -> Self {
        Self::launch()
            .at_the_threshold()
            .installs(Switch::Always)
            .spawn()
    }

    /// Start a game at the orb's menu with **the setting** deciding the
    /// installation, as a player's game does. `ORBS_SAVE=off` keeps no settings
    /// file, so it starts on, and the menu can turn it off for the session.
    #[must_use]
    pub fn at_the_threshold_by_setting() -> Self {
        Self::launch()
            .at_the_threshold()
            .installs(Switch::BySetting)
            .spawn()
    }

    /// Start a game **as a fresh game starts**: a laboratory and nothing else
    /// (§11.5), which every other scenario skips by opening the whole tower.
    #[must_use]
    pub fn sealed() -> Self {
        Self::launch().sealed().settled()
    }

    /// Wait for a file the game itself wrote into its working directory.
    ///
    /// `F6` says nothing on the transcript, so the file is the only evidence the
    /// key did anything — and the reason each game gets a directory of its own.
    pub fn wrote(&self, name: &str) -> String {
        let path = self.dir.join(name);
        let deadline = Instant::now() + PATIENCE;
        while Instant::now() < deadline {
            if let Ok(text) = std::fs::read_to_string(&path)
                && !text.is_empty()
            {
                return text;
            }
            std::thread::sleep(POLL);
        }
        panic!("{} was never written\n{}", path.display(), self.screen());
    }

    /// The game's own process, asked of tmux.
    #[must_use]
    pub fn pid(&self) -> Option<u32> {
        crate::tmux::pane_pid(&self.session)
    }

    /// Whether the game has written this file into its working directory yet.
    ///
    /// No wait: for a key that must write nothing, asked after the game has had
    /// the time a write would take.
    #[must_use]
    pub fn has_written(&self, name: &str) -> bool {
        self.dir.join(name).exists()
    }

    /// Wait for the first frame, so nothing reads an empty screen.
    pub(crate) fn settled(&self) {
        self.until(
            |screen| screen.contains(WIZARD),
            "the game never drew its first prompt",
        );
    }

    /// Whether the session is still running.
    #[must_use]
    pub fn alive(&self) -> bool {
        tmux(&["has-session", "-t", &self.session])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
    }

    /// Wait for the game to exit, for the scenarios about leaving.
    pub fn expect_gone(&self) {
        let deadline = Instant::now() + PATIENCE;
        while Instant::now() < deadline {
            if !self.alive() {
                return;
            }
            std::thread::sleep(POLL);
        }
        panic!("the game was still running {PATIENCE:?} after being told to leave");
    }

    /// Poll the screen until a condition holds, or fail with the whole screen.
    pub(crate) fn until(&self, mut done: impl FnMut(&str) -> bool, what: &str) {
        let deadline = Instant::now() + PATIENCE;
        let mut screen = String::new();
        while Instant::now() < deadline {
            screen = self.screen();
            if done(&screen) {
                return;
            }
            std::thread::sleep(POLL);
        }
        panic!("gave up {what}\n--- the screen was ---\n{screen}");
    }
}
