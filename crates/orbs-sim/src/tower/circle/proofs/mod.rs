//! The circle's claims, over every circuit rather than a sample.
//!
//! Exhaustive because it is cheap: 11,664 circuits, 216 ways to limn one. A
//! property asserted over a handful of seeds is a property of those seeds.
//!
//! The counts are pinned to numbers measured outside this code — a python
//! enumeration over the same rules, recorded in §19 — so a test here cannot
//! pass by the code agreeing with itself.

mod beast;
mod draws;
mod oracle;
mod searches;
mod space;
mod tables;
mod view;
