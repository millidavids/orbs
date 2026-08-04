mod input;
mod plugin;
mod prompt;
mod screen;

pub(crate) use input::Line;
pub use plugin::ShellPlugin;
pub(crate) use plugin::ShellSystems;
pub(crate) use prompt::{paint, paint_too_small};
pub(crate) use screen::{Screen, track_window};
