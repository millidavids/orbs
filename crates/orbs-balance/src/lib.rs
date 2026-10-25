//! Headless economy harness — balance as a sweep rather than as vibes.
//!
//! A synthetic player measuring §11.5's curves, §16's mitigation for an
//! untuneable economy. Headless (rule 1), so hours take seconds.
//!
//! A library so `tests/agrees.rs` can reach [`Policy`](policy::Policy),
//! [`drive::run`] and [`report::expected`]; `main.rs` is the CLI.
//!
//! ```text
//! cargo run -p orbs-balance -- list
//! cargo run -p orbs-balance -- run clarity --ticks 400
//! cargo run -p orbs-balance -- sweep --hours 1 --csv curves.csv
//! ```

pub mod drive;
pub mod policy;
pub mod report;
mod scrolling;
