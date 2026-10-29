//! The glass answers the tower: the corners draw in as a siege turns against
//! the garrison, and colour drains a little while a spell is broken (§4, §19
//! *the tower speaks up*).
//!
//! Enrichment, and window-only. Read from the shell's `Panel`, which is what
//! the painters draw from, so it carries nothing the Frame does not (rule 2):
//! the rampart's two bands and the rail's `‼` already say both facts.

use bevy::prelude::*;
use orbs_shell::Panel;
use orbs_sim::tower::Mark;

use super::settings::CrtSettings;

/// How long either level takes to reach where the tower puts it: `away`'s
/// ramp, for `away`'s reason. A whole-field step is the hazard; a level that
/// eases is not.
const RAMP: f32 = 0.6;

/// How grey a broken spell turns the glass.
///
/// Under peak's quarter: a fault clears only by walking in (a latch, never a
/// timer, §19), so a room nobody visits would grey the tube for a session. A
/// nudge toward the one action that clears it, not a punishment.
const FAULT_GREY: f32 = 0.15;

/// The switch that poses a strain for a capture, `pressure,fault`.
const POSE: &str = "ORBS_STRAIN";

/// How hard the tower is pressed, and whether a spell is broken, each eased.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub(crate) struct Strain {
    /// `0.0` an even fight or none, `1.0` the enemy alone on the field.
    pressure: f32,
    /// `0.0` no fault anywhere, `1.0` one latched.
    fault: f32,
    /// Set by [`POSE`]: held where it was put, for a screenshot.
    posed: bool,
}

impl Default for Strain {
    fn default() -> Self {
        let posed = std::env::var(POSE).ok().and_then(|text| {
            let (pressure, fault) = text.split_once(',')?;
            Some((
                pressure.trim().parse::<f32>().ok()?,
                fault.trim().parse::<f32>().ok()?,
            ))
        });
        posed.map_or(
            Self {
                pressure: 0.0,
                fault: 0.0,
                posed: false,
            },
            |(pressure, fault)| Self::held(pressure, fault),
        )
    }
}

impl Strain {
    /// Held at `pressure` and `fault`, whatever the tower does: a pose.
    pub(crate) const fn held(pressure: f32, fault: f32) -> Self {
        Self {
            pressure: level(pressure),
            fault: level(fault),
            posed: true,
        }
    }

    /// Carry both levels `seconds` toward where the panel puts them.
    pub(crate) fn advance(&mut self, panel: &Panel, seconds: f32) {
        if self.posed {
            return;
        }
        let pressure = panel.rampart.as_ref().map_or(0.0, |rampart| {
            margin(rampart.enemy.vigour, rampart.garrison.vigour)
        });
        let fault = if panel
            .briefs
            .iter()
            .any(|brief| brief.mark == Some(Mark::Fault))
        {
            1.0
        } else {
            0.0
        };
        let step = seconds / RAMP;
        self.pressure = toward(self.pressure, pressure, step);
        self.fault = toward(self.fault, fault, step);
    }

    /// The tube as the tower strains it. A tube that is off stays off, and
    /// nothing goes past `PEAK_THREAT` or below the state chosen.
    pub(crate) fn apply(self, state: CrtSettings) -> CrtSettings {
        if !state.on {
            return state;
        }
        let peak = CrtSettings::PEAK_THREAT;
        let mut out = state;
        out.vignette = (peak.vignette - state.vignette)
            .max(0.0)
            .mul_add(self.pressure, state.vignette);
        out.desaturation = state.desaturation.max(FAULT_GREY * self.fault);
        out
    }
}

/// How far the enemy outweighs the garrison: nought for an even fight or
/// better, one for an enemy with nobody against it. A share would have put
/// every siege at a half from the moment it began.
fn margin(enemy: u32, garrison: u32) -> f32 {
    let total = enemy.saturating_add(garrison);
    if total == 0 {
        return 0.0;
    }
    #[allow(clippy::cast_precision_loss)]
    let lead = enemy.saturating_sub(garrison) as f32 / total as f32;
    lead.clamp(0.0, 1.0)
}

/// A posed level in the glass's range. Not a number is none: it would reach
/// the shader and black the tube.
const fn level(posed: f32) -> f32 {
    if posed.is_nan() {
        0.0
    } else {
        posed.clamp(0.0, 1.0)
    }
}

