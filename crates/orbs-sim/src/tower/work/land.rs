//! Landing whatever's time has come.
//!
//! One pass per tick over both pools. §14 announces **completions only** — a
//! duration action finishing is the one event the player did not just cause, and
//! announcing progress instead would be unusable at endgame with ~25 actions in
//! flight.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use super::produce::transmute;
use super::slot::{Triaging, Working};
use crate::content::Prose;
use crate::parser::Verb;
use crate::session::Scrollback;
use crate::tick::Tick;
use crate::tower::node::{Name, NodeId, children_of};

/// Land anything whose time has come.
pub fn finish(world: &mut World) {
    let now = *world.resource::<Tick>();

    // Triage first, and separately: it draws on §9's other slot, so nothing here
    // may consult `CAPACITY` or `in_flight`. Landing it before production also
    // means a purge that completes on the same tick a brew does has already
    // cleared the instrument by the time anything asks what is in it.
    let scoured: Vec<Entity> = world
        .query::<(Entity, &Triaging)>()
        .iter(world)
        .filter(|(_, triage)| triage.finished(now))
        .map(|(entity, _)| entity)
        .collect();
    for place in scoured {
        world.entity_mut(place).remove::<Triaging>();
        let name = world
            .get::<Name>(place)
            .map_or_else(String::new, |name| name.0.clone());
        let contents = children_of(world, place);
        let emptied = u64::try_from(contents.len()).unwrap_or(u64::MAX);
        for node in contents {
            world.entity_mut(node).despawn();
        }
        let message = world.resource::<Prose>().line(
            if emptied == 0 {
                "purge_place_empty"
            } else {
                "purge_place"
            },
            &[("path", &name)],
        );
        world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Completion)
            .text(FieldName::Name, Verb::Purge.canonical())
            .text(FieldName::Path, &name)
            .text(FieldName::State, "emptied")
            .count(FieldName::Quantity, emptied)
            .text(FieldName::Message, &message)
            .role(Role::Success)
            .finish();
    }

    let landed: Vec<(Entity, Working)> = world
        .query::<(Entity, &Working)>()
        .iter(world)
        .filter(|(_, working)| working.finished(now))
        .map(|(entity, working)| (entity, *working))
        .collect();

    for (place, working) in landed {
        // The slot is released the moment the work lands, **before** anything is
        // collected. A finished instrument holds its product until siphoned but
        // holds no Focus — otherwise a capacity-1 player who walked away from a
        // finished mortar could never start anything again, which is a soft-lock
        // reachable in the first ten minutes.
        world.entity_mut(place).remove::<Working>();

        // **`transmutes()`, not `== Wield`.** §10.1's per-instrument verbs start
        // a run exactly as `wield` does, so asking for the one verb by name left
        // a finished `grind` releasing the slot, saying nothing useful, and
        // leaving the sage sitting whole in the mortar. See `Verb::transmutes`.
        if working.verb.transmutes() {
            transmute(world, place);
            continue;
        }

        let subject = name_of(world, working.subject);
        // `Source` is what makes a domain log a log: §3 keeps one stream, and
        // `peruse laboratory.log` is that stream filtered by where each line
        // happened.
        let source = world
            .get::<Name>(place)
            .map_or_else(String::new, |name| name.0.clone());
        world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Completion)
            .text(FieldName::Name, working.verb.canonical())
            .text(FieldName::Detail, &subject)
            .text(FieldName::Source, &source)
            .role(Role::Success)
            .finish();
    }
}

/// The name behind a stable id.
fn name_of(world: &mut World, id: NodeId) -> String {
    world
        .query::<(&NodeId, &Name)>()
        .iter(world)
        .find(|(node, _)| **node == id)
        .map_or_else(String::new, |(_, name)| name.0.clone())
}
