//! Authored content: what the orb says, and (later) what a recipe is.
//!
//! CLAUDE.md rule 6 — prose in data files, never string literals in Rust.

mod fuel;
mod load;
mod prose;
mod recipe;

pub use fuel::{Fuel, Fuels};
pub use load::ContentError;
pub use prose::Prose;
pub use recipe::{Recipe, Recipes};
