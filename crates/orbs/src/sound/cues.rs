//! What the orb sounds like, and what makes it make a sound.
//!
//! Rule 2 lets a frontend add enrichment the other cannot reproduce *"provided
//! it carries no information absent from the Frame"*. So a cue is chosen from
//! exactly what the marker glyph is chosen from: a record's `kind`, `role` and
//! `outcome`. [`Record::marker`] draws a character in front of the line, in the
//! same instant, out of the same three.
//!
//! `FieldName::Outcome` is an annotation and never printed as a labelled field,
//! but it is not invisible — `Record::marker` makes it a glyph and
//! `Record::style` an intensity. So a cue reading it says out loud what the
//! prompt already draws, and a player with the sound off has lost a channel and
//! not a fact. `every_cue_is_chosen_from_what_the_marker_is` keeps that true.
//!
//! [`Record::marker`]: orbs_render::Record::marker
//!
//! Only records the transcript draws. `Records::drawn` skips a spell's output
//! and anything quiet, both being *"in the log, not in the pane"* — and a cue
//! for a line nobody can see is information the Frame does not have. It is also
//! the difference between six bound spells being atmospheric and unusable.
//!
//! The watermark is a sequence number because `Records::sequence` survives both
//! a clear and a save's truncated tail — the two things that would make a
//! length-based watermark fire the whole transcript at once.

use bevy::prelude::*;
use orbs_render::{Outcome, RecordKind, Role};

use super::synth::{Cue, Tone};
use crate::sim::Tower;

/// A sound the orb can make.
///
/// Small on purpose. Seven cues a player can learn apart is worth more than
/// twenty they cannot, and §14 wants an ambient channel that does not compete
/// with speech. Three of the seven are §14's own: `done`, `ill` and `wrong` are
/// the completion, the warning and the sabotage it asks to be distinct.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Voice {
    /// A key pressed. Carries nothing: every key sounds the same.
    Key,
    /// A line submitted.
    Enter,
    /// A duration action finishing. §14 announces completions, never progress.
    Done,
    /// Something went well.
    Good,
    /// Something went wrong, or was breached.
    Ill,
    /// A surface you inspected came back tampered with.
    ///
    /// §14's third tone, and not a variation on [`Ill`](Self::Ill): a breach is
    /// happening to you now, a sabotage tell is something you went looking for
    /// and found — an alarm against a diagnosis.
    Wrong,
    /// The orb did not understand, or could not.
    Baulk,
}

impl Voice {
    /// Every cue, in declaration order.
    pub(crate) const ALL: [Self; 7] = [
        Self::Key,
        Self::Enter,
        Self::Done,
        Self::Good,
        Self::Ill,
        Self::Wrong,
        Self::Baulk,
    ];

    /// The word this cue is known by, for a test's failure message.
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Key => "key",
            Self::Enter => "enter",
            Self::Done => "done",
            Self::Good => "good",
            Self::Ill => "ill",
            Self::Wrong => "wrong",
            Self::Baulk => "baulk",
        }
    }

    /// The waveform.
    ///
    /// Read it as a table: a cue is a few sine tones, and the only decisions are
    /// pitch, length and order.
    pub(crate) fn cue(self) -> Cue {
        match self {
            // A tick, not a beep. Eight milliseconds of something high, quiet
            // enough to disappear under typing at speed.
            Self::Key => Cue::of(&[Tone::new(2_100.0, 0.008, 0.10)]),
            // The same gesture an octave down and a little longer: a line
            // landing rather than a key moving.
            Self::Enter => Cue::of(&[Tone::new(1_050.0, 0.022, 0.16)]),
            // Two notes rising a fifth. The only cue that arrives while you are
            // looking somewhere else, so it is the one that has to carry.
            Self::Done => Cue::of(&[
                Tone::new(660.0, 0.07, 0.30),
                Tone::new(990.0, 0.10, 0.28).after(0.06),
            ]),
            // The same interval, shorter and brighter. A thing you did, rather
            // than a thing that finished.
            Self::Good => Cue::of(&[
                Tone::new(880.0, 0.05, 0.24),
                Tone::new(1_320.0, 0.07, 0.22).after(0.04),
            ]),
            // Two notes a semitone apart, together: a beat rather than a rise or
            // a fall, which is the shape of a surface that says one thing and is
            // another. Unmistakable against the other six, and not an alarm.
            Self::Wrong => Cue::of(&[Tone::new(440.0, 0.22, 0.20), Tone::new(466.0, 0.22, 0.20)]),
            // Two notes *falling*, and low. The only cue in the set below
            // 300 Hz, so it is the one you hear through anything.
            Self::Ill => Cue::of(&[
                Tone::new(300.0, 0.09, 0.34),
                Tone::new(200.0, 0.16, 0.32).after(0.08),
            ]),
            // One flat note, no movement at all. §6 forbids a bare error, and
            // this is the tonal form of that: a refusal is not an alarm.
            Self::Baulk => Cue::of(&[Tone::new(370.0, 0.10, 0.22)]),
        }
    }
}

