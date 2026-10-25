//! `verify` bare — §8.1's expensive audit, started and landed.
//!
//! The model is [`tower::audit`]; this takes the production slot and says what
//! it found. §8.1 makes it Production-class, through [`tower::begin`]: the
//! looking is priced against the making, where a scour on the triage slot
//! answers a problem already found.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::Prose;
use crate::parser::Verb;
use crate::session::Scrollback;
use crate::tower::{self, Surface, audit};

/// Where the audit's run hangs — the named room, not the nameless root.
const TOWER: &str = "/tower";

/// Begin a full audit of the tower.
pub(super) fn sweep(world: &mut World) {
    let ticks = audit::ticks_for(held(world), world.resource::<Scrollback>().records().len());

    // `/tower`, not `tower::root`: that one is the filesystem root and is
    // deliberately nameless, so hanging the run there made `work_busy` read
    // *"the  is busy verifying"* — a refusal naming nothing.
    let Some(at) = tower::find_by_path(world, TOWER) else {
        return;
    };
    let Some(id) = world.get::<tower::NodeId>(at).copied() else {
        return;
    };

    // The tower node holds the run, because the audit is of the tower rather
    // than of any instrument. `in_flight` queries `Working` wherever it sits, so
    // this counts against the same pool a grind does — §8.1's contention.
    if !tower::begin(world, at, Verb::Verify, id, ticks) {
        return;
    }

    let message = world
        .resource::<Prose>()
        .line("verify_sweep_begins", &[("quantity", &ticks.to_string())]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Verify.canonical())
        .text(FieldName::Message, &message)
        .count(FieldName::Remaining, ticks)
        .role(Role::Cost)
        .finish();
}

/// Report what a finished audit found — `land::finish`'s fourth kind of work.
///
/// It makes no material and its sentence depends on what it found. It names
/// what is tampered, since the skill is knowing where to look (§8.1).
pub fn land_sweep(world: &mut World) {
    let found = tampered(world);
    let message = if found.is_empty() {
        world.resource::<Prose>().line("verify_sweep_sound", &[])
    } else {
        world
            .resource::<Prose>()
            .line("verify_sweep_found", &[("detail", &found.join(", "))])
    };
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Status)
        .text(FieldName::Name, Verb::Verify.canonical())
        .text(
            FieldName::State,
            if found.is_empty() {
                "sound"
            } else {
                "tampered"
            },
        )
        .text(FieldName::Message, &message)
        .role(if found.is_empty() {
            Role::Success
        } else {
            Role::Danger
        })
        .finish();
}

/// Say that a surface is still cooling, and how long is left.
///
/// It names the wait, so the cooldown is a decision (§6). The surface rides
/// `Kind`, since `State` is reserved for a verdict (rule 4).
pub(super) fn say_cooling(world: &mut World, surface: Surface, left: u64) {
    let message = world.resource::<Prose>().line(
        "verify_cooling",
        &[("state", surface.word()), ("quantity", &left.to_string())],
    );
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Verify.canonical())
        .text(FieldName::Kind, surface.word())
        .text(FieldName::Message, &message)
        .count(FieldName::Remaining, left)
        .role(Role::Cost)
        .finish();
}

/// How many spells the orb is holding — one half of the audit's cost.
fn held(world: &mut World) -> usize {
    world.query::<&tower::spell::Bound>().iter(world).count()
}

/// Everything in the tower that is currently lying, by name.
///
/// Sorted by name, since a query's order would differ between runs of one
/// seed — `assault::strike`'s rule.
fn tampered(world: &mut World) -> Vec<String> {
    let mut found: Vec<String> = world
        .query::<(Entity, &tower::Name)>()
        .iter(world)
        .map(|(entity, name)| (entity, name.0.clone()))
        .collect::<Vec<_>>()
        .into_iter()
        .filter(|(entity, _)| tower::poisoned(world, *entity))
        .map(|(_, name)| name)
        .collect();
    found.sort_unstable();
    found.dedup();
    found
}
