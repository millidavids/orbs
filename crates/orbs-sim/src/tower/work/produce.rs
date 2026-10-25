//! What an instrument makes.
//!
//! The back half of §10.1's loop: [`transmute`] turns an instrument's contents
//! into what its recipe yields and leaves both the product and the byproduct
//! *inside the tool*. Taking them out is somebody else's job — the next stage's
//! verb reaches in, `empty` shelves the lot, `purge` destroys it.
//!
//! The byproduct landing in the instrument is what makes clearing the first move
//! of the *next* loop rather than optional tidying, since a fouled instrument
//! matches no recipe at all (§7).

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use super::slot::say;
use crate::content::{Prose, Recipes};
use crate::parser::NounKind;
use crate::session::Scrollback;
use crate::tower::node::{Name, children_of};

/// A thing an instrument was asked to make, as opposed to what it left behind.
///
/// Marks the half of a run the recipe was for; a byproduct is unmarked, since
/// every one has a use (§10.1). The panel reads it to show a tool has finished
/// with something worth taking.
#[derive(Component, Debug, Clone, Copy)]
pub struct Product;

/// What is inside a place, in insertion order.
#[must_use]
pub fn contents(world: &mut World, place: Entity) -> Vec<Entity> {
    children_of(world, place)
}

/// Which of a recipe's products this run made.
///
/// A one-product recipe rolls nothing, or every grind would advance
/// [`RngStream::Archive`](crate::RngStream::Archive) (§19). Once per completion,
/// never per tick. The archive's stream, because the lectern is the only
/// instrument that draws.
fn draw(world: &mut World, choices: &[String]) -> String {
    let [only] = choices else {
        let mut rngs = world.resource_mut::<crate::rng::Rngs>();
        let rng = rngs.stream(crate::rng::RngStream::Archive);
        let at = rand::RngExt::random_range(rng, 0..choices.len());
        return choices[at].clone();
    };
    only.clone()
}

/// Turn an instrument's contents into what its recipe makes.
///
/// The recipe is looked up **again** here rather than carried on
/// [`Working`](super::Working). That keeps `Working` `Copy` and cannot disagree
/// with itself: the instrument is locked for the whole run, so its contents at
/// completion are the contents that started it, and re-deriving gives the same
/// answer by construction.
pub(super) fn transmute(world: &mut World, place: Entity) {
    let name = world
        .get::<Name>(place)
        .map_or_else(String::new, |name| name.0.clone());
    let held = contents(world, place);
    let holding = super::super::stock::holdings(world, place);

    let made = {
        let recipes = world.resource::<Recipes>();
        let known = super::super::known(world);
        recipes.matching(&name, &holding, &known).map(|recipe| {
            (
                // Only the outputs the player may make: a recipe that draws among
                // several fires once any is known, and the draw must not land on
                // one still gated. See `Recipes::reachable`.
                recipe
                    .outputs()
                    .into_iter()
                    .filter(|made| known.knows(recipes, made))
                    .map(str::to_owned)
                    .collect::<Vec<_>>(),
                recipe.leaves.clone(),
                recipe.potion,
                recipe.scroll,
                recipe.count,
                recipe.yields,
            )
        })
    };

    // Unreachable while the lock holds; if it ever is, the contents stay and it
    // is said, since a run ending is a completion (§14).
    let Some((choices, leaves, potion, scroll, count, yields)) = made else {
        let message = world
            .resource::<Prose>()
            .line("wield_nothing", &[("name", &name)]);
        say(world, &name, &name, "fouled", &message, Role::Cost);
        return;
    };

    // One draw a run, however many it yields, so a batch adds no roll.
    let output = draw(world, &choices);

    // What the recipe asked for, not everything in the vessel: the instrument is
    // charged a unit at a time, so a run spends what the recipe wants and the
    // rest stays rather than being destroyed by a recipe that never asked for
    // it. A literal `1` until a recipe could want four; `Recipe::count` now.
    for node in held {
        let Some(name) = world.get::<Name>(node).map(|name| name.0.clone()) else {
            continue;
        };
        super::super::stock::take(world, place, &name, count);
    }
    // A finished potion is an `Essence`, a scroll is a `Scroll`, and everything
    // else is crafting stock. The byproduct is always stock — §10.1 gives every
    // one of them a use.
    //
    // The same three-way rule `Recipes::kind_of` applies by name; the two agree
    // because `parse` refuses a recipe that sets both flags.
    let kind = if potion {
        NounKind::Essence
    } else if scroll {
        NounKind::Scroll
    } else {
        NounKind::Reagent
    };
    // The product is marked and the byproduct not — see [`Product`]. A recipe
    // that leaves nothing leaves no node.
    let byproduct = leaves
        .as_ref()
        .map(|leaves| (leaves, NounKind::Reagent, false));
    // A `fruitful` tool yields one more product, never byproduct, and flat
    // rather than a chance, which would be a draw on a path `meditate` repeats.
    let over = super::super::charmed(world, place, super::super::charm::Kind::Fruitful);
    let batch = yields + u32::from(over);
    for (product, kind, wanted) in std::iter::once((&output, kind, true)).chain(byproduct) {
        // Merged into whatever is already there, so a second run adds to the
        // pile rather than standing a second node beside it under the same name.
        let made = if wanted { batch } else { 1 };
        let node = super::super::stock::give(world, place, product, kind, made);
        if wanted {
            world.entity_mut(node).insert(Product);
        }
    }

    // The one place a run has succeeded: every other exit made nothing, and
    // `land::finish` also runs for a scour, so this is where work becomes
    // experience (§11.5). On the line the run is already writing rather than a
    // second record — what it earned belongs in the sentence about what it made.
    let earned = super::super::worth(world, &name);

    // Two lines, not one with a hole in it: `Prose` prints an unfilled
    // `{detail}` literally, which is how a typo surfaces (§19) — and *"and
    // leaves "* on every scroll the archive assembles is that firing on content
    // that is correct.
    let key = match (leaves.is_some(), batch > 1) {
        (true, false) => "wield_done",
        (false, false) => "wield_done_clean",
        (true, true) => "wield_done_batch",
        (false, true) => "wield_done_clean_batch",
    };
    let message = world.resource::<Prose>().line(
        key,
        &[
            ("source", &name),
            ("name", &output),
            ("quantity", &batch.to_string()),
            ("detail", leaves.as_deref().unwrap_or_default()),
            ("count", &earned.to_string()),
        ],
    );
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, &output)
        .count(FieldName::Quantity, earned)
        // The byproduct is the state the instrument is left in — and a *fact*,
        // so not `Detail`, which is prose a view draws in front of the message.
        .text(FieldName::State, leaves.as_deref().unwrap_or_default())
        .text(FieldName::Source, &name)
        // Where it happened. `Name` here is the **product**, which is why a
        // spell could not use it to know which instrument yielded — see
        // `FieldName::At`.
        .text(FieldName::At, &name)
        .text(FieldName::Message, &message)
        .role(Role::Success)
        .finish();

    // After the sentence about the run, never before it: a level bought by this
    // run is a consequence of it, and announcing the reward first reads as the
    // orb answering a question nobody asked. Counted as well as credited, since
    // what was made and where is what a room's mastery line reads.
    super::super::done(
        world,
        &super::super::Work::made(&name, &output, potion, scroll),
        earned,
    );
}

// `siphon` is retired (§19): the next tool reaches into an idle one directly.
