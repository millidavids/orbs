//! What all three of the forge's words need.
//!
//! The bailey's `shared.rs`, one room over.

use bevy_ecs::prelude::*;
use orbs_render::Role;

use crate::parser::Verb;
use crate::tower::charm;

/// The lattice, if the player is standing where one is.
pub(super) fn fixture(world: &World) -> Option<Entity> {
    crate::execute::readings::fixture(world, Verb::Imbue)
}

/// Say one line, in the forge's voice.
///
/// `args` interpolates the prose and nothing else; as fields too, the view
/// drew each a second time.
pub(super) fn say(world: &mut World, verb: Verb, key: &str, args: &[(&str, &str)], role: Role) {
    crate::execute::readings::filed(world, charm::LATTICE, verb, key, args, role);
}
