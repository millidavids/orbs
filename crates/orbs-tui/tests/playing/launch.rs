//! Starting a game in a terminal of its own, and a folder a game may keep.

use std::path::PathBuf;

use crate::play::{GRID, Game, LIFETIME, QUIET, WIZARD};
use crate::tmux::{available, next_id, pane_pid, tmux};

/// An on/off switch handed to the game: `0`, `1`, or the setting deciding.
#[derive(Debug, Clone, Copy)]
pub enum Switch {
    /// `0`.
    Never,
    /// `1`.
    Always,
    /// Blank: the setting decides.
    BySetting,
}

impl Switch {
    /// Blank rather than unset for the setting: unset, a value exported in the
    /// shell that started the tmux server reaches the game.
    const fn value(self) -> &'static str {
        match self {
            Self::Never => "0",
            Self::Always => "1",
            Self::BySetting => "",
        }
    }
}

/// How a game is started. [`Game::launch`] gives the scenarios' defaults: the
/// quiet seed, the game's own grid, no boot card, every room open, past the
/// front door, no installation, and nothing kept.
#[must_use]
pub struct Launch {
    seed: u64,
    cols: u16,
    rows: u16,
    boot: bool,
    sealed: bool,
    threshold: bool,
    installs: Switch,
    /// Whether anything may move: still by default, so a capture never lands
    /// half way through a roll or a line typing itself.
    motion: Switch,
    kept: Option<PathBuf>,
    /// Anything else the game is handed, each replacing a default of its name.
    env: Vec<(String, String)>,
}

impl Default for Launch {
    fn default() -> Self {
        Self {
            seed: QUIET,
            cols: GRID.0,
            rows: GRID.1,
            boot: false,
            sealed: false,
            threshold: false,
            installs: Switch::Never,
            motion: Switch::Never,
            kept: None,
            env: Vec::new(),
        }
    }
}

impl Launch {
    /// A chosen seed — for the scenarios that want a particular maze, ward or
    /// scroll draw.
    pub const fn seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// A chosen size, for the layout scenarios.
    pub const fn sized(mut self, cols: u16, rows: u16) -> Self {
        self.cols = cols;
        self.rows = rows;
        self
    }

    /// **§4's boot sequence**, which every other scenario skips. Nine and a half
    /// seconds of wall clock, the only such thing in the game.
    pub const fn booting(mut self) -> Self {
        self.boot = true;
        self
    }

    /// **As a fresh game starts**: a laboratory and nothing else (§11.5).
    pub const fn sealed(mut self) -> Self {
        self.sealed = true;
        self
    }

    /// **At the orb's menu**, which every other scenario passes. It cannot be
    /// left to `ORBS_DUMP`, which builds no `App` and so has no `Threshold`,
    /// clock gate or keyboard.
    pub const fn at_the_threshold(mut self) -> Self {
        self.threshold = true;
        self
    }

    /// Whether `new` at the menu installs the orb first.
    pub const fn installs(mut self, installs: Switch) -> Self {
        self.installs = installs;
        self
    }

    /// Whether anything may move — `ORBS_MOTION`, which seeds the `motion` row.
    pub const fn motion(mut self, motion: Switch) -> Self {
        self.motion = motion;
        self
    }

    /// Towers and settings kept in this folder rather than nowhere.
    pub fn kept_in(mut self, kept: &Kept) -> Self {
        self.kept = Some(kept.save());
        self
    }

    /// One more variable for the game — a reader to pin, say. It replaces the
    /// default of the same name rather than sitting beside it: two `-e` for one
    /// name leave the winner to tmux.
    pub fn env(mut self, key: &str, value: &str) -> Self {
        self.env.push((key.to_owned(), value.to_owned()));
        self
    }

    /// Start it, and wait for the first prompt.
    pub fn settled(self) -> Option<Game> {
        let game = self.spawn()?;
        game.settled();
        Some(game)
    }

