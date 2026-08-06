//! What each instrument turns its contents into (DESIGN.md §10.1).
//!
//! Authored in `content/recipes.toml`, not here (rule 6). This module knows the
//! *shape* of a recipe and nothing about which ones exist.
//!
//! # The material's state chooses the recipe, not the player
//!
//! §10.1: *"a material's **state** decides which recipes can fire — so the
//! material suggests the next operation rather than the player memorising a
//! sequence."* State is carried by the reagent's **name** — `sage`,
//! `ground-sage`, `sage-tincture` — rather than by a separate field. One name
//! per state means the tower's own contents are the state, `survey` shows it
//! without a special view, and there is no second representation to drift.
//!
//! # Replay
//!
//! Unlike [`Prose`](super::Prose), recipes **reach decisions**. Swapping this
//! file mid-session changes what the world does, so `(seed, submissions)` no
//! longer replays to the same world unless the content is versioned with it.
//! Hot-reload is therefore deliberately **not** wired up for recipes; they are
//! read once, at construction.

use std::collections::BTreeMap;

use bevy_ecs::resource::Resource;
use serde::Deserialize;

/// The compiled-in default, so a headless `Sim` needs no filesystem.
const BUILTIN: &str = include_str!("../../content/recipes.toml");

/// One transformation an instrument can perform.
#[derive(Debug, Clone, Deserialize)]
pub struct Recipe {
    /// The single input, for the instruments that take one.
    #[serde(default)]
    input: Option<String>,
    /// The inputs, for `flask_and_rod`, which combines two.
    #[serde(default)]
    inputs: Vec<String>,
    /// What comes out.
    pub output: String,
    /// The byproduct left behind. §10.1: every one of these has a use.
    pub leaves: String,
    /// How long it takes. A placeholder the balance CLI sweeps.
    pub ticks: u64,
    /// Whether the output is a finished potion rather than more crafting stock.
    ///
    /// The line between [`Essence`](crate::parser::NounKind::Essence) and
    /// [`Reagent`](crate::parser::NounKind::Reagent), and the
    /// end of §10.1's pipeline. Written in the content file rather than inferred
    /// from "nothing consumes it", because a byproduct nobody has found a use for
    /// yet would otherwise silently become a potion.
    #[serde(default)]
    pub potion: bool,
    /// Whether this needs the athanor alight (§10.1).
    ///
    /// **In the content file, for the same reason `potion` is.** This was a
    /// `matches!(instrument, "balneum_mariae" | "alembic")` in the executor while
    /// `recipes.toml` recorded the fact one line away as a *comment* — so a
    /// designer adding a sixth heated instrument would have edited the file, seen
    /// the note they had just written, and shipped a recipe that silently runs
    /// cold. Rule 6 puts authored facts in the data; nothing cross-checks a
    /// `matches!` against the TOML beside it.
    ///
    /// Per recipe rather than per instrument, because it is the *process* that
    /// wants heat: an instrument may well gain a cold recipe later.
    #[serde(default)]
    pub heat: bool,
}

impl Recipe {
    /// What this recipe consumes, however it was written.
    ///
    /// `input = "sage"` and `inputs = ["a", "b"]` are the same thing to everyone
    /// downstream; the two spellings exist so a one-in recipe does not have to
    /// be written as a list of one in the content file.
    #[must_use]
    pub fn inputs(&self) -> Vec<&str> {
        if self.inputs.is_empty() {
            self.input.as_deref().into_iter().collect()
        } else {
            self.inputs.iter().map(String::as_str).collect()
        }
    }
}

/// Every instrument's recipes, by instrument name.
#[derive(Debug, Clone, Deserialize, Resource)]
#[serde(transparent)]
pub struct Recipes {
    /// `BTreeMap` for the same reason as [`Prose`](super::Prose): two runs of one
    /// seed must produce the same bytes, and hash order varies per process.
    by_instrument: BTreeMap<String, Vec<Recipe>>,
}

/// The file's name, for an error a writer can act on.
const FILE: &str = "recipes.toml";

impl Recipes {
    /// The recipes compiled into the binary.
    ///
    /// # Panics
    ///
    /// If the built-in file is malformed — a build-time authoring error, covered
    /// by `the_builtin_file_parses`.
    #[must_use]
    pub fn builtin() -> Self {
        super::load::builtin(FILE, BUILTIN)
    }

    /// Parse a recipe file.
    ///
    /// # Errors
    ///
    /// [`ContentError`](super::ContentError) if the text is not valid TOML of the
    /// expected shape.
    pub fn parse(text: &str) -> Result<Self, super::ContentError> {
        super::load::parse(FILE, text)
    }

    /// The first recipe `instrument` can run on exactly `held`.
    ///
    /// Order in the file is the tie-break, matching how §6 resolves a scoring
    /// tie to whatever was registered first — so which recipe fires is a
    /// property of the content, never of iteration luck.
    ///
    /// Matching is on the **multiset** of what is in the instrument: a recipe
    /// wanting `[a, b]` fires on `[b, a]`, and does *not* fire when a third
    /// thing is also in there. Requiring an exact match is what makes clearing
    /// an instrument part of the loop rather than an optional tidy.
    #[must_use]
    pub fn matching(&self, instrument: &str, held: &[String]) -> Option<&Recipe> {
        self.by_instrument.get(instrument)?.iter().find(|recipe| {
            let mut wanted: Vec<&str> = recipe.inputs();
            if wanted.len() != held.len() {
                return false;
            }
            for item in held {
                match wanted.iter().position(|want| *want == item.as_str()) {
                    Some(at) => {
                        wanted.swap_remove(at);
                    }
                    None => return false,
                }
            }
            wanted.is_empty()
        })
    }

