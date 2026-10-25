//! Guards on the two architectural rules this crate can now break silently.
//!
//! The record model gives this crate `orbs-render`, so it can name [`Painter`]
//! and `Frame`, which rule 2 forbids. These read its own source, since what
//! the source may mention is no runtime behaviour.

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

/// Fail if any source mentions one of `needles`, naming the rule it breaks.
fn forbid(needles: &[&str], rule: &str) {
    for path in sources() {
        let source =
            fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        for name in needles {
            assert!(
                !source.contains(name),
                "{}: mentions `{name}` — {rule}",
                path.display(),
            );
        }
    }
}

#[test]
fn the_sim_never_reaches_for_the_painter() {
    // Rule 2: where a record lands is `orbs-render`'s, and changing that is a
    // §19 decision, not a use statement.
    forbid(
        &["Painter", "Frame", "ScreenLayout", "scale_for", "GRID"],
        "rule 2 keeps layout out of orbs-sim",
    );
}

#[test]
fn the_sim_stays_synchronous() {
    // Rule 8. Async introduces non-deterministic completion ordering, which is
    // exactly what replay, offline/online parity, and the balance harness
    // matching the live game all depend on not existing. `Sim::step(&mut self)`
    // makes cross-thread driving impossible; this keeps the inside honest too.
    forbid(
        &["async fn", ".await", "tokio"],
        "rule 8 forbids async in orbs-sim",
    );
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

    forbid(&["use bevy::"], "rule 1: an import from `bevy` the engine");
}

#[test]
fn the_sim_never_touches_the_filesystem() {
    // Rule 1: the sim never touches the filesystem; a save is written by
    // `orbs-shell`. `include_str!` is fine. A tripwire for the hurried
    // `std::fs::write`, not a wall — a manifest guard is
    // `the_sim_never_depends_on_bevy_the_engine`'s shape. Not
    // `read_to_string`, too common a method name.
    forbid(
        &["std::fs", "File::open", "File::create", "File::create_new"],
        "rule 1 keeps the filesystem out of orbs-sim",
    );
}
