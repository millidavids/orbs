//! What is true of one node, put back on it.
//!
//! Split from [`super::restore`], which owns the world's *shape* — the clock,
//! the streams, the tree, the resources. This owns what hangs off each node, and
//! the two halves are separated by a real seam rather than a line count: every
//! function below needs the tree to already exist.
//!
//! [`references`] is why the seam is there at all. `Working::subject` and
//! `Running::{spell, at}` are `NodeId`s that the document spells as **paths**,
//! so neither can be resolved while the tree is still being walked — the node a
//! path names may not have been created yet. A second pass over the finished
//! tree costs nothing and removes the ordering question entirely.

use bevy_ecs::prelude::*;
use orbs_render::FieldName;

use super::document::Save;
use super::naming;
use super::node::NodeSave;
use crate::session::Scrollback;
use crate::tick::Tick;
use crate::tower::{self, Maze, Stock, Ward, spell};

/// Put one node's components on, and take off anything the save says is absent.
///
/// **Both directions matter.** A raised athanor is cold and a saved one may be
/// burning; a raised log is clean and a saved one may be poisoned — but equally,
/// a log the player `purge`d is clean in the save and would stay poisoned if
/// this only ever inserted.
pub(super) fn apply(world: &mut World, entity: Entity, node: &NodeSave) {
    marker(world, entity, node.protected, tower::Protected);
    marker(world, entity, node.fixture, tower::Fixture);
    marker(world, entity, node.heat_source, tower::HeatSource);
    marker(world, entity, node.store, tower::Store);
    marker(world, entity, node.keep, tower::Keep);
    marker(world, entity, node.reading, tower::Reading);
    marker(world, entity, node.product, tower::Product);
    marker(world, entity, node.poisoned, tower::Poisoned);
    marker(world, entity, node.log, tower::Log);

    // **Removed then re-inserted, like every carried non-marker below.** A
    // fixture that stopped being one of a set between two builds must lose the
    // component, or a `for each` would walk a member the tower no longer groups.
    let mut at = world.entity_mut(entity);
    at.remove::<tower::Grouped>();
    if !node.group.is_empty() {
        at.insert(tower::Grouped(node.group.clone()));
    }

    let mut at = world.entity_mut(entity);
    at.remove::<tower::Operation>();
    at.remove::<Stock>();
    at.remove::<tower::Held>();
    at.remove::<tower::Domain>();
    at.remove::<tower::Banked>();
    at.remove::<tower::Ash>();
    at.remove::<tower::Bidden>();
    at.remove::<spell::Bound>();
    at.remove::<tower::Working>();
    at.remove::<tower::Triaging>();
    at.remove::<tower::Burning>();
    at.remove::<tower::Quickened>();
    at.remove::<tower::Substituted>();
    at.remove::<Maze>();
    at.remove::<Ward>();
    at.remove::<spell::Running>();

    if let Some(word) = node.operation.as_deref().and_then(naming::verb_from) {
        at.insert(tower::Operation(word));
    }
    if let Some(stock) = node.stock.as_deref() {
        at.insert(if stock == "endless" {
            Stock::Endless
        } else {
            Stock::Counted(stock.parse().unwrap_or(1))
        });
    }
    if let Some(lines) = node.held.clone() {
        at.insert(tower::Held(lines));
    }
    if let Some(domain) = node.domain.clone() {
        at.insert(tower::Domain(domain));
    }
    if let Some(banked) = node.banked {
        at.insert(tower::Banked(banked));
    }
    if let Some(ash) = node.ash.clone() {
        at.insert(tower::Ash(ash));
    }
    if let Some(bidden) = node.bidden.clone() {
        at.insert(tower::Bidden(bidden));
    }
    if let Some(said) = node.bound.clone() {
        at.insert(spell::Bound { said });
    }
    if let Some(triage) = node.triaging.as_ref() {
        at.insert(tower::Triaging {
            started: Tick::new(triage.started),
            ends: Tick::new(triage.ends),
        });
    }
    if let Some(fire) = node.burning.as_ref() {
        at.insert(tower::Burning {
            lit_at: Tick::new(fire.started),
            ticks: fire.ends.saturating_sub(fire.started),
        });
    }
    if let Some(quick) = node.quickened.as_ref() {
        at.insert(tower::Quickened {
            from: Tick::new(quick.started),
            ticks: quick.ends.saturating_sub(quick.started),
        });
    }
    if let Some(lie) = node.substituted.as_ref() {
        at.insert(tower::Substituted {
            was: lie.was.clone(),
            since: Tick::new(lie.since),
        });
    }
    if let Some(maze) = node.maze.as_ref() {
        at.insert(Maze::from_save(maze));
    }
    if let Some(ward) = node.ward.as_ref() {
        at.insert(Ward::from_save(ward));
    }
    // **`None` raises no course at all**, which is the graceful end of a
    // malformed one: the player walks into a sanctum they can `muster` in
    // rather than one jammed on a course that can never finish. See
    // `Course::from_save` for the four ways it says no.
    if let Some(course) = node.course.as_ref().and_then(tower::Course::from_save) {
        at.insert(course);
    }
    // `Running` is deliberately not here: it needs the whole tree in place to
    // resolve the two paths it carries, and it needs the record stream in place
    // to complain if it cannot. See `spells`.
}

