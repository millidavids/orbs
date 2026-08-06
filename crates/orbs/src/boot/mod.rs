mod drive;
mod plugin;
mod screen;
mod stage;

pub(crate) use drive::booted;
pub use plugin::BootPlugin;
pub(crate) use screen::paint;
pub(crate) use stage::{Boot, Stage};