    /// Every recipe an instrument knows, for the grimoire.
    #[must_use]
    pub fn for_instrument(&self, instrument: &str) -> &[Recipe] {
        self.by_instrument
            .get(instrument)
            .map_or(&[], Vec::as_slice)
    }

    /// **Every** way to make `output`, in file order.
    ///
    /// More than one is the point rather than an accident: §10.1's exit
    /// criterion is that the same goal has *two different right answers*
    /// depending on what the laboratory is holding, and a single route can only
    /// ever have one. This is what `grimoire` shows so the difference is
    /// readable **before** the player commits an instrument to it.
    #[must_use]
    pub fn routes(&self, output: &str) -> Vec<(&str, &Recipe)> {
        self.by_instrument
            .iter()
            .flat_map(|(instrument, recipes)| {
                recipes
                    .iter()
                    .filter(move |recipe| recipe.output == output)
                    .map(move |recipe| (instrument.as_str(), recipe))
            })
            .collect()
    }

    /// Every distinct output any instrument can produce.
    #[must_use]
    pub fn outputs(&self) -> Vec<&str> {
        let mut out: Vec<&str> = self
            .by_instrument
            .values()
            .flatten()
            .map(|recipe| recipe.output.as_str())
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }
}

impl Default for Recipes {
    fn default() -> Self {
        Self::builtin()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_builtin_file_parses() {
        let recipes = Recipes::builtin();
        assert!(!recipes.for_instrument("mortar_and_pestle").is_empty());
        assert!(!recipes.for_instrument("alembic").is_empty());
    }

    #[test]
    fn a_single_input_recipe_matches_its_reagent() {
        let recipes = Recipes::builtin();
        let held = vec!["sage".to_owned()];
        let recipe = recipes
            .matching("mortar_and_pestle", &held)
            .expect("sage grinds");
        assert_eq!(recipe.output, "ground-sage");
        assert_eq!(recipe.leaves, "husks");
    }

    #[test]
    fn a_two_input_recipe_matches_in_either_order() {
        // What is *in* an instrument has no order a player controls, so a recipe
        // that only fired one way round would be a coin flip on charge order.
        let recipes = Recipes::builtin();
        let forward = vec!["sage-tincture".to_owned(), "ground-salt".to_owned()];
        let backward = vec!["ground-salt".to_owned(), "sage-tincture".to_owned()];
        assert_eq!(
            recipes
                .matching("flask_and_rod", &forward)
                .map(|r| &r.output),
            recipes
                .matching("flask_and_rod", &backward)
                .map(|r| &r.output)
        );
        assert!(recipes.matching("flask_and_rod", &backward).is_some());
    }

    #[test]
    fn a_fouled_instrument_matches_nothing() {
        // Exact-multiset matching is what makes clearing part of §10.1's loop.
        // Leave the husks in and the mortar does not know what you want.
        let recipes = Recipes::builtin();
        let held = vec!["sage".to_owned(), "husks".to_owned()];
        assert!(recipes.matching("mortar_and_pestle", &held).is_none());
    }

    #[test]
    fn an_empty_instrument_matches_nothing() {
        let recipes = Recipes::builtin();
        assert!(recipes.matching("alembic", &[]).is_none());
    }

    #[test]
    fn the_wrong_instrument_matches_nothing() {
        // Sequencing is the puzzle: sage grinds, it does not distil.
        let recipes = Recipes::builtin();
        let held = vec!["sage".to_owned()];
        assert!(recipes.matching("alembic", &held).is_none());
    }

    #[test]
    fn every_byproduct_has_at_least_one_use() {
        // §10.1's rule, as a test. A byproduct that is only ever litter makes
        // `purge` into tidying — the exact feeling this item exists to remove.
        let recipes = Recipes::builtin();
        let mut byproducts: Vec<&str> = recipes
            .by_instrument
            .values()
            .flatten()
            .map(|recipe| recipe.leaves.as_str())
            .collect();
        byproducts.sort_unstable();
        byproducts.dedup();

        for byproduct in byproducts {
            let consumed = recipes
                .by_instrument
                .values()
                .flatten()
                .any(|recipe| recipe.inputs().contains(&byproduct));
            assert!(consumed, "{byproduct} is litter: nothing takes it as input");
        }
    }

    #[test]
    fn every_recipe_has_at_least_one_input() {
        // A recipe with no inputs would fire on an empty instrument for ever.
        let recipes = Recipes::builtin();
        for (instrument, list) in &recipes.by_instrument {
            for recipe in list {
                assert!(
                    !recipe.inputs().is_empty(),
                    "{instrument} has a recipe that consumes nothing"
                );
            }
        }
    }

    #[test]
    fn a_broken_file_is_an_error_rather_than_a_panic() {
        assert!(Recipes::parse("not toml [[[").is_err());
    }
}
