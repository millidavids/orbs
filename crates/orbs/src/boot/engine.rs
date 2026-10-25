//! What this binary is built out of — the one POST line only a frontend knows.
//!
//! The card is a true inventory (§4), and the terminal build links no Bevy, so
//! the engine line is handed to the shared painter, as [`crate::wizard`] is.

/// The exact Bevy pin, held to `Cargo.toml` by the test below.
///
/// A const, since the pin changes only by hand; a macro, since `concat!` takes
/// literals, so the number is written once.
macro_rules! bevy_version {
    () => {
        "0.20.0"
    };
}

/// Only the test reads this now — [`line`] builds the whole string at compile
/// time from the same macro, so there is nothing left to interpolate at runtime.
#[cfg(test)]
const BEVY: &str = bevy_version!();

/// The engine line for this frontend, as the POST card prints it.
///
/// A `const`, not a `format!`. This allocated, and `repaint` calls it once per
/// frame for the whole boot sequence — some 780 allocations for a string with no
/// runtime input. `Panel`, `Ghost` and `Bench::permitted` are all caches that
/// exist for the same reason.
pub(crate) const fn line() -> &'static str {
    concat!("bevy ", bevy_version!())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bevy_version_is_the_one_the_manifest_pins() {
        // Here, where `CARGO_MANIFEST_DIR` pins an engine (§19). Every
        // declaration, since the manifest declares bevy twice.
        let manifest = include_str!("../../Cargo.toml");
        let declared = manifest
            .lines()
            .filter(|line| line.trim_start().starts_with("bevy = "))
            .count();
        let pinned = manifest
            .lines()
            .filter(|line| line.trim_start().starts_with("bevy = "))
            .filter(|line| line.contains(&format!("\"={BEVY}\"")))
            .count();
        assert!(declared > 0, "Cargo.toml declares no bevy at all");
        assert_eq!(
            pinned, declared,
            "the splash says bevy {BEVY}; {pinned} of {declared} bevy \
             declarations in Cargo.toml pin that version",
        );
    }

    #[test]
    fn the_engine_line_is_drawable() {
        // The painter's own font lint runs over a stand-in engine string, so
        // this is what covers the real one. A glyph the atlas has no cell for
        // occupies a column and draws nothing.
        for glyph in line().chars() {
            assert!(
                orbs_render::is_renderable(glyph),
                "{glyph:?} in the engine line cannot be drawn"
            );
        }
    }
}
