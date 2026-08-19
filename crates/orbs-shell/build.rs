//! Capture the compiler that actually built this binary.
//!
//! The boot screen reports the Rust version, and reporting a *wrong* one would
//! undercut the only argument for the screen existing (see `boot::screen`).
//! There are three plausible sources and two of them are wrong:
//!
//! | Source | Value here | |
//! |---|---|---|
//! | `env!("CARGO_PKG_RUST_VERSION")` | **empty** | this crate does not inherit `rust-version` |
//! | the workspace's `rust-version` | `1.95` | a floor, not the compiler in use |
//! | `rust-toolchain.toml` | `1.96.0` | what the toolchain *should* be |
//! | `rustc -vV` | the truth | what it *was* |

use std::process::Command;

fn main() {
    // Only the manifest can change this, so nothing else needs re-running.
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-env-changed=RUSTC");

    println!("cargo::rustc-env=ORBS_RUSTC={}", rustc_version());
}

/// The compiler's version, or a placeholder that is obviously one.
///
/// A build that cannot ask `rustc` should not fail — it is a splash screen — but
/// it must not silently invent a number either, so the fallback is a value no
/// release could be mistaken for.
fn rustc_version() -> String {
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_owned());
    let Ok(output) = Command::new(rustc).arg("--version").output() else {
        return "0.0.0".to_owned();
    };
    let Ok(text) = String::from_utf8(output.stdout) else {
        return "0.0.0".to_owned();
    };
    // `rustc 1.96.0 (abcdef 2026-01-01)` -> `1.96.0`
    text.split_whitespace().nth(1).unwrap_or("0.0.0").to_owned()
}
