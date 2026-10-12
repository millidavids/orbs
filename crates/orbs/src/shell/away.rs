//! `away` — whether the glass clouds while the window is not the one you are
//! using.
//!
//! The tower keeps working: this is how the orb looks, never what it does, so a
//! window in the background is still an idle game running. The cues sound as
//! ever — §14's ambient channel, and for a backgrounded idle game a chime is the
//! one way a finished brew reaches you.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use orbs_shell::settings::Pair;

/// How long the glass takes to cloud, or to clear.
///
/// A ramp rather than a cut: a whole-field step in brightness on every
/// alt-tab is the hazard §19 names, and half a second is long enough to read
/// as the orb noticing.
const RAMP: f32 = 0.6;

/// How dark a clouded glass goes, as a share of its brightness.
const CLOUDED: f32 = 0.45;

/// Whether the glass clouds while the player looks away. Anything but `dim`
/// is off: nothing changes.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Away(bool);

impl From<bool> for Away {
    fn from(dims: bool) -> Self {
        Self(dims)
    }
}

impl Pair for Away {
    const KEY: &'static str = "away";
    const WORDS: [&'static str; 2] = ["off", "dim"];

    fn raised(self) -> bool {
        self.0
    }
}

impl Default for Away {
    fn default() -> Self {
        Self::remembered()
    }
}

/// Whether the player has looked away, and how far the glass has clouded.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Default)]
pub(crate) struct Absence {
    /// How clouded the glass is, `0.0` clear to `1.0` as dark as it goes.
    clouded: f32,
}

impl Absence {
    /// Carry the cloud `seconds` toward where `away` and `chose` put it.
    pub(crate) fn advance(&mut self, away: bool, chose: Away, seconds: f32) {
        let target = if away && chose.raised() { 1.0 } else { 0.0 };
        let step = seconds / RAMP;
        self.clouded = if self.clouded < target {
            (self.clouded + step).min(target)
        } else {
            (self.clouded - step).max(target)
        };
    }

    /// How bright the glass is drawn, `1.0` for as authored.
    pub(crate) fn brightness(self) -> f32 {
        CLOUDED.mul_add(-self.clouded, 1.0)
    }
}

/// Notice whether the window has the keyboard, every frame.
///
/// The window's own `focused` rather than the focus message: `input.rs` records
/// why a focus hook was never going to be enough, and a missed focus-in would
/// leave the orb clouded with the player looking straight at it.
pub(crate) fn notice(
    time: Res<Time>,
    window: Option<Single<&Window, With<PrimaryWindow>>>,
    chose: Res<Away>,
    mut absence: ResMut<Absence>,
) {
    let away = window.is_some_and(|window| !window.focused);
    let mut next = *absence;
    next.advance(away, *chose, time.delta_secs());
    // Only on a change: the sight pass watches this.
    absence.set_if_neq(next);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_glass_clouds_by_a_ramp_and_never_a_step() {
        let mut absence = Absence::default();
        let frame = 1.0 / 60.0;
        let mut last = absence.brightness();
        let mut frames = 0;
        while absence.brightness() > 1.0 - CLOUDED + 0.001 {
            absence.advance(true, Away(true), frame);
            let now = absence.brightness();
            assert!(
                last - now < 0.05,
                "a frame darkened the glass by {}",
                last - now
            );
            last = now;
            frames += 1;
            assert!(frames < 120, "the cloud never settled");
        }
        assert!(frames >= 30, "the glass clouded in {frames} frames: a step");
        // ...and it clears the same way when the player comes back.
        absence.advance(false, Away(true), frame);
        assert!(absence.brightness() < 1.0, "it cleared in one frame");
    }

    #[test]
    fn off_never_clouds_and_an_unknown_word_is_off() {
        let mut absence = Absence::default();
        absence.advance(true, Away(false), 1.0);
        assert!(absence.brightness() >= 1.0, "off clouded the glass");
        assert_eq!(Away::named("vanish"), Away(false));
    }
}
