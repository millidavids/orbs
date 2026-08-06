//! Watching authored content, and handing it to the sim on a tick boundary.
//!
//! CLAUDE.md rule 6 puts prose in data files so a writer can work without a
//! compiler. This is the half that makes "hot-reloadable" true rather than
//! aspirational: edit `prose.toml` with the game running and the next tick
//! speaks the new line.
//!
//! # Why the watcher is here and not in `orbs-sim`
//!
//! Rule 8 forbids async in the sim, and rule 3 makes a frontend a *caller*
//! rather than a host. A watcher inside the sim would also make every headless
//! test touch the filesystem, which is the thing rule 1 exists to prevent.
//! `notify` runs its own background thread and posts over a channel; a system
//! drains that channel and calls
//! [`Sim::set_prose`](orbs_sim::Sim::set_prose) between ticks.
//!
//! # Opt-in, so a shipped build cannot depend on a path that is not there
//!
//! Content is compiled into `orbs-sim` (`include_str!`), and this module does
//! nothing at all unless **`ORBS_CONTENT`** names a directory:
//!
//! ```bash
//! ORBS_CONTENT=crates/orbs-sim/content cargo run -p orbs
//! ```
//!
//! Absent that, the game runs on the built-in text and starts no thread.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, channel};

use bevy::prelude::*;
use notify::{RecursiveMode, Watcher as _};
use orbs_sim::Prose;

use super::driver::Tower;

/// Environment variable naming a directory of content files.
pub(crate) const CONTENT_DIR: &str = "ORBS_CONTENT";

/// The prose file's name within that directory.
const PROSE: &str = "prose.toml";

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
    prose: PathBuf,
}

/// Read the content directory once, with no watcher.
///
/// Shared with `ORBS_DUMP`, which builds no `App` and therefore cannot carry a
/// watcher — but is the tool CLAUDE.md says to reach for first, so it has to see
/// edited content or the rule-6 gate is only demonstrable through a window.
pub(crate) fn load() -> Option<Prose> {
    let dir = std::env::var_os(CONTENT_DIR).map(PathBuf::from)?;
    read(&dir.join(PROSE))
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

    // Load once up front, so a run starts on what is on disk rather than
    // waiting for the first edit.
    if let Some(loaded) = read(&prose) {
        app.world_mut().resource_mut::<Tower>().set_prose(loaded);
        info!("content: loaded {}", prose.display());
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
    // **Only `prose.toml` is live**, though the whole directory is watched.
    // `recipes.toml` reaches decisions, so swapping it mid-session would break
    // replay from `(seed, submissions)` unless the content were versioned with
    // it (see `Sim::new`); `fuel.toml` is the same. Editing either produced a
    // "reloaded prose.toml" line and no behaviour change, which reads as the
    // reload having worked — so say which file is live, once, where a writer
    // will see it.
    // Announced **after** the watch takes, not before: saying "watching" and then
    // "cannot watch" two lines later leaves a writer with two contradictory
    // claims and no way to tell which one the run is living under.
    if let Err(error) = watcher.watch(&dir, RecursiveMode::NonRecursive) {
        warn!("content: cannot watch {} ({error})", dir.display());
        return;
    }
    info!("content: watching {} (prose.toml is live)", dir.display());

    app.insert_resource(ContentWatch {
        _watcher: watcher,
        changes: Mutex::new(changes),
        prose,
    })
    // **Before `advance`, explicitly.** Both take `ResMut<Tower>` in
    // `FixedUpdate` with no constraint between them, which is an ambiguity rather
    // than an ordering: Bevy makes no promise about systems whose access
    // conflicts and which nothing separates. `reload`'s own doc claims it "lands
    // on a tick boundary", and only this edge makes that true — otherwise whether
    // swapped prose is visible to tick N or N+1 is up to the executor.
    .add_systems(
        FixedUpdate,
        reload
            .before(super::driver::advance)
            .run_if(resource_exists::<ContentWatch>),
    );
}

/// Read and parse the prose file, or explain why not.
fn read(path: &Path) -> Option<Prose> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) => {
            warn!("content: cannot read {} ({error})", path.display());
            return None;
        }
    };
    match Prose::parse(&text) {
        Ok(prose) => Some(prose),
        Err(error) => {
            // Keep what we have. A writer mid-edit saves broken TOML constantly,
            // and answering that by falling back to silence would make the tower
            // go mute at the exact moment someone is looking at it.
            warn!(
                "content: {} is not valid ({error}); keeping the last good text",
                path.display()
            );
            None
        }
    }
}

/// Apply any pending edit, between ticks.
///
/// In `FixedUpdate` alongside `advance`, so a reload lands on a **tick
/// boundary** and never part-way through a schedule.
fn reload(mut tower: ResMut<Tower>, watch: Res<ContentWatch>) {
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
    if let Some(prose) = read(&watch.prose) {
        tower.set_prose(prose);
        info!("content: reloaded {}", watch.prose.display());
    }
}
