//! The four lookups every reading-publishing domain needs, in one place.
//!
//! The lens and the sanctum publish their state the same way — clear a node's
//! children and re-raise them (`scry::publish`, `muster::publish`) — and both
//! grew the same four helpers to do it. They were byte-for-byte copies apart
//! from one constant, which is what CLAUDE.md's *"check for reuse before writing
//! new logic"* exists to stop.
//!
//! **The archive is deliberately not a caller.** `research::refresh` walks the
//! four ways by `Way::ALL` rather than by name and clears them inside its own
//! loop; pulling it in here would be fitting a third shape to two, which is how
//! a helper stops being one.
//!
//! # What is *not* here
//!
//! The record emitters. `scry::say` and `muster::say` differ only in which
//! `Source` they file under, and folding them together would take a `&'static
//! str` for the fixture's name — at which point the call reads `say(world,
//! Verb::Haul, PYLON, key, args, role)` and the parameter list is longer than
//! the body it saved. §19 records the same judgement about `tower::reach`'s
//! fifth axis: what a domain *says* is presentation and stays at the call site.

use bevy_ecs::prelude::*;

use crate::parser::Verb;
use crate::tower::{self, Cwd};

/// Take every reading off a node.
///
/// Clearing and re-raising rather than diffing is the shared idiom: a node
/// holding a stale reading is worse than one holding nothing, and no domain has
/// more than a handful of children to rebuild.
pub(super) fn clear(world: &mut World, node: Entity) {
    for held in tower::children_of(world, node) {
        world.entity_mut(held).despawn();
    }
}

/// The fixture carrying `verb`, where the player is standing.
///
/// **Where the player is standing**, like `research::stacks`: the readings and
/// the picture both follow the room, so neither can outrun the other by
/// following the player out of it. A publisher that runs on a *tick* — a bound
/// solver's — must take the entity instead, which is what
/// [`beside`](self::beside) is for.
pub(super) fn fixture(world: &World, verb: Verb) -> Option<Entity> {
    let cwd = world.resource::<Cwd>().0;
    beside(world, cwd, verb)
}

/// The child of `parent` carrying `verb`.
pub(super) fn beside(world: &World, parent: Entity, verb: Verb) -> Option<Entity> {
    tower::children_of(world, parent).into_iter().find(|node| {
        world
            .get::<tower::Operation>(*node)
            .is_some_and(|operation| operation.0 == verb)
    })
}

/// One of a room's `Role::Reading` fixtures, by name.
///
/// **Takes the room rather than reading `Cwd`**, and that is the whole reason
/// this is not one line inline: a press or a haul lands in the tick schedule,
/// *outside* `spell::run`'s domain swap, so a bound solver working while the
/// player is elsewhere would find nothing. §19 records the lens shipping that
/// bug and the sanctum being written against the fix.
pub(super) fn reading(world: &World, room: Entity, wanted: &str) -> Option<Entity> {
    tower::children_of(world, room).into_iter().find(|node| {
        world.get::<tower::Reading>(*node).is_some()
            && world
                .get::<tower::Name>(*node)
                .is_some_and(|name| name.0 == wanted)
    })
}

/// The room a fixture stands in.
pub(super) fn room_of(world: &World, node: Entity) -> Option<Entity> {
    world
        .get::<bevy_ecs::hierarchy::ChildOf>(node)
        .map(bevy_ecs::hierarchy::ChildOf::parent)
}
