//! A game, played through a real terminal, with the screen read back.
//!
//! # Why the assertions anchor on the *last block* and never on the screen
//!
//! The transcript keeps its history, and `Records` is never truncated. A driver
//! that waits for a substring to *appear anywhere* is therefore wrong twice
//! over, and both failures were measured before this file existed:
//!
//! - **Stale matches.** The clarity chain empties the mortar twice, six steps
//!   apart and both still on screen. The second wait returned in **1 ms**
//!   against the first one's line, the driver ran ahead of the game, and the
//!   suite went green on an assertion that was factually wrong.
//! - **Saturation.** Counting occurrences instead — wait for the count to rise —
//!   deadlocks: the pane holds exactly **8 command blocks** at 120×45, so from
//!   the ninth repeat onward each new line pushes an old one off and the count
//!   never rises again. It also runs *backwards* mid-chain, because the pane is
//!   a tail over `Records::drawn` and nothing about it is monotonic.
//!
//! So a match is scoped by **position**: everything since the last
//! `wizard $` header. A stale line is in an earlier block by construction, and a
//! repeat is still the newest block. It is also self-pacing — the echo appears
//! at once and the result lands on the next tick, so waiting for the result *is*
//! waiting for the world.
//!
//! # The screen's shape, which the parsing depends on
//!
//! ```text
//! │wizard $ grind sage                    │mp bm fr al at │   ← history, inside
//! │→ grind sage                           │███████████████│
//! │√ sage: dispensary to mortar_and_pestle │███████████████│
//! └───────────────────────────────────────┴───────────────┘
//!  wizard $ grind sa                                          ← live, outside
//! ```
//!
//! The rail shares every row, so a row is split on `│` and the transcript is the
//! first cell. The live prompt is drawn *outside* the border and is deliberately
//! not part of any block — it holds a half-typed line and the ghost completion.

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
/// Generous on purpose: the slowest single step measured is a little over a
/// tick, and a loaded CI box is slower than this desk. A timeout here should
/// mean *broken*, never *busy*.
const PATIENCE: Duration = Duration::from_secs(20);

/// How often the screen is re-read while waiting. A capture costs ~1.1 ms, so
/// polling is close to free and the resolution is worth more.
const POLL: Duration = Duration::from_millis(10);

/// A seed whose first two hundred ticks are free of ambient sabotage.
///
/// `drift` poisons a log at 1/300 per tick and `substitution` renames a base
/// reagent at 1/3600, both from the seeded `Threat` stream — so the schedule is
/// fixed in tick space and can simply be *chosen away* rather than tolerated.
/// Measured across seeds 0, 3, 11 and 42: seed 3 — the one every See-it line
/// uses — poisons a log inside 200 ticks, and seed 0 swaps a reagent inside
/// 7200. 11 and 42 are quiet through both.
pub const QUIET: u64 = 11;

/// The game's own grid, so a capture is directly comparable with `ORBS_DUMP`.
pub const GRID: (u16, u16) = (120, 45);

/// A running game.
pub struct Game {
    session: String,
    cols: u16,
    rows: u16,
    dir: std::path::PathBuf,
}

