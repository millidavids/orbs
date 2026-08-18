//! Headless economy harness — balance as a sweep rather than as vibes.
//!
//! Drives `orbs-sim` with a synthetic player and dumps DESIGN.md §11.5's curves,
//! so the numbers in the design are measured rather than asserted. §16 names it
//! the mitigation for the High-severity risk *"economy is wrong or untuneable"*,
//! and ROADMAP puts it at the head of Phase 2 for a reason it states outright:
//! **before** five domain phases author durations on top of unswept ones.
//!
//! It needs no window, no GPU and no `App` — `orbs-sim` is headless by
//! construction (CLAUDE.md rule 1), which is what makes a 25-hour session a
//! second of wall clock here.
//!
//! # Why there is a library and not only a binary
//!
//! `tests/agrees.rs` holds this crate's See-it claim — *a sweep's curve and a
//! hand-played session agree* — and a binary-only crate cannot be reached from an
//! integration test at all. The first version of that file drove `orbs_sim::Sim`
//! by hand for exactly that reason, and so passed without touching a
//! [`Policy`](policy::Policy), [`drive::run`] or [`report::expected`]: it
//! asserted the reference numbers were
//! *reachable* and never that the harness reached them.
//!
//! So the modules live here and `main.rs` is the CLI over them. Nothing is public
//! for the tests' sake alone; everything here is what the CLI already prints.
//!
//! ```text
//! cargo run -p orbs-balance -- list
//! cargo run -p orbs-balance -- run clarity --ticks 400
//! cargo run -p orbs-balance -- sweep --hours 1 --csv curves.csv
//! ```

pub mod drive;
pub mod policy;
pub mod report;
