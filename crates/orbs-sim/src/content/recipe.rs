//! What each instrument turns its contents into (DESIGN.md §10.1).
//!
//! Authored in `content/recipes.toml`, not here (rule 6). This module knows the
//! *shape* of a recipe and nothing about which ones exist.
//!
//! The material's state chooses the recipe (§10.1), carried by its name —
//! `sage`, `ground-sage` — so `survey` shows it and nothing can drift. Recipes
//! reach decisions, so unlike [`Prose`](super::Prose) they load once.

use std::collections::BTreeMap;

use bevy_ecs::resource::Resource;
use serde::Deserialize;

/// The compiled-in default, so a headless `Sim` needs no filesystem.
const BUILTIN: &str = include_str!("../../content/recipes.toml");

/// What a gathering instrument still lacks — see [`Recipes::missing`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lack {
    /// A kind it holds none of.
    Kind(String),
    /// More of a kind it holds too few of.
    More(String),
}

impl Lack {
    /// The material named.
    #[must_use]
    pub fn material(&self) -> &str {
        match self {
            Self::Kind(name) | Self::More(name) => name,
        }
    }
}

/// One transformation an instrument can perform.
#[derive(Debug, Clone, Deserialize)]
pub struct Recipe {
    /// The single input, for the instruments that take one.
    #[serde(default)]
    input: Option<String>,
    /// The inputs, for `flask_and_rod`, which combines two.
    #[serde(default)]
    inputs: Vec<String>,
    /// What comes out, for a recipe that makes one thing.
    #[serde(default)]
    output: Option<String>,
    /// What *may* come out, for a recipe whose product is drawn rather than
    /// fixed.
    ///
    /// The lectern's: what four fragments make is the archive's roll. A list,
    /// since [`matching`](Recipes::matching) takes the first recipe that
    /// matches and would leave the others unreachable.
    #[serde(default)]
    outputs: Vec<String>,
    /// The byproduct left behind, if the process leaves one. §10.1: every one
    /// of these has a use.
    ///
    /// Optional: *waste has a use* is the laboratory's mechanic, and forcing it
    /// elsewhere invents substances nobody designed. Omit the key to leave
    /// nothing.
    #[serde(default)]
    pub leaves: Option<String>,
    /// How long it takes. A placeholder the balance CLI sweeps.
    pub ticks: u64,
    /// Whether the output is a finished potion rather than more crafting stock.
    ///
    /// The line between [`Essence`](crate::parser::NounKind::Essence) and
    /// [`Reagent`](crate::parser::NounKind::Reagent). Authored, not inferred,
    /// or an unused byproduct becomes a potion.
    #[serde(default)]
    pub potion: bool,
    /// Whether the output is a scroll — finished work, like a potion, but read
    /// rather than drunk.
    ///
    /// A third kind, since a [`Scroll`](crate::parser::NounKind::Scroll) is an
    /// object you spend, not a quality. Setting both is refused at load.
    #[serde(default)]
    pub scroll: bool,
    /// How many of *each* input this consumes. One unless the file says so.
    ///
    /// A count, since an instrument holds a node per name with a
    /// [`Stock`](crate::tower::Stock) count. The match asks for *at least*
    /// that many.
    #[serde(default = "one")]
    pub count: u32,
    /// How many of the output one run makes. The byproduct is always one.
    ///
    /// Experience, renown and the stores still count the run, not the batch
    /// (§19, *"The tower counts runs, not things"*).
    #[serde(default = "one")]
    pub yields: u32,
    /// Whether this needs the athanor alight (§10.1).
    ///
    /// Authored (rule 6), and per recipe, since the process wants heat, not
    /// the instrument.
    #[serde(default)]
    pub heat: bool,
    /// Whether the player has to *find* this before it will fire (§10, `lens/`).
    ///
    /// Content, where [`Learned`](crate::tower::Learned) is player state, so
    /// `Recipes` stays immutable. Consulted only for whether a recipe fires
    /// and whether its product is sayable; every other query stays
    /// unfiltered — see [`outputs`](Recipes::outputs).
    #[serde(default)]
    pub secret: bool,
    /// Quintessence taken when a run starts, on top of the inputs. Stillness is
    /// the one that costs any (§19).
    #[serde(default)]
    pub quintessence: u32,
    /// Whether the player has to *earn* this before it will fire (§11.5).
    ///
    /// A secret is found in the lens; a gated product is earned on a mastery
    /// line (`tower::Opened`). Not both. Per output: `gated = true` gates all,
    /// `gated = ["quickening-scroll"]` one, and a recipe fires if any output
    /// is known.
    #[serde(default)]
    gated: Gate,
}

