//! Carrying out a tester's word, and the two records every word writes.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use super::word::DebugWord;
use super::{progress, shortcut};
use crate::content::Prose;
use crate::session::{Pending, Scrollback, Submissions};
use crate::tick::Tick;

/// Carry out a tester's word.
///
/// Recorded in the scrollback and `Submissions`, so it replays, but not in the
/// parse trace, which measures phrasings (§15).
pub fn open(world: &mut World, line: &str, word: DebugWord) {
    echo(world, line);
    match word {
        // Queued for the next tick like everything else.
        DebugWord::Spawn(order) => world.resource_mut::<Pending>().spawn(order),
        DebugWord::Swap => shortcut::swap(world),
        DebugWord::Ward => shortcut::ward(world),
        DebugWord::Course => shortcut::course(world),
        DebugWord::Circle => shortcut::circle(world),
        DebugWord::Siege => shortcut::siege(world),
        DebugWord::Learn(name) => progress::learn(world, line, name.as_deref()),
        DebugWord::Take(id) => progress::take(world, id.as_deref()),
        DebugWord::Reach(id) => progress::reach(world, id.as_deref()),
        DebugWord::Renown(total) => progress::renown(world, total),
    }
}

/// The line as typed, and its place in the journal.
///
/// The `Submission` is the load-bearing half: a debug word skips the parser,
/// so a replay without it would diverge.
fn echo(world: &mut World, line: &str) {
    let tick = *world.resource::<Tick>();
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Input)
        .text(FieldName::Message, line)
        .finish();
    world.resource_mut::<Submissions>().push(tick, line);
}

/// A tester's answer, unvoiced: a fact about what the word did.
pub(super) fn said(world: &mut World, message: &str) {
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Message, message)
        .finish();
}

/// One authored line under the word that said it.
pub(super) fn voiced(world: &mut World, word: &str, message: &str, role: Role) {
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, word)
        .text(FieldName::Message, message)
        .role(role)
        .finish();
}

/// A shortcut that found nothing to act on, saying so.
///
/// Silent, it would leave the next line to look wrong.
pub(super) fn nothing_to_give(world: &mut World, word: &str, key: &str) {
    let message = world.resource::<Prose>().line(key, &[]);
    voiced(world, word, &message, Role::Cost);
}
