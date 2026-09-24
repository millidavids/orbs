//! The waveform. Every sound in the game is made here, from numbers.
//!
//! Generated, never sampled, which is a licence decision: §13 permits either,
//! and a cue made in code has no provenance question, no file to ship and no
//! third party to credit (`assets/README.md`). It also matches §4 — a terminal
//! in a scrying orb makes electrical noises, not orchestral ones.
//!
//! A [`Cue`] is a handful of [`Tone`]s summed: a frequency, when it starts, how
//! long it runs, and how loud. Two tones a beat apart is a rise or a fall; one
//! tone eight milliseconds long is a click. The vocabulary is deliberately
//! small, so [`cues`](super::cues) reads as a table.
//!
//! The envelope is a raised cosine and has no parameters. A square gate on a
//! sine starts and ends mid-cycle, audible as a click at both edges — on a
//! short cue the clicks are most of what you hear.

use std::num::NonZero;
use std::time::Duration;

use bevy::audio::{ChannelCount, Decodable, Sample, SampleRate, Source};
use bevy::prelude::*;

/// Samples a second. CD rate, because `cpal` will not have to resample it.
pub(crate) const RATE: u32 = 44_100;

/// The same rate, as a float.
///
/// Written out rather than cast, so no arithmetic here needs an `#[expect]` for
/// a conversion that is exact. `the_two_rates_agree` keeps the pair honest.
const PER_SECOND: f32 = 44_100.0;

/// One partial of a cue.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Tone {
    /// Hertz.
    pub(crate) hz: f32,
    /// Seconds from the start of the cue.
    pub(crate) at: f32,
    /// Seconds long.
    pub(crate) secs: f32,
    /// Peak amplitude, 0 to 1, before the envelope.
    pub(crate) gain: f32,
}

impl Tone {
    /// A tone starting at the top of the cue.
    pub(crate) const fn new(hz: f32, secs: f32, gain: f32) -> Self {
        Self {
            hz,
            at: 0.0,
            secs,
            gain,
        }
    }

    /// The same tone, starting later.
    pub(crate) const fn after(mut self, at: f32) -> Self {
        self.at = at;
        self
    }
}

/// A sound the orb can make.
///
/// An asset rather than a component so that one cue is generated once and played
/// many times: `cues` builds the whole table at startup and keeps the handles.
#[derive(Asset, TypePath, Debug, Clone, PartialEq)]
pub(crate) struct Cue {
    /// The partials, summed.
    pub(crate) tones: Vec<Tone>,
    /// Whether this is a drone rather than a gesture.
    ///
    /// A cue is a gesture that rises out of silence and is gone, so the raised
    /// cosine stops both edges clicking. A bed is a *drone* — the sound of a
    /// machine being on, and being on has no shape.
    ///
    /// Shipped without this, the bed ran the cue envelope across its whole
    /// half-second lap: a 2 Hz tremolo at full depth for the length of the
    /// session, and no test could hear it. A drone needs no envelope because
    /// `LAP`'s whole-cycle rule already makes the waveform continuous across
    /// the loop point.
    pub(crate) sustained: bool,
}

impl Cue {
    /// A cue of these tones: a gesture, enveloped.
    pub(crate) fn of(tones: &[Tone]) -> Self {
        Self {
            tones: tones.to_vec(),
            sustained: false,
        }
    }

    /// A drone of these tones: steady, and meant to be looped.
    ///
    /// The caller owes the whole-cycle rule — see [`sustained`](Self::sustained)
    /// and `bed::LAP`.
    pub(crate) fn drone(tones: &[Tone]) -> Self {
        Self {
            tones: tones.to_vec(),
            sustained: true,
        }
    }

    /// How long the whole cue runs for, in seconds.
    ///
    /// The furthest edge of any tone, so a two-note cue is as long as its second
    /// note ends rather than as long as its first.
    pub(crate) fn secs(&self) -> f32 {
        self.tones
            .iter()
            .map(|tone| tone.at + tone.secs)
            .fold(0.0, f32::max)
    }

    /// The sample at `now` seconds, before clipping.
    ///
    /// Split out so it can be measured without a sound card — the only gate
    /// this file has that is not *listening*.
    fn at(&self, now: f32) -> f32 {
        let mut sample = 0.0;
        for tone in &self.tones {
            let into = now - tone.at;
            if into < 0.0 || into >= tone.secs || tone.secs <= 0.0 {
                continue;
            }
            // A raised cosine: nought at both edges, one in the middle. No
            // click, and nothing to tune. A drone has no envelope at all — see
            // `sustained`, and the tremolo that shipped without it.
            let shape = if self.sustained {
                1.0
            } else {
                (1.0 - (core::f32::consts::TAU * into / tone.secs).cos()) * 0.5
            };
            sample += (core::f32::consts::TAU * tone.hz * now).sin() * shape * tone.gain;
        }
        sample
    }
}

