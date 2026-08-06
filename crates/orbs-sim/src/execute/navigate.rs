//! Getting around, and looking at what is there.
//!
//! §7: *"The directory tree **is** the tower. Navigation is diegetic; paths are
//! places."* [`attend`] is also what makes a domain's contents nameable — the
//! scene holds every place but only the belongings of where you stand — so this
//! is the module every other verb depends on to have somewhere to act.
//!
//! **No prose here.** Rule 6 and §12 put authored text in content files; these
//! emit facts and let a later layer wrap sentences around them.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::parser::{Intent, NounKind, Verb};
use crate::session::Scrollback;
use crate::tower::{self, Cwd};

use super::{acknowledge, missing};

/// Go somewhere.
///
/// §7: *"Navigation is diegetic; paths are places."* Moving is also what makes a
/// domain's contents nameable at all — the scene holds every place but only the
/// belongings of where you stand, so `attend` is how a player reaches the nouns
/// a brewing command needs. See [`tower::rebuild`](crate::tower::rebuild).
pub(super) fn attend(intent: &Intent, world: &mut World) {
    let Some(target) = intent.arguments.first().map(|argument| &argument.value) else {
        acknowledge(Verb::Attend, world);
        return;
    };

    let root = root(world);
    let Some(node) = find_place(world, root, target) else {
        missing(Verb::Attend, target, world);
        return;
    };

    world.insert_resource(Cwd(node));
    let path = tower::path_of(world, node);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Attend.canonical())
        .text(FieldName::Path, &path)
        .role(Role::Success)
        .finish();
}

/// List what is at a place, or here.
///
/// `Verb::Survey` takes an **optional** place, and dropping it made the echo and
/// the listing disagree: `survey archive` restated `/tower/archive` and then
/// showed the laboratory. §6 makes the echo the thing players learn the vocabulary
/// from, so an echo that describes a different command than the one that ran is
/// worse than no echo at all.
pub(super) fn survey(intent: &Intent, world: &mut World) {
    let at = match intent.arguments.first() {
        Some(argument) => {
            let root = root(world);
            match find_place(world, root, &argument.value) {
                Some(node) => node,
                None => {
                    missing(Verb::Survey, &argument.value, world);
                    return;
                }
            }
        }
        None => world.resource::<Cwd>().0,
    };

    let here: Vec<(String, &'static str)> = tower::children_of(world, at)
        .into_iter()
        .filter_map(|node| {
            let name = world.get::<tower::Name>(node)?.0.clone();
            let kind = world.get::<tower::Nameable>(node)?.0;
            Some((name, kind.label()))
        })
        .collect();

    let mut scrollback = world.resource_mut::<Scrollback>();
    let records = scrollback.records_mut();
    for (name, kind) in here {
        records
            .push(RecordKind::Entry)
            .text(FieldName::Name, &name)
            .text(FieldName::Kind, kind)
            .finish();
    }
}

/// The tower root.
pub(super) fn root(world: &World) -> Entity {
    let mut at = world.resource::<Cwd>().0;
    while let Some(parent) = world.get::<ChildOf>(at).map(ChildOf::parent) {
        at = parent;
    }
    at
}

/// The place `target` names, by full path or by last segment (§7).
pub(super) fn find_place(world: &World, from: Entity, target: &str) -> Option<Entity> {
    let mut stack = vec![from];
    while let Some(node) = stack.pop() {
        if world.get::<tower::Nameable>(node).map(|n| n.0) == Some(NounKind::Place)
            && (tower::path_of(world, node) == target
                || world
                    .get::<tower::Name>(node)
                    .is_some_and(|n| n.0 == target))
        {
            return Some(node);
        }
        stack.extend(tower::children_of(world, node));
    }
    None
}