/// `from` moved `step` toward `to`, never past it.
fn toward(from: f32, to: f32, step: f32) -> f32 {
    if from < to {
        (from + step).min(to)
    } else {
        (from - step).max(to)
    }
}

/// Ease the strain toward the tower, every frame.
pub(super) fn notice(time: Res<Time>, panel: Res<Panel>, mut strain: ResMut<Strain>) {
    let mut next = *strain;
    next.advance(&panel, time.delta_secs());
    // Only on a change, as `away`'s does.
    strain.set_if_neq(next);
}

#[cfg(test)]
mod tests {
    use super::*;
    use orbs_render::{Rampart, SiegeSide};

    fn siege(enemy: u32, garrison: u32) -> Panel {
        let side = |vigour| SiegeSide {
            vigour,
            full: 30,
            ..SiegeSide::default()
        };
        Panel {
            rampart: Some(Rampart {
                enemy: side(enemy),
                garrison: side(garrison),
                ..Rampart::default()
            }),
            ..Panel::default()
        }
    }

    /// A glass at `level` on both channels.
    fn at(level: f32) -> Strain {
        Strain {
            pressure: level,
            fault: level,
            posed: false,
        }
    }

    fn still() -> Strain {
        at(0.0)
    }

    #[test]
    fn strain_ramps_and_never_steps() {
        let mut strain = still();
        let panel = siege(30, 0);
        let frame = 1.0 / 60.0;
        let mut last = strain.pressure;
        let mut frames = 0;
        while strain.pressure < 1.0 {
            strain.advance(&panel, frame);
            assert!(strain.pressure - last < 0.05, "a frame stepped the glass");
            last = strain.pressure;
            frames += 1;
            assert!(frames < 120, "the strain never settled");
        }
        assert!(frames >= 30, "strained in {frames} frames: a step");
    }

    #[test]
    fn an_even_fight_is_no_pressure_and_a_losing_one_rises() {
        assert!(margin(18, 18) < f32::EPSILON);
        assert!(margin(10, 18) < f32::EPSILON, "a winning fight strained");
        assert!(margin(27, 9) > 0.4);
        assert!((margin(30, 0) - 1.0).abs() < f32::EPSILON);
        assert!(margin(0, 0) < f32::EPSILON);
    }

    #[test]
    fn strain_never_exceeds_peak_threat_in_any_channel() {
        let full = at(1.0);
        let drawn = full.apply(CrtSettings::DEFAULT);
        let peak = CrtSettings::PEAK_THREAT;
        assert!(drawn.vignette <= peak.vignette + f32::EPSILON);
        assert!(drawn.desaturation <= peak.desaturation + f32::EPSILON);
        // ...and never takes the tube below what was chosen.
        let chosen = full.apply(peak);
        assert!(chosen.vignette >= peak.vignette - f32::EPSILON);
        assert!(chosen.desaturation >= peak.desaturation - f32::EPSILON);
    }

    #[test]
    fn a_tube_that_is_off_stays_off_under_strain() {
        assert_eq!(at(1.0).apply(CrtSettings::OFF), CrtSettings::OFF);
    }

    #[test]
    fn edge_off_takes_the_strained_vignette_and_the_fault_grey() {
        // One row for the rim and the glass's answer: `edge off` means the
        // glass never answers.
        use super::super::{Effect, Picture, Strength};
        let mut picture = Picture::FULL;
        picture.set(Effect::Edge, Strength::Off);
        let drawn =
            super::super::effective::effective(at(1.0).apply(CrtSettings::DEFAULT), picture, false);
        assert_eq!(drawn.vignette, CrtSettings::OFF.vignette);
        assert_eq!(drawn.desaturation, CrtSettings::OFF.desaturation);
    }

    #[test]
    fn a_pose_out_of_range_is_held_in_it() {
        let posed = Strain::held(f32::NAN, 7.0);
        assert!(
            posed.pressure.abs() < f32::EPSILON,
            "not a number reached the glass"
        );
        assert!((posed.fault - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn no_siege_no_fault_is_the_tube_as_chosen() {
        let mut strain = still();
        strain.advance(&Panel::default(), 1.0);
        assert_eq!(strain.apply(CrtSettings::DEFAULT), CrtSettings::DEFAULT);
    }
}
