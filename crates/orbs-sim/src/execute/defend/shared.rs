//! The rampart, and the one sentence every other file here says things with.
//!
//! Shared by every sibling, so a refusal is written once.

use bevy_ecs::prelude::*;
use orbs_render::Role;

use crate::parser::Verb;
use crate::tower::siege;

/// Where a bailey record is filed.
pub(super) const RAMPART: &str = siege::RAMPART;

/// The rampart, where the player is standing.
pub(super) fn fixture(world: &World) -> Option<Entity> {
    crate::execute::readings::fixture(world, Verb::Defend)
}

/// Say one thing, filed to the bailey.
pub(super) fn say(world: &mut World, verb: Verb, key: &str, args: &[(&str, &str)], role: Role) {
    crate::execute::readings::filed(world, RAMPART, verb, key, args, role);
}
