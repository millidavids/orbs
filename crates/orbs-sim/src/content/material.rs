//! What colour each material draws in (DESIGN.md §19).
//!
//! Authored in `content/materials.toml`, not here (rule 6). This module knows
//! the *shape* of the table and nothing about which materials exist.
//!
//! A tint is a hint. The instrument panel reads it and nothing else — no
//! recipe, no verb — so it is safe to hot-reload and `(seed, submissions)`
//! still replays identically.
//!
//! A name matching none of [`Tint`]'s eight families fails the *load* rather
//! than drawing the base hue: an untinted material draws in the base hue too,
//! so a fallback would make a typo indistinguishable from an omission.
//! `Recipe::heat` and `craft_of` have each paid for that once.

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
    /// `clarified-draught` is authored as `green` + `bone` rather than some
    /// third colour, so the flask's growing band and the product it leaves
    /// behind resolve through the same [`Wash`] and match by construction. A
    /// third colour would match only while two numbers happened to agree.
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
/// `Default` is the built-in table, not an empty one — a derived `Default`
/// gives an empty map that `Sim` installs with `init_resource`, and every
/// material silently loses its colour. That happened; `shell::panel`'s
/// cross-crate test caught it. `Fuels` next door is hand-written for the same
/// reason.
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
    /// the expected shape, or if any entry names a colour that does not exist —
    /// see this module's header for why that is a load failure.
    pub fn parse(text: &str) -> Result<Self, super::ContentError> {
        let parsed: Self = super::load::parse(FILE, text)?;
        // Both names: a mixture's second colour is as easy to typo as its
        // first, and just as invisible if it falls back.
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

    /// Every material that has been given a colour, alphabetically.
    ///
    /// A `BTreeMap`, so a failure message reads the same every run — the same
    /// reason `Recipes::instruments` exists, and used the same way:
    /// `debug_spawn` checks a tester can hold one of each, so a material no
    /// recipe names cannot have a colour and no way to reach it.
    #[must_use]
    pub fn names(&self) -> Vec<&str> {
        self.by_name.keys().map(String::as_str).collect()
    }

    /// The colour `name` draws in, if it has been given one.
    ///
    /// `None` is *"nobody has decided"* rather than *"this is colourless"*, so
    /// the panel draws it in the base hue.
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
        // `ground-sage` — so the pipeline shows one substance changing. A tint
        // that changed with the state would read as the mortar swapping its
        // contents.
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
        // Two files that nothing else makes agree: the painter averages the
        // flask's two ingredients for the growing band, and the product takes
        // its colour from *this* file. Disagreement is the band turning a
        // different colour the instant it finishes.
        //
        // Only new substances. The recycling route — `dregs` + `sediment` back
        // into `rock-salt` — must *not* follow the rule: salt reclaimed from
        // waste looks like the salt you started with. The data draws the line:
        // a material authored `with` a second colour claims to be a mixture,
        // and that claim has to match the recipe.
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
            // The recipe claims to be a mixture, so a missing colour fails
            // rather than skips. A `filter_map` here once let an untinted input
            // shrink the list, fail the pattern and skip the recipe silently —
            // which is how two tinctures and everything below them shipped
            // with no colour at all.
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
        // from. Same rule, one stage later.
        for recipe in recipes.for_instrument("alembic") {
            let inputs = recipe.inputs();
            let [input] = inputs.as_slice() else {
                continue;
            };
            let [made] = recipe.outputs()[..] else {
                continue;
            };
            // Both or neither, never one quietly. The flask's skip lived here
            // too: an untinted input failed the pair to match and the
            // distillation went unchecked.
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

        // ...and the two draughts *are* mixtures, so the clause above cannot
        // pass by nobody claiming anything.
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
        // Nobody has decided, so the bar draws in the base hue.
        assert_eq!(Materials::builtin().wash("retort"), None);
    }

    #[test]
    fn a_colour_that_does_not_exist_fails_the_load() {
        // Not a fallback: an untinted material draws in the base hue too, so a
        // silent one makes a typo indistinguishable from an omission.
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
