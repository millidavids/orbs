//! The orb's voice: cues generated in code.

mod cues;
mod keys;
mod kinds;
mod levels;
mod plugin;
mod ringing;
mod synth;

pub(crate) use cues::Heard;
pub(crate) use kinds::{Kind, Kinds};
pub(crate) use levels::{Levels, VOICE};
pub use plugin::SoundPlugin;
pub(crate) use ringing::RingMessage;
