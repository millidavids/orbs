//! What every puzzle domain needs to find its fixture, publish into it, and say
//! a line filed to its log.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::Prose;
use crate::parser::Verb;
use crate::session::Scrollback;
use crate::tower::{self, Cwd};

/// One authored line under `verb`, filed to `source` so the domain's log can
/// read it back (`files::in_domain`).
pub(super) fn filed(
    world: &mut World,
    source: &str,
    verb: Verb,
    key: &str,
    args: &[(&str, &str)],
    role: Role,
) {
    let message = world.resource::<Prose>().line(key, args);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, verb.canonical())
        .text(FieldName::Message, &message)
        .text(FieldName::Source, source)
        .role(role)
        .finish();
}

/// Take every reading off a node.
///
/// Clearing and re-raising is the shared idiom: a stale reading is worse than
/// none, and no domain has more than a handful.
pub(super) fn clear(world: &mut World, node: Entity) {
    for held in tower::children_of(world, node) {
        world.entity_mut(held).despawn();
    }
}

/// The fixture carrying `verb`, where the player is standing.
///
/// Like `research::stacks`, so the readings and the picture follow the room. A
/// publisher on a tick takes the entity instead — see [`beside`].
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
/// Takes the room, never `Cwd`: a press or a haul lands outside
/// `spell::run`'s domain swap (§19).
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