/// Every cue, built once and kept.
///
/// An asset per cue rather than a fresh one per sound: a cue is a few hundred
/// bytes of `Tone` and generating it per keystroke would be allocation on the
/// input path for no gain.
#[derive(Resource, Debug, Default)]
pub(crate) struct Voices {
    /// Handles, in [`Voice::ALL`] order.
    handles: Vec<Handle<Cue>>,
}

impl Voices {
    /// The handle for a cue.
    pub(crate) fn handle(&self, voice: Voice) -> Option<Handle<Cue>> {
        let at = Voice::ALL.iter().position(|kind| *kind == voice)?;
        self.handles.get(at).cloned()
    }
}

/// Build every cue once, at startup.
pub(super) fn voice_the_orb(mut commands: Commands, mut cues: ResMut<Assets<Cue>>) {
    let handles = Voice::ALL.map(|voice| cues.add(voice.cue())).to_vec();
    commands.insert_resource(Voices { handles });
}

/// Which cue a record asks for, if any.
///
/// The whole mapping, reading only what the transcript draws. `kind` chooses
/// first because a completion is a completion whatever its role; `outcome`
/// separates the echo the orb could not resolve from the one it could; `role` is
/// the fall-back, and is the split the styling uses.
const fn voice_of(kind: RecordKind, role: Role, outcome: Option<Outcome>) -> Option<Voice> {
    match kind {
        RecordKind::Completion => Some(Voice::Done),
        RecordKind::Echo => match outcome {
            // §6: the orb says what it could not read, and offers a correction.
            // That is a refusal, not a failure.
            Some(Outcome::Unresolved | Outcome::Incomplete | Outcome::Candidate) => {
                Some(Voice::Baulk)
            }
            _ => None,
        },
        // A record the player typed is already answered by `Enter`, and a second
        // cue on the same keystroke reads as a stutter.
        RecordKind::Input => None,
        // A reading that came back wrong is the sabotage tell, which §14 wants
        // distinct from a warning. `verify`'s check and its sweep both push
        // exactly this — `Status`, `Danger`, `state: tampered` — and nothing
        // else does: a breach is a `Completion` that cost you something.
        RecordKind::Status if matches!(role, Role::Danger) => Some(Voice::Wrong),
        _ => match role {
            Role::Danger => Some(Voice::Ill),
            Role::Success => Some(Voice::Good),
            Role::Normal | Role::Cost => None,
        },
    }
}

/// Where the record stream had got to last time this looked.
///
/// `None` until the first look, which is not a formality: a restored save opens
/// with a tail of records in it, and a watermark starting at nought would play
/// somebody's whole last session at them on load.
#[derive(Resource, Debug, Default)]
pub(crate) struct Heard(Option<u64>);

/// Ring a cue for anything new in the transcript.
pub(super) fn ring_for_records(
    tower: Res<Tower>,
    mut heard: ResMut<Heard>,
    mut ringing: MessageWriter<super::RingMessage>,
) {
    let records = tower.sim().scrollback().records();
    let now = records.sequence();
    let Some(last) = heard.0 else {
        heard.0 = Some(now);
        return;
    };
    heard.0 = Some(now);
    if now <= last {
        // A swap, or a restore that rewound the count. Take the new mark and
        // say nothing: the tower that just arrived has not done anything yet.
        return;
    }
    let mut asked: Vec<Voice> = records
        .since(last)
        .filter_map(|record| voice_of(record.kind(), record.role(), record.outcome()))
        .collect();

    // A cap, and not only insurance: `Heard` resets on a swap, which is what
    // fixed the restored tail ringing at once, but any other discontinuity would
    // do the same and offline catch-up (~29k steps) is planned. Beyond that, a
    // fistful of cues in one frame is a noise rather than a chord.
    //
    // It keeps `BURST`, not one. An earlier version popped the last and threw
    // the rest away, so a five-cue tick was quieter than a four-cue one, and the
    // test asserted `<= burst()`, which one satisfies. The newest survive,
    // because the newest record is the one at the bottom of the transcript.
    if asked.len() > BURST {
        asked.drain(..asked.len() - BURST);
    }
    for voice in asked {
        ringing.write(super::RingMessage(voice));
    }
}

/// How many cues may sound in one frame before it is a noise rather than a
/// chord.
///
/// Four is a tick's worth of work finishing at once with room to spare. See
/// [`ring_for_records`].
const BURST: usize = 4;

/// [`BURST`], for the test that holds it.
#[cfg(test)]
pub(super) const fn burst() -> usize {
    BURST
}

