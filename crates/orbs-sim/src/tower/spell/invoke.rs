//! `invoke` — casting a spell yourself.
//!
//! # Invoking is not automation, and that is the point
//!
//! §19 settles that **automation wins nothing at all until the first
//! Concentration level is bought**, and that buying it is the moment the game
//! becomes what it advertises. `invoke` runs at concentration 0, so it cannot be
//! allowed to win anything either.
//!
//! It does not. It types for you, and typing was already free — §5.0 makes
//! issuing an action cost nothing but the time the action takes, and §14 forbids
//! any mechanic requiring fast typing. So an invoked spell takes the **same**
//! durations, occupies the **same** production slot, and needs you standing
//! there watching it.
//!
//! What `bind` will add is the two things that matter: the spell running
//! **unattended**, and §8's script speed advantage. Both arrive together with
//! Concentration, which is what keeps that purchase the game's turn.
//!
//! It also gives the player a way to **test** a spell before committing a
//! Concentration slot to it — which at concentration 1, where binding a second
//! spell means letting the first one go, is the sharpest decision in the game.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::Prose;
use crate::parser::{Intent, Verb};
use crate::session::Scrollback;
use crate::tower::{self, Domain, Held, Name, NodeId};

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
    let depth = world.resource::<super::run::Depth>().0;
    if depth.is_some_and(|depth| depth + 1 >= super::MAX_DEPTH) {
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

    let Some(spell) = world.get::<NodeId>(node).copied() else {
        return;
    };
    // **A spell runs in its own domain, wherever the player is.** That is what
    // domain-scoping buys: `invoke morning` from the archive runs the laboratory
    // spell in the laboratory, because that is what it was written for.
    //
    // Not acting at a distance in §19's sense — that rule is about *typing* a
    // command at a room you are not in. Casting a spell you already wrote is a
    // different act, and `bind` is still what makes it unattended.
    let Some(at) = world
        .get::<Domain>(node)
        .cloned()
        .and_then(|Domain(named)| crate::execute::find_domain(world, &named))
        .and_then(|place| world.get::<NodeId>(place).copied())
    else {
        say(world, "spell_homeless", &wanted, Role::Danger);
        return;
    };

    // **Derived here, from the text, every time.** `Held` stays the truth: §8's
    // hot-reload re-resolves only changed *lines*, and §8.1's sabotage surface
    // is *"a line reordered"* — so an enemy edits the text and the program is
    // whatever the text now means.
    let lines = world
        .get::<Held>(node)
        .map(|held| held.0.clone())
        .unwrap_or_default();
    let program = super::program::parse(&lines);
    let complaints = program.complaints.clone();
    let seen = world.resource::<Scrollback>().records().sequence();

    world.entity_mut(node).insert(Running {
        spell,
        program,
        pc: vec![0],
        loops: Vec::new(),
        seen,
        depth: depth.map_or(0, |depth| depth + 1),
        at,
        waiting_since: None,
    });
    say(world, "spell_begun", &wanted, Role::Success);

    // What the orb had to fix to read it. Said **after** it begins, because the
    // spell runs either way — §8 forbids both refusing at save and halting at
    // cast, so this is a report rather than a rejection.
    for complaint in complaints {
        let message = world.resource::<Prose>().line(
            complaint.key,
            &[("name", &wanted), ("count", &complaint.line.to_string())],
        );
        world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Completion)
            .text(FieldName::Name, Verb::Invoke.canonical())
            .text(FieldName::Path, &wanted)
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
/// # Why `stop` had to learn about spells
///
/// Nothing but running out of program removed [`Running`], so an invoked spell
/// could not be called off — and `repeat` with no count never runs out. A
/// player who wrote one had a spell working the laboratory for ever with no way
/// to reach it, which is §6's dead end arrived at from a direction the parser
/// could not see: `stop` took a `Place`, a spell is a `Script`, so the sentence
/// never resolved to the thing it named.
///
/// **What it is working on is left alone.** Stopping the spell is not stopping
/// the mortar — `stop mortar_and_pestle` is still how you cancel a run, and a
/// spell called off mid-brew leaves the brew to finish, exactly as it would if
/// you had typed the line yourself and walked away.
pub fn stop_spell(world: &mut World, named: &str) -> bool {
    let wanted = crate::content::with_extension(named);
    let Some(node) = find(world, &wanted) else {
        return false;
    };
    if world.get::<Running>(node).is_none() {
        say(world, "spell_not_running", &wanted, Role::Cost);
        return true;
    }
    world.entity_mut(node).remove::<Running>();
    say(world, "spell_stopped", &wanted, Role::Success);
    true
}

/// The spell node called `wanted`, wherever it is kept.
fn find(world: &World, wanted: &str) -> Option<Entity> {
    let root = tower::root(world);
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if world.get::<tower::Nameable>(node).map(|kind| kind.0)
            == Some(crate::parser::NounKind::Script)
            && world.get::<Name>(node).is_some_and(|name| name.0 == wanted)
        {
            return Some(node);
        }
        stack.extend(tower::children_of(world, node));
    }
    None
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
