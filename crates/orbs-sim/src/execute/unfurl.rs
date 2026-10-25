//! Reading back through what the orb has said.
//!
//! A word for a key nobody could discover: `PageUp` announced itself only to
//! players already scrolled back. Using the word once puts the keys on screen.
//! The sim owns only the decision and asks, as `scribe` does, so scrolling
//! touches nothing a replay sees.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use super::Unfurling;
use crate::content::Prose;
use crate::parser::Verb;
use crate::session::Scrollback;

/// Hand the transcript the keyboard.
pub(super) fn unfurl(world: &mut World) {
    world.resource_mut::<Unfurling>().ask();

    let message = world.resource::<Prose>().line("unfurl_begins", &[]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Unfurl.canonical())
        .text(FieldName::Message, &message)
        .role(Role::Normal)
        .finish();
}
