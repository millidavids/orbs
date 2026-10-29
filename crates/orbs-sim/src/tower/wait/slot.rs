//! What holds the tower's one production slot (§9, §11.5).

use bevy_ecs::prelude::*;

use crate::tower::Working;
use crate::tower::node::Name;

/// The work holding the slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slot {
    /// The instrument at work.
    pub name: String,
    /// What it is doing, as the verb's participle.
    pub doing: &'static str,
    /// Ticks until it lands.
    pub left: u64,
}

/// The work holding the slot, if any. At [`CAPACITY`](crate::tower::CAPACITY)
/// 1 it is what every brew queues behind, so `status rates` names it.
#[must_use]
pub fn slot(world: &World) -> Option<Slot> {
    let now = *world.resource::<crate::tick::Tick>();
    let mut query = world.try_query::<(&Working, &Name)>()?;
    query
        .iter(world)
        .min_by_key(|(working, _)| working.started.get())
        .map(|(working, name)| Slot {
            name: name.0.clone(),
            doing: working.verb.participle(),
            left: working.ends.get().saturating_sub(now.get()),
        })
}
