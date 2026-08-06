//! Reading back through what the orb has said.
//!
//! # A verb for a key that already worked
//!
//! `PageUp` has scrolled the transcript since the transcript existed, and
//! nothing ever said so: the border advertises `PgDn newest` only once you are
//! *already* scrolled back, so the affordance announced itself exclusively to
//! players who had found it. In a game with no mouse and no menus, a key nobody
//! can discover is a key nobody has.
//!
//! So the way in is a word, like everything else here. Using it once puts the
//! keys on screen, which is the part that survives after the player stops
//! needing the word.
//!
//! # What the sim owns, and what it does not
//!
//! Only the *decision*. Where the transcript is scrolled to is frontend state
//! and always was — it is a fact about a pane, and rule 2 keeps panes out of
//! this crate entirely. What `unfurl` does here is ask, exactly as `scribe`
//! asks for the editor, and the frontend takes the request.
//!
//! That also keeps it out of the sim's determinism: reading is not a world
//! event. `(seed, submissions)` replays identically whether or not anyone
//! scrolled, because nothing here touches the world.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::Prose;
use crate::parser::Verb;
use crate::session::Scrollback;

/// A request to hand the transcript the keyboard.
///
/// **Taken rather than read**, for the reason `Opening` gives: `unfurl` asks
/// once, and a frontend polling a persistent flag would re-enter reading mode
/// every frame — including the frame after the player pressed Escape to leave.
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct Unfurling(bool);

impl Unfurling {
    /// Ask for the transcript to take the keyboard.
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
