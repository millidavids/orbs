//! Copies the trained weights into the build, so a game carries them rather
//! than reading the tree it was built in (DESIGN.md §19, *The weights are
//! compiled in*).

use std::io::ErrorKind;
use std::path::Path;

fn main() {
    let out = std::env::var("OUT_DIR").expect("cargo sets OUT_DIR for a build script");
    for name in ["reader", "scribe"] {
        let from = Path::new("weights").join(format!("{name}.bin"));
        // Absent is copied as empty, which loads as no reader, so a tree
        // without them still builds; `the_build_carries_the_weights_on_disk`
        // is what fails.
        let bytes = match std::fs::read(&from) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == ErrorKind::NotFound => Vec::new(),
            Err(error) => panic!("{}: {error}", from.display()),
        };
        std::fs::write(Path::new(&out).join(format!("{name}.bin")), bytes)
            .expect("OUT_DIR is writable");
    }
    // A path that does not exist reruns this on every build, so a tree with no
    // weights watches the crate, where `weights/` will appear.
    let watched = if Path::new("weights").is_dir() {
        "weights"
    } else {
        "."
    };
    println!("cargo::rerun-if-changed={watched}");
}
