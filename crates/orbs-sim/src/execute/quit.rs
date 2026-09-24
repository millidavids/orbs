//! Putting the orb down.
//!
//! A word, because every other way out is a key. `F10` and the terminal
//! build's `Ctrl-C` are both undiscoverable: §6 makes this a game you play by
//! *typing*, and a player who has learned twenty-five words for the tower has
//! learned nothing about how to stop — the argument `unfurl` already made for
//! `PageUp`.
//!
//! It reads right at every level too: `quit` closes the spell editor, the weave
//! screen, and now the orb. The word means *leave the thing you are in*.
//!
//! The sim owns only the *decision*. Whether leaving means dropping a window or
//! putting the terminal back is a frontend's business, so this asks — as
//! `scribe` asks for the editor and `unfurl` for the transcript — and the
//! frontend takes the request.
//!
//! That keeps leaving out of determinism: `(seed, submissions)` replays
//! identically whether or not anyone quit, because nothing here touches the
//! world.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::Prose;
use crate::parser::Verb;
use crate::session::Scrollback;

/// A request to put the orb down, and whether it has been confirmed.
///
/// Leaving is asked twice because nothing else in the game is irreversible and
/// unannounced: a `quit` meant for the spell editor, typed one surface too
/// high, would end the session and write the tower out. So the answer is
/// another `quit` — no new vocabulary, no modal surface, and the question says
/// what to type.
///
/// A word, not a keypress: a confirmation screen opening *because a keystroke
/// arrived* reads the keystrokes that opened it, a defect paid for once (§19).
///
/// The confirmed half is taken rather than read — see [`take`](Self::take) —
/// for the reason [`Unfurling`](super::Unfurling) gives: it fires once.
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
    /// Any other command answers *no*. A question that outlived the next line
    /// would be a `quit` typed a minute ago ending the session a minute later.
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

/// A request to open the orb's menu.
///
/// A take-once handshake, beside `scribe`, `unfurl`, `weave` and `wander` (and
/// the menagerie's `chorus`, until it went with the chant): the sim records the
/// decision and the frontend owns the screen.
///
/// Its own resource rather than a second flag on [`Quitting`] — the two are no
/// longer the same question (§19).
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct Menuing(bool);

impl Menuing {
    /// Whether a request is waiting, without taking it.
    #[must_use]
    pub const fn pending(&self) -> bool {
        self.0
    }

    /// Take the pending request, if there is one.
    pub const fn take(&mut self) -> bool {
        let asked = self.0;
        self.0 = false;
        asked
    }
}

/// Ask the frontend for the menu.
pub(super) fn menu(world: &mut World) {
    world.resource_mut::<Menuing>().0 = true;

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
