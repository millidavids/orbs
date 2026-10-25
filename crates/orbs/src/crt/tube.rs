//! The tube's states, the key that cycles them, and what reaches the render
//! world each frame.

use bevy::prelude::*;
use bevy::render::Extract;

use super::settings::{CrtSettings, CrtUniform};
use crate::fullscreen::Extracted;

/// The tube's own pass, so another can be ordered after it.
///
/// A set, not the generic system, for `sight`, which must touch a pixel last
/// since the tube puts hue back (§14).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct CrtPass;

/// Step through tube states: the tuned default, §4's peak-threat state, and off.
///
/// Peak threat has to be *reachable* rather than merely defined: §4 requires the
/// worst-case legibility test to run against "maximum flicker and vignette
/// pulse", and off is §14's accessibility requirement.
pub(super) fn cycle(settings: Option<Single<&mut CrtSettings>>) {
    let Some(mut settings) = settings else {
        return;
    };
    let (next, name) = after(**settings);
    **settings = next;
    info!("crt: {name}");
    // `F3` and the tube page are one setting seen twice. Without this, turning
    // the tube off with the key and then opening the settings page would show it
    // on, and the next launch would bring it back. §4 asks for a sticky setting;
    // the key is how a player reaches it fastest.
    orbs_shell::settings::set(crate::shell::CRT, named(&next));
}

/// The tube state `current` steps to, and what to call it.
///
/// Split out so a test can assert F3 reaches off (§14).
fn after(current: CrtSettings) -> (CrtSettings, &'static str) {
    if current == CrtSettings::DEFAULT {
        (CrtSettings::PEAK_THREAT, "peak threat")
    } else if current == CrtSettings::PEAK_THREAT {
        (CrtSettings::OFF, "off")
    } else {
        (CrtSettings::DEFAULT, "default")
    }
}

/// The three states `F3` reaches, by the names `ORBS_CRT` takes.
///
/// One table, so a name and the state it selects cannot drift apart.
pub(crate) const STATES: [(&str, CrtSettings); 3] = [
    ("default", CrtSettings::DEFAULT),
    ("peak", CrtSettings::PEAK_THREAT),
    ("off", CrtSettings::OFF),
];

/// What this tube state is called.
///
/// The table `ORBS_CRT` and `F3` share. A hand-built state reports as the
/// first.
pub(crate) fn named(settings: &CrtSettings) -> &'static str {
    STATES
        .iter()
        .find(|(_, state)| state == settings)
        .map_or(STATES[0].0, |(name, _)| *name)
}

/// The tube state `word` names, whole.
pub(crate) fn state_named(word: &str) -> Option<CrtSettings> {
    STATES
        .iter()
        .find(|(name, _)| *name == word)
        .map(|(_, state)| *state)
}

/// Which tube state to open with, from `ORBS_CRT`.
///
/// For See-it lines, since `ORBS_CAPTURE` presses no keys.
pub(crate) fn seeded() -> CrtSettings {
    chosen(orbs_shell::settings::seeded("ORBS_CRT", crate::shell::CRT).as_deref())
}

/// The rule [`seeded`] applies, without the environment.
///
/// Split out to test without the process-global environment, as
/// `save::chosen`; it agrees with `sight::chosen`.
fn chosen(value: Option<&str>) -> CrtSettings {
    // An exported-but-empty variable is *unset*, not a typo. `ORBS_CRT= orbs`
    // is the shell idiom for neutralising one, and a loop variable that came out
    // empty is the same thing arriving by accident; warning on either is noise
    // on every launch. `save::chosen` filters blanks for exactly this reason.
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return CrtSettings::default();
    };
    let Some((_, state)) = STATES
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(value))
    else {
        warn!("ORBS_CRT: no such tube state {value:?} — using the default");
        return CrtSettings::default();
    };
    *state
}

