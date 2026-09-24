//! What can go wrong between a file and a world.
//!
//! Nothing here is fatal to a session — `orbs-shell` reports and carries on,
//! `shortcuts::export_trace`'s posture. A save that will not open costs the
//! player their tower; one that also takes the game down costs them the
//! explanation too.

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
    /// Refused rather than half-read: an older build cannot know which fields
    /// changed meaning, and a tower loaded from a format it does not understand
    /// looks like a working game that is quietly wrong.
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
    /// Not symmetric with [`Ahead`](Self::Ahead): this build knows exactly what
    /// changed and cannot undo it. The lens rework is why — `WardSave` lost
    /// seven fields and a recorded reading's answers came from rules that no
    /// longer exist, so there is nothing to migrate them *to*.
    ///
    /// Refused rather than half-read: serde ignores the fields it no longer
    /// has, so the alternative is a tower that loads, looks right, and resumes
    /// a puzzle scored under different rules.
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
