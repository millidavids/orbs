//! The hum: the sound of a machine being switched on.
//!
//! §4's premise is a single curved CRT glowing in the dark, and a CRT is never
//! silent — the flyback transformer whines, the mains hums under it. The bed is
//! that, and it is the difference between a game with sound effects and a game
//! that sounds like it is running.
//!
//! It says one thing, and the screen says it too. In the eldritch register (§3,
//! [`Presentation::Eldritch`]) the bed gains a detuned partial a tritone up, and
//! loses it when the register goes — rule 2, since the register is already on
//! screen and a player with the sound off has lost nothing.
//!
//! It is also the one thing in §14's ambient channel allowed to be continuous.
//! Everything else is a cue: short, and over.
//!
//! [`Presentation::Eldritch`]: orbs_render::Presentation

use bevy::audio::Volume;
use bevy::prelude::*;
use orbs_render::Presentation;

use super::ringing::Saying;
use super::synth::{Cue, Tone};
use crate::sim::Tower;

/// How long one lap of the bed is.
///
/// A whole number of cycles of every tone in it, or the loop point steps the
/// waveform and you hear a click once a second forever. 0.5s at 50 Hz is 25
/// cycles, at 100 Hz 50, at 70 Hz — the tritone — 35.
///
/// Load-bearing again: while the bed ran the cue envelope the amplitude was
/// nought at both edges anyway, so this reasoning was decoration over a 2 Hz
/// tremolo. A drone has no envelope, so continuity across the loop point is the
/// only thing between the player and a tick twice a second — see
/// [`Cue::sustained`](super::synth::Cue::sustained).
const LAP: f32 = 0.5;

/// Marks the entity the bed is playing on.
#[derive(Component)]
pub(super) struct Bed;

/// Which bed is playing.
#[derive(Resource, Debug, Default, PartialEq, Eq, Clone, Copy)]
pub(super) struct Humming {
    /// Whether the orb was last speaking in the eldritch register.
    eldritch: bool,
    /// How loud it was started at.
    ///
    /// Part of *which bed is playing* rather than a separate thing to watch:
    /// `PlaybackSettings` changes do not reach audio already playing, so turning
    /// the hum down means starting a different one.
    level: &'static str,
    /// Whether anything is playing yet.
    started: bool,
}

/// The bed, with or without the register on it.
fn hum(eldritch: bool) -> Cue {
    // 50 Hz and its octave: mains, and the whine over it. Quiet — this sits
    // under everything else and must never be what you notice.
    let mut tones = vec![Tone::new(50.0, LAP, 0.05), Tone::new(100.0, LAP, 0.025)];
    if eldritch {
        // A tritone over the fundamental, detuned. It is the interval that
        // refuses to resolve, which is what the register is for.
        tones.push(Tone::new(70.0, LAP, 0.035));
    }
    // A drone, not a cue: `Cue::of` would run the gesture envelope across the
    // whole lap and make this a 2 Hz tremolo. See `Cue::sustained`.
    Cue::drone(&tones)
}

/// Start the bed, and change it when the register does.
///
/// One entity, respawned rather than retuned: `PlaybackSettings` changes do not
/// affect audio already playing (Bevy's own doc says so) and a source generated
/// in code has no pitch control, so the register moving means a new handle and a
/// new player. That happens rarely.
pub(super) fn keep_humming(
    mut commands: Commands,
    tower: Res<Tower>,
    mut cues: ResMut<Assets<Cue>>,
    mut humming: ResMut<Humming>,
    saying: Res<Saying>,
    levels: Res<super::Levels>,
    playing: Query<Entity, With<Bed>>,
) {
    let eldritch = tower.sim().scrollback().records().register() == Presentation::Eldritch;
    if humming.started && humming.eldritch == eldritch && humming.level == levels.hum {
        return;
    }
    // The bed proves the audio path is open at all, and an idle launch makes no
    // cues — so `ORBS_SOUND` says this too, or the switch answers nothing about
    // a game nobody has typed at.
    if saying.aloud() {
        eprintln!(
            "sound: hum ({}, {})",
            if eldritch { "eldritch" } else { "plain" },
            levels.hum,
        );
    }
    for entity in &playing {
        commands.entity(entity).despawn();
    }
    let gain = levels.hum_gain();
    // `off` leaves nothing playing at all, rather than a silent loop the mixer
    // keeps decoding for the life of the session.
    if gain > 0.0 {
        let handle = cues.add(hum(eldritch));
        commands.spawn((
            Bed,
            AudioPlayer(handle),
            PlaybackSettings::LOOP.with_volume(Volume::Linear(gain)),
        ));
    }
    *humming = Humming {
        eldritch,
        level: levels.hum,
        started: true,
    };
}

/// Stop the bed. The orb is being put down.
pub(super) fn quieten(mut commands: Commands, playing: Query<Entity, With<Bed>>) {
    for entity in &playing {
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_tone_in_the_bed_loops_without_a_click() {
        // The one thing a looped generated source can get wrong, inaudible in a
        // test and unbearable in a room: a lap that is not a whole number of
        // cycles steps the waveform twice a second for as long as the game is
        // open.
        for eldritch in [false, true] {
            for tone in hum(eldritch).tones {
                let cycles = tone.hz * LAP;
                assert!(
                    (cycles - cycles.round()).abs() < 0.0001,
                    "{} Hz is {cycles} cycles per lap and will click",
                    tone.hz,
                );
            }
        }
    }

    #[test]
    fn the_bed_is_steady_rather_than_a_tremolo() {
        // What shipped instead of a hum: the cue envelope across a half-second
        // lap swelled the bed from silence to full and back, twice a second, for
        // the session. Neither neighbouring test could hear it — one counts
        // cycles, the other compares gains — so this asserts on the samples.
        let rung: Vec<f32> = hum(false).decoder().collect();
        assert!(rung.len() > 1_000, "the bed produced almost nothing");
        let slice = rung.len() / 10;
        let peaks: Vec<f32> = rung
            .chunks(slice)
            .map(|chunk| chunk.iter().fold(0.0_f32, |big, s| big.max(s.abs())))
            .collect();
        let quietest = peaks.iter().copied().fold(f32::INFINITY, f32::min);
        let loudest = peaks.iter().copied().fold(0.0_f32, f32::max);
        assert!(
            quietest > loudest * 0.8,
            "the bed swells: peaks range {quietest} to {loudest} across one lap",
        );
    }

    #[test]
    fn the_bed_is_quieter_than_every_cue() {
        // §14 wants an ambient channel that does not compete with speech, and a
        // bed you can hear over a completion tone is not ambient.
        let loudest_bed = hum(true)
            .tones
            .iter()
            .map(|tone| tone.gain)
            .fold(0.0_f32, f32::max);
        for voice in super::super::cues::Voice::ALL {
            let quietest = voice
                .cue()
                .tones
                .iter()
                .map(|tone| tone.gain)
                .fold(f32::INFINITY, f32::min);
            assert!(
                loudest_bed < quietest,
                "the bed at {loudest_bed} drowns the `{}` cue at {quietest}",
                voice.name(),
            );
        }
    }

    #[test]
    fn the_register_changes_the_bed() {
        // Rule 2's other half: the enrichment has to actually track the thing
        // in the Frame, or it is decoration claiming to be a channel.
        assert_ne!(
            hum(false),
            hum(true),
            "the eldritch register sounds like the plain one",
        );
    }
}
