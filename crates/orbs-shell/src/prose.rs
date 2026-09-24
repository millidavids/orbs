//! Reading authored content off disk — the half with no watcher in it.
//!
//! CLAUDE.md rule 6 puts prose in data files so a writer can work without a
//! compiler. Finding and parsing that file is plain `std::fs`, and every
//! frontend needs it; keeping a *watch* on it is backend plumbing and lives in
//! the Bevy build's `sim::content`.
//!
//! `ORBS_DUMP` has always shared this path — it builds no `App` and therefore
//! cannot carry a watcher, but it is the tool CLAUDE.md says to reach for first,
//! so it has to see edited content or the rule-6 gate is only demonstrable
//! through a window.
//!
//! Opt-in, so a shipped build cannot depend on a path that is not there:
//! content is compiled in with `include_str!`, and this does nothing unless
//! `ORBS_CONTENT` names a directory:
//!
//! ```bash
//! ORBS_CONTENT=crates/orbs-sim/content cargo run -p orbs
//! ```
//!
//! Two files are live: `prose.toml` and `manual.toml`. The manual joined at
//! `0.16.8` for the reason rule 6 exists — `include_str!` alone does not satisfy
//! it, and a recompile per edit is the writing cost rule 6 was written to avoid.

use std::path::{Path, PathBuf};

use orbs_sim::{Prose, Sim, content::Manual};

/// Environment variable naming a directory of content files.
pub const CONTENT_DIR: &str = "ORBS_CONTENT";

/// The prose file's name within that directory.
pub const PROSE: &str = "prose.toml";

/// What [`load`] reports when `ORBS_CONTENT` does not name a directory at all.
///
/// Not a file name — it stands where one would, so a caller printing the list
/// says *"`ORBS_CONTENT` is set but the content directory did not load"*, which is
/// the sentence a mistyped path needs.
pub const DIRECTORY: &str = "the content directory";

/// The manual's file name within that directory.
///
/// A second live file, and the only one besides `prose.toml`. `recipes.toml` and
/// `fuel.toml` reach decisions, so swapping either mid-session would break
/// replay from `(seed, submissions)`; a chapter reaches nothing but a reader's
/// eyes. See `sim::content`'s watcher, which says the same to a writer.
pub const MANUAL: &str = "manual.toml";

/// Pour `ORBS_CONTENT` into a tower, if it names a directory.
///
/// The whole of rule 6's reach in one call, because there are two live files and
/// three places that load them by hand — the dump, the terminal build, and the
/// Bevy build's watcher. Two of the three take a `Sim` and use this; the watcher
/// keeps its own paths, since it re-reads without consulting the environment.
///
/// Silent when the variable is unset: content is compiled in, and a shipped
/// build must not depend on a path that is not there.
///
/// Returns the live files it could not read, so a caller with no `tracing`
/// subscriber can still say so. Not a nicety: a dump installs none, and without
/// the line a writer with malformed TOML sees their edit quietly not happen,
/// which looks exactly like the built-in fallback working.
#[must_use]
pub fn load(sim: &mut Sim) -> Vec<&'static str> {
    let mut missed = Vec::new();
    let Some(dir) = std::env::var_os(CONTENT_DIR).map(PathBuf::from) else {
        return missed;
    };
    // A directory that is not there is the thing most worth saying, and the
    // first attempt at the rule below silenced exactly that. A mistyped
    // `ORBS_CONTENT` is the likeliest mistake a writer makes, and it produced
    // nothing on stderr while the game drew the built-in text.
    if !dir.is_dir() {
        missed.push(DIRECTORY);
        return missed;
    }

    // A file that is not there is not a complaint. Both live files are optional:
    // a writer working on the prose keeps a directory with `prose.toml` and
    // nothing else, and reporting the absent manual every run is a warning that
    // trains people to stop reading warnings. A file that exists and will not
    // parse is what this was written for.
    let prose = dir.join(PROSE);
    if prose.exists() {
        match read(&prose) {
            Some(prose) => sim.set_prose(prose),
            None => missed.push(PROSE),
        }
    }
    let manual = dir.join(MANUAL);
    if manual.exists() {
        match read_manual(&manual) {
            Some(manual) => sim.set_manual(manual),
            None => missed.push(MANUAL),
        }
    }
    missed
}

/// Read and parse the manual file, or explain why not.
///
/// [`read`]'s pair, and mid-edit breakage is answered the same way: keep the
/// chapters already held rather than going blank while someone is looking.
pub fn read_manual(path: &Path) -> Option<Manual> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) => {
            tracing::warn!("content: cannot read {} ({error})", path.display());
            return None;
        }
    };
    match Manual::parse(&text) {
        Ok(manual) => Some(manual),
        Err(error) => {
            tracing::warn!(
                "content: {} is not valid ({error}); keeping the last good chapters",
                path.display()
            );
            None
        }
    }
}

/// Read and parse the prose file, or explain why not.
pub fn read(path: &Path) -> Option<Prose> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) => {
            tracing::warn!("content: cannot read {} ({error})", path.display());
            return None;
        }
    };
    match Prose::parse(&text) {
        Ok(prose) => Some(prose),
        Err(error) => {
            // Keep what we have. A writer mid-edit saves broken TOML constantly,
            // and answering that by falling back to silence would make the tower
            // go mute at the exact moment someone is looking at it.
            tracing::warn!(
                "content: {} is not valid ({error}); keeping the last good text",
                path.display()
            );
            None
        }
    }
}
