//! Reading a game back: the screen, its ink, the transcript, and the waits on
//! what they say.

use std::process::Command;

use crate::play::Game;
use crate::tmux::tmux;
use crate::transcript::{block, flatten, parse_tick, transcript};

impl Game {
    /// The whole screen, as text.
    #[must_use]
    pub fn screen(&self) -> String {
        let out = tmux(&["capture-pane", "-t", &self.session, "-p"])
            .output()
            .expect("tmux should capture the pane");
        String::from_utf8_lossy(&out.stdout).into_owned()
    }

    /// The screen with its colours, decoded to `glyph:colour/weight` per cell.
    ///
    /// The only way to see `theme.rs` from outside: a plain capture throws every
    /// attribute away, so a fire ramp climbing to white is invisible to every
    /// other instrument.
    #[must_use]
    pub fn ink(&self, first: usize, last: usize) -> String {
        let out = tmux(&["capture-pane", "-t", &self.session, "-p", "-e"])
            .output()
            .expect("tmux should capture the pane with escapes");
        let root = env!("CARGO_MANIFEST_DIR");
        let script = std::path::Path::new(root).join("../../scripts/ink.py");
        let mut child = Command::new(script)
            .args(["--cols", &format!("{first},{last}"), "--skip-blank"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("scripts/ink.py should run");
        if let Some(stdin) = child.stdin.take() {
            use std::io::Write;
            let mut stdin = stdin;
            let _ = stdin.write_all(&out.stdout);
        }
        let done = child.wait_with_output().expect("ink.py should finish");
        String::from_utf8_lossy(&done.stdout).into_owned()
    }

    /// The transcript column, with the rail and the borders taken off.
    ///
    /// Every row carries the rail too, so a naive match can find a word in the
    /// sidebar and believe it was in the transcript.
    #[must_use]
    pub fn pane(&self) -> String {
        transcript(&self.screen())
    }

    /// Everything since the last `wizard $` header — one command and its answer.
    ///
    /// The unit every assertion is scoped to; the module header says why.
    #[must_use]
    pub fn last_block(&self) -> String {
        block(&self.screen())
    }

    /// The world's clock, read off the rail's foot or the border title.
    ///
    /// Monotonic, present at every grid, and the one number on screen that no
    /// amount of prose, scrolling or sabotage can move.
    #[must_use]
    pub fn tick(&self) -> u64 {
        parse_tick(&self.screen()).unwrap_or(0)
    }

    /// Wait for text in the newest command block.
    pub fn expect(&self, needle: &str) -> &Self {
        self.until(
            |screen| flatten(&block(screen)).contains(&flatten(needle)),
            &format!("waiting for {needle:?} in the newest block"),
        );
        self
    }

    /// Wait for text anywhere in the transcript.
    ///
    /// For the handful of answers that are **not** a reply to the last line — a
    /// threshold announcing itself while a brew finishes, a spell's fault
    /// latching a room. Prefer [`Self::expect`]; this can match history.
    pub fn expect_somewhere(&self, needle: &str) -> &Self {
        self.until(
            |screen| flatten(&transcript(screen)).contains(&flatten(needle)),
            &format!("waiting for {needle:?} anywhere in the transcript"),
        );
        self
    }

    /// Wait for text anywhere on screen, rail and surfaces included.
    pub fn expect_drawn(&self, needle: &str) -> &Self {
        self.until(
            |screen| flatten(screen).contains(&flatten(needle)),
            &format!("waiting for {needle:?} on screen"),
        );
        self
    }

    /// Assert the newest command block does **not** contain something.
    ///
    /// A negative claim has no event to pace against, so this is only honest
    /// after an [`Self::expect`] on the same block has landed. Use
    /// [`Self::refute_after`] for the future.
    pub fn expect_absent(&self, needle: &str) -> &Self {
        let block = flatten(&block(&self.screen()));
        assert!(
            !block.contains(&flatten(needle)),
            "{needle:?} is in the newest block, and should not be:\n{}",
            self.screen(),
        );
        self
    }

    /// Wait for text to leave the **whole screen**.
    ///
    /// [`Self::expect_absent`] scopes to the newest block, which is the wrong
    /// question for a pane: the guide, the rail and the instrument panel are
    /// drawn outside every block, so asking it there passes without looking.
    ///
    /// This waits honestly, closing a pane being an event — a redraw. Named for
    /// the screen because `expect_gone` is taken.
    pub fn expect_off_screen(&self, needle: &str) -> &Self {
        self.until(
            |screen| !flatten(screen).contains(&flatten(needle)),
            &format!("waiting for {needle:?} to leave the screen"),
        );
        self
    }

    /// Let the world run, then assert something never showed up.
    ///
    /// A negative assertion cannot pace itself, which is why it takes a number of
    /// ticks: with no event to wait for, the only honest question is *after this
    /// much play, is it still absent*. Used sparingly.
    pub fn refute_after(&self, ticks: u64, needle: &str) -> &Self {
        self.wait_ticks(ticks);
        let pane = flatten(&self.pane());
        assert!(
            !pane.contains(&flatten(needle)),
            "{needle:?} reached the transcript, and should not have:\n{}",
            self.screen(),
        );
        self
    }
}