/// Insert or remove a unit component to match the save.
///
/// The value is passed rather than defaulted because none of these markers
/// implements `Default` — and should not. A `Protected` conjured out of nothing
/// is exactly the kind of component that turns up somewhere it was never put.
fn marker<T: Component>(world: &mut World, entity: Entity, wanted: bool, value: T) {
    let mut at = world.entity_mut(entity);
    if wanted {
        at.insert(value);
    } else {
        at.remove::<T>();
    }
}

/// The two components that name another node, put back once the tree exists.
///
/// `Working::subject` and `Running::{spell, at}` are `NodeId`s, and the document
/// spells them as paths — so neither can be resolved while [`apply`] is still
/// walking, because the node a path names may not have been created yet. A
/// second pass over the finished tree costs nothing and removes the ordering
/// question entirely.
pub(super) fn references(world: &mut World, save: &Save) {
    let found: Vec<(Entity, &NodeSave)> = save
        .nodes
        .iter()
        .filter_map(|node| Some((tower::find_by_path(world, &node.path)?, node)))
        .collect();

    for (entity, node) in found {
        if let Some(work) = node.working.as_ref() {
            let subject = tower::find_by_path(world, &work.subject)
                .and_then(|at| world.get::<tower::NodeId>(at).copied());
            // A run whose subject has gone is a run that can never land, so it
            // is dropped rather than resumed against nothing. Only a hand-edited
            // file can reach this: capture writes the path it read.
            if let (Some(subject), Some(verb)) = (subject, naming::verb_from(&work.verb)) {
                world.entity_mut(entity).insert(tower::Working {
                    verb,
                    subject,
                    started: Tick::new(work.started),
                    ends: Tick::new(work.ends),
                });
            }
        }
        spell(world, entity, node);
    }
}

