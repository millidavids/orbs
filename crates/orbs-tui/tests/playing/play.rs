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

use std::process::Command;
use std::time::{Duration, Instant};

/// A private tmux socket, so nothing here can see or be seen by the developer's
/// own sessions, their `~/.tmux.conf`, or a `remain-on-exit` that would keep a
/// window open after `quit` and hang the wait for it.
const SOCKET: &str = "orbs-play";

/// The prompt name, pinned. It falls back to `$USER` otherwise, which would make
/// every block header machine-dependent.
const WIZARD: &str = "wizard";

/// How long any wait may take before it is called a failure.
///
/// Generous on purpose: a timeout should mean *broken*, never *busy*.
const PATIENCE: Duration = Duration::from_secs(20);

/// How often the screen is re-read while waiting. A capture costs ~1.1 ms, so
/// polling is close to free and the resolution is worth more.
const POLL: Duration = Duration::from_millis(10);

/// A seed whose first two hundred ticks are free of ambient sabotage.
///
/// `drift` and `substitution` fire from the seeded `Threat` stream, so the
/// schedule is fixed in tick space and can be *chosen away*. Measured across
/// seeds 0, 3, 11 and 42: 3 poisons a log inside 200 ticks, 0 swaps a reagent
/// inside 7200, and 11 and 42 are quiet through both.
pub const QUIET: u64 = 11;

/// The game's own grid, so a capture is directly comparable with `ORBS_DUMP`.
pub const GRID: (u16, u16) = (120, 45);

/// How long a game this harness starts may live, whatever happens to the harness.
///
/// Ten minutes: a backstop for a killed run, not a budget for a slow scenario.
pub const LIFETIME: std::time::Duration = std::time::Duration::from_secs(600);

/// A running game.
pub struct Game {
    session: String,
    cols: u16,
    rows: u16,
    dir: std::path::PathBuf,
}

