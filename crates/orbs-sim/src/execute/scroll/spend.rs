//! Which scroll a `wield` names, and where it goes.
//!
//! Spent with `wield` rather than a word of its own: spending a scroll is
//! *setting a thing going*, so the verb learned a second argument kind
//! ([`NounKind::Workable`]) instead of the tower gaining another word.
//!
//! Rule 6, as everywhere in `execute`: facts here, sentences looked up by key.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::Prose;
use crate::parser::{Intent, NounKind};
use crate::session::Scrollback;
use crate::tower;

/// A scroll the archive can assemble, and what it does.
///
/// A table of facts, not prose — the shape of `Verb::canonical`. The words are
/// authored in `recipes.toml`; this says which the game can act on, and
/// `every_scroll_the_lectern_makes_can_be_spent` fails the build if the two
/// disagree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scroll {
    /// While open, the stacks gather rather than escape.
    Gleaning,
    /// While open, the laboratory works faster.
    Quickening,
    /// While open, grows the next herb the laboratory has never had.
    Verdant,
}

impl Scroll {
    /// Every scroll that has an effect behind it.
    pub const ALL: [Self; 3] = [Self::Gleaning, Self::Quickening, Self::Verdant];

    /// The name this is authored and carried under.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Gleaning => "gleaning-scroll",
            Self::Quickening => "quickening-scroll",
            Self::Verdant => "verdant-scroll",
        }
    }

    /// The scroll called `name`, if the game has one.
    #[must_use]
    pub fn of(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|scroll| scroll.word() == name)
    }
}

/// The base reagent the tower opens with, and the yardstick for where the
/// laboratory is.
pub(super) const SAGE: &str = "sage";

/// The scroll this `wield` spends, if it spends one rather than charging a tool.
///
/// The one expression of that question, for [`spend`] below and for
/// `tower::spell::block`, which must *not* charge a spend to the production
/// slot. Asking it twice went wrong once: a scripted `wield quickening-scroll`
/// waited out the brew it was meant to hurry.
///
/// Branches on the argument's *kind*, not its name: `Recipes::kind_of` decides
/// what a scroll is, from the content file.
pub(crate) fn spending(intent: &Intent) -> Option<&str> {
    intent
        .arguments
        .first()
        .filter(|argument| argument.kind == NounKind::Scroll)
        .map(|argument| argument.value.as_str())
}

/// Spend the scroll `intent` names.
///
/// Returns whether the argument was a scroll at all, so `wield` can fall through
/// to its instrument lookup.
pub(in crate::execute) fn spend(intent: &Intent, world: &mut World) -> bool {
    let Some(name) = spending(intent) else {
        return false;
    };
    let name = name.to_owned();
    super::calm::open(world, &name);
    true
}

/// The laboratory, wherever the tower keeps its sage.
pub(super) fn laboratory(world: &World) -> Option<Entity> {
    tower::home(world, SAGE).and_then(|shelf| tower::domain_of(world, shelf))
}

/// Say why the scroll was not spent, and leave it in the player's hands.
pub(super) fn refuse(world: &mut World, name: &str, key: &str) {
    say(world, key, name, &[], Role::Cost);
}

/// Say a scroll's line, with `name` and whatever else the key wants.
pub(super) fn say(world: &mut World, key: &str, name: &str, args: &[(&str, &str)], role: Role) {
    let mut all = vec![("name", name)];
    all.extend_from_slice(args);
    speak(world, key, name, &all, role);
}

/// [`say`], with every field already in hand, `name` among them.
pub(super) fn speak(world: &mut World, key: &str, name: &str, fields: &[(&str, &str)], role: Role) {
    let message = world.resource::<Prose>().line(key, fields);
    let kind = if role == Role::Success {
        RecordKind::Completion
    } else {
        RecordKind::Message
    };
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(kind)
        .text(FieldName::Name, name)
        .text(FieldName::Message, &message)
        .role(role)
        .finish();
}