#[cfg(test)]
pub(crate) fn table() -> Vec<(&'static str, Cue)> {
    Voice::ALL
        .into_iter()
        .map(|voice| (voice.name(), voice.cue()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cap_keeps_a_handful_rather_than_collapsing_to_one() {
        // The first version of the cap did `asked.pop()` and discarded the rest,
        // so five cues became one and a busier tick was *quieter* than a calm
        // one. The plugin test asserted `<= burst()`, which one satisfies —
        // so the bound was held and the behaviour was not.
        let mut asked: Vec<Voice> = std::iter::repeat_n(Voice::Done, BURST + 6).collect();
        if asked.len() > BURST {
            asked.drain(..asked.len() - BURST);
        }
        assert_eq!(asked.len(), BURST, "the cap did not keep a whole handful");

        // ...and under the cap nothing is dropped at all.
        let mut few: Vec<Voice> = std::iter::repeat_n(Voice::Good, BURST - 1).collect();
        if few.len() > BURST {
            few.drain(..few.len() - BURST);
        }
        assert_eq!(few.len(), BURST - 1);
    }

    #[test]
    fn a_completion_a_warning_and_a_sabotage_are_three_tones() {
        // §14, literally: *"distinct tones for completion/warning/sabotage give
        // screen-reader players an ambient channel that does not compete with
        // speech."* Three, and they have to differ in the waveform rather than
        // only in the enum, because the player hears one and not the other.
        let completion = voice_of(RecordKind::Completion, Role::Normal, None);
        let warning = voice_of(RecordKind::Message, Role::Danger, None);
        let sabotage = voice_of(RecordKind::Status, Role::Danger, None);
        assert_eq!(completion, Some(Voice::Done));
        assert_eq!(warning, Some(Voice::Ill));
        assert_eq!(sabotage, Some(Voice::Wrong));

        let heard: Vec<Cue> = [completion, warning, sabotage]
            .into_iter()
            .flatten()
            .map(Voice::cue)
            .collect();
        assert_ne!(heard[0], heard[1], "a completion sounds like a warning");
        assert_ne!(heard[1], heard[2], "a warning sounds like a sabotage");
        assert_ne!(heard[0], heard[2], "a completion sounds like a sabotage");
    }

    #[test]
    fn a_surface_that_verifies_sound_is_not_the_sabotage_tone() {
        // The other half of the same record: `verify` pushes `Status` either
        // way, and only the role differs. A cue on the kind alone would cry
        // sabotage every time a player checked something and found it fine.
        assert_eq!(
            voice_of(RecordKind::Status, Role::Success, None),
            Some(Voice::Good),
        );
    }

    #[test]
    fn every_cue_is_chosen_from_what_the_marker_is() {
        // Rule 2 as a test. `voice_of` takes only `kind`, `role` and `outcome`
        // — what the transcript's own styling is chosen from — so a cue cannot
        // know something the Frame does not. The signature is the proof; this
        // runs every combination so a future arm cannot need a fourth input.
        let mut heard = 0;
        for kind in RecordKind::ALL {
            for role in [Role::Normal, Role::Danger, Role::Cost, Role::Success] {
                for outcome in Outcome::ALL.map(Some).into_iter().chain([None]) {
                    if voice_of(kind, role, outcome).is_some() {
                        heard += 1;
                    }
                }
            }
        }
        assert!(heard > 0, "nothing in the transcript makes any sound");
    }

    #[test]
    fn a_completion_and_a_breach_do_not_sound_the_same() {
        // §14 asks for distinct completion, warning and sabotage tones. The
        // distinctness that matters is the one a player hears with the screen
        // off, so it is asserted on the waveform rather than on the enum.
        let done = Voice::Done.cue();
        let ill = Voice::Ill.cue();
        assert_ne!(done, ill, "a finished brew sounds like a breach");
        let lowest = |cue: &Cue| {
            cue.tones
                .iter()
                .map(|tone| tone.hz)
                .fold(f32::INFINITY, f32::min)
        };
        assert!(
            lowest(&ill) < lowest(&done),
            "the alarm is higher than the good news",
        );
    }

    #[test]
    fn a_refusal_is_not_an_alarm() {
        // §6: never a bare error, and a refusal names what to do instead. A
        // typo answered with the breach klaxon would be that rule broken in a
        // channel §6 never thought about.
        assert_eq!(
            voice_of(RecordKind::Echo, Role::Normal, Some(Outcome::Unresolved)),
            Some(Voice::Baulk),
        );
        assert_ne!(
            voice_of(RecordKind::Echo, Role::Normal, Some(Outcome::Unresolved)),
            Some(Voice::Ill),
        );
    }

    #[test]
    fn a_resolved_echo_is_silent() {
        // The common case, and the one that would make the game unbearable.
        // Every line a player types produces an echo; a cue on it would be a
        // second sound on the same keystroke as `Enter`.
        assert_eq!(
            voice_of(RecordKind::Echo, Role::Normal, Some(Outcome::Resolved)),
            None,
        );
        assert_eq!(voice_of(RecordKind::Input, Role::Normal, None), None);
    }

    #[test]
    fn every_voice_has_a_cue_with_sound_in_it() {
        for voice in Voice::ALL {
            let cue = voice.cue();
            assert!(
                !cue.tones.is_empty() && cue.secs() > 0.0,
                "the `{}` cue is silence",
                voice.name(),
            );
        }
    }
}
