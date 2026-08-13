//! Authored content: what the orb says, and (later) what a recipe is.
//!
//! CLAUDE.md rule 6 — prose in data files, never string literals in Rust.

mod fuel;
mod load;
mod material;
mod progression;
mod prose;
mod recipe;
mod spell;

pub use fuel::{Fuel, Fuels};
pub use load::ContentError;
pub use material::Materials;
pub use progression::Progression;
pub use prose::Prose;
pub use recipe::{Recipe, Recipes};
pub use spell::{EXTENSION, Spell, Spells, with_extension, without_extension};
