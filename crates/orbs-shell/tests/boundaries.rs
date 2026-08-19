//! The one architectural rule this crate can break silently.
//!
//! `orbs-shell` sits above `orbs-sim` and `orbs-render` and below two frontends,
//! and its whole value is that the *second* frontend can depend on it without
//! depending on an engine. That is a property of what the source is allowed to
//! **mention**, which no amount of runtime behaviour can express — so it is
//! asserted the way `orbs-sim` asserts the same class of rule, by reading the
//! crate's own source.
//!
//! `bevy_ecs` is allowed and `bevy` is not, which is architectural rule 1 as
//! written: *"what is forbidden is rendering, windowing, assets, and anything
//! requiring a GPU."* The standalone ECS is ~90 crates and adds no renderer; the
//! engine is ~340 and adds all of it.
//!
//! **A copy of `sources()` rather than a shared helper.** `orbs-sim`'s version
//! walks its own `CARGO_MANIFEST_DIR`; pointing it at a sibling would break the
//! moment either crate moved, and a boundary test that silently stops walking
//! anything is the failure mode §19 records for `orbs-balance`'s first
//! `agrees.rs` — green while measuring nothing.

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
fn the_shell_never_depends_on_bevy_the_engine() {
    let manifest = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("read Cargo.toml");
    for line in manifest.lines() {
        let declaration = line.trim_start();
        assert!(
            !(declaration.starts_with("bevy =") || declaration.starts_with("bevy.")),
            "orbs-shell declares a dependency on `bevy` the engine: {line}",
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

#[test]
fn the_shell_stays_synchronous() {
    // Rule 8, one crate further out. The sim is called from exactly one place
    // per frontend, synchronously; a shell that awaited anything between a
    // keystroke and a `Frame` would put a scheduler between them.
    for path in sources() {
        let source = fs::read_to_string(&path).unwrap_or_else(|error| panic!("{path:?}: {error}"));
        for name in ["async fn", ".await", "tokio"] {
            assert!(
                !source.contains(name),
                "{}: contains `{name}` — rule 8 forbids async here too",
                path.display(),
            );
        }
    }
}

#[test]
fn the_shell_never_resolves_a_colour() {
    // Rule 2's other half, and the one this crate is most likely to break as it
    // grows: `orbs-render` decides what appears and where, this crate decides
    // what a *screen* is, and **how a cell is finally drawn is a frontend's** —
    // a phosphor on a GPU, an ANSI index in a terminal. A `Style` resolved here
    // would be resolved once for two frontends that cannot agree about it.
    //
    // `Tint` and `Wash` are fine and are not listed: they are `orbs-render`'s
    // semantic names for a material's colour *family*, which is what appears,
    // not how it is drawn.
    for path in sources() {
        let source = fs::read_to_string(&path).unwrap_or_else(|error| panic!("{path:?}: {error}"));
        for name in ["Srgba", "LinearRgba", "resolve_tinted", "Phosphor"] {
            assert!(
                !source.contains(name),
                "{}: mentions `{name}` — resolving a colour belongs to a frontend",
                path.display(),
            );
        }
    }
}
