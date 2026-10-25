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
        // And put the name back, for a substituted thing. Removing `Poisoned`
        // alone reported `cleansed` and repaired nothing — the pile kept the
        // name the enemy gave it, so the spell that named it stayed broken and
        // the next roll renamed it again.
        let name = sabotage::restore(world, target).unwrap_or(name);
        // And the two adversarial surfaces, for the same reason. A rewritten
        // spell and a retimed one are `Poisoned` like everything else, so
        // without these two lines `purge` clears the mark, reports `cleansed`
        // and leaves the spell corrupt.
        super::super::assault::unwrite(world, target);
        super::super::assault::untime(world, target);
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
            // A fact, so `Path`: `Detail` is prose a view draws as a sentence.
            .text(FieldName::Path, &name)
            .text(FieldName::State, "protected")
            .text(FieldName::Message, &message)
            .role(Role::Danger)
            .finish();
        return;
    }

    // A place is emptied, never destroyed: `purge alembic` is everyday
    // maintenance (§7, §10.1). Being a place is what makes an instrument safe.
    if world.get::<Nameable>(target).map(|kind| kind.0) == Some(NounKind::Place) {
        // §10.1's lock covers clearing as much as charging: you cannot scour an
        // instrument out from under its own run, nor start a second scour.
        if let Some(why) = busy(world, target) {
            refuse_busy(world, Verb::Purge, target, why);
            return;
        }

        // In §9's triage slot, so a purge starts during a brew. A quickened
        // room scours quickly too (§19).
        let now = *world.resource::<Tick>();
        let ticks = super::quicken::hastened(world, target, PURGE_TICKS);
        world.entity_mut(target).insert(Triaging {
            started: now,
            ends: Tick::new(now.get().saturating_add(ticks)),
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
