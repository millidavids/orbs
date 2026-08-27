mod atlas;
mod blink;
// **A test instrument, so it is compiled only for tests.** It simulates colour
// vision deficiency to hold the palette to §14's claim; it draws nothing, and
// the correction filter §19 specified is deliberately not shipped — see the
// module header for the measurement that decided it. Gated rather than `pub`
// because an un-called simulator in a release build is dead code, and the gate
// says what it is more clearly than a comment would.
#[cfg(test)]
pub(crate) mod deficiency;
mod ember;
mod glyphs;
mod grid;
// `pub(crate)` because `tinting` resolves the colours the shared panel painter
// draws — the tint crosses `orbs-sim`, `orbs-render` and this crate, and every
// single-module test of it passes while the three are disconnected.
pub(crate) mod palette;
mod plugin;
pub(crate) mod tint;
mod tinting;

pub use plugin::RenderPlugin;
