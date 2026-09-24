//! `invoke` — casting a spell yourself.
//!
//! Invoking is not automation, and that is the point: §19 settles that
//! automation wins nothing until the first Concentration level is bought, and
//! `invoke` runs at concentration 0. It types for you, and typing was already
//! free (§5.0, §14), so an invoked spell takes the same durations, occupies the
//! same production slot, and needs you standing there.
//!
//! What `bind` adds is running unattended, and that is the whole of it — §11.5's
//! invariant about a script action being faster than a manual one was struck
//! when `bind` was built (§19). See
//! [`Running::unattended`](super::Running::unattended).
//!
//! It also lets a player *test* a spell before committing a Concentration slot,
//! which at concentration 1 — where binding a second spell means letting the
//! first go — is the sharpest decision in the game.

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

    // How deep this `invoke` is. `None` means a player typed it.
    //
    // §8 permits a spell to `invoke` another to a **call-depth limit of 3**, and
    // argues why the execution budget alone is not a sufficient recursion guard:
    // exhausting it makes every subsequent instruction *Budget starved*, which
    // logs at high verbosity only, so all automation would stop **silently**.
    // Depth-limiting makes runaway recursion loud and diagnosable.
    let caller = world.resource::<super::run::Caller>().0;
    if caller.is_some_and(|caller| caller.depth + 1 >= super::MAX_DEPTH) {
        say(world, "spell_too_deep", &wanted, Role::Danger);
        return;
    }

    let Some(node) = find(world, &wanted) else {
        say(world, "spell_unknown", &wanted, Role::Danger);
        return;
    };

    // Already running. Starting a second copy would double every action it takes
    // and make the log unreadable, and §8's contention rules are about *different*
    // scripts rather than a script racing itself.
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
/// One door, shared by `invoke` and `bind`: they differ in what they *buy* — an
/// invocation ends when the player leaves — and not at all in how a spell is
/// started. Two copies would be two places for the compile step, the depth
/// counter and the complaint report to drift apart.
///
/// `announce` is the prose key for "it has begun", or empty for a cast nobody
/// asked for, because one line per lap is the noise §19 cut from the editor's
/// saves. An empty `announce` is also what makes a lap a lap rather than a fresh
/// beginning — see `said` below, carried over rather than cleared.
///
/// `held` says the orb is holding this spell, which is the one thing the two
/// doors genuinely differ on. A parameter rather than a look at `Bound`, because
/// `bind` casts before it could read it back and because a spell cast by a held
/// spell is unattended without being held itself.
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
    // A spell runs in its own domain wherever the player is, which is what
    // domain-scoping buys. Not acting at a distance in §19's sense — that rule
    // is about *typing* a command at a room you are not in.
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

    // Derived here from the text, every time. `Held` stays the truth: an enemy
    // edits the text and the program is whatever the text now means.
    let lines = super::source(world, node);
    // Compiled, not merely parsed: this is where the loose phrasing in the file
    // becomes the tower's own names, since the file holds exactly what the
    // player typed.
    let program = super::compile(world, at_place, &lines);
    let complaints = program.complaints().to_vec();
    let seen = world.resource::<Scrollback>().records().sequence();

    // Carried across a lap, cleared for a beginning. `said` rations a bad name
    // to one report per line per casting, and a standing spell is cast again
    // every time it runs off the end — so clearing it here put the rationing
    // back to square one twice a second. `scribe` clears it when the *text*
    // changes, which is what makes a name worth complaining about again.
    //
    // Read off `Bound`, not off `Running`: `finish` removes the run between
    // laps, so the run is exactly the thing that cannot carry it.
    let already = if announce.is_empty() {
        world
            .get::<super::Bound>(node)
            .map(|bound| bound.said.clone())
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    // A spell the player invoked needs them standing there; one a bound spell
    // set going does not, and neither does a binding itself. See
    // [`Running::unattended`].
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
        // Empty at every cast, including a binding's re-cast: a spell that ran
        // off the end and starts again is a new pass over the same lines, and an
        // accumulator holding last lap's answer would make the first comparison
        // of this one ask about a world that has moved.
        vars: std::collections::BTreeMap::new(),
        // A cast opens on the spell's own body with nothing suspended behind it,
        // which is the same *"a new pass over the same lines"* the store above
        // argues for: a binding that laps mid-part would otherwise resume inside
        // a call the new pass never made.
        part: None,
        stack: Vec::new(),
        // One cursor at every cast, and `alongside` is the only thing that adds
        // a second. It mirrors the fields above rather than deriving from them,
        // because `swap_in` reads this slot before the first step and a strand
        // list that disagreed would step a cursor pointing nowhere.
        strands: vec![super::Strand {
            pc: vec![0],
            ..Default::default()
        }],
        spent: false,
    });
    if !announce.is_empty() {
        say(world, announce, wanted, role);
    }

    // What the orb had to fix to read it. Said **after** it begins, because the
    // spell runs either way — §8 forbids both refusing at save and halting at
    // cast, so this is a report rather than a rejection.
    //
    // Only when the cast was asked for: a bound spell recompiles on every lap,
    // so a spell with an unreadable line would print the same complaint for as
    // long as it is held.
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
///
/// `stop` had to learn about spells because nothing but running out of program
/// removed [`Running`], and `repeat` with no count never runs out — so a player
/// who wrote one had a spell working the laboratory for ever. §6's dead end from
/// a direction the parser could not see: `stop` took a `Place`, a spell is a
/// `Script`, so the sentence never resolved to the thing it named.
///
/// What it is working on is left alone: stopping the spell is not stopping the
/// mortar, and a spell called off mid-brew leaves the brew to finish exactly as
/// it would if you had typed the line yourself and walked away.
pub fn stop_spell(world: &mut World, named: &str) -> bool {
    let wanted = crate::content::with_extension(named);
    let Some(node) = find(world, &wanted) else {
        return false;
    };
    // Released first, or it stands straight back up: `stand` casts every bound
    // spell that is not running, so removing `Running` from a held spell would
    // put it back next tick. Letting go is what `stop` means for a held spell,
    // and is also how a slot is freed for another (§11.5).
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

/// The spell node called `wanted`, wherever it is kept.
///
/// One of three byte-identical copies of this walk, and now one call — see
/// `tower::reach` for the rule and what the other two cost.
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
