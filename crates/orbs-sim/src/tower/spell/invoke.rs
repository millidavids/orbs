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
//! What `bind` adds is **running unattended**, and that is the whole of it:
//! §11.5's invariant about a script action being faster than a manual one was
//! struck when `bind` was built (§19), so a binding sells one thing rather than
//! two. It is still the game's turn, because *walk away and come back to work
//! done* is the sentence pillar 3 is made of — and because "needs you standing
//! there" above was an assertion nothing enforced until `bind` gave it something
//! to be true against. See [`Running::unattended`](super::Running::unattended).
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
/// **One door, shared by `invoke` and `bind`.** They differ in what they *buy* —
/// an invocation ends when the player leaves, a binding does not — and not at
/// all in how a spell is started. Two copies of this would be two places for the
/// compile step, the depth counter and the complaint report to drift apart.
///
/// `announce` is the prose key for "it has begun", or **empty** for a cast
/// nobody asked for: a bound spell standing up again on its own says nothing,
/// because one line per lap is the noise §19 cut back from the editor's saves.
///
/// **An empty `announce` is also what makes a lap a lap** rather than a fresh
/// beginning — see `said` below, which is carried over rather than cleared.
///
/// `held` says the orb is holding this spell, which is the one thing the two
/// doors genuinely differ on. A parameter rather than a look at the `Bound`
/// component, because `bind` casts *before* it would be able to read it back and
/// because a spell cast by a held spell is unattended without being held itself.
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
    // **A spell runs in its own domain, wherever the player is.** That is what
    // domain-scoping buys: `invoke morning` from the archive runs the laboratory
    // spell in the laboratory, because that is what it was written for.
    //
    // Not acting at a distance in §19's sense — that rule is about *typing* a
    // command at a room you are not in. Casting a spell you already wrote is a
    // different act, and `bind` is what makes it survive you walking out.
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

    // **Derived here, from the text, every time.** `Held` stays the truth: §8's
    // hot-reload re-resolves only changed *lines*, and §8.1's sabotage surface
    // is *"a line reordered"* — so an enemy edits the text and the program is
    // whatever the text now means.
    let lines = world
        .get::<Held>(node)
        .map(|held| held.0.clone())
        .unwrap_or_default();
    // **Compiled, not merely parsed.** This is where the loose phrasing in the
    // file becomes the tower's own names — the moment §8 asks for and the file
    // no longer provides, since it holds exactly what the player typed.
    let program = super::compile(world, at_place, &lines);
    let complaints = program.complaints().to_vec();
    let seen = world.resource::<Scrollback>().records().sequence();

    // **Carried across a lap, cleared for a beginning.** `said` rations a bad
    // name to one report per line per casting, and a standing spell is cast
    // again every time it runs off the end — so clearing it here put the
    // rationing back to square one twice a second, which is the failure it
    // exists to prevent arriving through the fix for a different one. A lap is
    // not a new casting; `scribe` clears it when the *text* changes, which is
    // the event that makes a name worth complaining about again.
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
        said: already,
        unattended,
        // **Empty at every cast**, including a binding's re-cast. A spell that
        // has run off the end and starts again is a new pass over the same
        // lines, and an accumulator left holding last lap's answer would make
        // the first comparison of this one ask about a world that has moved.
        vars: std::collections::BTreeMap::new(),
    });
    if !announce.is_empty() {
        say(world, announce, wanted, role);
    }

    // What the orb had to fix to read it. Said **after** it begins, because the
    // spell runs either way — §8 forbids both refusing at save and halting at
    // cast, so this is a report rather than a rejection.
    //
    // **Only when the cast was asked for.** A bound spell recompiles on every
    // lap, and a spell with an unreadable line would otherwise print the same
    // complaint for as long as it is held — the failure the once-per-cast
    // rationing on `Running::said` exists to stop, arriving through the recast
    // that resets it.
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
    // **Released first, or it stands straight back up.** `stand` casts every
    // bound spell that is not running, so removing `Running` from a held spell
    // would put it back on the next tick — the player would type `stop` and
    // watch nothing happen. Letting go is what `stop` means for a spell the orb
    // is holding, which is also how a slot is freed for another (§11.5).
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
