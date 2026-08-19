//! Putting the orb down.
//!
//! # A word, because every other way out is a key
//!
//! `F10` has left the game since there was a game to leave, and the terminal
//! build added `Ctrl-C` because raw mode makes it ours to answer. Neither is
//! discoverable: §6 makes this a game you play by *typing*, and a player who has
//! learned twenty-five words for what happens inside the tower has learned
//! nothing about how to stop. In a game with no mouse and no menus, a key nobody
//! can find is a key nobody has — which is the argument `unfurl` already made
//! for `PageUp`, and this is the same shape.
//!
//! It also reads right at every level. `quit` closes the spell editor, `quit`
//! closes the weave screen, and now `quit` closes the orb: the word means *leave
//! the thing you are in*, whichever thing that is.
//!
//! # What the sim owns, and what it does not
//!
//! Only the *decision*. Whether leaving means dropping a window, or leaving raw
//! mode and putting the terminal back, is entirely a frontend's business — and
//! the two answers have nothing in common. So this asks, exactly as `scribe`
//! asks for the editor and `unfurl` for the transcript, and the frontend takes
//! the request.
//!
//! That also keeps it out of the sim's determinism: leaving is not a world
//! event. `(seed, submissions)` replays identically whether or not anyone quit,
//! because nothing here touches the world — a replay simply runs past it, which
//! is what you want when the recording outlives the session.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::Prose;
use crate::parser::Verb;
use crate::session::Scrollback;

/// A request to put the orb down.
///
/// **Taken rather than read**, for the reason [`Unfurling`](super::Unfurling)
/// gives: `quit` asks once. A frontend polling a persistent flag would be
/// correct here by luck — there is no frame after this one — but the shape is
/// the shape, and a fifth handshake that behaved differently from the other four
/// is the one somebody copies wrong.
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct Quitting(bool);

impl Quitting {
    /// Ask for the session to end.
    pub const fn ask(&mut self) {
        self.0 = true;
    }

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

/// Ask the frontend to put the orb down.
pub(super) fn quit(world: &mut World) {
    world.resource_mut::<Quitting>().ask();

    // **Said before it happens, and it is not ceremony.** The line lands in the
    // scrollback, which is the log a player can `peruse` next session — so a
    // recording ends with the player choosing to stop rather than simply
    // stopping, and a session that ended in a crash reads differently from one
    // that ended in a decision.
    let message = world.resource::<Prose>().line("quit_begins", &[]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Quit.canonical())
        .text(FieldName::Message, &message)
        .role(Role::Normal)
        .finish();
}
