mod atlas;
mod blink;
mod ember;
mod glyphs;
mod grid;
// `pub(crate)` because the panel's own tests resolve the colours it paints —
// the tint crosses `orbs-sim`, `orbs-render` and this crate, and every
// single-module test of it passes while the three are disconnected.
pub(crate) mod palette;
mod plugin;
pub(crate) mod tint;

pub use plugin::RenderPlugin;
