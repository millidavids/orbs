//! What colour each material draws in (DESIGN.md §19).
//!
//! Authored in `content/materials.toml`, not here (rule 6). This module knows
//! the *shape* of the table and nothing about which materials exist.
//!
//! # It is a hint, and the sim treats it as one
//!
//! Nothing here reaches a decision. A tint is read by the instrument panel and
//! by nothing else — no recipe consults it, no verb branches on it — which is
//! what separates this file from [`Recipes`](super::Recipes) and is why it is
//! **safe to hot-reload**: swapping it mid-session changes what the screen looks
//! like and cannot change what the world does, so `(seed, submissions)` still
//! replays identically.
//!
//! # An unknown colour is an error
//!
//! The eight families are fixed in [`Tint`] and this file selects one by name.
//! A name that matches none of them fails the *load*, rather than quietly
//! drawing the base hue — because a material nobody has tinted yet draws in the
//! base hue too, so a silent fallback would make a typo indistinguishable from
//! an omission. That is the defect `Recipe::heat` and `craft_of` have each
//! already paid for once.

use std::collections::BTreeMap;

use bevy_ecs::resource::Resource;
use orbs_render::{Tint, Wash};
use serde::Deserialize;

/// The compiled-in default, so a headless `Sim` needs no filesystem.
const BUILTIN: &str = include_str!("../../content/materials.toml");

/// The file's name, for an error a writer can act on.
const FILE: &str = "materials.toml";

/// What one material looks like.
#[derive(Debug, Clone, Deserialize)]
struct Entry {
    /// One of [`Tint`]'s eight names.
    tint: String,
    /// A second, for a material that **is** a mixture.
    ///
    /// `clarified-draught` is what sage tincture and ground salt become, and it
    /// is authored as `green` + `bone` rather than as some third colour — so the
    /// flask's growing band and the product it leaves behind resolve through the
    /// same [`Wash`] and are the same colour by construction. Naming a third
    /// colour would make them equal only for as long as two numbers happened to
    /// agree, and the jump when they stopped would be a player watching a
    /// mixture form and then change into something else.
    #[serde(default)]
    with: Option<String>,
}

impl Entry {
    /// Every colour name this entry mentions.
    fn names(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.tint.as_str()).chain(self.with.as_deref())
    }
}

/// Every tinted material, by name.
///
/// **`Default` is the built-in table, not an empty one** — deliberately, and
/// `Fuels` next door has the same hand-written impl for the same reason. A
/// derived `Default` gives an empty map, `Sim` installs it with
/// `init_resource`, and every material silently loses its colour while the file
/// on disk is perfectly correct. That is exactly what happened here, and the
/// cross-crate test in `shell::panel` is what caught it.
#[derive(Debug, Clone, Deserialize, Resource)]
#[serde(transparent)]
pub struct Materials {
    by_name: BTreeMap<String, Entry>,
}

impl Default for Materials {
    fn default() -> Self {
        Self::builtin()
    }
}

impl Materials {
    /// The material table compiled into the binary.
    ///
    /// # Panics
    ///
    /// If the built-in file is malformed — a build-time authoring error, covered
    /// by `the_builtin_file_parses`.
    #[must_use]
    pub fn builtin() -> Self {
        super::load::builtin(FILE, BUILTIN)
    }

    /// Parse a material file.
    ///
    /// # Errors
    ///
    /// [`ContentError`](super::ContentError) if the text is not valid TOML of
    /// the expected shape, **or if any entry names a colour that does not
    /// exist**. See this module's header for why the second is a load failure
    /// rather than a fallback.
    pub fn parse(text: &str) -> Result<Self, super::ContentError> {
        let parsed: Self = super::load::parse(FILE, text)?;
        // **Both names, not just the first.** A mixture's second colour is as
        // easy to typo as its first and just as invisible if it falls back.
        if let Some((name, colour)) = parsed
            .by_name
            .iter()
            .flat_map(|(name, entry)| entry.names().map(move |colour| (name, colour)))
            .find(|(_, colour)| Tint::from_name(colour).is_none())
        {
            return Err(super::ContentError::new(
                FILE,
                format!(
                    "`{name}` is tinted `{colour}`, which is not a colour. One of: {}",
                    Tint::ALL
                        .iter()
                        .map(|tint| tint.name())
                        .collect::<Vec<_>>()
                        .join(", "),
                ),
            ));
        }
        Ok(parsed)
    }