/// Which of a recipe's outputs are gated: all of them, or a named few.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum Gate {
    /// `gated = true` or absent.
    All(bool),
    /// `gated = ["a", "b"]`.
    Named(Vec<String>),
}

impl Default for Gate {
    fn default() -> Self {
        Self::All(false)
    }
}

impl Gate {
    /// Whether `name` is behind this gate.
    fn holds(&self, name: &str) -> bool {
        match self {
            Self::All(all) => *all,
            Self::Named(names) => names.iter().any(|gated| gated == name),
        }
    }

    /// Whether anything at all is behind it.
    const fn is_any(&self) -> bool {
        match self {
            Self::All(all) => *all,
            Self::Named(names) => !names.is_empty(),
        }
    }
}

/// The default for [`Recipe::count`]: a recipe wants one of each input.
const fn one() -> u32 {
    1
}

impl Recipe {
    /// What this recipe consumes, however it was written.
    ///
    /// `input = "sage"` and `inputs = ["a", "b"]` are one thing downstream.
    #[must_use]
    pub fn inputs(&self) -> Vec<&str> {
        if self.inputs.is_empty() {
            self.input.as_deref().into_iter().collect()
        } else {
            self.inputs.iter().map(String::as_str).collect()
        }
    }

    /// Everything this recipe can produce, however it was written.
    ///
    /// [`inputs`](Self::inputs)' two spellings. Every drawn product is real,
    /// so the vocabulary, topics, routes and `kind_of` all read this.
    #[must_use]
    pub fn outputs(&self) -> Vec<&str> {
        if self.outputs.is_empty() {
            self.output.as_deref().into_iter().collect()
        } else {
            self.outputs.iter().map(String::as_str).collect()
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
    /// [`ContentError`](super::ContentError) if the text is not valid TOML of
    /// the expected shape, if a recipe produces nothing, or if it claims to
    /// produce two kinds of thing at once — both parse and are unusable.
    pub fn parse(text: &str) -> Result<Self, super::ContentError> {
        let parsed: Self = super::load::parse(FILE, text)?;
        for (instrument, recipe) in parsed
            .by_instrument
            .iter()
            .flat_map(|(name, recipes)| recipes.iter().map(move |recipe| (name, recipe)))
        {
            if recipe.outputs().is_empty() {
                return Err(super::ContentError::new(
                    FILE,
                    format!("a `{instrument}` recipe names neither `output` nor `outputs`"),
                ));
            }
            // Not "whichever branch runs first": `kind_of` asks `potion` before
            // `scroll`, so a recipe setting both would quietly be an essence
            // and the scroll half would be authored, drawn and never read.
            if recipe.potion && recipe.scroll {
                return Err(super::ContentError::new(
                    FILE,
                    format!(
                        "a `{instrument}` recipe is both a potion and a scroll; \
                         its output cannot be two kinds of noun"
                    ),
                ));
            }
            // Found or earned, never both — `Learned` and `Opened` would each
            // answer for it and disagree.
            if recipe.secret && recipe.gated.is_any() {
                return Err(super::ContentError::new(
                    FILE,
                    format!(
                        "a `{instrument}` recipe is both secret and gated; \
                         a product is found or earned, not both"
                    ),
                ));
            }
            if let Gate::Named(names) = &recipe.gated
                && let Some(stray) = names
                    .iter()
                    .find(|name| !recipe.outputs().contains(&name.as_str()))
            {
                return Err(super::ContentError::new(
                    FILE,
                    format!("a `{instrument}` recipe gates `{stray}`, which it does not make"),
                ));
            }
            if recipe.yields == 0 {
                return Err(super::ContentError::new(
                    FILE,
                    format!("a `{instrument}` recipe yields nothing"),
                ));
            }
        }
        Ok(parsed)
    }

    /// The first recipe `instrument` can run on exactly `held`.
    ///
    /// See also [`missing`](Self::missing), for what a partial load lacks.
    ///
    /// File order is the tie-break. On the multiset held, so a third thing
    /// stops it — clearing is part of the loop. `held` is a name and count
    /// ([`Recipe::count`]); a recipe not `known` does not fire and the
    /// instrument reads `fouled`, which is honest.
    #[must_use]
    pub fn matching(
        &self,
        instrument: &str,
        held: &[(String, u32)],
        known: &crate::tower::Known<'_>,
    ) -> Option<&Recipe> {
        self.by_instrument.get(instrument)?.iter().find(|recipe| {
            if !self.reachable(recipe, known) {
                return false;
            }
            let mut wanted: Vec<&str> = recipe.inputs();
            if wanted.len() != held.len() {
                return false;
            }
            for (item, units) in held {
                // At least, not exactly: two sage fires a one-sage recipe and
                // leaves one behind, the behaviour already shipping before
                // recipes could ask for more than one.
                if *units < recipe.count {
                    return false;
                }
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

    /// Whether what is held is *part* of a recipe rather than none of one.
    ///
    /// [`matching`](Self::matching)'s complement, for the panel: a proper
    /// subset is collecting, not fouled — fewer kinds, or too few of one.
    #[must_use]
    pub fn gathering(
        &self,
        instrument: &str,
        held: &[(String, u32)],
        known: &crate::tower::Known<'_>,
    ) -> bool {
        self.missing(instrument, held, known).is_some()
    }

    /// What a gathering instrument still lacks, by the first recipe it is
    /// collecting for. [`gathering`](Self::gathering) is this being `Some`.
    #[must_use]
    pub fn missing(
        &self,
        instrument: &str,
        held: &[(String, u32)],
        known: &crate::tower::Known<'_>,
    ) -> Option<Lack> {
        if held.is_empty() {
            return None;
        }
        self.by_instrument
            .get(instrument)?
            .iter()
            .find_map(|recipe| {
                if !self.reachable(recipe, known) {
                    return None;
                }
                let inputs = recipe.inputs();
                let mut wanted = inputs.clone();
                if held.len() > wanted.len() {
                    return None;
                }
                let all_wanted = held.iter().all(|(item, _)| {
                    wanted
                        .iter()
                        .position(|want| *want == item.as_str())
                        .is_some_and(|at| {
                            wanted.swap_remove(at);
                            true
                        })
                });
                if !all_wanted {
                    return None;
                }
                // Short of a kind, or short of a count of one — either way the
                // player is collecting rather than holding leavings. What is left
                // of `wanted` is the kinds not held. Named in the recipe's own
                // order, so the answer does not move between ticks.
                if let Some(kind) = inputs.iter().find(|input| wanted.contains(input)) {
                    return Some(Lack::Kind((*kind).to_owned()));
                }
                inputs.iter().find_map(|input| {
                    held.iter()
                        .any(|(item, units)| item == input && *units < recipe.count)
                        .then(|| Lack::More((*input).to_owned()))
                })
            })
    }

    /// Every instrument the recipes name, alphabetically.
    ///
    /// The authority on what an instrument is, which `progression.toml`'s keys
    /// are checked against. Sorted, so an error reads the same every run.
    #[must_use]
    pub fn instruments(&self) -> Vec<&str> {
        self.by_instrument.keys().map(String::as_str).collect()
    }

    /// Every recipe an instrument knows, for the manual.
    #[must_use]
    pub fn for_instrument(&self, instrument: &str) -> &[Recipe] {
        self.by_instrument
            .get(instrument)
            .map_or(&[], Vec::as_slice)
    }

    /// *Every* way to make `output`, in file order.
    ///
    /// More than one is §10.1's point: the same goal, two right answers, which
    /// `recall` shows.
    #[must_use]
    pub fn routes(&self, output: &str) -> Vec<(&str, &Recipe)> {
        self.by_instrument
            .iter()
            .flat_map(|(instrument, recipes)| {
                recipes
                    .iter()
                    .filter(move |recipe| recipe.outputs().contains(&output))
                    .map(move |recipe| (instrument.as_str(), recipe))
            })
            .collect()
    }

    /// Every reagent name the laboratory knows, consumed or produced.
    ///
    /// The vocabulary, not the stock: a name is fixed whether or not any is on
    /// the shelf.
    #[must_use]
    pub fn vocabulary(&self) -> Vec<&str> {
        let mut out: Vec<&str> = self
            .by_instrument
            .values()
            .flatten()
            .flat_map(|recipe| {
                recipe
                    .outputs()
                    .into_iter()
                    .chain(recipe.inputs())
                    .chain(recipe.leaves.as_deref())
            })
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    /// Every substance the laboratory has a *word* for.
    ///
    /// [`vocabulary`](Self::vocabulary) and the fuels, which have no recipe. A
    /// word, not a thing on a shelf, so the parser tells *none here* from a
    /// typo (`Scene::knowing`, §19).
    #[must_use]
    pub fn substances(world: &bevy_ecs::world::World) -> Vec<String> {
        let mut names: Vec<String> = world
            .resource::<Self>()
            .vocabulary()
            .into_iter()
            .map(str::to_owned)
            .collect();
        names.extend(
            world
                .resource::<crate::content::Fuels>()
                .names()
                .map(str::to_owned),
        );
        // And what the circle binds outside a recipe (§19, *Supply*): a demon
        // is finished work no instrument makes, invisible to `vocabulary` for
        // charcoal's reason.
        names.extend(
            world
                .resource::<crate::content::Spendables>()
                .demons()
                .map(str::to_owned),
        );
        names.sort_unstable();
        names.dedup();
        names
    }

    /// What kind of noun `name` is when it exists in the world.
    ///
    /// `produce`'s rule, asked by name: a potion is an
    /// [`Essence`](crate::parser::NounKind::Essence), a scroll a
    /// [`Scroll`](crate::parser::NounKind::Scroll), everything else stock.
    #[must_use]
    pub fn kind_of(&self, name: &str) -> crate::parser::NounKind {
        let made_by = |wanted: fn(&Recipe) -> bool| {
            self.by_instrument
                .values()
                .flatten()
                .any(|recipe| recipe.outputs().contains(&name) && wanted(recipe))
        };
        if made_by(|recipe| recipe.potion) {
            crate::parser::NounKind::Essence
        } else if made_by(|recipe| recipe.scroll) {
            crate::parser::NounKind::Scroll
        } else {
            crate::parser::NounKind::Reagent
        }
    }

    /// Whether the player may fire this recipe at all.
    ///
    /// One rule for [`matching`](Self::matching) and
    /// [`gathering`](Self::gathering). Any output known, not every one, or the
    /// lectern waits for its last scroll.
    fn reachable(&self, recipe: &Recipe, known: &crate::tower::Known<'_>) -> bool {
        recipe.outputs().iter().any(|made| known.knows(self, made))
    }

    /// Whether a name is one the player has to earn before they can make it.
    ///
    /// Every route to it must gate it, for the reason
    /// [`is_secret`](Self::is_secret) gives: a product one recipe gates and
    /// another gives away is not gated.
    #[must_use]
    pub fn is_gated(&self, name: &str) -> bool {
        let mut routes = self
            .by_instrument
            .values()
            .flatten()
            .filter(|recipe| recipe.outputs().contains(&name))
            .peekable();
        routes.peek().is_some() && routes.all(|recipe| recipe.gated.holds(name))
    }

    /// Every product that has to be earned, in the file's own order.
    ///
    /// What `progression.toml`'s `recipe:` keys are checked against, and what an
    /// open tower holds — see `tower::Opened`.
    #[must_use]
    pub fn gated(&self) -> Vec<&str> {
        let mut out: Vec<&str> = Vec::new();
        for recipe in self.by_instrument.values().flatten() {
            for made in recipe.outputs() {
                if recipe.gated.holds(made) && self.is_gated(made) && !out.contains(&made) {
                    out.push(made);
                }
            }
        }
        out
    }

    /// Whether a name is one the player has to find before they can make it.
    ///
    /// Every route to it must be secret, or it is not one (§10.1).
    #[must_use]
    pub fn is_secret(&self, name: &str) -> bool {
        let mut routes = self
            .by_instrument
            .values()
            .flatten()
            .filter(|recipe| recipe.outputs().contains(&name))
            .peekable();
        routes.peek().is_some() && routes.all(|recipe| recipe.secret)
    }

    /// Every product that has to be found, in the file's own order.
    ///
    /// The order *is* the reveal sequence — `execute::scry` takes the first
    /// unfound one rather than drawing at random, so `recipes.toml` decides
    /// what a player meets first. §19 argues the same for `recall`'s primary
    /// route.
    #[must_use]
    pub fn secrets(&self) -> Vec<&str> {
        let mut out: Vec<&str> = Vec::new();
        for recipe in self.by_instrument.values().flatten() {
            if !recipe.secret {
                continue;
            }
            for made in recipe.outputs() {
                if self.is_secret(made) && !out.contains(&made) {
                    out.push(made);
                }
            }
        }
        out
    }

    /// Every distinct output any instrument can produce.
    ///
    /// Unfiltered by [`Recipe::secret`]: `debug_spawn` reaches every material
    /// through it, and the verdant unlock subtracts it to find base reagents,
    /// so filtering would grow an undiscovered potion as an herb.
    #[must_use]
    pub fn outputs(&self) -> Vec<&str> {
        let mut out: Vec<&str> = self
            .by_instrument
            .values()
            .flatten()
            .flat_map(Recipe::outputs)
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

    /// What the player knows at tick 0 — nothing found, nothing earned,
    /// everything ordinary available. Spelled out so a test reads as *a new,
    /// sealed tower*.
    fn fresh() -> (crate::tower::Learned, crate::tower::Opened) {
        (
            crate::tower::Learned::default(),
            crate::tower::Opened::default(),
        )
    }

    #[test]
    fn a_single_input_recipe_matches_its_reagent() {
        let recipes = Recipes::builtin();
        let held = vec![("sage".to_owned(), 1)];
        let (learned, opened) = fresh();
        let known = crate::tower::Known::new(&learned, &opened);
        let recipe = recipes
            .matching("mortar_and_pestle", &held, &known)
            .expect("sage grinds");
        assert_eq!(recipe.outputs(), ["ground-sage"]);
        assert_eq!(recipe.leaves.as_deref(), Some("husks"));
    }

    #[test]
    fn a_two_input_recipe_matches_in_either_order() {
        // What is *in* an instrument has no order a player controls, so a recipe
        // that only fired one way round would be a coin flip on charge order.
        let recipes = Recipes::builtin();
        let forward = vec![
            ("sage-tincture".to_owned(), 1),
            ("ground-salt".to_owned(), 1),
        ];
        let backward = vec![
            ("ground-salt".to_owned(), 1),
            ("sage-tincture".to_owned(), 1),
        ];
        let (learned, opened) = fresh();
        let known = crate::tower::Known::new(&learned, &opened);
        assert_eq!(
            recipes
                .matching("flask_and_rod", &forward, &known)
                .map(Recipe::outputs),
            recipes
                .matching("flask_and_rod", &backward, &known)
                .map(Recipe::outputs)
        );
        assert!(
            recipes
                .matching("flask_and_rod", &backward, &known)
                .is_some()
        );
    }

    #[test]
    fn a_fouled_instrument_matches_nothing() {
        // Exact-multiset matching is what makes clearing part of §10.1's loop.
        // Leave the husks in and the mortar does not know what you want.
        let recipes = Recipes::builtin();
        let held = vec![("sage".to_owned(), 1), ("husks".to_owned(), 1)];
        let (learned, opened) = fresh();
        let known = crate::tower::Known::new(&learned, &opened);
        assert!(
            recipes
                .matching("mortar_and_pestle", &held, &known)
                .is_none()
        );
    }

    #[test]
    fn an_empty_instrument_matches_nothing() {
        let recipes = Recipes::builtin();
        let (learned, opened) = fresh();
        let known = crate::tower::Known::new(&learned, &opened);
        assert!(recipes.matching("alembic", &[], &known).is_none());
    }

    #[test]
    fn a_secret_recipe_does_not_fire_until_it_is_found() {
        // §10's discovery, at its narrowest: the alembic holds potash, the
        // recipe exists, and nothing happens — because the player has not been
        // told it exists.
        let recipes = Recipes::builtin();
        let held = vec![("potash".to_owned(), 1)];
        let (learned, opened) = fresh();
        let known = crate::tower::Known::new(&learned, &opened);
        assert!(
            recipes.matching("alembic", &held, &known).is_none(),
            "a secret fired before it was found",
        );

        let secrets = recipes.secrets();
        assert!(!secrets.is_empty(), "nothing is authored secret");
        for made in &secrets {
            assert!(recipes.is_secret(made), "{made} is not hidden");
            assert!(!recipes.is_gated(made), "{made} is both found and earned");
        }
    }

    #[test]
    fn a_gated_recipe_does_not_fire_until_it_is_earned() {
        // §11.5's other door: warding is authored gated, so a sealed tower
        // holding its draught makes nothing — and an open one makes it.
        let recipes = Recipes::builtin();
        let gated = recipes.gated();
        assert!(
            gated.contains(&"warding"),
            "warding is not gated: {gated:?}"
        );
        for made in &gated {
            assert!(recipes.is_gated(made), "{made} is not gated on every route");
            assert!(!recipes.is_secret(made), "{made} is both earned and found");
        }

        let held = vec![("fixed-draught".to_owned(), 1)];
        let (learned, opened) = fresh();
        let known = crate::tower::Known::new(&learned, &opened);
        assert!(
            recipes.matching("alembic", &held, &known).is_none(),
            "a gated recipe fired before it was earned",
        );
        let all = crate::tower::Opened::all(
            &recipes,
            &super::super::Charms::builtin(),
            &super::super::Spendables::builtin(),
        );
        let known = crate::tower::Known::new(&learned, &all);
        assert!(
            recipes.matching("alembic", &held, &known).is_some(),
            "an earned recipe did not fire",
        );
    }

    #[test]
    fn a_recipe_that_draws_fires_once_any_of_its_outputs_is_known() {
        // The lectern is one recipe with three outputs, two of them gated. A
        // recipe-level gate would seal the third; per output, the sealed tower
        // still assembles a gleaning scroll.
        let recipes = Recipes::builtin();
        let held = vec![("fragment".to_owned(), 4)];
        let (learned, opened) = fresh();
        let known = crate::tower::Known::new(&learned, &opened);
        assert!(
            recipes.matching("lectern", &held, &known).is_some(),
            "the lectern was sealed by its gated outputs",
        );
        assert!(recipes.is_gated("quickening-scroll"));
        assert!(!recipes.is_gated("gleaning-scroll"));
    }

    #[test]
    fn a_recipe_cannot_be_both_secret_and_gated_nor_gate_what_it_does_not_make() {
        assert!(
            Recipes::parse(
                "[[alembic]]\ninput = \"x\"\noutput = \"y\"\nticks = 1\nsecret = true\ngated = true\n"
            )
            .is_err(),
            "a recipe both found and earned was accepted",
        );
        assert!(
            Recipes::parse(
                "[[lectern]]\ninput = \"x\"\noutputs = [\"y\", \"z\"]\nticks = 1\ngated = [\"w\"]\n"
            )
            .is_err(),
            "a gate on a product the recipe does not make was accepted",
        );
        assert!(
            Recipes::parse(
                "[[lectern]]\ninput = \"x\"\noutputs = [\"y\", \"z\"]\nticks = 1\ngated = [\"z\"]\n"
            )
            .is_ok(),
        );
    }

    #[test]
    fn an_ordinary_recipe_is_never_treated_as_secret() {
        // The other half, and the one that would break the whole laboratory if
        // it slipped: everything shipped before Phase 2 must still fire on a
        // tower that has found nothing.
        let recipes = Recipes::builtin();
        for made in [
            "ground-sage",
            "sage-tincture",
            "clarified-draught",
            "clarity",
        ] {
            assert!(!recipes.is_secret(made), "{made} became a secret");
        }
    }

    #[test]
    fn the_wrong_instrument_matches_nothing() {
        // Sequencing is the puzzle: sage grinds, it does not distil.
        let recipes = Recipes::builtin();
        let held = vec![("sage".to_owned(), 1)];
        let (learned, opened) = fresh();
        let known = crate::tower::Known::new(&learned, &opened);
        assert!(recipes.matching("alembic", &held, &known).is_none());
    }

    #[test]
    fn every_byproduct_has_at_least_one_use() {
        // §10.1's rule: a byproduct only ever litter makes `purge` tidying. A
        // recipe that leaves nothing is skipped (see [`Recipe::leaves`]).
        let recipes = Recipes::builtin();
        let mut byproducts: Vec<&str> = recipes
            .by_instrument
            .values()
            .flatten()
            .filter_map(|recipe| recipe.leaves.as_deref())
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
