//! Where a thing lives, derived rather than listed.
//!
//! A rule and not a table, because `debug_spawn`'s whole value is that a new
//! item is testable the moment it is authored. A hand-kept list of where each
//! thing goes is a promise kept until somebody is busy, and the person who finds
//! out is the one who assumed the tool was right and went looking for the bug in
//! the game.
//!
//! So it is a rule over the content: add a material to `recipes.toml` and it has
//! a home the same tick, read off the recipes that name it —
//! `every_material_has_a_home_a_move_can_reach` fails the build otherwise.
//!
//! The rule is where the game itself would leave it:
//!
//! 1. Finished work — an [`Essence`](crate::parser::NounKind::Essence) or a
//!    [`Scroll`](crate::parser::NounKind::Scroll) — goes to the arsenal, the
//!    room it is *for* and the one place reachable from every other.
//! 2. Anything a recipe produces belongs to the domain that produces it. Dust is
//!    the archive's leavings even though a mortar will grind it.
//! 3. Anything else — a base reagent, a fuel — belongs to the domain that
//!    consumes it. Nothing makes sage; the laboratory is where it is wanted.
//!
//! ...and within a domain, the store, never an instrument. A shelf is inert, so
//! the thing sits there until it is `move`d, which is what a tester wants.
//! Dropped into an instrument it would charge the tool, and a mortar holding
//! something no recipe wants reads `fouled`.

use bevy_ecs::prelude::*;

use super::node::{Fixture, Name, Store, children_of, root};
use crate::content::{Fuels, Recipes};
use crate::parser::NounKind;

/// Where `named` belongs, or `None` if the tower has nowhere for it.
///
/// `None` is a real answer and not a shrug: it means the content names a
/// material whose domain has no shelf, which is a build-time authoring problem
/// the lint below reports by name.
#[must_use]
pub fn home(world: &World, named: &str) -> Option<Entity> {
    // 1. Finished work keeps itself.
    //
    // A troop is finished work that no recipe makes, the one shape rules 2 and 3
    // cannot see: `kind_of` derives a kind from the recipes producing a name,
    // and a held beast is not a recipe — so a troop reads as a `Reagent` that
    // nothing makes and nothing yet consumes, and the rule answers `None` for a
    // material the game produces every time a beast is held.
    //
    // Named here rather than given a recipe, because inventing a recipe nobody
    // can fire would put a lie in `recall troop`.
    //
    // It also keeps troops out of the verdant unlock: `execute::scroll` derives
    // a base reagent as one the vocabulary knows and nothing makes, whose home
    // is the laboratory's shelf. §19 records that defect shipping once, with
    // dregs and ash on the dispensary.
    if named == crate::execute::TROOP
        || matches!(
            world.resource::<Recipes>().kind_of(named),
            NounKind::Essence | NounKind::Scroll
        )
    {
        return super::keep(world);
    }

    // 2 and 3. The domain that makes it, else the domain that wants it.
    let instrument = making(world, named).or_else(|| wanting(world, named))?;
    store_of(world, &instrument)
}

/// An instrument whose recipes *produce* `named`, as an output or a leaving.
fn making(world: &World, named: &str) -> Option<String> {
    let recipes = world.resource::<Recipes>();
    recipes.instruments().into_iter().find_map(|instrument| {
        recipes
            .for_instrument(instrument)
            .iter()
            .any(|recipe| {
                recipe.outputs().contains(&named) || recipe.leaves.as_deref() == Some(named)
            })
            .then(|| instrument.to_owned())
    })
}

/// An instrument whose recipes *consume* `named` — or which burns it.
fn wanting(world: &World, named: &str) -> Option<String> {
    let recipes = world.resource::<Recipes>();
    let consumed = recipes.instruments().into_iter().find_map(|instrument| {
        recipes
            .for_instrument(instrument)
            .iter()
            .any(|recipe| recipe.inputs().contains(&named))
            .then(|| instrument.to_owned())
    });
    consumed.or_else(|| {
        // Fuel has no recipe, because the athanor transforms nothing — the one
        // instrument with a content file of its own and no entry in
        // `recipes.toml`. Without this arm charcoal has no home at all, and it
        // is the reagent a tester reaches for first.
        world
            .resource::<Fuels>()
            .names()
            .any(|fuel| fuel == named)
            .then(|| super::ATHANOR.to_owned())
    })
}

/// The store standing in the same domain as the fixture called `instrument`.
///
/// Walked from the root because a home is a fact about the world rather than
/// about where the player is: `debug_spawn ground-sage` from the archive still
/// means the laboratory's shelf, which is the whole point of the item having a
/// home at all.
fn store_of(world: &World, instrument: &str) -> Option<Entity> {
    let mut stack = vec![root(world)];
    while let Some(node) = stack.pop() {
        let children = children_of(world, node);
        let holds_it = children.iter().any(|child| {
            world.get::<Fixture>(*child).is_some()
                && world
                    .get::<Name>(*child)
                    .is_some_and(|name| name.0 == instrument)
        });
        if holds_it {
            return children
                .iter()
                .copied()
                .find(|child| world.get::<Store>(*child).is_some());
        }
        stack.extend(children);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Sim;

    /// The leaf name of wherever `named` belongs.
    fn home_of(sim: &Sim, named: &str) -> String {
        let world = sim.world();
        let node = home(world, named).unwrap_or_else(|| panic!("`{named}` has no home"));
        world
            .get::<Name>(node)
            .map_or_else(String::new, |name| name.0.clone())
    }

    #[test]
    fn every_material_has_a_home_a_move_can_reach() {
        // The structural guarantee, and the reason this module is a rule: a
        // material authored tomorrow gets a home from the recipes that name it,
        // and if it does not the build says which one and stops — rather than a
        // tester typing `debug_spawn <thing>` and being told there is nowhere to
        // put it.
        let sim = Sim::new(1);
        let world = sim.world();
        for material in crate::content::Materials::builtin().names() {
            let node = home(world, material)
                .unwrap_or_else(|| panic!("`{material}` is authored and has nowhere to live"));
            // A `Store` or the arsenal, and nothing else: those are exactly the
            // two things `pipeline::reachable` can see into, so anything else
            // would be a home a `move` cannot pick up from.
            assert!(
                world.get::<Store>(node).is_some()
                    || world.get::<super::super::Keep>(node).is_some(),
                "`{material}` lives somewhere no `move` can reach",
            );
        }
        // And the fuels, which have no recipe and so take the other arm.
        for fuel in crate::content::Fuels::builtin().names() {
            assert!(home(world, fuel).is_some(), "`{fuel}` has nowhere to live");
        }
    }

    #[test]
    fn a_thing_lives_in_the_room_that_makes_it() {
        let sim = Sim::new(1);

        // Base stock: nothing makes it, the laboratory wants it.
        assert_eq!(home_of(&sim, "sage"), "dispensary");
        assert_eq!(
            home_of(&sim, "charcoal"),
            "dispensary",
            "fuel has no recipe"
        );

        // Made in the laboratory, so it stays there.
        assert_eq!(home_of(&sim, "ground-sage"), "dispensary");

        // Made in the archive, so it stays there. `fragment` is the archive's
        // only stock now: the lectern's `dust` went with the byproduct mechanic,
        // which §10.1 keeps in the laboratory.
        assert_eq!(home_of(&sim, "fragment"), "cabinet");

        // Finished work keeps itself, wherever it was made.
        assert_eq!(home_of(&sim, "clarity"), super::super::ARSENAL);
        assert_eq!(home_of(&sim, "gleaning-scroll"), super::super::ARSENAL);
    }
}
