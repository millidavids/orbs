//! Reading authored content off disk — the half with no watcher in it.
//!
//! CLAUDE.md rule 6 puts prose in data files so a writer can work without a
//! compiler. Finding and parsing that file is plain `std::fs`, and every
//! frontend needs it; keeping a *watch* on it is backend plumbing and lives in
//! [`super::content`].
//!
//! `ORBS_DUMP` has always shared this path — it builds no `App` and therefore
//! cannot carry a watcher, but it is the tool CLAUDE.md says to reach for first,
//! so it has to see edited content or the rule-6 gate is only demonstrable
//! through a window.
//!
//! # Opt-in, so a shipped build cannot depend on a path that is not there
//!
//! Content is compiled into `orbs-sim` (`include_str!`), and this does nothing
//! at all unless **`ORBS_CONTENT`** names a directory:
//!
//! ```bash
//! ORBS_CONTENT=crates/orbs-sim/content cargo run -p orbs
//! ```

use std::path::{Path, PathBuf};

use orbs_sim::Prose;

/// Environment variable naming a directory of content files.
pub const CONTENT_DIR: &str = "ORBS_CONTENT";

/// The prose file's name within that directory.
pub const PROSE: &str = "prose.toml";

/// Read the content directory once, with no watcher.
pub fn load() -> Option<Prose> {
    let dir = std::env::var_os(CONTENT_DIR).map(PathBuf::from)?;
    read(&dir.join(PROSE))
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
