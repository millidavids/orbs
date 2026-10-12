//! The rows' words, and the two settings held only so their rows tell the
//! truth.

use bevy::prelude::*;

use orbs_shell::settings;

/// The word the tube's own state is set by.
pub(crate) const CRT: &str = "crt";
/// The word the phosphor is set by.
pub(crate) const THEME: &str = "theme";
/// The word §14's hue accommodation is set by.
pub(crate) const SIGHT: &str = "sight";
/// The word the linear stream is set by.
///
/// `orbs-shell`'s, because both builds bind `F5` and both must write the same
/// key.
pub(super) const LINEAR: &str = orbs_shell::LINEAR_SETTING;
/// The word the parse driver is set by — see `Row::key` for why it is not
/// `driver`, which is what the file says.
pub(super) const READING: &str = "reading";
/// The word §9's focus mode is set by.
///
/// `orbs-shell`'s, because both builds bind `F4` and `Screen::default` reads it
/// for both.
pub(crate) const FOCUS: &str = settings::FOCUS;
/// The word the cue volume is set by.
pub(super) const VOICE: &str = crate::sound::VOICE;
/// One louder than `full`.
pub(super) const ELEVEN: &str = "eleven";

/// Whether §4's sticky boot skip is on.
///
/// The one setting that had no resource, which cost twice. Reading the *file*
/// through `settings::skips_boot()` meant a blocking read and a TOML parse per
/// frame — `follow_the_keys` rebuilds rows while the menu is open, and at the
/// threshold it cannot be closed, so the game parsed `orbs-settings.toml` sixty
/// times a second, invisibly to every instrument. It also meant a silent no-op
/// wherever the write did not land: the row flipped, nothing applied it, and the
/// next rebuild snapped it back without complaint.
///
/// `Boot::default` still reads the file, once, at startup. Nothing else does.
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
