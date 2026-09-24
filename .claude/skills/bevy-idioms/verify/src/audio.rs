//! Procedural audio: a `Decodable` asset generated in code, not loaded.
//!
//! O.R.B.S. ships no sampled audio (DESIGN.md §19, `assets/README.md`: every
//! asset carries a `PROVENANCE.md`, and a cue generated in code has no licence
//! question at all). That makes the custom-`Decodable` path load-bearing rather
//! than exotic, and CLAUDE.md forbids recalling Bevy syntax — so it is verified
//! here first.
//!
//! What 0.19 actually wants, confirmed by this file compiling:
//!
//! - `rodio` 0.22 is what `bevy_audio` re-exports. `Sample` is `f32`,
//!   `SampleRate` is `NonZero<u32>` and `ChannelCount` is `NonZero<u16>` — all
//!   three were plain integers in older rodio, which is the trap.
//! - `Source` requires four methods: `current_span_len`, `channels`,
//!   `sample_rate`, `total_duration`.
//! - `Decodable::Decoder` must be `Source + Send + Iterator<Item = Sample>`.
//! - `app.add_audio_source::<T>()` needs the `AddAudioSource` trait in scope,
//!   and both `AudioPlugin` and `AssetPlugin` registered first.
//! - `AudioPlayer::new` is **hard-coded to `AudioSource`**. A custom type is
//!   constructed with tuple syntax: `AudioPlayer(handle)`.

use std::num::NonZero;
use std::time::Duration;

use bevy::audio::{AddAudioSource, ChannelCount, Decodable, Sample, SampleRate, Source};
use bevy::prelude::*;

/// How many samples a second the cues are generated at.
const RATE: u32 = 44_100;

/// One partial of a cue.
#[derive(Debug, Clone, Copy)]
pub struct Tone {
    /// Hertz.
    pub hz: f32,
    /// Seconds from the start of the cue.
    pub at: f32,
    /// Seconds long.
    pub secs: f32,
    /// Peak amplitude, 0..1.
    pub gain: f32,
}

/// A cue, generated rather than sampled.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct Cue {
    /// The partials, summed.
    pub tones: Vec<Tone>,
}

impl Cue {
    /// How long the whole cue runs for.
    fn secs(&self) -> f32 {
        self.tones
            .iter()
            .map(|tone| tone.at + tone.secs)
            .fold(0.0, f32::max)
    }
}

/// [`Cue`] as a stream of samples.
pub struct Ringing {
    cue: Cue,
    /// Samples emitted so far.
    at: u64,
    /// Total samples the cue comes to.
    len: u64,
}

impl Iterator for Ringing {
    type Item = Sample;

    fn next(&mut self) -> Option<Sample> {
        if self.at >= self.len {
            return None;
        }
        #[expect(clippy::cast_precision_loss, reason = "a cue is well under 2^24 samples")]
        let now = self.at as f32 / RATE as f32;
        self.at += 1;
        let mut sample = 0.0;
        for tone in &self.cue.tones {
            let into = now - tone.at;
            if into < 0.0 || into >= tone.secs {
                continue;
            }
            // A raised-cosine envelope: no click at either edge, and no
            // parameters to tune.
            let shape = (1.0 - (core::f32::consts::TAU * into / tone.secs).cos()) * 0.5;
            sample += (core::f32::consts::TAU * tone.hz * now).sin() * shape * tone.gain;
        }
        Some(sample.clamp(-1.0, 1.0))
    }
}

impl Source for Ringing {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> ChannelCount {
        NonZero::new(1).expect("one channel")
    }

    fn sample_rate(&self) -> SampleRate {
        NonZero::new(RATE).expect("a non-zero rate")
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f32(self.cue.secs()))
    }
}

impl Decodable for Cue {
    type Decoder = Ringing;

    fn decoder(&self) -> Ringing {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a cue is a fraction of a second"
        )]
        let len = (self.secs() * RATE as f32) as u64;
        Ringing {
            cue: self.clone(),
            at: 0,
            len,
        }
    }
}

/// Registration, and playing one.
pub fn plugin(app: &mut App) {
    app.add_audio_source::<Cue>()
        .add_systems(Startup, ring_once);
}

fn ring_once(mut commands: Commands, mut cues: ResMut<Assets<Cue>>) {
    let handle = cues.add(Cue {
        tones: vec![Tone {
            hz: 880.0,
            at: 0.0,
            secs: 0.08,
            gain: 0.4,
        }],
    });
    // Tuple syntax, not `AudioPlayer::new` — that constructor is hard-coded to
    // `AudioSource`.
    commands.spawn((
        AudioPlayer(handle),
        PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(0.5)),
    ));
}

/// The global volume a settings page would write to.
pub fn set_volume(mut global: ResMut<GlobalVolume>) {
    global.volume = bevy::audio::Volume::Linear(0.25);
}
