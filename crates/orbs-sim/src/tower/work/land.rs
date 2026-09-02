//! Landing whatever's time has come.
//!
//! One pass per tick over both pools. §14 announces **completions only** — a
//! duration action finishing is the one event the player did not just cause, and
//! announcing progress instead would be unusable at endgame with ~25 actions in
//! flight.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use super::produce::transmute;
use super::slot::{Bidden, Triaging, Working};
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
        // A scour ends its instrument's run too, so the credit goes with it —
        // see `slot::stop`. Taken rather than read: nothing here is attributed,
        // because a purge is the player's or the spell's own line and was
        // already credited when it was issued.
        world.entity_mut(place).remove::<Bidden>();
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
            .text(FieldName::At, &name)
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
        // **Credited to whoever asked for it**, ticks after they asked. A spell
        // charges the mortar and the mortar yields on its own schedule, in this
        // system rather than in the runner — so a completion the player did not
        // cause was the one line a loop still put in their transcript.
        //
        // Taken rather than read: the run is over, and a credit left behind
        // would be spent on the next thing this instrument does, whoever starts
        // it.
        let bidden = world.entity_mut(place).take::<Bidden>();
        world
            .resource_mut::<Scrollback>()
            .records_mut()
            .attribute(bidden.as_ref().map(|spell| spell.0.as_str()));

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
            world
                .resource_mut::<Scrollback>()
                .records_mut()
                .attribute(None);
            continue;
        }

        // **A press is the third kind of completed work**, beside a transmuting
        // run and the generic one below. It makes no material, so `transmute`
        // is wrong; and what it says depends on what the ward answered, so the
        // generic sentence is wrong too. It also credits on its own terms — the
        // yield scales with how few presses it took — which is the one thing
        // `worth(verb)` below cannot express.
        if working.verb == crate::parser::Verb::Probe {
            crate::execute::land_probe(world, place);
            world
                .resource_mut::<Scrollback>()
                .records_mut()
                .attribute(None);
            continue;
        }

        // **A fall is the fifth kind**, on the press's argument exactly: it
        // makes no material, so `transmute` is wrong, and what it says depends
        // on whether every glyph came up lit, so the generic sentence below is
        // wrong too. It also credits nothing — what a charm is worth is the
        // charm.
        if working.verb == crate::parser::Verb::Anneal {
            crate::execute::land_fall(world, place);
            world
                .resource_mut::<Scrollback>()
                .records_mut()
                .attribute(None);
            continue;
        }

        // **An audit is the fourth kind**, on the press's argument exactly: it
        // makes no material, so `transmute` is wrong, and what it says depends
        // on what it found, so the generic sentence below is wrong too. §8.1's
        // expensive `verify` — see `execute::audit`.
        if working.verb == crate::parser::Verb::Verify {
            crate::execute::land_sweep(world);
            world
                .resource_mut::<Scrollback>()
                .records_mut()
                .attribute(None);
            continue;
        }

        let subject = name_of(world, working.subject);
        // `Source` is what makes a domain log a log: §3 keeps one stream, and
        // `peruse laboratory.log` is that stream filtered by where each line
        // happened.
        let source = world
            .get::<Name>(place)
            .map_or_else(String::new, |name| name.0.clone());
        // **The archive earns too.** This branch is the other kind of completed
        // work — `divine` in one of the two rooms the game opens with — and
        // leaving it at nothing would make half the opening game pay nothing at
        // all. Keyed by the **verb**, because the archive has no instrument to
        // key on; `progression.toml` documents that exception.
        let earned = super::super::worth(world, working.verb.canonical());
        world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Completion)
            .text(FieldName::Name, working.verb.canonical())
            .count(FieldName::Quantity, earned)
            .text(FieldName::Detail, &subject)
            .text(FieldName::Source, &source)
            // Where it happened, always in the same field — what a spell reads
            // to know the mortar has finished. See `FieldName::At`.
            .text(FieldName::At, &source)
            .role(Role::Success)
            .finish();
        // After the sentence about the work, for the reason `transmute` gives.
        super::super::credit(world, earned);
        world
            .resource_mut::<Scrollback>()
            .records_mut()
            .attribute(None);
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
