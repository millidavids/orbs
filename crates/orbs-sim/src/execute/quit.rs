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

/// A request to put the orb down, and whether it has been confirmed.
///
/// # Why leaving is asked twice
///
/// **Nothing else in the game is irreversible and unannounced.** Every other
/// word can be undone, waited out or simply repeated; this one ends the session,
/// and the tower is written out on the way — so a `quit` meant for the spell
/// editor, typed one surface too high, would end the game instead of closing a
/// buffer. §6 makes this a game played by typing, which means the cost of a
/// mistyped word has to be bounded.
///
/// So `quit` **asks**, and the answer is another `quit`. No new vocabulary, no
/// modal surface, and the question says what to type — which is what the
/// refusals do everywhere else.
///
/// **A word, not a keypress.** A confirmation screen would be a fourth surface
/// that takes the keyboard, and one that opens *because a keystroke arrived*
/// reads the keystrokes that opened it — a defect this project has now paid for
/// once (§19).
///
/// The confirmed half is **taken rather than read** — see [`take`](Self::take)
/// — for the reason [`Unfurling`](super::Unfurling) gives: it fires once.
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
    /// **Any other command answers *no*.** A question that outlived the next
    /// line would be a `quit` typed a minute ago ending a session a minute
    /// later, which is the surprise the question exists to prevent.
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

    // **Said before it happens, and it is not ceremony.** The line lands in the
    // scrollback, which is the log a player can `peruse` next session — so a
    // recording ends with the player choosing to stop rather than simply
    // stopping, and a session that ended in a crash reads differently from one
    // that ended in a decision.
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
/// The sixth take-once handshake, beside `scribe`, `unfurl`, `weave`, `wander`
/// and `chorus`: the sim records the decision and the frontend owns the screen.
///
/// **Its own resource rather than a second flag on [`Quitting`]**, because the
/// two are no longer the same question. They were for one iteration — §19 has
/// why that was superseded.
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
