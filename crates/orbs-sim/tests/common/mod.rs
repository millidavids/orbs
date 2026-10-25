//! The helpers every integration test was writing for itself.
//!
//! Each test binary compiles this module and uses a subset of it.
#![allow(dead_code)]

mod helpers;

// A binary whose tests are all debug-only uses none of it in a release build.
#[allow(unused_imports)]
pub use helpers::*;
