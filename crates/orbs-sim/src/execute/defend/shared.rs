//! The rampart, and the one sentence every other file here says things with.
//!
//! **Two functions and a constant, extracted because all four siblings need
//! them.** `say` in particular is the reason this file exists rather than the
//! helpers living with the verbs: a refusal written twice is a refusal that can
//! disagree with itself, and §6 forbids a bare error in either copy.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::Prose;
use crate::parser::Verb;
use crate::session::Scrollback;
use crate::tower::siege;

/// Where a bailey record is filed.
pub(super) const RAMPART: &str = siege::RAMPART;

/// The rampart, where the player is standing.
pub(super) fn fixture(world: &World) -> Option<Entity> {
    crate::execute::readings::fixture(world, Verb::Defend)
}

/// Say one thing, filed to the bailey.
pub(super) fn say(world: &mut World, verb: Verb, key: &str, args: &[(&str, &str)], role: Role) {
    let message = world.resource::<Prose>().line(key, args);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, verb.canonical())
        .text(FieldName::Message, &message)
        .text(FieldName::Source, RAMPART)
        .role(role)
        .finish();
}
