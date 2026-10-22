//! Typing at a game: lines, text, keys, a resize, and the waits that pace them.

use std::time::{Duration, Instant};

use crate::play::{Game, PATIENCE, POLL, WIZARD};
use crate::tmux::tmux;
use crate::transcript::{block_with_header, parse_tick, prompt_taken};

impl Game {
    /// Type a line and press Enter.
    ///
    /// `send-keys -l`, and Enter on a call of its own. Without the literal flag
    /// tmux reads a word as a key name wherever one matches, and `end` closes
    /// every `repeat` and `if` in the spell language.
    ///
    /// The wait carries every assertion that follows: a moment early and `rfind`
    /// still points at the *previous* command.
    pub fn send(&self, line: &str) -> &Self {
        self.type_raw(line);
        let header = format!("{WIZARD} $ {line}");
        self.until(
            // The first condition makes a *repeated* line safe: an emptied
            // prompt means this line, not the identical one before it, was
            // consumed.
            |screen| prompt_taken(screen, line) && block_with_header(screen).starts_with(&header),
            &format!("waiting for the prompt to take {line:?}"),
        );
        self
    }

    /// Type text and **stop** — no Enter.
    ///
    /// For everything that answers to where the caret is rather than to a
    /// finished line: the scribing guide, Tab completion, the prompt's ghost.
    /// Enter would answer a different question, and `-l` matters most here
    /// because these are the lines that end mid-word.
    pub fn send_text(&self, text: &str) -> &Self {
        if text.is_empty() {
            return self;
        }
        let status = tmux(&["send-keys", "-t", &self.session, "-l", "--", text])
            .status()
            .expect("tmux should send text");
        assert!(status.success(), "tmux refused the text {text:?}");
        self
    }

    /// Type a line and press Enter, with no wait for a prompt block.
    ///
    /// For the surfaces: the editor and the weave screen take whole lines too,
    /// and neither draws a `wizard $` header for them.
    pub fn type_raw(&self, line: &str) -> &Self {
        if !line.is_empty() {
            let status = tmux(&["send-keys", "-t", &self.session, "-l", "--", line])
                .status()
                .expect("tmux should send text");
            assert!(status.success(), "tmux refused the line {line:?}");
        }
        let status = tmux(&["send-keys", "-t", &self.session, "Enter"])
            .status()
            .expect("tmux should send Enter");
        assert!(status.success(), "tmux refused Enter after {line:?}");
        self
    }

    /// Press a named key — `Up`, `Escape`, `F5`, `PageUp`.
    ///
    /// No wait: `Sim::walk` is the third entry point and spends no world time,
    /// so a player walks as fast as they can press and so does this.
    pub fn press(&self, key: &str) -> &Self {
        let status = tmux(&["send-keys", "-t", &self.session, key])
            .status()
            .expect("tmux should send a key");
        assert!(status.success(), "tmux refused the key {key}");
        self
    }

    /// Send a line and wait for its answer, in the block that line opened.
    pub fn does(&self, line: &str, expect: &str) -> &Self {
        self.send(line);
        self.expect(expect)
    }

    /// Skip time, and wait for the clock rather than for the word.
    ///
    /// `does("meditate 20", "meditate")` caught three scenarios at once: the
    /// prompt echoes `→ meditate 20` the instant the line is taken, so the wait
    /// is satisfied before a tick passes. The clock cannot be fooled that way.
    pub fn meditates(&self, ticks: u64) -> &Self {
        let target = self.tick() + ticks;
        self.send(&format!("meditate {ticks}"));
        self.until(
            |screen| parse_tick(screen).is_some_and(|now| now >= target),
            &format!("waiting for the world to reach tick {target}"),
        );
        self
    }

    /// Leave the editor, and wait for the save it queued to actually land.
    ///
    /// A save queues its write for the next tick, so a `peruse` typed straight
    /// afterwards runs before the spell exists — `ORBS_THEN` is the dump side of
    /// the same thing (§19).
    pub fn closes_editor(&self) -> &Self {
        self.type_raw("quit");
        self.wait_ticks(2)
    }

    /// Send a line that opens a full-pane surface, and wait for the surface.
    ///
    /// There is no block: `scribe`, `weave` and `wander` take the whole pane, so
    /// the answer is a *screen* rather than a record. The word still goes
    /// through the prompt, so [`Self::send`]'s wait still applies.
    pub fn opens(&self, line: &str, drawn: &str) -> &Self {
        self.send(line);
        self.expect_drawn(drawn)
    }

    /// Let the world run for a number of ticks.
    /// Keep the count small: `PATIENCE` is 20 seconds and the world runs at 1 Hz.
    /// Scenarios here wait 2–4; one asked for 20 and failed the moment the suite
    /// ran six sessions in parallel.
    pub fn wait_ticks(&self, ticks: u64) -> &Self {
        let target = self.tick() + ticks;
        self.until(
            |screen| parse_tick(screen).is_some_and(|now| now >= target),
            &format!("waiting for tick {target}"),
        );
        self
    }

    /// Resize the window, as a player dragging the corner would.
    pub fn resize(&mut self, cols: u16, rows: u16) -> &Self {
        let status = tmux(&[
            "resize-window",
            "-t",
            &self.session,
            "-x",
            &cols.to_string(),
            "-y",
            &rows.to_string(),
        ])
        .status()
        .expect("tmux should resize the window");
        assert!(status.success(), "tmux refused {cols}x{rows}");
        self.cols = cols;
        self.rows = rows;
        // Wait for the size rather than sleeping, and ask tmux rather than the
        // picture: below the 80×22 floor the game draws a small "too small" card
        // with blank around it, so row 0 is not `cols` wide.
        let deadline = Instant::now() + PATIENCE;
        while Instant::now() < deadline {
            let out = tmux(&[
                "display-message",
                "-p",
                "-t",
                &self.session,
                "#{window_width}x#{window_height}",
            ])
            .output()
            .expect("tmux should report the window size");
            if String::from_utf8_lossy(&out.stdout).trim() == format!("{cols}x{rows}") {
                // The size is in; give the loop its next frame to draw at it.
                std::thread::sleep(Duration::from_millis(120));
                return self;
            }
            std::thread::sleep(POLL);
        }
        panic!("the window never became {cols}x{rows}");
    }
}
