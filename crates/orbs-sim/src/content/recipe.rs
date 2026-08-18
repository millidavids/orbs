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
    /// What comes out, for a recipe that makes one thing.
    #[serde(default)]
    output: Option<String>,
    /// What **may** come out, for a recipe whose product is drawn rather than
    /// fixed.
    ///
    /// The lectern is the only one: four fragments are four fragments however
    /// they were won, so what they assemble into is the archive's roll rather
    /// than a fact about the inputs. Written as a list here rather than as
    /// several recipes because [`matching`](Recipes::matching) returns the
    /// *first* recipe whose inputs match — three `[[lectern]]` blocks all
    /// wanting four fragments would leave two of them permanently unreachable,
    /// and nothing would say so.
    #[serde(default)]
    outputs: Vec<String>,
    /// The byproduct left behind, if the process leaves one. §10.1: every one of
    /// these has a use.
    ///
    /// # Optional, because byproducts are the laboratory's mechanic
    ///
    /// §10.1 builds the whole *waste has a use* loop around brewing — husks
    /// become a weak tincture, dregs and sediment become salt, ash becomes
    /// potash — and it is the laboratory that makes it interesting, because the
    /// laboratory is where a second route to the same draught can exist.
    ///
    /// Replicating it into every other domain buys nothing and costs each one a
    /// substance nobody has decided anything about. The lectern's assembly left
    /// `dust` for exactly one reason — that the field was compulsory — and the
    /// dust then had nowhere to go and nothing to become that `ash` did not
    /// already become.
    ///
    /// So a recipe may leave nothing, and the two ways to say so are the same:
    /// omit the key.
    #[serde(default)]
    pub leaves: Option<String>,
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
    /// Whether the output is a scroll — finished work, like a potion, but read
    /// rather than drunk.
    ///
    /// A third kind rather than a second flavour of `potion`, because the two
    /// are different nouns: an [`Essence`](crate::parser::NounKind::Essence) is
    /// §10.1's *quality* a recipe yields, and a
    /// [`Scroll`](crate::parser::NounKind::Scroll) is an object you spend. A
    /// recipe setting both is refused at load — the output would have to be two
    /// kinds at once, and `kind_of` would answer whichever branch came first.
    #[serde(default)]
    pub scroll: bool,
    /// How many of **each** input this consumes. One unless the file says so.
    ///
    /// # Why a count rather than the input repeated
    ///
    /// `inputs = ["fragment", "fragment", "fragment", "fragment"]` reads like it
    /// should work and cannot: what an instrument holds is a *node per name* with
    /// a [`Stock`](crate::tower::Stock) count on it, so four fragments are one
    /// entry, not four. The alternative — expanding a held stack into one name
    /// per unit — breaks something that already works: an instrument may hold two
    /// sage against a one-sage recipe and still be `charged`, which is what
    /// *"charged a unit at a time, so a run spends a unit"* means.
    ///
    /// So the recipe says how many it wants and the match asks for **at least**
    /// that many. Two sage still fires a one-sage recipe and leaves one behind;
    /// two fragments do not fire a four-fragment one.
    #[serde(default = "one")]
    pub count: u32,
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
    /// Whether the player has to **find** this before it will fire (§10, `lens/`).
    ///
    /// **Content, not player state.** This says a recipe *is* discoverable;
    /// [`Learned`](crate::tower::Learned) says whether it has been discovered.
    /// Keeping them apart is what lets `Recipes` stay immutable — it is loaded
    /// once and deliberately not hot-reloadable, because a recipe reaches a
    /// decision and swapping one mid-session would break replay from
    /// `(seed, submissions)`.
    ///
    /// **Only two questions consult it**: whether a recipe fires
    /// ([`matching`](Recipes::matching), [`gathering`](Recipes::gathering)) and
    /// whether its product is a word the player can say (`tower::scene`).
    /// Everything else here is a *content* query and must stay unfiltered —
    /// `debug_spawn` reaches every material by design, and `execute::scroll`'s
    /// verdant unlock derives base reagents as *"in the vocabulary and made by
    /// nothing"*, so a filtered `outputs` would drop an undiscovered potion out
    /// of "made" and offer it as an endless herb.
    #[serde(default)]
    pub secret: bool,
}