/// Put one part-way-through spell back.
///
/// # Why the program is re-derived rather than carried
///
/// `tower::spell::program` is explicit that the **text is the single source of
/// truth** and the program is rebuilt at every cast. Carrying a compiled tree in
/// the save would give a spell two sources of truth that a hand-edited file
/// could put out of step.
///
/// # ...and why re-deriving it blind would not be safe
///
/// `pc` is a *path into the tree the spell was cast against* — `[2, 1]` is the
/// second step inside the third — and two things can have moved since: the
/// spell's own text, because editing one mid-flight is a shipped feature, and a
/// reagent's name, because §8.1's substitution surface is the whole point of the
/// domain. Walk a stale `pc` into a freshly compiled tree and the orb runs the
/// wrong line, or `program::at` returns nothing and the run ends with no reason
/// given.
///
/// So the save carries a fingerprint of the text it compiled, and a mismatch
/// **ends the run and says so** rather than resuming into a program that is not
/// the one the position belongs to. A spell that is `Bound` will be cast again
/// on the next tick from the top, which is the correct recovery and needs no
/// code here at all.
fn spell(world: &mut World, entity: Entity, node: &NodeSave) {
    {
        let Some(run) = node.running.as_ref() else {
            return;
        };
        let Some(spell) = tower::find_by_path(world, &run.spell) else {
            return;
        };
        let Some(at) = tower::find_by_path(world, &run.at) else {
            return;
        };
        let lines = world
            .get::<tower::Held>(spell)
            .map(|held| held.0.clone())
            .unwrap_or_default();

        if fingerprint(&lines) != run.fingerprint {
            moved_under_it(world, &run.spell);
            return;
        }

        let (Some(spell_id), Some(at_id)) = (
            world.get::<tower::NodeId>(spell).copied(),
            world.get::<tower::NodeId>(at).copied(),
        ) else {
            return;
        };

        let program = spell::compile(world, spell, &lines);
        world.entity_mut(entity).insert(spell::Running {
            spell: spell_id,
            program,
            pc: run.pc.clone(),
            loops: run.loops.iter().copied().map(loop_from).collect(),
            seen: run.seen,
            depth: run.depth,
            unattended: run.unattended,
            at: at_id,
            waiting_since: run.waiting_since.map(Tick::new),
            said: run.said.clone(),
            vars: run.vars.clone(),
            part: run.part.clone(),
            stack: run
                .stack
                .iter()
                .map(|frame| spell::Descent {
                    part: frame.part.clone(),
                    pc: frame.pc.clone(),
                    loops: frame.loops.iter().copied().map(loop_from).collect(),
                    vars: frame.vars.clone(),
                })
                .collect(),
        });
    }
}

/// Say that a spell's text changed while it was not running.
///
/// In voice and in the log rather than silently, because the player is about to
/// find a spell they left running has stopped, and §8's failure taxonomy is
/// *"scripts always log and never halt"* — halting quietly is the one thing it
/// forbids.
fn moved_under_it(world: &mut World, spell: &str) {
    let name = spell.rsplit_once('/').map_or(spell, |(_, leaf)| leaf);
    let message = world
        .resource::<crate::content::Prose>()
        .line("spell_moved_under_it", &[("name", name)]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(orbs_render::RecordKind::Message)
        .text(FieldName::Name, name)
        .text(FieldName::Message, &message)
        .role(orbs_render::Role::Danger)
        .finish();
}

/// A `Loop`, as one number.
///
/// `repeat 5` counts down, `repeat until` counts nothing, and a branch of an
/// `if` runs once. A save with a position and no counts would resume every
/// enclosing loop from its first turn, so the counts have to travel — and one
/// integer per open block reads better in a file than a tagged table would.
pub(super) fn loop_code(open: &spell::Loop) -> i64 {
    match open {
        spell::Loop::Repeat(Some(turns)) => i64::from(*turns),
        spell::Loop::Repeat(None) => -1,
        spell::Loop::Branch => -2,
        // **Below the two markers, one step per member.** `-3` is the first,
        // `-4` the second. A `for each` is the third block shape and the first
        // to carry a number *and* need a tag, which is why it takes a range
        // rather than a single value.
        spell::Loop::Each(index) => -3 - i64::from(*index),
    }
}

/// ...and back.
fn loop_from(code: i64) -> spell::Loop {
    match code {
        -1 => spell::Loop::Repeat(None),
        -2 => spell::Loop::Branch,
        // A hand-edited file could put anything here; a set walked from a member
        // that does not exist simply ends, which is what `for each` does when it
        // runs off the end anyway.
        each if each <= -3 => spell::Loop::Each(u32::try_from(-3 - each).unwrap_or(0)),
        turns => spell::Loop::Repeat(Some(u32::try_from(turns).unwrap_or(0))),
    }
}

/// A cheap, stable hash of a spell's text.
///
/// **FNV-1a written out, not `DefaultHasher`.** `RandomState` is seeded per
/// process, so a fingerprint taken today and compared tomorrow would never
/// match — the check would fire on every load and quietly stop every running
/// spell, which is the opposite of what it is for.
pub(super) fn fingerprint(lines: &[String]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for line in lines {
        for byte in line.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        hash ^= u64::from(b'\n');
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}
