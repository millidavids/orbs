mod chorusing;
mod commanding;
mod editing;
mod input;
mod motion;
mod plugin;
mod reading;
mod revealing;
mod wandering;
mod weaving;
mod window;

// Everything a painter or a surface needs is `orbs-shell`'s now, and re-exported
// here so a Bevy system reads `shell::Line` exactly as it did when this module
// owned the file. What is left below is the half that needs an engine: keyboard
// events with a press/release model, run conditions, the camera, and the system
// graph that orders them.
pub(crate) use orbs_shell::{
    Bench, Editor, EditorOutcome, Ghost, Line, Linear, Offered, PaneTransition, Panel, Passing,
    Reveal, Screen, Scroll, Tapestry, View, WeaveOutcome, paint, paint_booting, paint_too_small,
};

pub(crate) use chorusing::Chorus;
pub(crate) use editing::Editing;
pub use plugin::ShellPlugin;
pub(crate) use plugin::ShellSystems;
pub(crate) use wandering::Walk;
pub(crate) use weaving::Loom;
pub(crate) use window::track_window;
