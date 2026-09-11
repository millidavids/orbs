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

/// How long a game this harness starts may live, whatever happens to the harness.
///
/// **Ten minutes, and nothing here should come near it.** The whole suite is
/// under thirty seconds; this is the backstop for a run that was killed, not a
/// budget for a slow scenario. A test that genuinely wants longer is one to start
/// deliberately rather than one to leave sitting.
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
        // Best effort: a panicking test still has to leave the server clean, and
        // a failure to kill a session it may already have lost is not worth
        // replacing the original assertion message with.
        let _ = tmux(&["kill-session", "-t", &self.session]).status();
        // **And the scratch directory, which nothing was removing.** One per
        // game, ~100 per run: a suite that leaves its own litter behind put 600
        // of them in `/tmp` before anybody noticed. Removed here rather than in
        // the script so an ordinary run leaves nothing at all; `play.sh` sweeps
        // what a killed run could not, exactly as it does for the server.
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
        Self::spawn_with(QUIET, GRID.0, GRID.1, true, false)
    }

    /// Start a game **as a fresh game starts**: a laboratory and nothing else
    /// (§11.5), which every other scenario skips by opening the whole tower.
    #[must_use]
    pub fn sealed() -> Self {
        let game = Self::spawn_with(QUIET, GRID.0, GRID.1, false, true);
        game.settled();
        game
    }

    fn open(seed: u64, cols: u16, rows: u16) -> Self {
        let game = Self::spawn(seed, cols, rows);
        game.settled();
        game
    }

    fn spawn(seed: u64, cols: u16, rows: u16) -> Self {
        Self::spawn_with(seed, cols, rows, false, false)
    }

    fn spawn_with(seed: u64, cols: u16, rows: u16, boot: bool, sealed: bool) -> Self {
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
            // **Every room open**, which is the tower these scenarios were
            // written against: a fresh *game* is a laboratory and nothing else
            // (§11.5), and a scenario that walks into the archive on its first
            // line would otherwise be refused in voice. `Game::sealed` is the
            // one door to the start a player gets.
            format!("ORBS_SEALED={}", u8::from(sealed)),
            // **A harness-spawned game gets a lifetime; a player's never does.**
            // The whole suite is under thirty seconds, so ten minutes is a
            // backstop rather than a budget: nothing here should approach it, and
            // a scenario that wants longer has to be run deliberately rather than
            // left to sit.
            //
            // It closes the half `drive::watch_for_hangup` cannot. That handles a
            // game whose terminal *died*; this handles one whose terminal is
            // perfectly alive and whose harness is not — a detached tmux server
            // is nobody's child, so `SIGKILL`ing `cargo test` leaves the sessions
            // running normally, drawing at 30fps, until something kills the
            // server. Measured at ~2.4% CPU each, which is survivable and still
            // not something to leave lying about.
            format!("ORBS_LIFETIME={}", LIFETIME.as_secs()),
            // **No crossings, for a scripted run.** This build holds a settled
            // `Passing` and does not animate one today, so it is belt and braces
            // — but a suite that types a command and reads the screen back must
            // not be able to catch a screen part-way through leaving, and it
            // should not be *this* file's job to notice when that changes.
            // `ORBS_FIRE=0` is the same call for the same reason.
            "ORBS_PASSAGE=0".to_owned(),
            // **Both readers off, so a scenario cannot pass or fail on whether
            // somebody ran the trainer.** Weights are a gitignored build
            // artefact: a fresh clone has none, so a suite that ran with them
            // would behave one way here and another on CI, and the difference
            // would show up as an unrelated scenario failing. `dumps.sh` turns
            // them off in its `run` helper for the same reason, and
            // `Readers::from_environment` refuses to install one under
            // `cargo test` for it too.
            //
            // A scenario that is *about* a reader names one — `stub` is a fixed
            // table and is what such a scenario should pin.
            "ORBS_AUGURY=off".to_owned(),
            "ORBS_SCRIVENER=off".to_owned(),
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

    /// Type text and **stop** — no Enter.
    ///
    /// For everything that answers to where the caret is rather than to a
    /// finished line: the scribing guide, Tab completion, the prompt's ghost.
    /// Pressing Enter would move the caret to the next line and answer a
    /// different question.
    ///
    /// **`-l`, and it matters more here than anywhere.** Without the literal
    /// flag tmux reads a word as a *key name* wherever one matches — `end`
    /// becomes the End key, `up` an arrow — and the half-typed lines this exists
    /// for are exactly the ones that end mid-word.
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

    /// Wait for text to leave the **whole screen**.
    ///
    /// [`Self::expect_absent`] scopes to the newest command block, which is the
    /// right question for a transcript and the wrong one for a pane: the
    /// scribing guide, the tower rail and the instrument panel are all drawn
    /// outside every block, so asking that about one of them passes without
    /// looking at it. Both of this feature's first negative scenarios did.
    ///
    /// This waits, unlike `expect_absent`, and honestly: closing a pane **is**
    /// an event — a redraw — so there is something to pace against, and the
    /// alternative is a race against the frame that has not landed yet.
    ///
    /// Named for the screen rather than shortened to `expect_gone`, which is
    /// already taken and means *the game has exited*.
    pub fn expect_off_screen(&self, needle: &str) -> &Self {
        self.until(
            |screen| !flatten(screen).contains(&flatten(needle)),
            &format!("waiting for {needle:?} to leave the screen"),
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
    /// **Keep the count small — `PATIENCE` is 20 seconds and the world runs at
    /// 1 Hz**, so anything near twenty has no margin at all and passes only on
    /// an idle machine. Every scenario here waits 2–4; a siege scenario asked
    /// for 20 and failed the moment the suite ran six sessions in parallel.
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
            // **Cut at a corner, because a pane beside the transcript has two
            // edges with no `│` in them.** The archive's inline map splits off
            // cleanly on every row but two: its top and bottom borders are
            // `┌───┐` and `└───┘`, so they stay in this cell, glued to whatever
            // the transcript says on that row. `research`'s answer wraps to
            // `…a way out` / `is in them`, and once the map's bottom edge came
            // to share the first of those rows the flattened block read `a way
            // out └───┘ is in them` — twelve scenarios waited on that sentence
            // and all twelve gave up, from `v0.13.19` at the latest.
            //
            // Safe because nothing the game *says* is spelled with a box
            // corner: `Painter::border` is the only thing that draws one.
            let cell = cell.find(['┌', '└']).map_or(cell, |at| &cell[..at]);
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
    // **Leader runs collapse too, and that is the same argument one step on.**
    // `status` draws its readings as `experience ..... 0` so the values share a
    // column, and seven scenarios asserting `experience 0` broke the day it did
    // — every one of them about *what the reading was*, none about how the gap
    // to it was filled. Whitespace was already normalised here for exactly that
    // reason; a run of dots is the same kind of nothing.
    //
    // A **run**, never a single `.`, so a sentence keeps its full stops and a
    // file keeps its extension: `orbs-save.toml` and `laboratory.log` are needles
    // scenarios really do write.
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

/// A pane beside the transcript never becomes part of what the transcript says.
///
/// **Pure, and not `#[ignore]`d**, because the helper it holds is what every
/// scenario reads the game through, and it had no test at all — which is how a
/// defect in it arrived as twelve unrelated-looking scenario failures in the
/// archive and the maze rather than as one failure here.
///
/// The rows are the ones `research` answered on when those twelve failed, cut
/// narrower: the map's top edge beside an instrument row, its side beside the
/// echo, and its bottom edge sharing the first row of a sentence that wraps.
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
/// **The scenario that took the machine down.** 573 orphaned `orbs-tui`
/// processes once reached a load average of 581 on a 32-core box with swap
/// exhausted — leaked by harness runs that were `SIGKILL`ed, so neither
/// `Game::drop` nor `play.sh`'s trap could tear the tmux sessions down. Each
/// orphan then span at ~11% CPU for ever, because a dead pty makes
/// `crossterm::event::read` loop inside itself rather than return.
///
/// Nothing a parent writes can run after it is killed, so the fix is in the game
/// — `drive::watch_for_hangup` — and this is what holds it.
///
/// # It watches one pid, and the first version did not
///
/// Counting `orbs-tui` processes globally passes alone and **fails in the
/// suite**, because a hundred other scenarios are running games at the same time
/// and the count never reaches zero. tmux is asked for this session's own pane
/// pid instead, which is the only number that answers the question being asked.
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

    // Generous against `HANGUP_CHECK`'s half second: what is under test is
    // *does it ever exit*, and a loaded machine must not make that a flake.
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
/// The other half, and the one `watch_for_hangup` cannot see: a detached tmux
/// server is nobody's child, so a `SIGKILL`ed harness leaves its games running
/// **normally** against a live pty. [`LIFETIME`] is what ends those.
///
/// Ten minutes is far too long to sit in a test, so what is checked here is that
/// the cap is *wired* — a one-second lifetime really does stop a game — rather
/// than the shipped number itself, which is asserted directly.
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

    // **Five, not one.** The probe needs two seconds to come up and be asked its
    // pid, and a one-second lifetime means the game is already gone by then —
    // the first version of this test failed on exactly that, reporting a game
    // that never started when what had happened was a game that had finished.
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
    // **`ORBS_SAVE=off` here as everywhere**: a probe must not read or write a
    // tower another scenario is using.
    start
        .arg("-e")
        .arg("ORBS_SAVE=off")
        .arg(format!("ORBS_BOOT=0 {}", env!("CARGO_BIN_EXE_orbs-tui")));
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
