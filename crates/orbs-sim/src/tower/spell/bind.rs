//! `bind` — giving a spell to the orb to hold.
//!
//! What `bind` adds is running unattended (§8): an invocation ends when the
//! player leaves its domain (`run::advance`), and a bound spell does not. A bound
//! spell also stands — cast again when it runs off the end, because a slot held
//! by a finished spell is held by nothing — so *walk away and come back to work
//! done* needs no `repeat`. How many may be held is [`tower::concentration`].

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::{Prose, with_extension};
use crate::parser::{Intent, Verb};
use crate::session::Scrollback;
use crate::tower::{self, Name};

use super::run::Running;

/// A spell the orb is holding.
///
/// Beside [`Running`], not instead of it: a held spell is un-`Running` for the
/// tick between laps.
#[derive(Component, Debug, Clone, Default)]
pub struct Bound {
    /// Lines whose bad name has already been complained about, across laps.
    ///
    /// Here because `finish` takes `Running` away between laps, and one typo was
    /// reported every two ticks. Cleared when the text changes.
    pub said: Vec<usize>,
}

/// Every spell the orb is currently holding, by name.
///
/// By [`NodeId`](crate::NodeId), not query order, since a refusal names them.
/// `&World` through `try_query`, so a sidebar can draw it; `None` is a world
/// where nothing was ever bound.
#[must_use]
pub fn held(world: &World) -> Vec<String> {
    let Some(mut query) = world.try_query::<(&crate::NodeId, &Name, &Bound)>() else {
        return Vec::new();
    };
    let mut found: Vec<(crate::NodeId, String)> = query
        .iter(world)
        .map(|(id, name, _)| (*id, name.0.clone()))
        .collect();
    found.sort_unstable_by_key(|(id, _)| *id);
    found.into_iter().map(|(_, name)| name).collect()
}

/// `bind <spell>` — ask the orb to hold a spell.
pub fn bind(intent: &Intent, world: &mut World) {
    let Some(argument) = intent.arguments.first() else {
        // The parser prompts for the slot instead; answered, because §3 forbids
        // a silent return.
        say(world, "spell_unknown", "", Role::Danger);
        return;
    };
    let wanted = with_extension(&argument.value);

    let Some(node) = find(world, &wanted) else {
        say(world, "spell_unknown", &wanted, Role::Danger);
        return;
    };
    if world.get::<Bound>(node).is_some() {
        say(world, "bind_already", &wanted, Role::Cost);
        return;
    }

    // The game's first gate on what you have earned, so the refusal teaches: at
    // concentration 0 a player cannot know holding a spell is a thing.
    let room = tower::concentration(world);
    if room == 0 {
        say(world, "bind_untrained", &wanted, Role::Danger);
        return;
    }
    let holding = held(world);
    if holding.len() >= room {
        // Naming what is held is the whole refusal: at concentration 1 it is the
        // sharpest decision in the game (§11.5).
        let message = world.resource::<Prose>().line(
            "bind_full",
            &[
                ("name", &wanted),
                ("count", &room.to_string()),
                ("detail", &holding.join(", ")),
            ],
        );
        push(world, &wanted, &message, Role::Danger);
        return;
    }

    world.entity_mut(node).insert(Bound::default());

    // Already in flight from `invoke x`, the recommended test, so it is taken up
    // where it stands rather than restarted at line 1.
    if let Some(mut running) = world.get_mut::<Running>(node) {
        running.unattended = true;
        say(world, "bind_done", &wanted, Role::Success);
        // The grimoire's mastery line, after the sentence, on both doors.
        crate::tower::note(world, crate::tower::BOUND);
        return;
    }

    super::invoke::cast(world, node, &wanted, Role::Success, "bind_done", true);
    crate::tower::note(world, crate::tower::BOUND);
}

/// Let go of a spell the orb is holding. Returns whether it was holding one.
///
/// `stop` does this as well as ending a run, because *stop doing that* is one
/// idea. So there is no way to pause a bound spell without giving up the slot.
pub fn release(world: &mut World, node: Entity, named: &str) -> bool {
    if world.get::<Bound>(node).is_none() {
        return false;
    }
    world.entity_mut(node).remove::<Bound>();
    say(world, "bind_released", named, Role::Success);
    true
}

/// Cast every bound spell that has run off the end.
///
/// A system, because standing belongs to the tick: `run::advance` ends a spell
/// and this starts it again next tick.
pub fn stand(world: &mut World) {
    let idle: Vec<(Entity, String)> = world
        .query_filtered::<(Entity, &Name), (With<Bound>, Without<super::Running>)>()
        .iter(world)
        .map(|(node, name)| (node, name.0.clone()))
        .collect();
    for (node, named) in idle {
        // Silent: a line every lap is noise; the work still reports itself.
        super::invoke::cast(world, node, &named, Role::Normal, "", true);
    }
}

/// The spell node called `wanted`, wherever it is kept — `tower::reach`'s walk.
fn find(world: &World, wanted: &str) -> Option<Entity> {
    tower::reach::look(world)
        .scope(tower::reach::Scope::Tower)
        .kind(crate::parser::NounKind::Script)
        .naming(tower::reach::Naming::Script)
        .find(wanted)
}

/// Say one authored line about `named`.
fn say(world: &mut World, key: &str, named: &str, role: Role) {
    let message = world.resource::<Prose>().line(key, &[("name", named)]);
    push(world, named, &message, role);
}

/// Push one record about a spell, in the shape `invoke`'s already take.
fn push(world: &mut World, named: &str, message: &str, role: Role) {
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Bind.canonical())
        .text(FieldName::Path, named)
        .text(FieldName::Message, message)
        .role(role)
        .finish();
}
