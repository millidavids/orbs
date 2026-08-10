//! What an instrument makes.
//!
//! The back half of §10.1's loop: [`transmute`] turns an instrument's contents
//! into what its recipe yields, and leaves both the product and the byproduct
//! **inside the tool**. Taking them out is somebody else's job — the next
//! stage's verb reaches in for what it needs, `empty` shelves the lot, `purge`
//! destroys it.
//!
//! §7: *"alchemical byproduct accumulates and must be purged manually or by a
//! bound cleanup script."* The byproduct lands **in the instrument**, which is
//! what makes clearing the first move of the *next* loop rather than optional
//! tidying — a fouled instrument matches no recipe at all.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use super::slot::say;
use crate::content::{Prose, Recipes};
use crate::parser::NounKind;
use crate::session::Scrollback;
use crate::tower::node::{Name, children_of};

/// A thing an instrument was asked to make, as opposed to what it left behind.
///
/// Marks the half of a finished run that the recipe was **for**. Byproducts
/// carry no marker — §10.1's rule is that every one of them has a use, so
/// "waste" is a judgement about *this* brew rather than a property of the
/// reagent.
///
/// **`siphon` read this and is retired** (§19). `tower::panel` still does: the
/// marker is what makes an instrument show as having finished with something
/// worth taking, which is a fact about the tool rather than about any verb.
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
        say(world, &name, &name, "fouled", &message, Role::Cost);
        return;
    };

    // **One of each input, not everything in the vessel.** The instrument is
    // charged a unit at a time now, so a run spends a unit — and what is left
    // over stays where it is rather than being destroyed by a recipe that never
    // asked for it.
    for node in held {
        let Some(name) = world.get::<Name>(node).map(|name| name.0.clone()) else {
            continue;
        };
        super::super::stock::take(world, place, &name, 1);
    }
    // A finished potion is an `Essence`; everything else is crafting stock. The
    // byproduct is always stock — §10.1 gives every one of them a use.
    let kind = if potion {
        NounKind::Essence
    } else {
        NounKind::Reagent
    };
    // The product is marked, the byproduct is not — which is what the panel
    // reads to say a tool has finished with something worth having. Telling them
    // apart by **name** would mean the laboratory knowing which reagents are
    // "waste", which §10.1 explicitly refuses: every byproduct is some other
    // recipe's input, and route B of the clarified draught is exactly the husks
    // route A leaves behind.
    for (product, kind, wanted) in [(&output, kind, true), (&leaves, NounKind::Reagent, false)] {
        // Merged into whatever is already there, so a second run adds to the
        // pile rather than standing a second node beside it under the same name.
        let node = super::super::stock::give(world, place, product, kind, 1);
        if wanted {
            world.entity_mut(node).insert(Product);
        }
    }

    // **The one place a run has succeeded.** Every other exit from this function
    // is a run that ended without making anything, and `land::finish` also runs
    // for a scour — so this is where work becomes experience (§11.5). It goes on
    // the line the run is already writing rather than pushing a second record: a
    // run says what it made, and what it earned belongs in that sentence.
    let earned = super::super::worth(world, &name);

    let message = world.resource::<Prose>().line(
        "wield_done",
        &[
            ("source", &name),
            ("name", &output),
            ("detail", &leaves),
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
        .text(FieldName::State, &leaves)
        .text(FieldName::Source, &name)
        // Where it happened. `Name` here is the **product**, which is why a
        // spell could not use it to know which instrument yielded — see
        // `FieldName::At`.
        .text(FieldName::At, &name)
        .text(FieldName::Message, &message)
        .role(Role::Success)
        .finish();

    // **After the sentence about the run, never before it.** A level bought by
    // this run is a consequence of it, and announcing the reward first reads as
    // the orb answering a question nobody asked.
    super::super::credit(world, earned);
}

// `siphon` lived here and is **retired** (§19). It lifted the `Product` out of
// an instrument and set it on the laboratory floor, which was the fourth move of
// §10.1's loop back when a stage's output had to be carried by hand.
//
// `reachable` searches idle instruments, so the next tool takes the output
// directly — `digest ground-sage` needs nothing drawn off first — and with
// `siphon` gone nothing can put a reagent on the floor at all. The bench and the
// shelf are one place, and `empty` is how a tool is cleared into it.
//
// [`Product`] survives it: `tower::panel` reads the marker to show that a tool
// has finished with something in it, which is a fact about the instrument rather
// than about any verb.
