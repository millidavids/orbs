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

    /// It came from a build whose format this one has moved on *from*.
    ///
    /// **The other direction, and it is not symmetric.** [`Ahead`](Self::Ahead)
    /// is a build that cannot know what changed; this is one that knows exactly
    /// what changed and cannot undo it. The lens rework is why it exists:
    /// `WardSave` lost seven fields and `shift` changed vocabulary — a recorded
    /// reading's answers were produced by rules that no longer exist, so there
    /// is nothing to migrate them *to*.
    ///
    /// Refused rather than half-read, for [`Ahead`](Self::Ahead)'s reason: serde
    /// ignores the fields it no longer has, so the alternative is a tower that
    /// loads, looks right, and resumes a puzzle whose answers were scored under
    /// different rules.
    #[error(
        "the save is from an earlier version of the game (format {found}, this build reads {understood})"
    )]
    Behind {
        /// The format the file claims.
        found: u32,
        /// The format this build knows.
        understood: u32,
    },
}
