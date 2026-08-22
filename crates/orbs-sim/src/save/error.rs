//! What can go wrong between a file and a world.
//!
//! Nothing here is fatal to a session. `orbs-shell` reports and carries on, the
//! same posture `shortcuts::export_trace` already takes for the parse trace:
//! *"a failed export must not take the session down with it — the tester whose
//! run it was recording is still playing."* A save that will not open costs the
//! player their tower; a save that will not open **and** takes the game down
//! with it costs them the tower and the explanation.

use thiserror::Error;

/// Why a save could not be written, read, or believed.
#[derive(Debug, Error)]
pub enum SaveError {
    /// The document could not be rendered as TOML.
    ///
    /// A bug rather than anything a world can reach: every field is a scalar, a
    /// string, or a sequence of them.
    #[error("the save could not be written: {reason}")]
    Unwritable {
        /// What `toml` said.
        reason: String,
    },

    /// The text is not the TOML this writes.
    #[error("the save could not be read: {reason}")]
    Malformed {
        /// What `toml` said.
        reason: String,
    },

    /// It came from a build whose format has moved on.
    ///
    /// **Refused rather than half-read.** An older build meeting a newer save
    /// cannot know which fields changed meaning, and a tower loaded from a
    /// format it does not understand is worse than no tower — it looks like a
    /// working game that is quietly wrong.
    #[error(
        "the save is from a later version of the game (format {found}, this build reads {understood})"
    )]
    Ahead {
        /// The format the file claims.
        found: u32,
        /// The format this build knows.
        understood: u32,
    },
}
