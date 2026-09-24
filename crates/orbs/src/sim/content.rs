//! Watching authored content, and handing it to the sim on a tick boundary.
//!
//! CLAUDE.md rule 6 puts prose in data files so a writer can work without a
//! compiler. This is the half that makes "hot-reloadable" true rather than
//! aspirational: edit `prose.toml` with the game running and the next tick
//! speaks the new line.
//!
//! Finding and parsing the file is [`orbs_shell::load`]'s, because every frontend
//! needs that and only a hosted one can keep a background thread.
//!
//! The watcher is here rather than in `orbs-sim` because rule 8 forbids async
//! there and rule 3 makes a frontend a caller, not a host — and a watcher inside
//! the sim would make every headless test touch the filesystem. `notify` runs
//! its own thread and posts over a channel; a system drains it and calls
//! [`Sim::set_prose`](orbs_sim::Sim::set_prose) between ticks.
//!
//! Opt-in, so a shipped build cannot depend on a path that is not there: content
//! is compiled in with `include_str!`, and this module does nothing unless
//! `ORBS_CONTENT` names a directory:
//!
//! ```bash
//! ORBS_CONTENT=crates/orbs-sim/content cargo run -p orbs
//! ```
//!
//! Absent that, the game runs on the built-in text and starts no thread.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, channel};
use std::time::SystemTime;

use bevy::prelude::*;
use notify::{RecursiveMode, Watcher as _};

use orbs_shell::{CONTENT_DIR, MANUAL, PROSE, read, read_manual};

use super::driver::Tower;

/// A live watch on the content directory.
///
/// Holds the `notify` watcher: dropping it stops the background thread, so it
/// must outlive the app rather than the function that built it.
#[derive(Resource)]
pub(crate) struct ContentWatch {
    /// Kept alive for its `Drop`, never read.
    _watcher: notify::RecommendedWatcher,
    /// `Receiver` is `Send` but not `Sync`, and a Bevy resource must be both.
    /// `try_recv` only needs `&self`, so a mutex is enough and is never
    /// contended — exactly one system reads this, on one thread.
    changes: Mutex<Receiver<()>>,
    prose: Live,
    /// The manual, live since `0.16.8` for rule 6's own reason — ten thousand
    /// words behind `include_str!` is a recompile per sentence.
    manual: Live,
}

/// One file `ORBS_CONTENT` can replace, and the moment it last did.
struct Live {
    path: PathBuf,
    /// The modification time already applied.
    ///
    /// Not an optimisation — it stops the watch feeding itself. Re-reading the
    /// file lands in the directory watch that asked for the re-read, so at
    /// `0.16.8` one `touch` reloaded `prose.toml` on every tick for the life of
    /// the run, saying so once a second while the game looked fine.
    ///
    /// Draining the channel after the read would also break the loop, but would
    /// swallow an edit saved during it. Comparing the stamp costs one `stat` on
    /// a spurious event and nothing else.
    ///
    /// It accepts two writes inside one filesystem timestamp tick — nanoseconds
    /// here, and the next save corrects it.
    stamp: Option<SystemTime>,
}

impl Live {
    /// Start watching a file, taking its current stamp as already applied.
    fn new(path: PathBuf) -> Self {
        let stamp = modified(&path);
        Self { path, stamp }
    }

    /// Whether what is on disk is newer than what has been applied.
    fn moved(&mut self) -> bool {
        let now = modified(&self.path);
        if now == self.stamp {
            return false;
        }
        self.stamp = now;
        true
    }
}

/// A file's modification time, or `None` if it cannot be asked for.
///
/// Absent is a stable answer, not an error: a content directory with no
/// `manual.toml` in it stays `None` and never reloads, and one that gains the
/// file later moves from `None` to `Some` and does.
fn modified(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path)
        .and_then(|data| data.modified())
        .ok()
}