    /// Start it and wait for nothing — for a boot card or a menu, where there is
    /// no prompt to wait for.
    ///
    /// `None` when tmux is not installed, so a scenario cannot start a game
    /// without asking.
    pub fn spawn(self) -> Option<Game> {
        if !available() {
            return None;
        }
        let binary = env!("CARGO_BIN_EXE_orbs-tui");
        // Unique per game, so scenarios inside one test binary run concurrently
        // without seeing each other's windows.
        let session = format!("play-{}-{}", std::process::id(), next_id());
        // A directory of its own, because `F6` writes `orbs-parse.tsv` relative
        // to the working directory — under `cargo test` the crate root, so a
        // file in the repository and a collision between any two scenarios.
        let dir = std::env::temp_dir().join(&session);
        std::fs::create_dir_all(&dir).expect("a scratch directory for the game");

        let (cols, rows) = (self.cols.to_string(), self.rows.to_string());
        let mut args: Vec<String> = [
            "new-session",
            "-d",
            "-s",
            &session,
            "-x",
            &cols,
            "-y",
            &rows,
            "-c",
            &dir.to_string_lossy(),
        ]
        .iter()
        .map(|arg| (*arg).to_owned())
        .collect();

        // `-e`, and not optional: tmux does not pass the client's environment to
        // a new session on an already-running server, and fails *silently* — so
        // a seed set with `Command::env` is ignored wherever a server is up.
        let save = self.kept.as_ref().map_or_else(
            // So no scenario inherits another's tower: a scenario that types
            // `quit` writes a save.
            || "off".to_owned(),
            |save| save.to_string_lossy().into_owned(),
        );
        let mut pairs = vec![
            format!("ORBS_SEED={}", self.seed),
            format!("ORBS_WIZARD={WIZARD}"),
            format!("ORBS_SAVE={save}"),
            // Every room open, the tower these scenarios were written against;
            // a fresh *game* is a laboratory and nothing else (§11.5).
            format!("ORBS_SEALED={}", u8::from(self.sealed)),
            // A harness-spawned game gets a lifetime; a player's never does. It
            // closes the half `hangup::watch_for_hangup` cannot — a detached tmux
            // server is nobody's child, so `SIGKILL`ing `cargo test` leaves the
            // sessions drawing at 30fps until something kills them.
            format!("ORBS_LIFETIME={}", LIFETIME.as_secs()),
            // No crossings unless motion was asked for outright: a suite that
            // types a command and reads the screen back must not catch one
            // part-way through leaving. `Passing` caches this at construction,
            // so a scenario that wants to watch one needs it gone, not the row.
            format!(
                "ORBS_PASSAGE={}",
                if matches!(self.motion, Switch::Always) {
                    ""
                } else {
                    "0"
                }
            ),
            // Both readers off: weights are a gitignored build artefact, so a
            // scenario would behave one way here and another on CI. `dumps.sh`
            // and `Readers::from_environment` do the same. A scenario *about* a
            // reader names one; `stub` is the fixed table to pin.
            "ORBS_AUGURY=off".to_owned(),
            "ORBS_SCRIVENER=off".to_owned(),
            // Off unless asked, for the threshold's reason: `new` at the menu
            // would otherwise reach the practice tower, and the scenarios that
            // type it are about the tower that follows.
            format!("ORBS_INSTALL={}", self.installs.value()),
            format!("ORBS_MOTION={}", self.motion.value()),
        ];
        // The skip, and why ninety-odd scenarios cost a minute rather than a
        // quarter of an hour. `0` is the only skip there is (§19); absent is how
        // the sequence runs, which is what [`Launch::booting`] wants.
        if !self.boot {
            pairs.push("ORBS_BOOT=0".to_owned());
        }
        // Past the front door: a scenario is about a tower, and typing past the
        // menu would test the door ninety times over. `ORBS_DUMP` builds no
        // `App`, so every capture is byte-identical whatever the threshold does.
        if !self.threshold {
            pairs.push("ORBS_THRESHOLD=0".to_owned());
        }
        for (key, value) in &self.env {
            let prefix = format!("{key}=");
            pairs.retain(|pair| !pair.starts_with(&prefix));
            pairs.push(format!("{key}={value}"));
        }
        for pair in pairs {
            args.push("-e".to_owned());
            args.push(pair);
        }
        args.push(binary.to_owned());

        let borrowed: Vec<&str> = args.iter().map(String::as_str).collect();
        let status = tmux(&borrowed)
            .status()
            .expect("tmux should start a session");
        assert!(status.success(), "tmux refused to start {session}");

        // Only a kept game needs its pid: its `Drop` waits for the exit-save, or
        // the folder it is about to remove is written back into existence.
        let pid = self.kept.as_ref().and_then(|_| pane_pid(&session));
        Some(Game {
            session,
            cols: self.cols,
            rows: self.rows,
            dir,
            pid,
        })
    }
}

/// A folder a game keeps its towers and settings in, outliving the games
/// launched into it — for the scenarios about what a relaunch remembers.
///
/// Under the temp directory with the `play-` prefix, so `scripts/play.sh`'s
/// sweep collects what a killed run leaves. Declare it before the games that use
/// it, so it is dropped after them.
pub struct Kept {
    dir: PathBuf,
}

impl Default for Kept {
    fn default() -> Self {
        let dir =
            std::env::temp_dir().join(format!("play-{}-kept-{}", std::process::id(), next_id()));
        std::fs::create_dir_all(&dir).expect("a folder to keep towers in");
        Self { dir }
    }
}

impl Kept {
    /// The file `ORBS_SAVE` names. It names a file, not a folder: the settings
    /// and every other slot sit beside it.
    #[must_use]
    pub fn save(&self) -> PathBuf {
        self.dir.join("orbs-save.toml")
    }
}

impl Drop for Kept {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}
