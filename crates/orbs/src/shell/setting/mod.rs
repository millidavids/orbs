//! What this build can set, and what setting it does.
//!
//! This build's half of [`Row`](orbs_shell::settings::Row). Every row is read
//! from the thing it sets, never the file, so a page shows what is in effect.
//! A function key writes back, so `F3` and the tube page are one setting.

mod apply;
mod held;
mod pair;
mod rows;
#[cfg(test)]
mod tests;

pub(crate) use apply::{apply, follow_the_keys};
pub(crate) use held::{CRT, Naming, Skipping, THEME};
pub(crate) use pair::pair;
pub(crate) use rows::{Settable, defaults};
