mod plugin;
mod preview;
mod screen;

pub use plugin::ShellPlugin;
pub(crate) use preview::{paint, paint_too_small};
pub(crate) use screen::{Screen, track_window};
