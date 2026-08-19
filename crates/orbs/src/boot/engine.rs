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
/// act in Phase 9c, so a number that can only change when someone edits the
/// manifest is exactly as live as it needs to be.
const BEVY: &str = "0.19.0";

/// The engine line for this frontend, as the POST card prints it.
pub(crate) fn line() -> String {
    format!("bevy {BEVY}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bevy_version_is_the_one_the_manifest_pins() {
        // CLAUDE.md pins Bevy exactly and upgrading is a deliberate Phase 9c
        // act. This is what stops the splash drifting away from the manifest
        // silently when that window arrives.
        //
        // **It stays in this crate on purpose.** Moved beside the painter it
        // would resolve `CARGO_MANIFEST_DIR` to a crate that pins no engine, and
        // pass by asserting nothing — which is exactly how `orbs-balance`'s
        // first `agrees.rs` was green while measuring nothing (§19).
        let manifest = include_str!("../../Cargo.toml");
        assert!(
            manifest.contains(&format!("\"={BEVY}\"")),
            "the splash says bevy {BEVY}, which Cargo.toml does not pin",
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
