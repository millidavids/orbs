//! `bind` — giving a spell to the orb to hold.
//!
//! §8 says what `bind` adds is running unattended. Gating the *cast* buys
//! nothing — [`Running`] fixes the domain at cast and never reads the player's
//! position again — so the difference is at the other end: an invocation ends
//! when the player leaves the domain it runs in (`run::advance`), and a bound
//! spell does not. A spell is still nameable and castable from anywhere; what
//! needs your presence is the *running*.
//!
//! A bound spell stands: it is cast again when it runs off the end, because a
//! slot held by a finished spell is a slot held by nothing. That makes *"walk
//! away, come back to work done"* true without a `repeat`.
//!
//! Nothing here counts anything — how many may be held is
//! [`tower::concentration`]. At 0 the orb cannot hold one at all, which is
//! §11.5's *"the game's turn"* and the first gate in the game on progression
//! rather than on where you stand.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::{Prose, with_extension};
use crate::parser::{Intent, Verb};
use crate::session::Scrollback;
use crate::tower::{self, Name};

use super::run::Running;

/// A spell the orb is holding.
///
/// Beside [`Running`] rather than instead of it — *"is it working right now"*
/// and *"is the orb holding it"* are different questions. A spell that has run
/// off the end is un-`Running` for the tick before this casts it again.
#[derive(Component, Debug, Clone, Default)]
pub struct Bound {
    /// Lines whose bad name has already been complained about, across laps.
    ///
    /// The rationing has to outlive the run, because holding is what laps:
    /// `Running::said` rations per casting and `finish` takes `Running` away
    /// between laps, so one typo was reported every two ticks for as long as
    /// the spell was held. Copied out at the end of a lap and back in at the
    /// start of the next.
    ///
    /// Cleared where `Running::said` is cleared — when the *text* changes.
    pub said: Vec<usize>,
}

/// Every spell the orb is currently holding, by name.
///
/// Ordered by [`NodeId`](crate::NodeId) rather than by query order —
/// `tower::node` records archetype order as a bug no test catches, and this
/// feeds a refusal that names them.
///
/// `&World`, through `try_query`, so the frontend can read it while drawing a
/// frame: §8 wants what is held in the sidebar, and a sidebar has no `&mut`.
/// `None` is a world where nothing has ever been bound, which is the empty
/// answer.
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
        // Unreachable through the parser — `bind`'s only slot is required, so a
        // bare `bind` becomes a numbered prompt. Answered rather than ignored,
        // because §3 forbids a silent return.
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

    // The first gate in the game about what you have earned. §6 forbids a bare
    // refusal, so the sentence teaches: a player at concentration 0 does not
    // know holding a spell is a thing, and cannot find the word by looking.
    let room = tower::concentration(world);
    if room == 0 {
        say(world, "bind_untrained", &wanted, Role::Danger);
        return;
    }
    let holding = held(world);
    if holding.len() >= room {
        // Naming what is held is the whole refusal: at concentration 1 this is
        // the sharpest decision in the game (§11.5), and a player who has to go
        // and look up what they hold cannot make it.
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

    // Already in flight, so it is taken up where it stands. `invoke x` then
    // `bind x` is the sequence the design recommends, and casting again would
    // throw away what the test had done and restart at line 1.
    if let Some(mut running) = world.get_mut::<Running>(node) {
        running.unattended = true;
        say(world, "bind_done", &wanted, Role::Success);
        // A spell bound is what the grimoire's mastery line counts — after the
        // sentence, on both doors.
        crate::tower::note(world, crate::tower::BOUND);
        return;
    }

    // Cast now, through the one door — a bound spell that waited for something
    // to start it would be a slot held by nothing.
    super::invoke::cast(world, node, &wanted, Role::Success, "bind_done", true);
    crate::tower::note(world, crate::tower::BOUND);
}

/// Let go of a spell the orb is holding. Returns whether it was holding one.
///
/// `stop` does this as well as ending a run, because *"stop doing that"* is one
/// idea. The consequence, named rather than solved: with a bound spell always
/// running there is no way to pause one without giving up the slot.
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
/// A system, because standing is a property of the tick rather than of any
/// command. `run::advance` removes `Running` and this puts it back on the next
/// tick, so one place ends a spell and one starts it again.
pub fn stand(world: &mut World) {
    let idle: Vec<(Entity, String)> = world
        .query_filtered::<(Entity, &Name), (With<Bound>, Without<super::Running>)>()
        .iter(world)
        .map(|(node, name)| (node, name.0.clone()))
        .collect();
    for (node, named) in idle {
        // Silent: `spell_begun` every lap is the noise §19 cut back from the
        // editor's saves. The work it does still reports itself.
        super::invoke::cast(world, node, &named, Role::Normal, "", true);
    }
}

/// The spell node called `wanted`, wherever it is kept.
///
/// One of three byte-identical copies of this walk, and now one call — see
/// `tower::reach`.
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
