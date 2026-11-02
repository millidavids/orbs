//! The helpers every integration test was writing for itself.
//!
//! Each test binary compiles this module and uses a subset of it.
#![allow(dead_code, reason = "each binary uses a different subset")]

mod helpers;

#[allow(
    unused_imports,
    reason = "a binary whose tests are all debug-only uses none of it in a release build"
)]
pub use helpers::*;