    /// The colour `name` draws in, if it has been given one.
    ///
    /// `None` is *"nobody has decided"* rather than *"this is colourless"*, and
    /// the panel draws it in the base hue — which is what an untinted material
    /// has always looked like.
    /// Every material that has been given a colour, alphabetically.
    ///
    /// **What the game has, against what any other list claims it has.** A
    /// `BTreeMap`, so the order is stable and a failure message reads the same
    /// every run — the same reason `Recipes::instruments` exists, and it is used
    /// the same way: `debug_spawn` checks that a tester can hold one of each of
    /// these, and a material no recipe names would otherwise have a colour, a
    /// manual route and no way to reach it.
    #[must_use]
    pub fn names(&self) -> Vec<&str> {
        self.by_name.keys().map(String::as_str).collect()
    }

    /// The colour `name` draws in, if it has been given one.
    ///
    /// `None` is *"nobody has decided"* rather than *"this is colourless"*, and
    /// the panel draws it in the base hue — which is what an untinted material
    /// has always looked like.
    #[must_use]
    pub fn wash(&self, name: &str) -> Option<Wash> {
        let entry = self.by_name.get(name)?;
        let tint = Tint::from_name(&entry.tint)?;
        Some(match entry.with.as_deref().and_then(Tint::from_name) {
            Some(second) => Wash::mixing(tint, second),
            None => Wash::plain(tint),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_builtin_file_parses() {
        let materials = Materials::builtin();
        assert_eq!(materials.wash("sage"), Some(Wash::plain(Tint::Green)));
        assert_eq!(materials.wash("husks"), Some(Wash::plain(Tint::Brown)));
    }

    #[test]
    fn every_builtin_colour_is_a_real_one() {
        // The load-time check, exercised on the file that actually ships. A
        // typo here would otherwise reach a player as a bar in the base hue.
        assert!(
            Materials::parse(BUILTIN).is_ok(),
            "the shipped table names a colour that does not exist",
        );
    }

    #[test]
    fn a_material_stays_the_same_colour_through_its_states() {
        // §10.1 carries a material's *state* in its name — `sage`,
        // `ground-sage` — so the pipeline shows one substance changing rather
        // than several. A tint that changed with the state would say the mortar
        // had swapped its contents for something else, which is the opposite of
        // what the picture is for.
        let materials = Materials::builtin();
        for (raw, worked) in [("sage", "ground-sage"), ("rock-salt", "ground-salt")] {
            assert_eq!(
                materials.wash(raw),
                materials.wash(worked),
                "{raw} changes colour when it is worked",
            );
        }
    }

    #[test]
    fn the_flasks_products_are_the_colour_of_what_makes_them() {
        // **Two files that have to agree, and nothing else makes them.** The
        // flask's growing band is the average of its two ingredients, computed
        // by the painter; the product it leaves behind takes its colour from
        // *this* file. If they disagree, a player watches a mixture form over
        // eight ticks and then sees it turn into a different colour the instant
        // it finishes — which is the panel contradicting itself.
        //
        // **Only the outputs that are *new substances*.** The flask also runs a
        // recycling route — `dregs` + `sediment` back into `rock-salt` — and that
        // one must **not** follow the rule: rock salt is a reagent the game
        // already has, mined and named, and salt reclaimed from waste has to look
        // like the salt you started with. Reconstituting a known substance is not
        // the same act as compounding a new one, and only the second is what the
        // flask's picture is about.
        //
        // The line is drawn by the data itself: a material authored `with` a
        // second colour is claiming to *be* a mixture, and that claim has to
        // match the recipe. One that is not simply has an identity of its own.
        let materials = Materials::builtin();
        let recipes = super::super::Recipes::builtin();
        let mut checked = 0;
        for recipe in recipes.for_instrument("flask_and_rod") {
            // A drawing recipe has no single product to check a colour against,
            // and the flask has none — the lectern is the only one that draws.
            let [made] = recipe.outputs()[..] else {
                continue;
            };
            let named = recipe.inputs();
            let [left, right] = named.as_slice() else {
                continue;
            };
            let Some(output) = materials.wash(made) else {
                continue;
            };
            if output.with.is_none() {
                continue;
            }
            // **From here the recipe *claims* to be a mixture, so a missing
            // colour is a failure and not a reason to look away.** This used to
            // `filter_map` the inputs and destructure two out of the result, so
            // an untinted input shrank the list, the pattern failed, and the
            // whole recipe was skipped **silently**. That is how
            // `mugwort-tincture` and `valerian-tincture` shipped with no colour
            // at all: the omission did not fail this check, it switched the check
            // off — for `keen-draught`, `quiet-draught` and both potions beneath
            // them. A lint that cannot fail is worse than no lint.
            let first = materials.wash(left).unwrap_or_else(|| {
                panic!("`{made}` is a mixture of `{left}`, which has no colour")
            });
            let second = materials.wash(right).unwrap_or_else(|| {
                panic!("`{made}` is a mixture of `{right}`, which has no colour")
            });
            checked += 1;
            assert_eq!(
                output,
                Wash::mixing(first.tint, second.tint),
                "`{made}` claims to be a mixture but is not the colour of the \
                 {:?} it is made from",
                recipe.inputs(),
            );
        }
        assert!(checked > 0, "no combining recipe was actually checked");

        // The alembic's turn: a potion is the colour of the draught it came
        // from. Same rule, one stage later — an essence that changed to an
        // unrelated colour the instant it finished would be the discontinuity
        // the flask's mixtures exist to remove.
        for recipe in recipes.for_instrument("alembic") {
            let inputs = recipe.inputs();
            let [input] = inputs.as_slice() else {
                continue;
            };
            let [made] = recipe.outputs()[..] else {
                continue;
            };
            // **Both or neither, and never one quietly.** The same skip that hid
            // the flask's missing tints lived here too: an untinted input made
            // the pair fail to match and the distillation went unchecked. A
            // distillation whose draught has a colour must produce one, and one
            // whose draught has none is a material nobody has tinted — which is
            // the other lint's business, not a reason to skip this one.
            let (from, into) = match (materials.wash(input), materials.wash(made)) {
                (Some(from), Some(into)) => (from, into),
                (Some(_), None) => {
                    panic!("`{made}` is distilled from the coloured `{input}` and has no colour")
                }
                (None, Some(_)) => {
                    panic!("`{made}` has a colour and the `{input}` it comes from has none")
                }
                (None, None) => continue,
            };
            assert_eq!(
                from, into,
                "`{made}` is not the colour of the `{input}` it was distilled from",
            );
        }

        // ...and the two draughts *are* mixtures, so the clause above cannot be
        // satisfied by nobody claiming anything.
        for draught in ["clarified-draught", "fixed-draught"] {
            assert!(
                materials
                    .wash(draught)
                    .is_some_and(|wash| wash.with.is_some()),
                "`{draught}` stopped being authored as a mixture — the flask's \
                 band and the product it leaves would drift apart",
            );
        }
    }

    #[test]
    fn an_untinted_material_is_not_an_error() {
        // The honest default: nobody has decided, so the bar draws in the base
        // hue exactly as it did before this file existed.
        assert_eq!(Materials::builtin().wash("retort"), None);
    }

    #[test]
    fn a_colour_that_does_not_exist_fails_the_load() {
        // **Not a fallback.** A material nobody has tinted also draws in the
        // base hue, so a silent one would make a typo indistinguishable from an
        // omission — and the writer would never find out.
        let broken = Materials::parse("[sage]\ntint = \"chartreuse\"\n");
        let error = broken.expect_err("an invented colour should not load");
        let message = error.to_string();
        assert!(message.contains("sage"), "{message}");
        assert!(message.contains("chartreuse"), "{message}");
        assert!(message.contains("green"), "it should list the real ones");
    }

    #[test]
    fn a_broken_file_is_an_error_rather_than_a_panic() {
        assert!(Materials::parse("not toml [[[").is_err());
    }
}
