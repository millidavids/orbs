//! `invoke` — casting a spell yourself.
//!
//! Not automation: it types for you, at concentration 0, so an invoked spell
//! takes the same durations and slot and needs you standing there. Running
//! unattended is the whole of what `bind` adds (§19) — see
//! [`Running::unattended`](super::Running::unattended). It is how a player tests
//! a spell before giving it the one slot concentration 1 allows.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::Prose;
use crate::parser::{Intent, Verb};
use crate::session::Scrollback;
use crate::tower::{self, Domain, Held, NodeId};

use super::run::Running;

/// Begin running the spell an intent names.
pub fn invoke(intent: &Intent, world: &mut World) {
    let Some(argument) = intent.arguments.first() else {
        return;
    };
    let wanted = crate::content::with_extension(&argument.value);

    // §8's call-depth limit, which makes runaway recursion loud — see
    // [`MAX_DEPTH`](super::MAX_DEPTH). `None` means a player typed it.
    let caller = world.resource::<super::run::Caller>().0;
    if caller.is_some_and(|caller| caller.depth + 1 >= super::MAX_DEPTH) {
        say(world, "spell_too_deep", &wanted, Role::Danger);
        return;
    }

    let Some(node) = find(world, &wanted) else {
        say(world, "spell_unknown", &wanted, Role::Danger);
        return;
    };

    // A second copy would double every action and make the log unreadable.
    if world.get::<Running>(node).is_some() {
        say(world, "spell_already", &wanted, Role::Cost);
        return;
    }

    let empty = world
        .get::<Held>(node)
        .is_none_or(|held| held.0.iter().all(|line| line.trim().is_empty()));
    if empty {
        say(world, "spell_empty", &wanted, Role::Cost);
        return;
    }

    cast(world, node, &wanted, Role::Success, "spell_begun", false);
}

/// Put a spell into flight, however it was asked for.
///
/// One door for `invoke` and `bind`, so the compile step, the depth counter and
/// the complaints cannot drift apart. An empty `announce` is a lap nobody asked
/// for: no line, and the previous lap's rationing carried over. `held` is a
/// parameter because `bind` casts before `Bound` can be read back, and a spell a
/// held one casts is unattended without being held.
pub(super) fn cast(
    world: &mut World,
    node: Entity,
    wanted: &str,
    role: Role,
    announce: &str,
    held: bool,
) {
    let caller = world.resource::<super::run::Caller>().0;
    let Some(spell) = world.get::<NodeId>(node).copied() else {
        return;
    };
    // A spell runs in its own domain wherever the player is; §19's rule against
    // acting at a distance is about typing.
    let Some(at_place) = world
        .get::<Domain>(node)
        .cloned()
        .and_then(|Domain(named)| crate::execute::find_domain(world, &named))
    else {
        say(world, "spell_homeless", wanted, Role::Danger);
        return;
    };
    let Some(at) = world.get::<NodeId>(at_place).copied() else {
        say(world, "spell_homeless", wanted, Role::Danger);
        return;
    };

    // From the text every time, compiled into the tower's own names.
    let lines = super::source(world, node);
    let program = super::compile(world, at_place, &lines);
    let complaints = program.complaints().to_vec();
    let seen = world.resource::<Scrollback>().records().sequence();

    // Carried across a lap, from `Bound` because `finish` removes the run
    // between laps; clearing it reset the rationing twice a second. `scribe`
    // clears it when the text changes.
    let already = if announce.is_empty() {
        world
            .get::<super::Bound>(node)
            .map(|bound| bound.said.clone())
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let unattended = held || caller.is_some_and(|caller| caller.unattended);

    world.entity_mut(node).insert(Running {
        spell,
        program,
        pc: vec![0],
        loops: Vec::new(),
        seen,
        depth: caller.map_or(0, |caller| caller.depth + 1),
        at,
        waiting_since: None,
        biding: None,
        said: already,
        unattended,
        // Every cast is a new pass over the same lines: no accumulator from last
        // lap, and nothing suspended mid-part.
        vars: std::collections::BTreeMap::new(),
        part: None,
        stack: Vec::new(),
        // One cursor, mirroring the fields above: `swap_in` reads it first.
        strands: vec![super::Strand {
            pc: vec![0],
            ..Default::default()
        }],
        spent: false,
    });
    if !announce.is_empty() {
        say(world, announce, wanted, role);
    }

    // After it begins, because it runs either way: a report, not a rejection.
    // Not on a lap, or a held spell would repeat its complaints for as long as
    // it is held.
    if announce.is_empty() {
        return;
    }
    for complaint in complaints {
        let message = world.resource::<Prose>().line(
            complaint.key,
            &[("name", wanted), ("count", &complaint.line.to_string())],
        );
        world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Completion)
            .text(FieldName::Name, Verb::Invoke.canonical())
            .text(FieldName::Path, wanted)
            .count(FieldName::Quantity, complaint.line as u64)
            .text(FieldName::Message, &message)
            .role(Role::Cost)
            .finish();
    }
}

/// Stop the spell `named`, if one by that name is running.
///
/// Returns whether it found one, so `stop` can fall through to an instrument.
/// What it was working on is left alone: calling a spell off mid-brew leaves the
/// brew to finish, as walking away from a typed line would.
pub fn stop_spell(world: &mut World, named: &str) -> bool {
    let wanted = crate::content::with_extension(named);
    let Some(node) = find(world, &wanted) else {
        return false;
    };
    // Released first, or `stand` casts it straight back next tick.
    let released = super::bind::release(world, node, &wanted);

    if world.get::<Running>(node).is_none() {
        if !released {
            say(world, "spell_not_running", &wanted, Role::Cost);
        }
        return true;
    }
    world.entity_mut(node).remove::<Running>();
    if !released {
        say(world, "spell_stopped", &wanted, Role::Success);
    }
    true
}

/// The spell node called `wanted`, wherever it is kept — `tower::reach`'s walk.
fn find(world: &World, wanted: &str) -> Option<Entity> {
    tower::reach::look(world)
        .scope(tower::reach::Scope::Tower)
        .kind(crate::parser::NounKind::Script)
        .naming(tower::reach::Naming::Script)
        .find(wanted)
}

fn say(world: &mut World, key: &str, name: &str, role: Role) {
    let message = world.resource::<Prose>().line(key, &[("name", name)]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Invoke.canonical())
        .text(FieldName::Path, name)
        .text(FieldName::Message, &message)
        .role(role)
        .finish();
}