/// A [`Cue`] being played: a stream of samples.
pub(crate) struct Ringing {
    cue: Cue,
    /// Samples emitted so far.
    at: u64,
    /// Samples the whole cue comes to.
    len: u64,
}

impl Iterator for Ringing {
    type Item = Sample;

    fn next(&mut self) -> Option<Sample> {
        if self.at >= self.len {
            return None;
        }
        #[expect(
            clippy::cast_precision_loss,
            reason = "a cue is under a second, so well inside f32's exact integer range"
        )]
        let now = self.at as f32 / PER_SECOND;
        self.at += 1;
        // Clipped rather than normalised: the table is authored so nothing sums
        // past one, and a cue that did should distort audibly rather than
        // quietly rescale every other cue around it.
        Some(self.cue.at(now).clamp(-1.0, 1.0))
    }
}

impl Source for Ringing {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> ChannelCount {
        // Mono. §4 is one screen on a desk, and a cue that came from the left
        // would be saying where something is — which the Frame does not.
        NonZero::new(1).expect("one channel is not zero")
    }

    fn sample_rate(&self) -> SampleRate {
        NonZero::new(RATE).expect("the rate is not zero")
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
            reason = "a cue is a fraction of a second; `secs` is a sum of positives"
        )]
        let len = (self.secs() * PER_SECOND) as u64;
        Ringing {
            cue: self.clone(),
            at: 0,
            len,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every sample of a cue, as a decoder produces them.
    fn rung(cue: &Cue) -> Vec<f32> {
        cue.decoder().collect()
    }

    #[test]
    fn the_two_rates_agree() {
        // Two spellings of one number (§19). The failure it prevents is every
        // cue playing at the wrong speed.
        assert!((f64::from(RATE) - f64::from(PER_SECOND)).abs() < 0.5);
    }

    #[test]
    fn a_cue_starts_and_ends_at_silence() {
        // The reason the envelope exists: a square gate on a sine steps from
        // nought to mid-cycle, a click at each end, and on an eight-millisecond
        // cue the clicks are most of what you hear.
        let cue = Cue::of(&[Tone::new(880.0, 0.08, 0.5)]);
        let samples = rung(&cue);
        assert!(!samples.is_empty(), "the cue produced no sound at all");
        assert!(
            samples[0].abs() < 0.01,
            "it begins with a step of {}",
            samples[0],
        );
        let last = samples[samples.len() - 1];
        assert!(last.abs() < 0.05, "it ends with a step of {last}");
    }

    #[test]
    fn a_cue_never_clips_by_accident() {
        // Clipping is deliberate in `next` and a bug in the table: a cue whose
        // tones sum past one reads as a broken sound card, not a loud cue.
        //
        // Measured before the clamp. Collecting from `Ringing::next` measured a
        // peak of 1.0 by construction, so the assertion could not fail however
        // loud the table got; `Cue::at` is the sum itself.
        for (name, cue) in super::super::cues::table() {
            let mut peak = 0.0_f32;
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "a cue is a fraction of a second; `secs` is a sum of positives"
            )]
            let steps = (cue.secs() * PER_SECOND) as usize;
            for step in 0..steps {
                #[expect(clippy::cast_precision_loss, reason = "under 2^24 samples")]
                let now = step as f32 / PER_SECOND;
                peak = peak.max(cue.at(now).abs());
            }
            assert!(
                peak <= 1.0,
                "the `{name}` cue peaks at {peak} before the clamp and will distort",
            );
        }
    }

    #[test]
    fn a_second_tone_sounds_after_the_first() {
        // Two tones a beat apart is the whole of how a rise differs from a fall,
        // so `after` has to actually delay one.
        let cue = Cue::of(&[
            Tone::new(440.0, 0.04, 0.5),
            Tone::new(660.0, 0.04, 0.5).after(0.04),
        ]);
        assert!(
            (cue.secs() - 0.08).abs() < 0.0001,
            "the cue is {} long, not 0.08",
            cue.secs(),
        );
        // Nothing of the second tone is audible while the first is at its peak.
        let early = cue.at(0.02);
        let late = cue.at(0.06);
        assert!(
            early.abs() > 0.0 && late.abs() > 0.0,
            "one of the two notes is silent: {early}, {late}",
        );
    }

    #[test]
    fn a_tone_with_no_length_is_silence_rather_than_a_panic() {
        // A guard rather than a hope: `secs` divides the envelope, and a zero
        // would be a NaN sample fed to the sound card.
        let cue = Cue::of(&[Tone::new(440.0, 0.0, 1.0)]);
        assert!(
            rung(&cue).iter().all(|sample| sample.is_finite()),
            "a zero-length tone produced something that is not a number",
        );
    }
}
