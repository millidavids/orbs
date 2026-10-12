//! What this build can set, and what setting it does.
//!
//! [`Row`](orbs_shell::settings::Row) is data and the menu draws it; this is
//! the half that knows what a phosphor theme is, how many there are, and which
//! one is on. `values.rs` in `orbs-shell` carries the argument for the split —
//! the two builds do not have the same settings, and the menu must not be the
//! place that knows which.
//!
//! Every row is read *from the thing it sets*, never from the file
//! (`Menu::show_driver`'s rule): what a page shows is what is in effect. An
//! `ORBS_SAVE=off` run has no file and still has a tube, and `ORBS_CRT=off`
//! outranks whatever is written down.
//!
//! A function key writes back, so `F3` and the tube page are one setting seen
//! twice. The keys still cycle, because that is faster than a menu, and call
//! [`remember`] on the way past.

mod apply;
mod held;
mod rows;
#[cfg(test)]
mod tests;

pub(crate) use apply::{apply, follow_the_keys, remember};
pub(crate) use held::{CRT, FOCUS, Naming, SIGHT, Skipping, THEME};
pub(crate) use rows::{Settable, defaults};
