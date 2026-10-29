mod away;
mod commanding;
mod echoing;
mod editing;
mod input;
mod installing;
mod manualling;
mod menuing;
mod modal;
mod motion;
mod plugin;
mod reading;
mod revealing;
mod setting;
mod thresholding;
mod wandering;
mod weaving;
mod window;

// Everything a painter or a surface needs is `orbs-shell`'s now, and re-exported
// here so a Bevy system reads `shell::Line` exactly as it did when this module
// owned the file. What is left below is the half that needs an engine: keyboard
// events with a press/release model, run conditions, the camera, and the system
// graph that orders them.
pub(crate) use orbs_shell::{
    Bench, Climb, Editor, EditorOutcome, Ghost, Line, Linear, Offered, PaneTransition, Panel,
    Passing, Reveal, Screen, Scroll, Tapestry, View, WeaveOutcome, paint, paint_booting,
    paint_too_small,
};

pub(crate) use away::{Absence, Away};
pub(crate) use editing::Editing;
pub(crate) use input::SubmittedMessage;
pub(crate) use installing::{Installing, Installs};
pub(crate) use manualling::{OpenManualMessage, Reading};
pub(crate) use menuing::{SettingMessage, Standing, SwapMessage};
pub(crate) use motion::{Stillness, moving, moving_at_launch};
pub use plugin::ShellPlugin;
pub(crate) use plugin::{ShellSystems, reset_for_swap};
pub(crate) use setting::apply as apply_setting;
// A function key and the settings page are **one setting seen twice**: the key
// cycles and writes its word down, so the page cannot disagree on the next
// launch. These are the words the window's own keys write.
pub(crate) use setting::{CRT, THEME, defaults as default_settings, pair};
#[cfg(test)]
pub(crate) use thresholding::Opened as ThresholdOpened;
pub(crate) use thresholding::playing;
pub(crate) use wandering::Walk;
pub(crate) use weaving::Loom;
pub(crate) use window::track_window;
