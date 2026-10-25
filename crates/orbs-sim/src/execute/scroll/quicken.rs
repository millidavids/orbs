//! What a quickening draw does: the laboratory works faster for a window.
//!
//! Each draw sets [`Quickened`](tower::Quickened) on the laboratory, and
//! `tower::begin` reads it when a run starts — where §10.1 checks the athanor,
//! and for the same reason: a run starting inside the window stays short even
//! if the window closes.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use super::spend::{laboratory, refuse};
use crate::content::Prose;
use crate::session::Scrollback;
use crate::tower;

/// Set the laboratory working `by` times quick for `ticks`.
///
/// `opening` also hurries what is running. Divided from *now*, once, so
/// `meditate 60` is sixty `meditate 1`s.
pub(super) fn window(world: &mut World, name: &str, by: u64, ticks: u64, opening: bool) {
    let Some(domain) = laboratory(world) else {
        refuse(world, name, "scroll_nowhere");
        return;
    };
    let now = *world.resource::<crate::tick::Tick>();
    world.entity_mut(domain).insert(tower::Quickened {
        from: now,
        ticks,
        by,
    });
    if !opening {
        return;
    }

    // `in_flight` is the production pool and `CAPACITY` is 1, so at most one
    // run, and only if it is in the laboratory.
    let hurried = tower::in_flight(world)
        .into_iter()
        .map(|(_, at)| at)
        .find(|at| tower::domain_of(world, *at) == Some(domain));
    if let Some(at) = hurried
        && let Some(mut working) = world.get_mut::<tower::Working>(at)
        && let Some(ends) = tower::hurried_from(now, working.ends, by, working.hurried_by)
    {
        working.ends = ends;
        working.hurried_by = by;
    }

    let place = world
        .get::<tower::Name>(domain)
        .map_or_else(String::new, |name| name.0.clone());
    let message = world.resource::<Prose>().line(
        "scroll_quickening",
        &[
            ("name", name),
            ("source", &place),
            ("count", &by.to_string()),
        ],
    );
    // The room and the window as fields, not only prose (rule 4).
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, name)
        .text(FieldName::At, &place)
        .count(FieldName::Remaining, ticks)
        .text(FieldName::Message, &message)
        .role(Role::Success)
        .finish();
}
