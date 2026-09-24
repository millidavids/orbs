//! Guards on the two architectural rules this crate can now break silently.
//!
//! `orbs-sim` gained a dependency on `orbs-render` when the record model landed:
//! a record is the interface between the two crates, and an interface belongs to
//! whichever side both can depend on. The cost is that this crate can now name
//! [`Painter`] and `Frame`, which rule 2 forbids it from using.
//!
//! A comment would not hold and this does — the instrument §16 already uses,
//! failing the build if `unscii-16-full` appears in `assets/`.
//!
//! These read the crate's own source, deliberately: what is being asserted is
//! what the source may mention, which no runtime behaviour can express.

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
    // Rule 2. The sim emits records; where one lands on the grid is
    // `orbs-render`'s job. Changing that is a §19 decision, not a use statement.
    //
    // `Fidelity` was the fifth name until §19 fixed the grid and deleted it, and
    // guarding a name that cannot exist is a boundary test getting weaker — so
    // `scale_for` and `GRID` took its place.
    const FORBIDDEN: [&str; 5] = ["Painter", "Frame", "ScreenLayout", "scale_for", "GRID"];

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

#[test]
fn the_sim_never_touches_the_filesystem() {
    // Rule 1. This crate must compile and test headlessly, in milliseconds, with
    // no window — and `crates/orbs-sim/Cargo.toml` states the corollary in as
    // many words: *"this crate never watches a file — a frontend owns the
    // watcher and hands new content in at a tick boundary."* `orbs/src/sim/
    // content.rs` gives the reason the other way round: *"a watcher inside the
    // sim would also make every headless test touch the filesystem, which is the
    // thing rule 1 exists to prevent."*
    //
    // Until the save format landed, nothing here was *tempted*. `save::capture`
    // and `save::restore` turn a world into a document and back, and the obvious
    // next line is the one that writes it to a file. It belongs in `orbs-shell`,
    // beside `prose::read` and `shortcuts::export_trace`, and this is what says
    // so at build time rather than in a comment nobody reads in a hurried phase.
    //
    // `include_str!` is untouched: content compiled *into* the binary is the
    // opposite of this, and is what makes the headless promise keepable.
    //
    // A tripwire, not a wall, and worth saying so. A substring scan is trivially
    // walked around — `use std::fs;` then a bare `fs::write`, or `OpenOptions`,
    // or `Command::new("cp")` — and it cannot see a filesystem-capable
    // dependency arriving in `Cargo.toml`. It is aimed at the accidental
    // `std::fs::write` in a hurried phase, which is how the rule would actually
    // be broken. `the_sim_never_depends_on_bevy_the_engine` reads the manifest
    // and is the shape to copy if a dependency needs guarding too.
    for path in sources() {
        let source = fs::read_to_string(&path).unwrap_or_else(|error| panic!("{path:?}: {error}"));
        // `std::fs` covers the module however it is spelled at the call site.
        // A bare `read_to_string` is deliberately *not* a needle: it is an
        // inherent method name common enough to fail this build for a reason
        // that has nothing to do with rule 1.
        for name in ["std::fs", "File::open", "File::create", "File::create_new"] {
            assert!(
                !source.contains(name),
                "{}: reaches for `{name}` — rule 1 keeps the filesystem out of orbs-sim",
                path.display(),
            );
        }
    }
}
