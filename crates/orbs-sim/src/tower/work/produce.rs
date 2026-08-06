//! What an instrument makes, and collecting it.
//!
//! The back half of §10.1's loop: [`transmute`] turns an instrument's contents
//! into what its recipe yields, and [`siphon`] takes the product out while
//! leaving the byproduct behind.
//!
//! §7: *"alchemical byproduct accumulates and must be purged manually or by a
//! bound cleanup script."* The byproduct lands **in the instrument**, which is
//! what makes clearing the first move of the *next* loop rather than optional
//! tidying — a fouled instrument matches no recipe at all.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use super::slot::{Busy, busy, refuse_busy, say};
use crate::content::{Prose, Recipes};
use crate::parser::{NounKind, Verb};
use crate::session::Scrollback;
use crate::tower::node::{Cwd, Name, Nameable, NodeIds, children_of};

/// A thing an instrument was asked to make, as opposed to what it left behind.
///
/// Marks the half of a finished run that [`siphon`] collects. Byproducts carry no
/// marker — §10.1's rule is that every one of them has a use, so "waste" is a
/// judgement about *this* brew rather than a property of the reagent.
#[derive(Component, Debug, Clone, Copy)]
pub struct Product;

/// What is inside a place, in insertion order.
#[must_use]
pub fn contents(world: &mut World, place: Entity) -> Vec<Entity> {
    children_of(world, place)
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
    let holding: Vec<String> = held
        .iter()
        .filter_map(|node| world.get::<Name>(*node).map(|n| n.0.clone()))
        .collect();

    let made = world
        .resource::<Recipes>()
        .matching(&name, &holding)
        .map(|recipe| (recipe.output.clone(), recipe.leaves.clone(), recipe.potion));

    // A recipe that stopped matching mid-run should not be reachable — the lock
    // sees to that — but leaving the contents alone is the only safe answer if it
    // ever becomes reachable, because consuming them without producing anything
    // would destroy the player's reagents for nothing.
    //
    // **And it must say so.** `finish` `continue`s after calling this, so
    // returning quietly made `transmute` the one path that can end a `Wield` with
    // no record at all: the echo appeared and then nothing, ever — no completion,
    // no refusal, no message. §14 announces completions, and a run ending is a
    // completion whether or not it produced anything.
    let Some((output, leaves, potion)) = made else {
        let message = world
            .resource::<Prose>()
            .line("wield_nothing", &[("name", &name)]);
        say(world, &name, "fouled", &message, Role::Cost);
        return;
    };

    for node in held {
        world.entity_mut(node).despawn();
    }
    // A finished potion is an `Essence`; everything else is crafting stock. The
    // byproduct is always stock — §10.1 gives every one of them a use.
    let kind = if potion {
        NounKind::Essence
    } else {
        NounKind::Reagent
    };
    // The product is marked, the byproduct is not. That is the whole difference
    // `siphon` and `purge` read: **`siphon` takes what you meant to make, `purge`
    // clears what you did not.** Telling them apart by name would mean the
    // laboratory knowing which reagents are "waste", which §10.1 explicitly
    // refuses — every byproduct is some other recipe's input.
    for (product, kind, wanted) in [(&output, kind, true), (&leaves, NounKind::Reagent, false)] {
        let id = world.resource_mut::<NodeIds>().issue();
        let node = world
            .spawn((id, Name(product.clone()), Nameable(kind)))
            .id();
        world.entity_mut(node).insert(ChildOf(place));
        if wanted {
            world.entity_mut(node).insert(Product);
        }
    }

    let message = world.resource::<Prose>().line(
        "wield_done",
        &[("source", &name), ("name", &output), ("detail", &leaves)],
    );
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, &output)
        // The byproduct is the state the instrument is left in — and a *fact*,
        // so not `Detail`, which is prose a view draws in front of the message.
        .text(FieldName::State, &leaves)
        .text(FieldName::Source, &name)
        .text(FieldName::Message, &message)
        .role(Role::Success)
        .finish();
}

/// Collect what an instrument made, leaving what it fouled itself with.
///
/// The fourth move of §10.1's loop, and the end of `siphon` being a dark verb.
/// The product lands where the player is standing, loose and nameable, ready to
/// be `move`d onward — which is what makes the pipeline typeable without ever
/// naming an instrument's insides (§19).
pub fn siphon(world: &mut World, place: Entity) -> bool {
    let name = world
        .get::<Name>(place)
        .map_or_else(String::new, |name| name.0.clone());

    // **Through `busy`, not `Working` alone.** `slot::busy` exists precisely so
    // no caller checks half the lock, and this one did: a scour in flight left
    // `siphon` free to lift the product out of an instrument a `purge` was four
    // ticks from emptying, so `purge x; siphon x` rescued exactly what the scour
    // was started to destroy — while `move`, `wield`, `purge` and `begin` all
    // refuse on the same state.
    match busy(world, place) {
        Some(Busy::Working) => {
            let message = world
                .resource::<Prose>()
                .line("siphon_working", &[("name", &name)]);
            say(world, &name, "working", &message, Role::Cost);
            return false;
        }
        Some(why) => {
            refuse_busy(world, Verb::Siphon, place, why);
            return false;
        }
        None => {}
    }

    let cwd = world.resource::<Cwd>().0;
    let taken: Vec<Entity> = contents(world, place)
        .into_iter()
        .filter(|node| world.get::<Product>(*node).is_some())
        .collect();

    if taken.is_empty() {
        let message = world
            .resource::<Prose>()
            .line("siphon_empty", &[("name", &name)]);
        say(world, &name, "empty", &message, Role::Cost);
        return false;
    }

    let collected: Vec<String> = taken
        .iter()
        .filter_map(|node| world.get::<Name>(*node).map(|held| held.0.clone()))
        .collect();
    for node in taken {
        world.entity_mut(node).insert(ChildOf(cwd));
        world.entity_mut(node).remove::<Product>();
    }

    let listed = collected.join(", ");
    let message = world
        .resource::<Prose>()
        .line("siphon_done", &[("name", &listed), ("source", &name)]);
    say(world, &listed, "collected", &message, Role::Success);
    true
}