/// Copy the settings across to the render world once per frame.
pub(super) fn extract(
    settings: Extract<Query<&CrtSettings>>,
    picture: Extract<Option<Res<super::Picture>>>,
    stillness: Extract<Option<Res<crate::shell::Stillness>>>,
    time: Extract<Res<Time>>,
    tube: Extract<Res<Tube>>,
    mut commands: Commands,
) {
    let Some(settings) = settings.iter().next() else {
        return;
    };
    let still = stillness
        .as_deref()
        .is_some_and(|stillness| stillness.still());
    commands.insert_resource(Extracted(CrtUniform::new(
        super::effective::effective(
            *settings,
            picture.as_deref().copied().unwrap_or(super::Picture::FULL),
            still,
        ),
        time.elapsed_secs(),
        **tube,
    )));
}

/// The tube's geometry in the window, published by the renderer for the CRT.
///
/// The tube is the picture, not the window: periodic terms divide the cell
/// size (§9), shaped ones are measured against [`fill`](Self::fill_x) (§19).
#[derive(Resource, Debug, Clone, Copy)]
pub struct Tube {
    /// Physical pixels per cell, horizontally and vertically.
    pub width: f32,
    pub height: f32,
    /// The picture's share of the window on each axis, in `(0, 1]`.
    ///
    /// One of the two is always 1.0 — the picture fits inside on both axes and
    /// touches on at least one — and the other is what the bars eat.
    pub fill_x: f32,
    pub fill_y: f32,
    /// How far the picture's middle sits from the window's, as a share of the
    /// window, top-down. Zero but for a whole fit on an odd window, which
    /// `Screen::middle` moves half a pixel onto a whole one.
    pub shift_x: f32,
    pub shift_y: f32,
}

impl Default for Tube {
    /// A window exactly the size of the picture: native cells, no bars.
    fn default() -> Self {
        Self {
            width: f32::from(orbs_render::CELL_WIDTH),
            height: f32::from(orbs_render::CELL_HEIGHT),
            fill_x: 1.0,
            fill_y: 1.0,
            shift_x: 0.0,
            shift_y: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pressing_f3_reaches_off_and_comes_back() {
        // §14 requires the tube be disableable. That is a property of the
        // *cycle*, not of `CrtSettings::OFF` existing — a key that never arrives
        // at off is the same as no key at all.
        let mut state = CrtSettings::DEFAULT;
        let mut seen = Vec::new();
        for _ in 0..3 {
            let (next, name) = after(state);
            state = next;
            seen.push(name);
        }
        assert_eq!(seen, ["peak threat", "off", "default"]);
        assert_eq!(state, CrtSettings::DEFAULT, "the cycle did not close");
    }

    #[test]
    fn an_unrecognised_state_lands_somewhere_the_player_can_see() {
        // The fallback arm. Something world-driven will eventually leave the
        // settings equal to no preset — §4 wires vignette to threat and flash to
        // breach — and the wrong answer would be to leave the key doing nothing.
        let odd = CrtSettings {
            vignette: 0.61,
            ..CrtSettings::DEFAULT
        };
        assert_eq!(after(odd).0, CrtSettings::DEFAULT);
    }

    /// `ORBS_CRT` and `ORBS_SIGHT` must agree about case and about blanks.
    ///
    /// Both case-insensitive.
    #[test]
    fn a_tube_state_is_named_the_way_a_sight_is() {
        for (name, state) in STATES {
            assert_eq!(chosen(Some(name)), state);
            assert_eq!(chosen(Some(&name.to_uppercase())), state);
            assert_eq!(chosen(Some(&format!("  {name}  "))), state);
        }
    }

    /// An exported-but-empty variable is unset, and must not warn.
    #[test]
    fn a_blank_reads_as_unset_rather_than_as_a_typo() {
        assert_eq!(chosen(None), CrtSettings::default());
        assert_eq!(chosen(Some("")), CrtSettings::default());
        assert_eq!(chosen(Some("   ")), CrtSettings::default());
    }

    #[test]
    fn a_typo_falls_back_to_the_default_tube() {
        assert_eq!(chosen(Some("of")), CrtSettings::default());
        assert_eq!(chosen(Some("peak threat")), CrtSettings::default());
    }
}
