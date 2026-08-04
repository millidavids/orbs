mod plugin;
mod screen;
mod stage;

pub use plugin::BootPlugin;
pub(crate) use plugin::booted;
pub(crate) use screen::{arrived, paint};
pub(crate) use stage::{Boot, Stage};
