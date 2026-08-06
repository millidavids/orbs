//! Destruction, which is maintenance.
//!
//! §7: destruction is *"a tool, not a trap"* — useful, everyday, and scriptable.
//! §10.1's loop **opens** by clearing an instrument of what the last brew left in
//! it, which is why a place is emptied rather than deleted and why this runs in
//! §9's triage slot rather than the production one.
//!
//! Three tiers, and they are different answers to different questions: a
//! *poisoned* surface is cleansed (§8.1's notice → `verify` → `purge` loop), a
//! [`Protected`] target refuses in character, and everything else is emptied or
//! destroyed.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use super::slot::{PURGE_TICKS, Triaging, busy, refuse_busy};
use crate::content::Prose;
use crate::parser::{NounKind, Verb};
use crate::session::Scrollback;
use crate::tick::Tick;
use crate::tower::node::{Name, Nameable, Protected};
use crate::tower::sabotage;

/// Destroy something.
///
/// Nothing irreplaceable is destructible in Phase 0: byproduct regenerates with
/// the next brew, so `undo` is not load-bearing for §15's gate.
pub fn purge(world: &mut World, target: Entity) {
    let name = world
        .get::<Name>(target)
        .map_or_else(String::new, |name| name.0.clone());

    // Purging a *poisoned* surface clears the interference rather than
    // destroying the surface. §7 makes destruction maintenance, and the loop
    // that gives sabotage its point is notice → `verify` → `purge`: finding
    // tampering you cannot do anything about is its own dead end.
    if sabotage::poisoned(world, target) {
        world.entity_mut(target).remove::<sabotage::Poisoned>();
        world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Completion)
            .text(FieldName::Name, Verb::Purge.canonical())
            .text(FieldName::Detail, &name)
            .text(FieldName::State, "cleansed")
            .role(Role::Success)
            .finish();
        return;
    }

    if world.get::<Protected>(target).is_some() {
        let message = world
            .resource::<Prose>()
            .line("purge_protected", &[("path", &name)]);
        world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Completion)
            .text(FieldName::Name, Verb::Purge.canonical())
            // The target is a *fact*, so it belongs in `Path`, not `Detail` —
            // `FieldName::Detail` is secondary prose subordinate to `Message`,
            // and a view that honours prose draws it as a sentence. It sat in
            // `Detail` while nothing read the distinction; the moment the
            // refusal gained a written line, the target started leaking in
            // front of it as a bare `tower`.
            .text(FieldName::Path, &name)
            .text(FieldName::State, "protected")
            .text(FieldName::Message, &message)
            .role(Role::Danger)
            .finish();
        return;
    }

    // A place is **emptied**, never destroyed. §10.1's loop opens with clearing
    // an instrument of what the last brew left in it, and `purge alembic` has to
    // mean that — deleting the alembic would leave a laboratory that cannot
    // distil, from a verb §7 calls everyday maintenance.
    //
    // Being a place is what makes an instrument safe, which is why instruments
    // are not `Protected`: that tier is for the root, the live domains, and the
    // dispensary, which refuse above and are never emptied either.
    if world.get::<Nameable>(target).map(|kind| kind.0) == Some(NounKind::Place) {
        // §10.1's lock covers clearing as much as charging: you cannot scour an
        // instrument out from under its own run, nor start a second scour.
        if let Some(why) = busy(world, target) {
            refuse_busy(world, Verb::Purge, target, why);
            return;
        }

        // Clearing **takes time** — §11.5 puts it in the Triage band — but it
        // runs in §9's triage slot, which is a different pool from the
        // production one. So a purge starts happily during a brew, and this is
        // the one action in the domain that does not answer to `CAPACITY`.
        let now = *world.resource::<Tick>();
        world.entity_mut(target).insert(Triaging {
            started: now,
            ends: Tick::new(now.get().saturating_add(PURGE_TICKS)),
        });
        let message = world
            .resource::<Prose>()
            .line("purge_begins", &[("path", &name)]);
        world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Completion)
            .text(FieldName::Name, Verb::Purge.canonical())
            .text(FieldName::Path, &name)
            .text(FieldName::At, &name)
            .text(FieldName::State, "scouring")
            .text(FieldName::Message, &message)
            .role(Role::Cost)
            .finish();
        return;
    }

    world.entity_mut(target).despawn();
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Purge.canonical())
        .text(FieldName::Detail, &name)
        .role(Role::Success)
        .finish();
}
