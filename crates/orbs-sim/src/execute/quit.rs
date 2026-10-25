//! Putting the orb down.
//!
//! A word, because every other way out is an undiscoverable key — `unfurl`'s
//! argument for `PageUp`. `quit` means *leave the thing you are in*, at every
//! level. The sim owns only the decision and asks, as `scribe` does, so
//! leaving touches nothing a replay sees.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::Prose;
use crate::parser::Verb;
use crate::session::Scrollback;

/// A request to put the orb down, and whether it has been confirmed.
///
/// Asked twice, since a `quit` typed one surface too high would end the
/// session. The answer is another `quit`, a word rather than a keypress, which
/// would read the keystrokes that opened it (§19). The confirmed half is taken,
/// so it fires once.
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct Quitting {
    /// The orb has asked, and the next line answers.
    asked: bool,
    /// The answer was yes.
    leaving: bool,
}

impl Quitting {
    /// Whether the orb is waiting to be told again.
    #[must_use]
    pub const fn is_asking(&self) -> bool {
        self.asked
    }

    /// Forget the question, because the player did something else.
    ///
    /// Any other command answers *no*, so an old `quit` cannot end the session
    /// later.
    pub const fn never_mind(&mut self) {
        self.asked = false;
    }

    /// Whether a confirmed request is waiting, without taking it.
    #[must_use]
    pub const fn pending(&self) -> bool {
        self.leaving
    }

    /// Take the pending request, if there is one.
    pub const fn take(&mut self) -> bool {
        let leaving = self.leaving;
        self.leaving = false;
        leaving
    }
}

/// Ask to leave, or leave if the orb already asked.
pub(super) fn quit(world: &mut World) {
    let asked = world.resource::<Quitting>().is_asking();
    let key = if asked { "quit_begins" } else { "quit_asks" };
    if asked {
        let mut quitting = world.resource_mut::<Quitting>();
        quitting.asked = false;
        quitting.leaving = true;
    } else {
        world.resource_mut::<Quitting>().asked = true;
    }

    // Said before it happens: the line lands in the scrollback a player can
    // `peruse` next session, so a session that ended in a crash reads
    // differently from one that ended in a decision.
    let message = world.resource::<Prose>().line(key, &[]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Quit.canonical())
        .text(FieldName::Message, &message)
        .role(Role::Normal)
        .finish();
}

/// Ask the frontend for the menu.
pub(super) fn menu(world: &mut World) {
    world.resource_mut::<super::Menuing>().ask();

    let message = world.resource::<Prose>().line("menu_begins", &[]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Menu.canonical())
        .text(FieldName::Message, &message)
        .role(Role::Normal)
        .finish();
}
