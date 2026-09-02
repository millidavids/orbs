//! What this binary is built out of — the one POST line only a frontend knows.
//!
//! The boot card is a **diegetic inventory of the machine** (§4), which is the
//! whole argument for it existing: `tower/boot.rs` is built by walking the world
//! so it cannot go stale, and a POST in front of it printing invented numbers
//! would be the same lie one screen earlier.
//!
//! So the engine line cannot live with the painter. The painter is shared —
//! `orbs-tui` draws the same card — and a terminal build does not link Bevy at
//! all. A shared painter hard-coding `bevy 0.19.0` would put a component on the
//! inventory that is not in the machine.
//!
//! This is [`crate::wizard`]'s shape: a fact only the frontend can know, handed
//! to the shared layer, which still decides where it goes and how it reads.

/// The exact Bevy pin, held to `Cargo.toml` by the test below.
///
/// A const rather than a build script because the version is *pinned* —
/// CLAUDE.md commits to `=0.19.0` and upgrading it is a deliberate one-window
/// act in Phase 11c, so a number that can only change when someone edits the
/// manifest is exactly as live as it needs to be.
/// **A macro so the number is written once**, because `concat!` takes literals
/// and not consts — and two copies of a version is exactly the drift this whole
/// file exists to prevent.
macro_rules! bevy_version {
    () => {
        "0.19.0"
    };
}

/// Only the test reads this now — [`line`] builds the whole string at compile
/// time from the same macro, so there is nothing left to interpolate at runtime.
#[cfg(test)]
const BEVY: &str = bevy_version!();

/// The engine line for this frontend, as the POST card prints it.
///
/// **A `const`, not a `format!`.** This allocated, and `repaint` calls it once
/// per frame for the whole boot sequence — some 780 allocations for a string
/// with no runtime input at all. The surrounding code is unusually careful about
/// exactly this: `Panel`, `Ghost` and `Bench::permitted` are all caches that
/// exist because "twenty allocations at 60 Hz for 1 Hz data" was judged worth
/// removing.
pub(crate) const fn line() -> &'static str {
    concat!("bevy ", bevy_version!())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bevy_version_is_the_one_the_manifest_pins() {
        // CLAUDE.md pins Bevy exactly and upgrading is a deliberate Phase 11c
        // act. This is what stops the splash drifting away from the manifest
        // silently when that window arrives.
        //
        // **It stays in this crate on purpose.** Moved beside the painter it
        // would resolve `CARGO_MANIFEST_DIR` to a crate that pins no engine, and
        // pass by asserting nothing — which is exactly how `orbs-balance`'s
        // first `agrees.rs` was green while measuring nothing (§19).
        // **Every declaration, not merely one.** `crates/orbs/Cargo.toml`
        // declares bevy twice — once plainly and once under
        // `cfg(target_os = "linux")` — and a `contains` passes if *either* still
        // says `0.19.0`. That is the exact shape of a half-finished Phase 11c
        // upgrade, and the half most likely to be left behind is the one that
        // decides what a Linux build actually links.
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
