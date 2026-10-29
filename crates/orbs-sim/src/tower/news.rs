//! The tower's news reaching the player, wherever they stand and whoever did
//! the work (§19, *the tower speaks up*).
//!
//! A station reached, a room opened, a rank or a level is drawn even when a
//! spell earned it (`RecordBuilder::news`), and latches `!` on the room's box
//! when the player is elsewhere — the rail's *something happened over there*,
//! which a lens find alone used to raise.

use bevy_ecs::prelude::*;

use super::node::{Cwd, Name, domain_of};

/// Latch news on `room` unless the player is standing in it.
///
/// The player's own position, not `Cwd`: a spell's line runs with `Cwd` set to
/// the spell's room, so a ward a solver breaks would always look attended.
pub(crate) fn latch(world: &mut World, room: &str) {
    let cwd = world
        .resource::<super::spell::Caller>()
        .0
        .map_or_else(|| world.resource::<Cwd>().0, |casting| casting.player);
    let here = domain_of(world, cwd)
        .and_then(|domain| world.get::<Name>(domain))
        .is_some_and(|name| name.0 == room);
    if !here && super::DOMAINS.contains(&room) {
        super::mark(world, room, super::Mark::News);
    }
}