/// Put `ORBS_CONTENT`'s prose into a tower that has just been built.
///
/// Called again whenever a tower is, which is why it is a function. `install`
/// loads once at App-build time and the watcher keeps *that* tower current, but
/// a `Sim` swapped in from the menu is a new world holding the built-in text —
/// so a writer would load a save and silently get the shipped prose back.
pub(crate) fn apply(tower: &mut Tower) {
    let Some(dir) = std::env::var_os(CONTENT_DIR).map(PathBuf::from) else {
        return;
    };
    // Absent is not a complaint, which `orbs_shell::load` decided and this path
    // did not follow: a writer working on the prose keeps a directory with
    // `prose.toml` and nothing else, and without these guards `content: cannot
    // read .../manual.toml` was logged on every launch and every swap. A file
    // that is there and will not parse is still said, loudly, by `read`.
    let prose = dir.join(PROSE);
    if prose.exists()
        && let Some(loaded) = read(&prose)
    {
        tower.set_prose(loaded);
    }
    let manual = dir.join(MANUAL);
    if manual.exists()
        && let Some(loaded) = read_manual(&manual)
    {
        tower.set_manual(loaded);
    }
}

/// Install the watcher, if `ORBS_CONTENT` names a readable directory.
///
/// Failure is always a warning and never fatal — a missing or unreadable
/// content directory means the built-in text, which is a complete game.
pub(crate) fn install(app: &mut App) {
    let Some(dir) = std::env::var_os(CONTENT_DIR).map(PathBuf::from) else {
        return;
    };
    let prose = dir.join(PROSE);
    let manual = dir.join(MANUAL);

    // Load once up front, so a run starts on what is on disk rather than
    // waiting for the first edit. Guarded on existence for `apply`'s reason: a
    // live file that is simply not there is an ordinary content directory, not
    // something to warn about once a launch.
    if prose.exists()
        && let Some(loaded) = read(&prose)
    {
        app.world_mut().resource_mut::<Tower>().set_prose(loaded);
        info!("content: loaded {}", prose.display());
    }
    if manual.exists()
        && let Some(loaded) = read_manual(&manual)
    {
        app.world_mut().resource_mut::<Tower>().set_manual(loaded);
        info!("content: loaded {}", manual.display());
    }

    let (tx, changes) = channel();
    let mut watcher = match notify::recommended_watcher(move |event| {
        // Coalesced to a bare "something changed": the file is re-read from
        // scratch on the next tick, so *what* changed is not interesting, and a
        // save that arrives as several events must not queue several reloads.
        if let Ok(notify::Event { .. }) = event {
            let _ = tx.send(());
        }
    }) {
        Ok(watcher) => watcher,
        Err(error) => {
            warn!("content: no watcher ({error}); running on built-in text");
            return;
        }
    };

    // The *directory*, not the file. Editors save by writing a temporary file
    // and renaming it over the target, which severs a watch on the inode and
    // makes exactly one edit work before the watch goes quiet.
    //
    // Only `prose.toml` and `manual.toml` are live, though the whole directory
    // is watched. `recipes.toml` reaches decisions, so swapping it mid-session
    // would break replay from `(seed, submissions)` (see `Sim::new`); `fuel.toml`
    // is the same. Editing either said "reloaded prose.toml" and changed no
    // behaviour, so name the live files once where a writer will see it.
    //
    // Announced after the watch takes: "watching" followed by "cannot watch"
    // leaves a writer with two contradictory claims.
    if let Err(error) = watcher.watch(&dir, RecursiveMode::NonRecursive) {
        warn!("content: cannot watch {} ({error})", dir.display());
        return;
    }
    info!(
        "content: watching {} ({PROSE} and {MANUAL} are live)",
        dir.display()
    );

    app.insert_resource(ContentWatch {
        _watcher: watcher,
        changes: Mutex::new(changes),
        // Stamped *after* the load above, so the run starts level with disk and
        // the first event that arrives has something to be newer than.
        prose: Live::new(prose),
        manual: Live::new(manual),
    })
    // Before `advance`, explicitly. Both take `ResMut<Tower>` in `FixedUpdate`
    // with nothing between them, which is an ambiguity rather than an ordering —
    // so only this edge makes `reload`'s "lands on a tick boundary" true.
    // The message is registered by `ShellPlugin`, which reads it and is always
    // present; this module is behind `ORBS_CONTENT`.
    .add_systems(
        FixedUpdate,
        reload
            .before(super::driver::advance)
            .run_if(resource_exists::<ContentWatch>),
    );
}

