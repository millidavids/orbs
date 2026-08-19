mod drive;
mod engine;
mod plugin;

pub(crate) use drive::booted;
pub(crate) use engine::line as engine;
// The card itself and the clock behind it are `orbs-shell`'s — a terminal boots
// too. What stays here is what the *engine line* says and what drives the clock.
pub(crate) use orbs_shell::{Boot, Stage};
pub use plugin::BootPlugin;
