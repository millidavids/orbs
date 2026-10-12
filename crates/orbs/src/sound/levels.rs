//! How loud the orb is, and where that is remembered.
//!
//! A word, not a number, because the menu is typed and has no slider — a volume
//! is one of [`orbs_shell::settings::LEVELS`]. `values.rs` carries that
//! argument; what belongs here is that the mapping from word to gain lives in
//! `orbs-shell` too, so a terminal build gaining audio cannot pick different
//! numbers for the same words.

use bevy::prelude::*;
use orbs_shell::settings::{LEVELS, loudness};

/// The key the cue volume is kept under.
pub(crate) const VOICE: &str = "voice";

/// The level a setting arrives at when nobody has chosen.
const DEFAULT: &str = "full";

/// How loud the orb is.
///
/// A resource, read by the settings page like everything else: `setting`'s
/// rule is that a row shows what is in effect, asked of the thing that holds it
/// and never of the file. This is the thing that holds it.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Levels {
    /// The cues.
    pub(crate) voice: &'static str,
}

impl Levels {
    /// The level remembered under `key`, or the default.
    ///
    /// Resolved against [`LEVELS`] rather than trusted, so a settings file from
    /// a later build naming a level this one has never heard of falls back
    /// rather than producing a gain of nought and a silently mute game.
    /// `Theme::default` answers a stranger phosphor the same way.
    fn chosen(key: &str) -> &'static str {
        orbs_shell::settings::get(key)
            .and_then(|word| LEVELS.into_iter().find(|level| *level == word))
            .unwrap_or(DEFAULT)
    }

    /// How loud a cue should be.
    pub(crate) fn voice_gain(&self) -> f32 {
        loudness(self.voice)
    }

    /// Set the voice to `value`, if it is a level, and say whether it moved.
    pub(crate) fn set(&mut self, value: &str) -> bool {
        let Some(level) = LEVELS.into_iter().find(|level| *level == value) else {
            return false;
        };
        let moved = self.voice != level;
        self.voice = level;
        moved
    }
}

impl Default for Levels {
    fn default() -> Self {
        Self {
            voice: Self::chosen(VOICE),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_level_this_build_has_never_heard_of_falls_back() {
        // A settings file written by a later build. The failure this prevents is
        // the worst kind: a gain of nought, a game with no sound, and a settings
        // page showing a word that explains it to nobody.
        let mut levels = Levels { voice: DEFAULT };
        assert!(!levels.set("cacophonous"));
        assert_eq!(levels.voice, DEFAULT);
        assert!(levels.voice_gain() > 0.0);
    }

    #[test]
    fn off_really_is_silence_and_full_really_is_one() {
        // The two ends have to be exact. A `full` at 0.98 is a rounding error
        // nobody can hear; an `off` at 0.01 is a player who asked for silence
        // and did not get it, which is the one they will notice.
        assert_eq!(Levels { voice: "off" }.voice_gain(), 0.0);
        assert_eq!(Levels { voice: "full" }.voice_gain(), 1.0);
    }

    #[test]
    fn every_level_is_louder_than_the_one_before_it() {
        // `LEVELS` is the order cycling visits, so a gain table out of order
        // would make `voice` step *down* on the way to `full`.
        let gains: Vec<f32> = LEVELS.into_iter().map(loudness).collect();
        for pair in gains.windows(2) {
            assert!(pair[1] > pair[0], "the levels are not in order: {gains:?}",);
        }
    }
}
