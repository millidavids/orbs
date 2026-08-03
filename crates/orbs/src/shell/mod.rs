mod plugin;
mod preview;
mod screen;

pub use plugin::ShellPlugin;
pub(crate) use preview::paint;
pub(crate) use screen::{Screen, track_window};
