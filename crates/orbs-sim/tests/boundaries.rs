//! Guards on the two architectural rules this crate can now break silently.
//!
//! `orbs-sim` gained a dependency on `orbs-render` when the record model landed:
//! a record is the *interface* between the two crates, and an interface belongs
//! to whichever side both can depend on (see `orbs_render::record`). The cost is
//! that this crate can now **name** [`Painter`] and `Frame`, which architectural
//! rule 2 forbids it from using — `orbs-render` decides what appears and where,
//! and the sim is not a participant in that decision.
//!
//! A comment would not hold. This does, and it is the same instrument the design
//! already reaches for elsewhere: §16 fails the build if `unscii-16-full` ever
//! appears in `assets/`, for exactly the same reason — a rule that is only
//! written down is a rule that gets broken during a hurried phase.
//!
//! These read the crate's own source. That is unusual and deliberate: the thing
//! being asserted is what the source is *allowed to mention*, which no amount of
//! runtime behaviour can express.

use std::fs;
use std::path::{Path, PathBuf};

/// Every `.rs` file under `src/`.
fn sources() -> Vec<PathBuf> {
    fn walk(dir: &Path, found: &mut Vec<PathBuf>) {
        let entries =
            fs::read_dir(dir).unwrap_or_else(|error| panic!("read {}: {error}", dir.display()));
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, found);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                found.push(path);
            }
        }
    }

    let mut found = Vec::new();
    walk(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut found,
    );
    assert!(!found.is_empty(), "found no sources to check");
    found
}

#[test]
fn the_sim_never_reaches_for_the_painter() {
    // Rule 2. The sim emits records; deciding where a record lands on the grid
    // is `orbs-render`'s job and a frontend's after that. If this ever needs to
    // change, it is a design decision for DESIGN.md §19, not a use statement.
    const FORBIDDEN: [&str; 4] = ["Painter", "Frame", "ScreenLayout", "Fidelity"];

    for path in sources() {
        let source = fs::read_to_string(&path).unwrap_or_else(|error| panic!("{path:?}: {error}"));
        for name in FORBIDDEN {
            assert!(
                !source.contains(name),
                "{}: mentions `{name}` — rule 2 keeps layout out of orbs-sim",
                path.display(),
            );
        }
    }
}

#[test]
fn the_sim_stays_synchronous() {
    // Rule 8. Async introduces non-deterministic completion ordering, which is
    // exactly what replay, offline/online parity, and the balance harness
    // matching the live game all depend on not existing. `Sim::step(&mut self)`
    // makes cross-thread driving impossible; this keeps the inside honest too.
    for path in sources() {
        let source = fs::read_to_string(&path).unwrap_or_else(|error| panic!("{path:?}: {error}"));
        for name in ["async fn", ".await", "tokio"] {
            assert!(
                !source.contains(name),
                "{}: contains `{name}` — rule 8 forbids async in orbs-sim",
                path.display(),
            );
        }
    }
}

#[test]
fn the_sim_never_depends_on_bevy_the_engine() {
    // Rule 1. `bevy_ecs` standalone is ~90 crates and adds no renderer; `bevy`
    // is ~340 and adds all of it. The underscore form is what a `use` would
    // spell, the hyphenated form is what a manifest would.
    let manifest = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("read Cargo.toml");
    for line in manifest.lines() {
        let declaration = line.trim_start();
        assert!(
            !(declaration.starts_with("bevy =") || declaration.starts_with("bevy.")),
            "orbs-sim declares a dependency on `bevy` the engine: {line}",
        );
    }

    for path in sources() {
        let source = fs::read_to_string(&path).unwrap_or_else(|error| panic!("{path:?}: {error}"));
        assert!(
            !source.contains("use bevy::"),
            "{}: imports from `bevy` the engine",
            path.display(),
        );
    }
}
