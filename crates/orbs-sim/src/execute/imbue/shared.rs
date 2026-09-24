//! What all three of the forge's words need.
//!
//! The bailey's `shared.rs`, one room over, and for the same reason: the
//! alternative cut — one file per verb — would put three copies of *find the
//! lattice, refuse if there is none* in three places.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::Prose;
use crate::parser::Verb;
use crate::session::Scrollback;
use crate::tower::{self, charm};

/// The lattice, if the player is standing where one is.
pub(super) fn fixture(world: &World) -> Option<Entity> {
    crate::execute::readings::fixture(world, Verb::Imbue)
}

/// Say one line, in the forge's voice.
///
/// `FieldName::Source` is the lattice, which is what makes `peruse forge.log`
/// work: `execute::files::in_domain` matches `Source` against the domain, so a
/// line without it lands nowhere a player can read it back.
///
/// `args` interpolates the prose and nothing else. Writing each one into a
/// record field too made the view draw it *after* the sentence, so `forge_set`
/// read the tool's name twice. A field is for something a reader would `sift`
/// on, not a second copy of the message.
pub(super) fn say(world: &mut World, verb: Verb, key: &str, args: &[(&str, &str)], role: Role) {
    let message = world.resource::<Prose>().line(key, args);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, verb.canonical())
        .text(FieldName::Message, &message)
        .text(FieldName::Source, charm::LATTICE)
        .role(role)
        .finish();
}

/// Whether a siege is being fought anywhere, which is what the surcharge asks.
pub(super) fn besieged(world: &mut World) -> bool {
    world
        .query::<&tower::Siege>()
        .iter(world)
        .any(tower::Siege::running)
}