impl Drop for Game {
    fn drop(&mut self) {
        // Best effort: a panicking test still has to leave the server clean, and
        // a failure to kill a session it may already have lost is not worth
        // replacing the original assertion message with.
        let _ = tmux(&["kill-session", "-t", &self.session]).status();
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
/// **A missing tmux is not a broken game**, so a scenario says so and returns
/// rather than failing. It is printed rather than silent because a suite that
/// quietly tests nothing is worse than one that is red.
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
    /// Nine and a half seconds of wall clock, and the only thing in the game
    /// that is pure wall clock — so it exists nowhere but here. No wait for a
    /// prompt, deliberately: the whole point is the time before there is one.
    #[must_use]
    pub fn booting() -> Self {
        Self::spawn_with(QUIET, GRID.0, GRID.1, true)
    }

    fn open(seed: u64, cols: u16, rows: u16) -> Self {
        let game = Self::spawn(seed, cols, rows);
        game.settled();
        game
    }

    fn spawn(seed: u64, cols: u16, rows: u16) -> Self {
        Self::spawn_with(seed, cols, rows, false)
    }

    fn spawn_with(seed: u64, cols: u16, rows: u16, boot: bool) -> Self {
        let binary = env!("CARGO_BIN_EXE_orbs-tui");
        // Unique per game, so scenarios inside one test binary run concurrently
        // without seeing each other's windows.
        let session = format!("play-{}-{}", std::process::id(), next_id());
        // **A directory of its own, because `F6` writes `orbs-parse.tsv`
        // relative to the working directory.** Under `cargo test` that would be
        // the crate root — a file written into the repository, and a collision
        // between any two scenarios that press it.
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

        // **`-e`, and it is not optional.** tmux does not pass the client's
        // environment to a new session on an already-running server, and it
        // fails *silently* — so a seed set with `Command::env` would be ignored
        // on any machine that already had a tmux server up, and every scenario
        // would quietly run on the default world with nothing to show for it.
        // **`ORBS_SAVE=off`, so no scenario can inherit another's tower.**
        // `spawn_with` already gives each scenario a unique directory and hands
        // it to `tmux -c`, so this is deliberate isolation rather than a fix for
        // a live collision — but a scenario that types `quit` writes a save, and
        // a suite whose scenarios could load one another's is one where a
        // failure depends on the order the threads happened to run in.
        for pair in [
            format!("ORBS_SEED={seed}"),
            format!("ORBS_WIZARD={WIZARD}"),
            "ORBS_SAVE=off".to_owned(),
        ] {
            args.push("-e".to_owned());
            args.push(pair);
        }
        // **The skip, and it is why ninety-odd scenarios cost a minute rather
        // than a quarter of an hour.** §4's sequence runs in this frontend too
        // now; `0` is the only skip there is, and §19 removed the keypress one.
        // Absent is how the sequence runs, which is what [`Game::booting`] wants.
        if !boot {
            args.push("-e".to_owned());
            args.push("ORBS_BOOT=0".to_owned());
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
    /// **`F6` says nothing on the transcript**, which is the whole reason this
    /// exists: the only evidence that the key did anything is the file, and the
    /// only reason each game gets a directory of its own is so that two
    /// scenarios pressing it cannot collide — or write into the repository.
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
    /// **The only way to see `theme.rs` from outside.** A plain capture throws
    /// every attribute away, so a fault in the colour table — a fire ramp that
    /// climbs to white, a fault marker drawn in the success hue — is invisible
    /// to every other instrument in the project.
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
    /// This is the unit every assertion is scoped to. See the module header for
    /// the two ways a whole-screen search goes wrong.
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
    /// **`send-keys -l`, and Enter on a call of its own.** Without the literal
    /// flag tmux reads the argument as a key name wherever one matches: `end`
    /// becomes the End key, `up` an arrow, `home` Home. `end` closes every
    /// `repeat` and every `if` in the spell language, so a spell typed without
    /// this loses its blocks and the editor never sees the word.
    /// Type a line at the prompt and wait for the game to open its block.
    ///
    /// **The wait is not politeness, it is the correctness of every assertion
    /// that follows.** Scoping a match to the newest block is only sound once
    /// the newest block is the one just typed; ask a moment too early and
    /// `rfind` is still pointing at the *previous* command, whose answer is
    /// already on screen. That is the stale match this whole file is built to
    /// avoid, arriving through the one door the block-scoping left open — and it
    /// was not theoretical: `debug_spawn ground-salt` matched the `all along` of
    /// the `debug_spawn sage-tincture` before it, the driver ran a command
    /// ahead, and `mix` was typed against a shelf that had nothing on it yet.
    pub fn send(&self, line: &str) -> &Self {
        self.type_raw(line);
        let header = format!("{WIZARD} $ {line}");
        self.until(
            // Two conditions, and the first is what makes a *repeated* line
            // safe: the live prompt is drawn outside the border and still holds
            // the text until the game takes it, so an emptied prompt means this
            // line — not the identical one before it — has been consumed.
            |screen| prompt_taken(screen, line) && block_with_header(screen).starts_with(&header),
            &format!("waiting for the prompt to take {line:?}"),
        );
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
    /// **`does("meditate 20", "meditate")` is a trap and it caught three
    /// scenarios at once.** The prompt echoes `→ meditate 20` the instant the
    /// line is taken, so a wait for the word *"meditate"* is satisfied before a
    /// single tick has passed — the driver walks on, and a spell that was
    /// supposed to have run for twenty ticks has run for none. Two of the three
    /// long playthroughs failed exactly there, one of them after asking for
    /// seven thousand ticks and getting zero.
    ///
    /// The clock cannot be fooled that way, so this waits for it.
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
    /// **A save queues its write for the next tick like every other effect**, so
    /// a `peruse` typed straight afterwards runs before the spell exists and
    /// offers the other readables instead — which looks exactly like a bug and
    /// is not one. §19 records `ORBS_THEN` existing for the same reason on the
    /// dump side.
    pub fn closes_editor(&self) -> &Self {
        self.type_raw("quit");
        self.wait_ticks(2)
    }

    /// Send a line that opens a full-pane surface, and wait for the surface.
    ///
    /// **A block-scoped wait cannot work here, because there is no block.**
    /// `scribe`, `weave` and `wander` take the whole pane, transcript and all —
    /// so the answer to the word is a *screen*, not a record, and waiting for it
    /// in the newest command block waits for something that has been painted
    /// over. The word still goes through the prompt, so [`Self::send`]'s own
    /// wait still applies; only the assertion changes scope.
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
    /// For the handful of answers that are **not** a reply to the last line —
    /// a threshold announcing itself while a brew finishes, a spell's fault
    /// latching a room. Prefer [`Self::expect`]; this one can match history.
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
    /// **It waits for nothing, and that is what makes it safe.** A negative
    /// claim has no event to pace itself against, so this is only honest after
    /// an [`Self::expect`] on the same block has already landed — the answer is
    /// on screen whole, and the question is what is missing from it. Use
    /// [`Self::refute_after`] when the claim is about the future instead.
    pub fn expect_absent(&self, needle: &str) -> &Self {
        let block = flatten(&block(&self.screen()));
        assert!(
            !block.contains(&flatten(needle)),
            "{needle:?} is in the newest block, and should not be:\n{}",
            self.screen(),
        );
        self
    }

    /// Let the world run, then assert something never showed up.
    ///
    /// **A negative assertion cannot pace itself**, which is why it takes a
    /// number of ticks: there is no event to wait for, so the only honest
    /// question is *after this much play, is it still absent*. Used sparingly.
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
        // A resize is a redraw, and the redraw is the thing under test — so wait
        // for the screen to actually be that wide rather than sleeping.
        // **Ask tmux, not the picture.** Waiting for row 0 to be exactly `cols`
        // wide works while the game is drawing a full-width border and fails the
        // moment it is not — and the most interesting resize of all is the one
        // below the 80×22 floor, where the answer is a small "too small" card
        // with a great deal of blank around it.
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
        // prompt row has no border at all and is dropped with everything else
        // outside the pane, which is correct: it holds a half-typed line and a
        // ghost completion, neither of which the game has been told yet.
        if let Some(cell) = row.split('│').nth(1) {
            out.push_str(cell.trim_end());
            out.push('\n');
        }
    }
    out
}

/// Whether the live prompt has given up the line that was typed into it.
///
/// The prompt row is drawn **outside** the pane border, on the last row, and is
/// the only part of the screen holding text the game has not been told about
/// yet. It also carries the ghost completion, so this asks whether the typed
/// line is gone rather than whether the row is bare.
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
/// **The header has to go, and leaving it in made assertions pass on the echo.**
/// `send` waits until the newest block starts with `wizard $ <line>`, so by the
/// time anything is asserted the block is guaranteed to contain the command
/// text — and `does("sift charcoal laboratory.log", "charcoal")` was then
/// satisfied by its own argument, with the log empty and the claim untested.
/// Nine assertions shipped that way.
///
/// It is the same failure as the stale match and the saturating count, arriving
/// through the one door left: not *"an older answer"* but *"the question,
/// mistaken for the answer"*. Dropping the first line costs nothing and closes
/// it, because a needle can no longer be found in the words the test typed.
fn block(screen: &str) -> String {
    let full = block_with_header(screen);
    full.split_once('\n')
        .map_or_else(String::new, |(_, body)| body.to_owned())
}

/// One long line, for matching a phrase the pane may have wrapped.
///
/// **`RecordView` wraps rather than clips**, with a two-cell continuation
/// indent, so a sentence longer than the pane is split mid-phrase across two
/// rows. `research`'s answer is the everyday example: at 120×45 it lands as
///
/// ```text
/// √ the page opens into shelves that do not end. a way out
///     is in them
/// ```
///
/// and a needle written the way a person reads it matches neither row. Every
/// wait flattens before it matches, so an assertion is about *what the game
/// said* rather than about where this particular pane width broke the line —
/// which also keeps the resize scenarios from needing different needles from
/// everything else.
fn flatten(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The world's clock, wherever this grid happens to draw it.
///
/// **Never a plain search for `tick`**, and the two traps are both on screen at
/// once: the athanor says *"fuel for 600 ticks"* and `status` prints its own
/// `tick N` into the transcript, which is a snapshot of whenever it was typed
/// rather than the clock. So this reads only the two places the *live* reading
/// is drawn — the rail's foot where there is a rail, and the session border's
/// title where the grid is too small for one.
fn parse_tick(screen: &str) -> Option<u64> {
    // **The rail is the rightmost pane, and counting from the left does not
    // find it.** `│transcript││rail│` splits to five cells — two borders meet in
    // the middle, leaving an empty one between them — but open a maze and the
    // inline map adds a pane, so the rail moves from the fourth cell to the
    // fifth and a fixed index silently reads blank. Counting from the right is
    // stable under any number of panes.
    //
    // The length guard is what keeps `status`'s own `tick N` — which is a
    // snapshot in the *transcript*, not the clock — from being read as the rail
    // at a grid that has no rail at all.
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
