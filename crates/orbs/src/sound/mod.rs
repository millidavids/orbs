//! The orb's voice: cues generated in code, and the hum under them.

mod bed;
mod cues;
mod keys;
mod levels;
mod plugin;
mod ringing;
mod synth;

pub(crate) use cues::Heard;
pub(crate) use levels::{HUM, Levels, VOICE};
pub use plugin::SoundPlugin;
pub(crate) use ringing::RingMessage;