/// Apply any pending edit, between ticks.
///
/// In `FixedUpdate` alongside `advance`, so a reload lands on a tick boundary
/// and never part-way through a schedule.
fn reload(
    mut tower: ResMut<Tower>,
    mut watch: ResMut<ContentWatch>,
    mut restocking: MessageWriter<ManualChangedMessage>,
) {
    let mut changed = false;
    {
        // A poisoned lock would mean the only other holder panicked; there is no
        // other holder, so treat it as empty rather than taking the tower down
        // over a hot-reload convenience.
        let Ok(changes) = watch.changes.lock() else {
            return;
        };
        // Drain the whole queue before re-reading: one save can arrive as
        // several events, and re-reading per event would parse the same bytes
        // repeatedly for no gain.
        while changes.try_recv().is_ok() {
            changed = true;
        }
    }
    if !changed {
        return;
    }
    // The event says only "something in this directory moved", so which file was
    // saved is asked here rather than carried: the watch is coalesced on
    // purpose, and the alternative is trusting a path that arrives through a
    // rename. `Live::moved` also keeps a read from queueing the next reload.
    if watch.prose.moved()
        && let Some(prose) = read(&watch.prose.path)
    {
        tower.set_prose(prose);
        info!("content: reloaded {}", watch.prose.path.display());
    }
    if watch.manual.moved()
        && let Some(manual) = read_manual(&watch.manual.path)
    {
        tower.set_manual(manual);
        info!("content: reloaded {}", watch.manual.path.display());
        // Said, so an open reader can re-assemble. The book is a snapshot taken
        // when the manual opened; without this the log claimed the edit landed
        // and the screen went on showing the old chapter. See
        // `shell::manualling::restock_requested`.
        restocking.write(ManualChangedMessage);
    }
}

/// The authored manual has been replaced on disk.
///
/// A message rather than the watcher reaching into the shell: `Reading` is the
/// shell's and the book is assembled from the `Sim`, which is this module's.
#[derive(Message, Debug, Clone, Copy)]
pub(crate) struct ManualChangedMessage;

#[cfg(test)]
mod tests {
    use super::*;

    /// A scratch directory that removes itself.
    ///
    /// A guard rather than tidying by hand: these keyed the directory on
    /// `std::process::id()` and removed nothing, so every `cargo test` run left
    /// one behind for ever. `Drop` runs on a failing assertion too, which a line
    /// at the end of each test does not.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new() -> Self {
            let dir = std::env::temp_dir().join(format!(
                "orbs-content-{}-{:?}",
                std::process::id(),
                std::thread::current().id(),
            ));
            std::fs::create_dir_all(&dir).expect("a scratch directory");
            Self(dir)
        }

        fn file(&self, name: &str) -> PathBuf {
            self.0.join(name)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).ok();
        }
    }

    #[test]
    fn a_file_that_has_not_moved_is_not_read_again() {
        // The regression `Live`'s stamp exists for: re-reading on a spurious
        // event lands in the watch that asked for the read, so one `touch` of
        // `prose.toml` reloaded it on every tick for the life of the run.
        let scratch = Scratch::new();
        let path = scratch.file("still.toml");
        std::fs::write(&path, "one").expect("a file to watch");
        let mut live = Live::new(path);
        assert!(
            !live.moved(),
            "a file nobody touched asked to be read again",
        );
        assert!(!live.moved(), "and asked twice");
    }

    #[test]
    fn a_file_that_is_written_is_read_once() {
        // The other half: breaking the loop must not cost the reload itself.
        // Filesystem stamps here are nanoseconds, so a second write is a second
        // stamp without contriving a delay.
        let scratch = Scratch::new();
        let path = scratch.file("moved.toml");
        std::fs::write(&path, "one").expect("a file to watch");
        let mut live = Live::new(path.clone());
        std::fs::write(&path, "two").expect("a second write");
        assert!(live.moved(), "an edited file was not noticed");
        assert!(!live.moved(), "and was then noticed a second time");
    }

    #[test]
    fn a_file_that_is_not_there_never_reloads_and_can_still_arrive() {
        // A content directory with no `manual.toml` in it is ordinary — the
        // terminal build's own scratch dirs are like that — and must not report
        // a reload every tick. One that gains the file later must.
        let scratch = Scratch::new();
        let path = scratch.file("later.toml");
        let mut live = Live::new(path.clone());
        assert!(!live.moved(), "a file that does not exist claimed to move");
        std::fs::write(&path, "arrived").expect("the file arriving");
        assert!(live.moved(), "a file that arrived was not picked up");
    }
}
