//! The rows' words, and the two settings held only so their rows tell the
//! truth.

use bevy::prelude::*;

use orbs_shell::settings;

/// The word the tube's own state is set by.
pub(crate) const CRT: &str = "crt";
/// The word the phosphor is set by.
pub(crate) const THEME: &str = "theme";
/// The word the cue volume is set by.
pub(super) const VOICE: &str = crate::sound::VOICE;
/// One louder than `full`.
pub(super) const ELEVEN: &str = "eleven";

/// Whether §4's sticky boot skip is on.
///
/// A resource, or the rows read the file every frame. `Boot::default` reads it
/// once, at startup.
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct Skipping(pub(super) bool);

impl Skipping {
    /// Whether the boot sequence is skipped.
    pub(crate) const fn on(self) -> bool {
        self.0
    }
}

impl Default for Skipping {
    fn default() -> Self {
        Self(settings::skips_boot())
    }
}

/// The name a new tower is raised with, if the player chose one.
///
/// Held, not read from the file at `new`, for [`Skipping`]'s reason: a choice
/// the file could not take must still count for the rest of the session.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub(crate) struct Naming(pub(crate) Option<String>);

impl Default for Naming {
    fn default() -> Self {
        Self(settings::chosen_name())
    }
}
