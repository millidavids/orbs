//! How loud the orb is, and where that is remembered.
//!
//! Two levels, not one, because the reasons for turning them down differ and a
//! single volume forces a choice nobody should have to make: the hum is
//! atmosphere some people find tiring within a minute, where the cues are §14's
//! ambient channel and the part a screen-reader player is using. *Hum off, voice
//! full* has to be reachable.
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
/// The key the bed volume is kept under.
pub(crate) const HUM: &str = "hum";

/// The level a setting arrives at when nobody has chosen.
const DEFAULT: &str = "full";

/// How loud the orb is.
///
/// A resource, read by the settings page like everything else: `setting.rs`'s
/// rule is that a row shows what is in effect, asked of the thing that holds it
/// and never of the file. This is the thing that holds it.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Levels {
    /// The cues.
    pub(crate) voice: &'static str,
    /// The bed.
    pub(crate) hum: &'static str,
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

    /// How loud the bed should be.
    pub(crate) fn hum_gain(&self) -> f32 {
        loudness(self.hum)
    }

    /// Set one of the two, if `key` names one.
    ///
    /// Returns whether anything changed, which is what tells the bed to start
    /// again at the new volume.
    pub(crate) fn set(&mut self, key: &str, value: &str) -> bool {
        let Some(level) = LEVELS.into_iter().find(|level| *level == value) else {
            return false;
        };
        let slot = match key {
            VOICE => &mut self.voice,
            HUM => &mut self.hum,
            _ => return false,
        };
        let moved = *slot != level;
        *slot = level;
        moved
    }
}

impl Default for Levels {
    fn default() -> Self {
        Self {
            voice: Self::chosen(VOICE),
            hum: Self::chosen(HUM),
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
        let mut levels = Levels {
            voice: DEFAULT,
            hum: DEFAULT,
        };
        assert!(!levels.set(VOICE, "cacophonous"));
        assert_eq!(levels.voice, DEFAULT);
        assert!(levels.voice_gain() > 0.0);
    }

    #[test]
    fn off_really_is_silence_and_full_really_is_one() {
        // The two ends have to be exact. A `full` at 0.98 is a rounding error
        // nobody can hear; an `off` at 0.01 is a player who asked for silence
        // and did not get it, which is the one they will notice.
        let silent = Levels {
            voice: "off",
            hum: "off",
        };
        assert_eq!(silent.voice_gain(), 0.0);
        assert_eq!(silent.hum_gain(), 0.0);
        let loud = Levels {
            voice: "full",
            hum: "full",
        };
        assert_eq!(loud.voice_gain(), 1.0);
    }

    #[test]
    fn the_two_levels_move_independently() {
        // *Hum off, voice full* is the state this module is split for: the bed
        // is atmosphere and the cues are §14's ambient channel, and turning one
        // down must never reach the other.
        let mut levels = Levels {
            voice: DEFAULT,
            hum: DEFAULT,
        };
        assert!(levels.set(HUM, "off"));
        assert_eq!(levels.hum, "off");
        assert_eq!(levels.voice, DEFAULT, "silencing the hum silenced the cues");
        assert!(
            !levels.set(HUM, "off"),
            "an unchanged level claimed to move"
        );
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
