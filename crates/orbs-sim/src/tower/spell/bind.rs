//! `bind` — giving a spell to the orb to hold.
//!
//! # What it buys, and why `invoke` had to change for it to buy anything
//!
//! §8 says what `bind` adds is running **unattended**. The obvious way to make
//! that mean something — narrow `invoke` to the room you are standing in — does
//! not work: [`Running`](super::Running) fixes the spell's domain at cast and
//! nothing checks the player's position afterwards, so
//! `attend laboratory; invoke brewing; attend archive` already runs unattended.
//! Gating the *cast* would have bought a walk back and nothing else.
//!
//! So the difference is at the other end. **An invocation ends when the player
//! leaves the domain it runs in** (`run::advance`); a bound spell does not. That
//! is §19's own reading of `invoke` — *"it needs you standing there"* — made
//! true rather than asserted, and it is a difference you feel in one keystroke.
//!
//! It does not contradict `scene::rebuild`'s *"a spell is not a domain, it is
//! the book you carry"*: a spell stays nameable and castable from anywhere. What
//! needs your presence is the *running*.
//!
//! # A bound spell stands
//!
//! It is cast again when it runs off the end, because a slot held by a spell
//! that has finished is a slot held by nothing. That is what makes
//! *"walk away, come back to work done"* true for a spell that is not already
//! wrapped in a `repeat` — and what makes the portfolio question §8 describes a
//! real one, because the thing you are holding is *working*, not merely stored.
//!
//! # Concentration is what limits it
//!
//! Nothing here counts anything: how many may be held is
//! [`tower::concentration`](crate::tower::concentration), derived from what the
//! player has earned. At 0 the orb cannot hold one at all, which is the tower
//! worked entirely by hand — §11.5's *"the game's turn"*, and the first thing in
//! the game that is gated on progression rather than on where you stand.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::{Prose, with_extension};
use crate::parser::{Intent, Verb};
use crate::session::Scrollback;
use crate::tower::{self, Name, Nameable};

use super::run::Running;

/// A spell the orb is holding.
///
/// Beside [`Running`](super::Running) rather than instead of it: a bound spell
/// is *also* running, and the two answer different questions —
/// *"is it working right now"* and *"is the orb holding it"*. A spell that has
/// run off the end is un-`Running` for the tick before this casts it again.
#[derive(Component, Debug, Clone, Default)]
pub struct Bound {
    /// Lines whose bad name has already been complained about, across laps.
    ///
    /// **The rationing has to outlive the run, because holding is what laps.**
    /// `Running::said` holds a missing name to one report per line per casting,
    /// and `finish` takes `Running` away between every lap — so a held spell with
    /// one typo in it reported that typo every two ticks for as long as it was
    /// held, which is the failure the rationing exists to prevent arriving
    /// through the mechanism that makes holding work. Copied out at the end of a
    /// lap and back in at the start of the next.
    ///
    /// Cleared where `Running::said` is cleared — when the *text* changes, which
    /// is the event that makes a name worth complaining about again.
    pub said: Vec<usize>,
}

/// Every spell the orb is currently holding, by name.
///
/// Ordered by [`NodeId`](crate::NodeId) rather than by query order —
/// `tower::node` records archetype order as a bug that changes what a phrase
/// resolves to with no test catching it, and this feeds a refusal that names
/// them.
///
/// **`&World`, through `try_query`**, so the frontend can read it while drawing
/// a frame — §8 wants what is held in the sidebar, and a sidebar has no `&mut`.
/// `None` is a world in which nothing has ever been bound, which is exactly the
/// empty answer.
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
        // bare `bind` becomes a numbered prompt rather than an intent. Answered
        // rather than ignored, because a silent return is the one thing §3
        // forbids and a `_ =>` that can never run is a claim nobody checks.
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

    // **The first gate in the game that is about what you have earned.** §6
    // forbids a bare refusal, so the sentence has to teach: a player at
    // concentration 0 does not know there is such a thing as holding a spell,
    // and the word that fixes it is not one they can find by looking around.
    let room = tower::concentration(world);
    if room == 0 {
        say(world, "bind_untrained", &wanted, Role::Danger);
        return;
    }
    let holding = held(world);
    if holding.len() >= room {
        // Naming what is held is the whole refusal: at concentration 1 this is
        // the sharpest decision in the game (§11.5), and it cannot be made by a
        // player who has to go and look up what they are already holding.
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

    // **Already in flight, so it is taken up where it stands.** `invoke` gives
    // the player a way to test a spell before committing a slot to it, which
    // makes `invoke x` then `bind x` the sequence the design recommends — and
    // casting again would silently throw away everything the test had done and
    // restart at line 1. The run is the same run; what changed is who is
    // watching it.
    if let Some(mut running) = world.get_mut::<Running>(node) {
        running.unattended = true;
        say(world, "bind_done", &wanted, Role::Success);
        return;
    }

    // Cast now, through the one door — a bound spell that waited for something
    // to start it would be a slot held by nothing.
    super::invoke::cast(world, node, &wanted, Role::Success, "bind_done", true);
}

/// Let go of a spell the orb is holding. Returns whether it was holding one.
///
/// `stop` does this as well as ending a run, because *"stop doing that"* is one
/// idea and holding is one of the two ways the orb can be doing something with a
/// spell. **The consequence, named rather than solved:** with a bound spell
/// always running there is no way to pause one without giving up the slot.
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
/// **A system, because standing is a property of the tick rather than of any
/// command.** A spell finishes inside `run::advance`, which removes `Running`;
/// this puts it back on the next tick, so there is exactly one place that ends a
/// spell and exactly one that starts it again.
pub fn stand(world: &mut World) {
    let idle: Vec<(Entity, String)> = world
        .query_filtered::<(Entity, &Name), (With<Bound>, Without<super::Running>)>()
        .iter(world)
        .map(|(node, name)| (node, name.0.clone()))
        .collect();
    for (node, named) in idle {
        // **Silent.** `spell_begun` on every lap would be one line per pass for
        // as long as the spell is held, which is the noise §19 cut back from the
        // editor's saves. The work it does still reports itself.
        super::invoke::cast(world, node, &named, Role::Normal, "", true);
    }
}

/// The spell node called `wanted`, wherever it is kept.
fn find(world: &World, wanted: &str) -> Option<Entity> {
    let root = tower::root(world);
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if world.get::<Nameable>(node).map(|kind| kind.0) == Some(crate::parser::NounKind::Script)
            && world.get::<Name>(node).is_some_and(|name| name.0 == wanted)
        {
            return Some(node);
        }
        stack.extend(tower::children_of(world, node));
    }
    None
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
