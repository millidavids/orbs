mod atlas;
// `pub(crate)` for `shell::motion`'s test, which holds the caret to §14's
// switch through the system rather than through a flag.
pub(crate) mod blink;
// A test instrument, so it is compiled only for tests. It simulates colour
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
mod legibility;
// `pub(crate)` because `tinting` resolves the colours the shared panel painter
// draws — the tint crosses `orbs-sim`, `orbs-render` and this crate, and every
// single-module test of it passes while the three are disconnected.
pub(crate) mod palette;
mod plugin;
pub(crate) mod tint;
mod tinting;
mod weight;

pub use plugin::RenderPlugin;
// The phosphor as a *setting*. `shell::setting` reads the theme that is in
// effect from here and the list of them from `palette`.
pub(crate) use plugin::Theme;
// `contrast high`, a filter beside the theme rather than a theme.
pub(crate) use legibility::Contrast;
// `weight heavy`, the same font a pixel bolder.
pub(crate) use weight::Weight;
