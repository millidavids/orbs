//! One instruction: a command, a `wait`, a `bide` or a `pull`, and what happens
//! to it. §8's failure taxonomy lives here: a spell logs and moves on.

use bevy_ecs::prelude::*;
use orbs_render::Role;

use crate::parser::{Mode, Resolution, Verb, analyse};
use crate::session::Scrollback;
use crate::tick::Tick;
use crate::tower::{self, Cwd};

use super::block::{Blocked, would_block};
use super::guard::advance_pc;
use super::run::{LONGEST_BIDE, PATIENCE, Running, line_of, node_of, walking};
use super::saying::{already_said, say_blocked, say_failure, say_missing, say_waiting};
use super::speech::Refused;

/// What happened to one instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Progress {
    /// It ran, or failed in a way the taxonomy logs and continues past.
    Done,
    /// It cannot proceed yet; the spell keeps its place and tries next tick.
    Blocked,
}

/// Hold until something the spell named happens.
///
/// Only what arrived since the cursor, so a `wait` waits for something *new* and
/// a loop's second turn does not return on the first turn's event.
pub(super) fn wait_for(
    world: &mut World,
    entity: Entity,
    state: &Running,
    wanted: &str,
) -> Progress {
    let stream = world.resource::<Scrollback>().records();
    let dropped = stream.dropped();
    let start = usize::try_from(state.seen.saturating_sub(dropped)).unwrap_or(0);

    let found = stream
        .iter()
        .skip(start)
        .filter_map(|record| super::watch::watch(&record))
        .any(|event| event.names(wanted));
    let now = stream.sequence();

    if found {
        if let Some(mut running) = world.get_mut::<Running>(entity) {
            running.seen = now;
            running.waiting_since = None;
        }
        advance_pc(world, entity);
        return Progress::Done;
    }

    let tick = *world.resource::<Tick>();
    let since = match state.waiting_since {
        Some(since) => since,
        None => {
            say_waiting(world, state, wanted);
            if let Some(mut running) = world.get_mut::<Running>(entity) {
                running.waiting_since = Some(tick);
            }
            tick
        }
    };

    if tick.get().saturating_sub(since.get()) >= PATIENCE {
        say_failure(world, state, "spell_gave_up", wanted, Role::Danger);
        if let Some(mut running) = world.get_mut::<Running>(entity) {
            running.waiting_since = None;
            running.seen = now;
        }
        advance_pc(world, entity);
        return Progress::Done;
    }
    hold(world, entity, super::Hold::Watching(wanted.to_owned()))
}

/// The fault a spell line says when its verb is handed words it cannot use:
/// one for a verb taking nothing, one for a verb taking only a closed few.
/// Read at cast and at run, so the two say the same.
pub(super) const fn takes_nothing_key(verb: Verb) -> &'static str {
    if verb.narrows().is_some() {
        "spell_takes_only"
    } else {
        "spell_takes_nothing"
    }
}

/// Yield, saying what on. The one way a turn ends blocked.
///
/// One hold a spell, however many cursors are held: the one that says most,
/// the first cursor's on a tie, so a `bide` beside a blocked cursor does not
/// hide what blocks it.
fn hold(world: &mut World, entity: Entity, why: super::Hold) -> Progress {
    if let Some(mut running) = world.get_mut::<Running>(entity)
        && running.held.as_ref().is_none_or(|held| why.outranks(held))
    {
        running.held = Some(why);
    }
    Progress::Blocked
}

/// Execute one line.
pub(super) fn run_line(world: &mut World, entity: Entity, state: &Running, line: &str) -> Progress {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        advance_pc(world, entity);
        return Progress::Done;
    }

    let scene = tower::scene_at(world, world.resource::<Cwd>().0);
    let resolution = analyse(line, &scene, Mode::Calm).resolution;
    let for_the_tower = super::whole::is_tower(world, state.at);
    match Refused::reading(&resolution, for_the_tower) {
        Some(Refused::Forbidden) => {
            say_failure(
                world,
                state,
                Refused::Forbidden.of_line(),
                line,
                Role::Danger,
            );
            advance_pc(world, entity);
            return Progress::Done;
        }
        Some(Refused::TowerOnly) => {
            return refused(
                world,
                entity,
                state,
                Refused::TowerOnly.of_line(),
                line,
                Role::Danger,
            );
        }
        None => {}
    }
    let intent = match resolution {
        Resolution::Resolved { intent, .. } => intent,
        // A verb handed words it cannot use — see `reading::one`.
        Resolution::TakesNothing { verb, extra, .. } => {
            say_failure(world, state, takes_nothing_key(verb), &extra, Role::Danger);
            advance_pc(world, entity);
            return Progress::Done;
        }
        // §8's *Referent missing*: skips, logs, continues.
        Resolution::Ambiguous { .. }
        | Resolution::Incomplete { .. }
        | Resolution::Elsewhere { .. }
        | Resolution::InSpell { .. }
        | Resolution::Unresolved { .. } => {
            say_failure(world, state, "spell_missing", line, Role::Danger);
            advance_pc(world, entity);
            return Progress::Done;
        }
    };

    // Here rather than in `invoke`, so it is said once a line, not a lap.
    if let (Verb::Invoke, Some(wanted)) = (intent.verb, intent.arguments.first())
        && let Some((key, detail, role)) =
            super::invoke::refused_from(world, state, wanted.value.as_str())
    {
        return refused(world, entity, state, key, &detail, role);
    }

    if let Some(blocked) = would_block(world, &intent) {
        return wait(world, entity, state, &blocked);
    }

    // The same dispatch a typed line takes, or the live game and the balance
    // harness stop being the same game (§13).
    crate::execute::execute_one(&intent, world);
    if let Some(mut running) = world.get_mut::<Running>(entity) {
        running.waiting_since = None;
    }
    advance_pc(world, entity);
    Progress::Done
}

