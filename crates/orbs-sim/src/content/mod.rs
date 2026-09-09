//! Authored content: what the orb says, and (later) what a recipe is.
//!
//! CLAUDE.md rule 6 — prose in data files, never string literals in Rust.

mod deed;
mod forge;
mod fuel;
mod length;
mod load;
mod material;
mod phrasing;
mod progression;
mod prose;
mod recipe;
mod siege;
mod spell;

pub use deed::Deed;
pub use forge::{Charm as CharmCost, Charms};
pub use fuel::{Fuel, Fuels};
pub use length::Length;
pub use load::ContentError;
pub use material::Materials;
pub use phrasing::{CORPUS_CAP, Example, Phrasing, Phrasings, Refusal, Span, corpus_scene};
pub use progression::{Catalogue, Milestone, Progression, Station};
pub use prose::Prose;
pub use recipe::{Recipe, Recipes};
pub use siege::{Spendable, Spendables};
pub use spell::{EXTENSION, Spell, Spells, with_extension, without_extension};
