//! Every open room's rate and wait, together: what `status rates` lists and
//! the balance sweep reports (§19, *the tower speaks up*).

use bevy_ecs::prelude::*;

use super::throughput::{Hourly, Throughput};

/// One open room: how fast it makes what its ledger counts, and what it waits
/// on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Yield {
    /// The room.
    pub room: &'static str,
    /// The ledger row its rate is measured by; `ledger_<id>` names it.
    pub row: Option<String>,
    /// How fast.
    pub rate: Hourly,
    /// What it waits on.
    pub wait: super::wait::Wait,
}

/// Every open room's [`Yield`], in the rail's order.
#[must_use]
pub fn yields(world: &World) -> Vec<Yield> {
    let now = world.resource::<crate::tick::Tick>().get();
    let rooms = super::brief::rooms_of(world);
    let casts = super::running_spells(world);
    let opened = world.resource::<super::Opened>();
    let throughput = world.resource::<Throughput>();
    super::DOMAINS
        .into_iter()
        .filter(|name| opened.is_open(name))
        .filter_map(|name| {
            let room = rooms.iter().find(|(had, _)| had == name)?.1;
            let row = super::ledger_of(world, name);
            Some(Yield {
                room: name,
                rate: row
                    .as_ref()
                    .map_or(Hourly::Unknown, |row| throughput.rate(&row.key, now)),
                row: row.map(|row| row.id),
                wait: super::wait::wait_of(world, room, &casts),
            })
        })
        .collect()
}