/// Refuse the line `say_failure`'s way, said once a line rather than every time
/// it runs, and move on.
fn refused(
    world: &mut World,
    entity: Entity,
    state: &Running,
    key: &str,
    detail: &str,
    role: Role,
) -> Progress {
    let at = line_of(state).and_then(|line| usize::try_from(line).ok());
    if !at.is_some_and(|line| already_said(world, entity, line)) {
        say_failure(world, state, key, detail, role);
    }
    advance_pc(world, entity);
    Progress::Done
}

/// Spend `ticks` doing nothing, then move on.
///
/// `waiting_since` carries the countdown rather than a second counter, which
/// also saves it for free. It never reaches `PATIENCE`: a bide is blocked on
/// nothing, so it says nothing.
pub(super) fn bide(world: &mut World, entity: Entity, state: &Running, delay: u32) -> Progress {
    let now = *world.resource::<Tick>();
    // Latched as a pair on the tick the bide begins: `waiting_since` alone cannot
    // say whether it has started.
    let (since, ticks) = match (state.waiting_since, state.biding) {
        (Some(since), Some(ticks)) => (since, ticks),
        _ => {
            let ticks = delay.min(LONGEST_BIDE);
            if let Some(mut running) = world.get_mut::<Running>(entity) {
                running.waiting_since = Some(now);
                running.biding = Some(ticks);
            }
            (now, ticks)
        }
    };
    // `ticks - 1` because completing spends a tick of its own: `bide n` means the
    // next line runs n ticks from here.
    let spent = now.get().saturating_sub(since.get());
    let wanted = u64::from(ticks.saturating_sub(1));
    if spent >= wanted {
        if let Some(mut running) = world.get_mut::<Running>(entity) {
            running.waiting_since = None;
            running.biding = None;
        }
        advance_pc(world, entity);
        return Progress::Done;
    }
    hold(world, entity, super::Hold::Biding(wanted - spent))
}

/// Hold the spell where it is, saying so once.
fn wait(world: &mut World, entity: Entity, state: &Running, blocked: &Blocked) -> Progress {
    let now = *world.resource::<Tick>();
    let since = match state.waiting_since {
        Some(since) => since,
        None => {
            // One line, on the first tick; a record per tick would bury the log.
            say_blocked(world, state, blocked);
            if let Some(mut running) = world.get_mut::<Running>(entity) {
                running.waiting_since = Some(now);
            }
            now
        }
    };

    if now.get().saturating_sub(since.get()) >= PATIENCE {
        say_failure(world, state, "spell_gave_up", blocked.name(), Role::Danger);
        if let Some(mut running) = world.get_mut::<Running>(entity) {
            running.waiting_since = None;
        }
        advance_pc(world, entity);
        return Progress::Done;
    }
    hold(world, entity, super::Hold::Busy(blocked.clone()))
}

/// Take the oldest name out of a satchel and bind it, or yield until there is
/// one.
///
/// In the spell's room, never `Cwd`: every domain has a satchel under one name,
/// so a producer would fill one queue while its consumer drained another.
/// Yielding is not waiting — it never touches `waiting_since` or reaches
/// [`PATIENCE`]. A thing that holds no satchel is a missing name, said once.
pub(super) fn pull(
    world: &mut World,
    entity: Entity,
    state: &Running,
    name: &str,
    from: &str,
) -> Progress {
    let line = super::program::at(walking(state), &state.pc).map_or(0, |step| step.line);
    // Asked here as well as at cast, `may_issue`'s rule; silent, because
    // `compile::check_learned` said it at cast.
    if !tower::holds(world, tower::Grant::Satchel) {
        advance_pc(world, entity);
        return Progress::Done;
    }
    let at = node_of(world, state.at).and_then(|room| {
        tower::children_of(world, room)
            .into_iter()
            .find(|node| {
                world
                    .get::<tower::Name>(*node)
                    .is_some_and(|it| it.0 == crate::parser::leaf(from))
            })
            .filter(|node| world.get::<tower::Satchel>(*node).is_some())
    });
    let Some(at) = at else {
        say_missing(world, entity, state, line, &[from.to_owned()]);
        advance_pc(world, entity);
        return Progress::Done;
    };

    let Some(mut satchel) = world.get_mut::<tower::Satchel>(at) else {
        say_missing(world, entity, state, line, &[from.to_owned()]);
        advance_pc(world, entity);
        return Progress::Done;
    };
    let Some(taken) = satchel.take() else {
        return hold(world, entity, super::Hold::Pulling(from.to_owned()));
    };

    if let Some(mut running) = world.get_mut::<Running>(entity) {
        running.vars.insert(name.to_owned(), taken);
    }
    advance_pc(world, entity);
    Progress::Done
}
