//! What a verdant draw does: cultivation (§19, *Supply*).
//!
//! Each draw grows the next herb the laboratory has never held, and at the
//! authored threshold it flowers on the shelf for good, endless like sage. The
//! growth is the tower's [`Cultivation`](tower::Cultivation), so a supply that
//! runs dry keeps it.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use super::spend::{SAGE, refuse};
use crate::content::{Prose, Recipes};
use crate::session::Scrollback;
use crate::tower::{self, supply::Supplies};

/// Whether there is a herb left to grow.
pub(super) fn growing(world: &World) -> bool {
    tower::home(world, SAGE).is_some_and(|shelf| !locked(world, shelf).is_empty())
}

/// Grow the next herb by `worth`, flowering each one that reaches
/// `flowers_at`, and roll the supply up once nothing is left to grow.
pub(super) fn grow(world: &mut World, name: &str, worth: u32, flowers_at: u32) {
    let flowered = world
        .resource_mut::<tower::Cultivation>()
        .grow(worth, flowers_at);
    for _ in 0..flowered {
        if !flower(world, name) {
            break;
        }
    }
    if !growing(world) {
        world.resource_mut::<tower::Cultivation>().restore(0);
        world.resource_mut::<Supplies>().close(name);
        refuse(world, name, "scroll_all_flowered");
    }
}

/// Put the next base reagent the laboratory has never had on its shelf.
/// Returns whether one flowered.
///
/// Alphabetical, from `Recipes::vocabulary`, so no second roll is needed.
fn flower(world: &mut World, name: &str) -> bool {
    let Some(shelf) = tower::home(world, SAGE) else {
        return false;
    };
    let Some(herb) = locked(world, shelf).first().cloned() else {
        return false;
    };

    let kind = world.resource::<Recipes>().kind_of(&herb);
    tower::give_endless(world, shelf, &herb, kind);

    let message = world
        .resource::<Prose>()
        .line("scroll_verdant", &[("name", name), ("detail", &herb)]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, name)
        .text(FieldName::State, &herb)
        .text(FieldName::At, "dispensary")
        .text(FieldName::Message, &message)
        .role(Role::Success)
        .finish();
    true
}

/// Base reagents the laboratory has not been given yet, alphabetically.
///
/// *Base*: known to the vocabulary and made by nothing — not an output, a
/// leaving, a fuel or ash. *The laboratory's*, by [`tower::home`]. Held is
/// asked of the shelf's true names, or a swap regrows sage.
fn locked(world: &World, shelf: Entity) -> Vec<String> {
    let recipes = world.resource::<Recipes>();
    let fuels = world.resource::<crate::content::Fuels>();

    let mut made: Vec<String> = recipes.outputs().into_iter().map(str::to_owned).collect();
    for instrument in recipes.instruments() {
        made.extend(
            recipes
                .for_instrument(instrument)
                .iter()
                .filter_map(|recipe| recipe.leaves.clone()),
        );
    }
    for fuel in fuels.names() {
        made.push(fuel.to_owned());
        if let Some(burnt) = fuels.get(fuel) {
            made.push(burnt.leaves.clone());
        }
    }

    let shelved: Vec<String> = tower::children_of(world, shelf)
        .into_iter()
        .filter_map(|node| {
            world
                .get::<tower::Substituted>(node)
                .map(|lie| lie.was.clone())
                .or_else(|| world.get::<tower::Name>(node).map(|name| name.0.clone()))
        })
        .collect();

    recipes
        .vocabulary()
        .into_iter()
        .filter(|name| !made.iter().any(|made| made == name))
        .filter(|name| tower::home(world, name) == Some(shelf))
        .filter(|name| !shelved.iter().any(|held| held == name))
        .map(str::to_owned)
        .collect()
}
