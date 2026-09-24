//! Turning a request into a sound: the message, the switch, and the one system
//! that spawns a player.
//!
//! Split out of `plugin.rs`, which CLAUDE.md reserves for Bevy registration —
//! *"System bodies and helpers go in sibling files"* — and which was carrying
//! this system, a const and a resource while its own header claimed otherwise.

use bevy::audio::Volume;
use bevy::prelude::*;

use super::cues::{Voice, Voices};

/// Ring a cue.
///
/// It carries the [`Voice`], not the handle. A message rather than each system
/// spawning its own player, so one place decides how loud a cue is and what
/// becomes of the entity — and naming the cue rather than the asset is what let
/// that place grow a per-category volume without every writer learning about it.
#[derive(Message, Debug, Clone, Copy)]
pub(crate) struct RingMessage(pub(crate) Voice);

/// Environment variable that prints every cue instead of only playing it.
///
/// You cannot diff a sound, and `dumps.sh` is blind to this subsystem by
/// construction — so without a switch the only gate audio has is a person with
/// speakers, which is §15's *"debug affordance nobody will maintain"* when
/// invented late and *"the instrument before the thing it measures"* when not.
///
/// `bevy_log` caps the subscriber at `INFO`, so the `trace!` below is invisible
/// in an ordinary run; this writes to stderr directly for the reason the dump
/// does — it is the half of this that works with no sound card and no window.
///
/// ```bash
/// ORBS_SOUND=1 cargo run -p orbs 2>&1 | grep '^sound:'
/// ```
const SOUND: &str = "ORBS_SOUND";

/// Whether cues are being printed as well as played.
#[derive(Resource, Debug, Clone, Copy)]
pub(super) struct Saying(bool);

impl Saying {
    /// Whether to print.
    pub(super) const fn aloud(self) -> bool {
        self.0
    }
}

impl Default for Saying {
    fn default() -> Self {
        Self(std::env::var_os(SOUND).is_some_and(|value| value != "0"))
    }
}

/// Play whatever asked to be played.
///
/// `PlaybackSettings::DESPAWN` rather than `REMOVE`: the entity exists for the
/// length of one cue and has nothing else on it, so the tidier of the two is the
/// one that leaves nothing behind.
pub(super) fn ring(
    mut commands: Commands,
    voices: Res<Voices>,
    saying: Res<Saying>,
    levels: Res<super::Levels>,
    mut ringing: MessageReader<RingMessage>,
) {
    let gain = levels.voice_gain();
    for RingMessage(voice) in ringing.read().copied() {
        let Some(handle) = voices.handle(voice) else {
            continue;
        };
        // The one way to check the audio without a sound card, which is what CI
        // and a headless run both are. See [`SOUND`].
        trace!("sound: {}", voice.name());
        if saying.aloud() {
            eprintln!("sound: {}", voice.name());
        }
        // Silence is not spawning. A player who asked for `off` should cost the
        // game nothing, and an `AudioPlayer` at zero gain is an entity, a decode
        // and a mixer slot per keystroke for a sound nobody hears. The message
        // is still read, so `ORBS_SOUND` still reports it — which is what makes
        // the switch usable for checking the table rather than the speakers.
        if gain > 0.0 {
            commands.spawn((
                AudioPlayer(handle),
                PlaybackSettings::DESPAWN.with_volume(Volume::Linear(gain)),
            ));
        }
    }
}
