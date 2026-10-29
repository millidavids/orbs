//! The tour in the window (DESIGN.md §19, *the tour*). The catalogue is
//! `orbs_shell::tour`'s.

mod begin;
mod inject;
mod plugin;
mod shots;
mod start;
mod state;
#[cfg(test)]
mod tests;
mod yours;

pub(crate) use start::{Ask, before_the_app};