/// The default for [`Recipe::count`]: a recipe wants one of each input.
const fn one() -> u32 {
    1
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

    /// Everything this recipe can produce, however it was written.
    ///
    /// The counterpart of [`inputs`](Self::inputs), and the same two spellings
    /// for the same reason: a recipe that makes one thing says `output = "x"`
    /// rather than a list of one. A recipe that **draws** says `outputs`, and
    /// every name in it is a real product — so this is what the vocabulary, the
    /// scene's topics, `recall`'s routes and `kind_of` all read. Anything asking
    /// *"can this make x"* has to consider all of them or a rolled product would
    /// be a word the parser does not know.
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
    /// [`ContentError`](super::ContentError) if the text is not valid TOML of the
    /// expected shape, **or if a recipe produces nothing, or claims to produce
    /// two kinds of thing at once**. Both parse perfectly and are unusable —
    /// `Materials::parse` refuses an unknown colour for the same reason and in
    /// the same place.
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
            // **Not "whichever branch runs first".** `kind_of` asks `potion`
            // before `scroll`, so a recipe setting both would quietly be an
            // essence and the scroll half would be authored, drawn, and never
            // read — the silent-fallback shape the colour check exists to stop.
            if recipe.potion && recipe.scroll {
                return Err(super::ContentError::new(
                    FILE,
                    format!(
                        "a `{instrument}` recipe is both a potion and a scroll; \
                         its output cannot be two kinds of noun"
                    ),
                ));
            }
        }
        Ok(parsed)
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
    /// `held` is what the instrument contains: each name once, with how many
    /// units of it are there. See [`Recipe::count`] for why the count is carried
    /// rather than the name repeated.
    /// `learned` is what the player has found. A secret recipe they have not
    /// found does not fire, and the instrument reads `fouled` — which is honest:
    /// they are holding two things that make nothing, as far as they know.
    #[must_use]
    pub fn matching(
        &self,
        instrument: &str,
        held: &[(String, u32)],
        learned: &crate::tower::Learned,
    ) -> Option<&Recipe> {
        self.by_instrument.get(instrument)?.iter().find(|recipe| {
            if !self.reachable(recipe, learned) {
                return false;
            }
            let mut wanted: Vec<&str> = recipe.inputs();
            if wanted.len() != held.len() {
                return false;
            }
            for (item, units) in held {
                // **At least, not exactly.** Two sage fires a one-sage recipe and
                // leaves one behind — the behaviour that was already shipping
                // before recipes could ask for more than one.
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
    /// **The complement of [`matching`](Self::matching), and the panel needs
    /// both.** An instrument holding nothing a recipe wants is fouled; one
    /// holding a proper subset of a recipe's inputs is collecting a set, and
    /// telling the player it will not start is exactly wrong. The lectern is the
    /// first instrument this can happen to — it wants four distinct shards, so
    /// three of them matched nothing — but any multi-input recipe has it.
    ///
    /// **Two ways to be part-way there**, and the lectern reaches both: fewer
    /// *kinds* than a recipe names, or every kind but not enough of one.
    #[must_use]
    pub fn gathering(
        &self,
        instrument: &str,
        held: &[(String, u32)],
        learned: &crate::tower::Learned,
    ) -> bool {
        if held.is_empty() {
            return false;
        }
        self.by_instrument.get(instrument).is_some_and(|recipes| {
            recipes.iter().any(|recipe| {
                if !self.reachable(recipe, learned) {
                    return false;
                }
                let mut wanted: Vec<&str> = recipe.inputs();
                if held.len() > wanted.len() {
                    return false;
                }
                // Short of a kind, or short of a count of one — either way the
                // player is collecting rather than holding leavings.
                let short = held.len() < wanted.len()
                    || held.iter().any(|(_, units)| *units < recipe.count);
                short
                    && held.iter().all(|(item, _)| {
                        wanted
                            .iter()
                            .position(|want| *want == item.as_str())
                            .is_some_and(|at| {
                                wanted.swap_remove(at);
                                true
                            })
                    })
            })
        })
    }

    /// Every instrument the recipes name, alphabetically.
    ///
    /// **The authority on what an instrument is**, for content that has to agree
    /// with this file — `progression.toml` prices work by instrument, and a key
    /// there matching nothing here would earn nothing while looking deliberate.
    /// A `BTreeMap`, so the order is stable and an error message reads the same
    /// every run.
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

    /// **Every** way to make `output`, in file order.
    ///
    /// More than one is the point rather than an accident: §10.1's exit
    /// criterion is that the same goal has *two different right answers*
    /// depending on what the laboratory is holding, and a single route can only
    /// ever have one. This is what `recall` shows so the difference is
    /// readable **before** the player commits an instrument to it.
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
    /// **The vocabulary, not the stock.** A reagent's *name* is a fixed fact
    /// about the recipes; whether any is on the shelf right now is not. That
    /// distinction is what `scribe::dropped_argument` needs: the orb must be
    /// able to tell `grind sage` typed when the sage happens to be spent from
    /// `look around`, where the extra word never named anything.
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

    /// Every substance the laboratory has a **word** for.
    ///
    /// The union of [`vocabulary`](Self::vocabulary) and the fuels, rather than
    /// either half: `vocabulary` is every name a recipe can produce or consume
    /// and misses fuel, because the athanor transforms nothing and so has no
    /// recipe — which is exactly the reagent a tester reaches for first.
    ///
    /// **A word, not a thing on a shelf**, and that distinction is the whole
    /// point. `ground-sage` is a word the laboratory knows whether or not any
    /// exists right now, which is what lets the parser tell *"there is none
    /// here"* from *"you have mistyped something"* — see `Scene::knowing`. It was
    /// telling neither, and `digest ground-sage` on an empty shelf quietly
    /// digested **ground-salt** instead: two characters apart in eleven, well
    /// inside the typo band, and a wrong action rather than a refusal (§19).
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
        names.sort_unstable();
        names.dedup();
        names
    }

    /// What kind of noun `name` is when it exists in the world.
    ///
    /// **The rule `produce` applies, asked by name instead of by recipe.** A
    /// finished potion is an [`Essence`](crate::parser::NounKind::Essence) —
    /// §10.1's *quality* a recipe yields — a scroll is a
    /// [`Scroll`](crate::parser::NounKind::Scroll), and everything else is
    /// crafting stock. `produce` knows which because it has the recipe it just
    /// ran in hand; anything working from a name alone (`debug_spawn`) has to
    /// ask.
    ///
    /// A name no recipe produces is stock: that is every input and every
    /// byproduct, which is what most of the vocabulary is.
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
    /// One place, asked by both [`matching`](Self::matching) and
    /// [`gathering`](Self::gathering) — two expressions of one rule disagreeing
    /// is the defect §19 records most often, and here it would show as an
    /// instrument reading `gathering` for a set it will never assemble.
    fn reachable(&self, recipe: &Recipe, learned: &crate::tower::Learned) -> bool {
        !recipe.secret
            || recipe
                .outputs()
                .iter()
                .all(|made| learned.knows(self, made))
    }

    /// Whether a name is one the player has to find before they can make it.
    ///
    /// **Every route to it must be secret.** A name one recipe hides and another
    /// gives away freely is not a secret, and treating it as one would hide a
    /// product the player can already make by the other route — §10.1's *"the
    /// same goal, two right answers"* is a shape the content is built around.
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
    /// The order **is** the reveal sequence — `execute::scry` takes the first
    /// unfound one rather than drawing at random, so `recipes.toml` decides what
    /// a player meets first. §19 makes the same argument for `recall`'s primary
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
    /// **Unfiltered, deliberately** — see [`Recipe::secret`]. This is a content
    /// query: `debug_spawn` reaches every material through it, and
    /// `execute::scroll`'s verdant unlock subtracts it from the vocabulary to
    /// find base reagents, so filtering here would offer an undiscovered potion
    /// as an inexhaustible herb.
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

    /// What the player knows at tick 0 — nothing found, everything ordinary
    /// available. The default, spelled out so a test reads as *a new tower*.
    fn fresh() -> crate::tower::Learned {
        crate::tower::Learned::default()
    }

    #[test]
    fn a_single_input_recipe_matches_its_reagent() {
        let recipes = Recipes::builtin();
        let held = vec![("sage".to_owned(), 1)];
        let recipe = recipes
            .matching("mortar_and_pestle", &held, &fresh())
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
        assert_eq!(
            recipes
                .matching("flask_and_rod", &forward, &fresh())
                .map(Recipe::outputs),
            recipes
                .matching("flask_and_rod", &backward, &fresh())
                .map(Recipe::outputs)
        );
        assert!(
            recipes
                .matching("flask_and_rod", &backward, &fresh())
                .is_some()
        );
    }

    #[test]
    fn a_fouled_instrument_matches_nothing() {
        // Exact-multiset matching is what makes clearing part of §10.1's loop.
        // Leave the husks in and the mortar does not know what you want.
        let recipes = Recipes::builtin();
        let held = vec![("sage".to_owned(), 1), ("husks".to_owned(), 1)];
        assert!(
            recipes
                .matching("mortar_and_pestle", &held, &fresh())
                .is_none()
        );
    }

    #[test]
    fn an_empty_instrument_matches_nothing() {
        let recipes = Recipes::builtin();
        assert!(recipes.matching("alembic", &[], &fresh()).is_none());
    }

    #[test]
    fn a_secret_recipe_does_not_fire_until_it_is_found() {
        // §10's discovery, at its narrowest: the alembic holds potash, the
        // recipe exists, and nothing happens — because the player has not been
        // told it exists.
        let recipes = Recipes::builtin();
        let held = vec![("potash".to_owned(), 1)];
        assert!(
            recipes.matching("alembic", &held, &fresh()).is_none(),
            "a secret fired before it was found",
        );

        let secrets = recipes.secrets();
        assert!(!secrets.is_empty(), "nothing is authored secret");
        for made in &secrets {
            assert!(recipes.is_secret(made), "{made} is not hidden");
        }
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
        assert!(recipes.matching("alembic", &held, &fresh()).is_none());
    }

    #[test]
    fn every_byproduct_has_at_least_one_use() {
        // §10.1's rule, as a test. A byproduct that is only ever litter makes
        // `purge` into tidying — the exact feeling this item exists to remove.
        //
        // **A recipe that leaves nothing is not a recipe that leaves litter**,
        // so it is skipped rather than counted as one. Leaving something is the
        // laboratory's mechanic and no other domain replicates it (see
        // [`Recipe::leaves`]); the rule polices the byproducts that exist, not
        // the absence of one.
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