impl Drop for Game {
    fn drop(&mut self) {
        // Best effort: a failed kill must not replace a panicking test's own
        // assertion message.
        let _ = tmux(&["kill-session", "-t", &self.session]).status();
        // And the scratch directory — one per game, 600 in `/tmp` before anybody
        // noticed. Here rather than in the script so an ordinary run leaves
        // nothing; `play.sh` sweeps what a killed run could not.
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// Run a tmux command against the private socket.
fn tmux(args: &[&str]) -> Command {
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

impl Game {
    /// Start a game at the standard grid on the quiet seed.
    #[must_use]
    pub fn start() -> Self {
        Self::open(QUIET, GRID.0, GRID.1)
    }

    /// Start a game on a chosen seed — for the scenarios that want a particular
    /// maze, ward or scroll draw.
    #[must_use]
    pub fn seeded(seed: u64) -> Self {
        Self::open(seed, GRID.0, GRID.1)
    }

    /// Start a game at a chosen size, for the layout scenarios.
    #[must_use]
    pub fn sized(cols: u16, rows: u16) -> Self {
        Self::open(QUIET, cols, rows)
    }

    /// Start a game **with §4's boot sequence**, which every other scenario skips.
    ///
    /// Nine and a half seconds of wall clock, the only such thing in the game.
    /// No wait for a prompt: the point is the time before there is one.
    #[must_use]
    pub fn booting() -> Self {
        Self::spawn_with(QUIET, GRID.0, GRID.1, true, false, false)
    }

    /// Start a game **at the orb's menu**, which every other scenario skips.
    ///
    /// It cannot be left to `ORBS_DUMP`, which builds no `App` and so has no
    /// `Threshold`, clock gate or keyboard. No wait for a prompt: there is not
    /// one.
    #[must_use]
    pub fn at_the_threshold() -> Self {
        Self::spawn_with(QUIET, GRID.0, GRID.1, false, false, true)
    }

    /// Start a game **as a fresh game starts**: a laboratory and nothing else
    /// (§11.5), which every other scenario skips by opening the whole tower.
    #[must_use]
    pub fn sealed() -> Self {
        let game = Self::spawn_with(QUIET, GRID.0, GRID.1, false, true, false);
        game.settled();
        game
    }

    fn open(seed: u64, cols: u16, rows: u16) -> Self {
        let game = Self::spawn(seed, cols, rows);
        game.settled();
        game
    }

    fn spawn(seed: u64, cols: u16, rows: u16) -> Self {
        Self::spawn_with(seed, cols, rows, false, false, false)
    }

    fn spawn_with(
        seed: u64,
        cols: u16,
        rows: u16,
        boot: bool,
        sealed: bool,
        threshold: bool,
    ) -> Self {
        let binary = env!("CARGO_BIN_EXE_orbs-tui");
        // Unique per game, so scenarios inside one test binary run concurrently
        // without seeing each other's windows.
        let session = format!("play-{}-{}", std::process::id(), next_id());
        // A directory of its own, because `F6` writes `orbs-parse.tsv` relative
        // to the working directory — under `cargo test` the crate root, so a
        // file in the repository and a collision between any two scenarios.
        let dir = std::env::temp_dir().join(&session);
        std::fs::create_dir_all(&dir).expect("a scratch directory for the game");

        let (cols_arg, rows_arg) = (cols.to_string(), rows.to_string());
        let mut args: Vec<String> = [
            "new-session",
            "-d",
            "-s",
            &session,
            "-x",
            &cols_arg,
            "-y",
            &rows_arg,
            "-c",
            &dir.to_string_lossy(),
        ]
        .iter()
        .map(|arg| (*arg).to_owned())
        .collect();

        // `-e`, and not optional: tmux does not pass the client's environment to
        // a new session on an already-running server, and fails *silently* — so
        // a seed set with `Command::env` is ignored wherever a server is up.
        //
        // `ORBS_SAVE=off`, so no scenario inherits another's tower: a scenario
        // that types `quit` writes a save.
        for pair in [
            format!("ORBS_SEED={seed}"),
            format!("ORBS_WIZARD={WIZARD}"),
            "ORBS_SAVE=off".to_owned(),
            // Every room open, the tower these scenarios were written against;
            // a fresh *game* is a laboratory and nothing else (§11.5).
            // `Game::sealed` is the door to a player's start.
            format!("ORBS_SEALED={}", u8::from(sealed)),
            // A harness-spawned game gets a lifetime; a player's never does. It
            // closes the half `drive::watch_for_hangup` cannot — a detached tmux
            // server is nobody's child, so `SIGKILL`ing `cargo test` leaves the
            // sessions drawing at 30fps until something kills them.
            format!("ORBS_LIFETIME={}", LIFETIME.as_secs()),
            // No crossings: a suite that types a command and reads the screen
            // back must not catch one part-way through leaving, and noticing
            // when that starts to matter is not this file's job. `ORBS_FIRE=0`
            // too.
            "ORBS_PASSAGE=0".to_owned(),
            // Both readers off: weights are a gitignored build artefact, so a
            // scenario would behave one way here and another on CI. `dumps.sh`
            // and `Readers::from_environment` do the same. A scenario *about* a
            // reader names one; `stub` is the fixed table to pin.
            "ORBS_AUGURY=off".to_owned(),
            "ORBS_SCRIVENER=off".to_owned(),
        ] {
            args.push("-e".to_owned());
            args.push(pair);
        }
        // The skip, and why ninety-odd scenarios cost a minute rather than a
        // quarter of an hour. `0` is the only skip there is (§19); absent is how
        // the sequence runs, which is what [`Game::booting`] wants.
        if !boot {
            args.push("-e".to_owned());
            args.push("ORBS_BOOT=0".to_owned());
        }
        // Past the front door: a scenario is about a tower, and typing past the
        // menu would test the door ninety times over. The door gets its own
        // scenario, [`Game::at_the_threshold`] — `ORBS_DUMP` builds no `App`, so
        // all 147 captures are byte-identical whatever the threshold does.
        if !threshold {
            args.push("-e".to_owned());
            args.push("ORBS_THRESHOLD=0".to_owned());
        }
        args.push(binary.to_owned());

        let borrowed: Vec<&str> = args.iter().map(String::as_str).collect();
        let status = tmux(&borrowed)
            .status()
            .expect("tmux should start a session");
        assert!(status.success(), "tmux refused to start {session}");

        Self {
            session,
            cols,
            rows,
            dir,
        }
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

    /// Wait for the first frame, so nothing reads an empty screen.
    fn settled(&self) {
        self.until(
            |screen| screen.contains(WIZARD),
            "the game never drew its first prompt",
        );
    }

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
    fn until(&self, mut done: impl FnMut(&str) -> bool, what: &str) {
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

/// A session name nothing else in this process will pick.
fn next_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

/// The transcript column of every row, with the rail and borders removed.
fn transcript(screen: &str) -> String {
    let mut out = String::new();
    for row in screen.lines() {
        // `│transcript│rail│` splits to ["", transcript, rail, ""]. The live
        // prompt row has no border and drops out, correctly: it holds a
        // half-typed line the game has not been told about.
        if let Some(cell) = row.split('│').nth(1) {
            // Cut at a corner: a map's `┌───┐` and `└───┘` have no `│` in them,
            // so they stay in this cell glued to the transcript — once giving
            // `a way out └───┘ is in them`, and twelve scenarios gave up. Safe
            // because only `Painter::border` draws a box corner.
            let cell = cell.find(['┌', '└']).map_or(cell, |at| &cell[..at]);
            out.push_str(cell.trim_end());
            out.push('\n');
        }
    }
    out
}

/// Whether the live prompt has given up the line that was typed into it.
///
/// The prompt row is drawn **outside** the pane border and is the only text the
/// game has not been told about. It carries the ghost completion too, so this
/// asks whether the typed line is gone, not whether the row is bare.
fn prompt_taken(screen: &str, line: &str) -> bool {
    let header = format!("{WIZARD} $");
    screen
        .lines()
        .rfind(|row| !row.starts_with('│') && row.contains(&header))
        .is_none_or(|row| !row.contains(line))
}

/// The newest command block, header included — what [`Game::send`] waits for.
fn block_with_header(screen: &str) -> String {
    let pane = transcript(screen);
    let header = format!("{WIZARD} $");
    pane.rfind(&header)
        .map_or_else(|| pane.clone(), |at| pane[at..].to_owned())
}

/// What the game said back — the newest block **without the line that was typed**.
///
/// The header has to go, or assertions pass on the echo:
/// `does("sift charcoal laboratory.log", "charcoal")` was satisfied by its own
/// argument, with the log empty and the claim untested. Nine shipped that way.
fn block(screen: &str) -> String {
    let full = block_with_header(screen);
    full.split_once('\n')
        .map_or_else(String::new, |(_, body)| body.to_owned())
}

/// One long line, for matching a phrase the pane may have wrapped.
///
/// `RecordView` wraps rather than clips, with a two-cell continuation indent, so
/// a sentence longer than the pane splits mid-phrase. `research`'s answer at
/// 120×45 lands as
///
/// ```text
/// √ the page opens into shelves that do not end. a way out
///     is in them
/// ```
///
/// and a needle as a person reads it matches neither row. Every wait flattens
/// first, so an assertion is about *what the game said* rather than where this
/// width broke the line.
fn flatten(text: &str) -> String {
    // Leader runs collapse too: `status` draws `experience ..... 0`, and seven
    // scenarios asserting `experience 0` broke the day it did.
    //
    // A **run**, never a single `.`, so a sentence keeps its full stops and a
    // file its extension: `orbs-save.toml` and `laboratory.log` are real needles.
    let mut out = String::with_capacity(text.len());
    let mut dots = 0usize;
    for ch in text.chars() {
        if ch == '.' {
            dots += 1;
            continue;
        }
        if dots > 0 {
            // Two or more is a leader; one is punctuation and is kept.
            out.push_str(if dots == 1 { "." } else { " " });
            dots = 0;
        }
        out.push(ch);
    }
    if dots == 1 {
        out.push('.');
    } else if dots > 1 {
        out.push(' ');
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The world's clock, wherever this grid happens to draw it.
///
/// Never a plain search for `tick`: the athanor says *"fuel for 600 ticks"* and
/// `status` prints a snapshot `tick N`. Only the two places the *live* reading
/// is drawn — the rail's foot, and the border title where there is no rail.
fn parse_tick(screen: &str) -> Option<u64> {
    // From the right, because an inline map adds a pane and a fixed index from
    // the left silently reads blank. The length guard keeps `status`'s snapshot
    // `tick N` from being read as the rail at a grid that has none.
    for row in screen.lines() {
        let cells: Vec<&str> = row.split('│').collect();
        if cells.len() >= 5
            && let Some(cell) = cells.get(cells.len() - 2)
            && let Some(rest) = cell.trim().strip_prefix("tick")
            && let Ok(tick) = rest.trim().parse()
        {
            return Some(tick);
        }
    }
    let title = screen.lines().next()?;
    let at = title.find("tick ")?;
    title[at + "tick ".len()..]
        .split_whitespace()
        .next()?
        .parse()
        .ok()
}

/// A pane beside the transcript never becomes part of what the transcript says.
///
/// Pure, and not `#[ignore]`d: every scenario reads the game through this
/// helper, and a defect in it arrived as twelve unrelated-looking failures.
///
/// The rows are the ones `research` answered on when those twelve failed: the
/// map's top edge beside an instrument row, its side beside the echo, and its
/// bottom edge sharing the first row of a wrapped sentence.
#[test]
fn a_pane_beside_the_transcript_does_not_leak_into_it() {
    let screen = "\
│  orb 0 181 cold start      ┌ stacks ──────────┐│le st ││
│→ research                  │██████████████████││   ░░ ││
│√ the page opens into shelves that do not end. a way out  └──────────────────┘│   ░░ ││
│    is in them                                                                │   ░░ ││";
    let read = flatten(&transcript(screen));
    assert!(
        read.contains("a way out is in them"),
        "the map's edge split the sentence: {read:?}",
    );
    assert!(
        !read.contains("stacks"),
        "the map's title was read as transcript: {read:?}",
    );
    assert!(read.contains("orb 0 181 cold start"), "{read:?}");
}

/// A game whose terminal is destroyed exits instead of spinning.
///
/// 573 orphans once reached a load average of 581: a dead pty makes
/// `crossterm::event::read` loop inside itself, and nothing a `SIGKILL`ed parent
/// wrote could tear the sessions down. The fix is `drive::watch_for_hangup`.
///
/// It watches one pid — counting `orbs-tui` processes globally never reaches
/// zero in a suite running a hundred games — so tmux is asked for this session's
/// own pane pid.
#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_game_whose_terminal_dies_does_not_outlive_it() {
    if !available() {
        return;
    }
    let socket = format!("hangup-{}", std::process::id());
    let Some(pid) = probe(&socket) else {
        panic!("the probe game never started, so this proves nothing");
    };

    let _ = Command::new("tmux")
        .args(["-L", &socket, "kill-server"])
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
/// The half `watch_for_hangup` cannot see: a detached tmux server is nobody's
/// child, so a `SIGKILL`ed harness leaves its games running against a live pty.
/// [`LIFETIME`] ends those.
///
/// Ten minutes is too long to sit in a test, so this checks the cap is *wired*
/// and asserts the shipped number directly.
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
    let socket = format!("lifetime-{}", std::process::id());
    let Some(pid) = probe_with(&socket, &["ORBS_LIFETIME=5"]) else {
        panic!("the probe game never started, so this proves nothing");
    };

    for _ in 0..40 {
        if !running(pid) {
            let _ = Command::new("tmux")
                .args(["-L", &socket, "kill-server"])
                .status();
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
    let _ = Command::new("kill").args(["-9", &pid.to_string()]).status();
    let _ = Command::new("tmux")
        .args(["-L", &socket, "kill-server"])
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
