//! Moving past a step, and the questions that decide whether a loop goes round.

use bevy_ecs::prelude::*;

use crate::tower::{self, Cwd};

use super::run::{Running, node_of};
use super::strand::standing_for;

/// Move past the step just run, closing and repeating blocks as needed.
///
/// The program is cloned out because `step_past` needs it while `Running` is
/// borrowed mutably on the same entity.
pub(super) fn advance_pc(world: &mut World, entity: Entity) {
    let Some((program, part)) = world
        .get::<Running>(entity)
        .map(|state| (state.program.clone(), state.part.clone()))
    else {
        return;
    };
    let body = super::program::tree(program.body(), part.as_deref())
        .cloned()
        .unwrap_or_default();
    // Every guard answered before `Running` is borrowed mutably.
    let (at, pc, vars) = world.get::<Running>(entity).map_or_else(
        || (None, Vec::new(), std::collections::BTreeMap::new()),
        |state| {
            (
                node_of(world, state.at),
                state.pc.clone(),
                state.vars.clone(),
            )
        },
    );
    let guards = guard_answers(world, &body, at, &pc, &vars);
    if let Some(mut running) = world.get_mut::<Running>(entity) {
        let Running { pc, loops, .. } = &mut *running;
        let again = |at: &[usize], popped: super::Loop| {
            let answer = guards.iter().find(|(path, _)| path.as_slice() == at);
            match (popped, answer) {
                // No entry is no set, so the loop stops rather than walking
                // nothing for ever.
                (super::Loop::Each(index), Some((_, Continues::Members(many)))) => {
                    index + 1 < *many
                }
                (super::Loop::Each(_), _) => false,
                // An unguarded `repeat` has no entry and goes round.
                (_, Some((_, Continues::Guard(more)))) => *more,
                (_, _) => true,
            }
        };
        if !super::program::step_past(&body, pc, loops, again) {
            // Off the end: `at` returns `None` next time, the one place a spell
            // ends.
            pc.clear();
        }
    }
}

/// Answer `condition` in the spell's own room, not the player's.
///
/// One function, because `until` once answered where the player stood while
/// `if` swapped, and a bound spell saw nothing and stopped.
pub(super) fn asked_where_the_spell_is(
    world: &mut World,
    at: Option<Entity>,
    condition: &crate::parser::Condition,
) -> (Option<bool>, Vec<String>) {
    let player = world.resource::<Cwd>().0;
    let Some(room) = at else {
        return (None, Vec::new());
    };
    world.insert_resource(Cwd(room));
    let asked = super::watch::holds(world, condition);
    world.insert_resource(Cwd(player));
    asked
}

/// What a block that has run off the end needs to know to go round again.
#[derive(Debug, Clone, Copy)]
enum Continues {
    /// A `repeat until`: whether the loop may take another turn.
    Guard(bool),
    /// A `for each`: how many members its set has, now.
    Members(u32),
}

/// Whether each guarded loop the path is inside may take another turn, by path.
///
/// An unanswerable `until` stops the loop, the opposite of `if`: declining to
/// act is safe, and a `repeat` that declined to *stop* would run for ever.
fn guard_answers(
    world: &mut World,
    body: &super::Block,
    at: Option<Entity>,
    pc: &[usize],
    vars: &std::collections::BTreeMap<String, String>,
) -> Vec<(Vec<usize>, Continues)> {
    fn walk(
        world: &World,
        body: &super::Block,
        pc: &[usize],
        vars: &std::collections::BTreeMap<String, String>,
        path: &mut Vec<usize>,
        out: &mut Vec<(Vec<usize>, Continues)>,
    ) {
        for (index, step) in body.iter().enumerate() {
            path.push(index);
            match &step.kind {
                super::Kind::Repeat { until, body, .. } => {
                    // Only an ancestor of `pc` is running. A bound name stands
                    // for its place, as it does in an `if`.
                    if let Some(condition) = until
                        && pc.starts_with(path)
                    {
                        let condition = standing_for(vars, condition);
                        let more = matches!(super::watch::holds(world, &condition).0, Some(false));
                        out.push((path.clone(), Continues::Guard(more)));
                    }
                    walk(world, body, pc, vars, path, out);
                }
                super::Kind::Each { group, body } => {
                    if pc.starts_with(path) {
                        let room = world.resource::<Cwd>().0;
                        let many = tower::group_at(world, room, group).len();
                        out.push((
                            path.clone(),
                            Continues::Members(u32::try_from(many).unwrap_or(u32::MAX)),
                        ));
                    }
                    walk(world, body, pc, vars, path, out);
                }
                super::Kind::If {
                    body, otherwise, ..
                } => {
                    // `enter_branch` pushes the half and the step, so a loop in
                    // an `if` is two elements deeper — matching `step_past`.
                    for (half, block) in [body, otherwise].into_iter().enumerate() {
                        path.push(half);
                        walk(world, block, pc, vars, path, out);
                        path.pop();
                    }
                }
                // A definition's loops belong to another frame.
                super::Kind::Part { .. }
                | super::Kind::Call { .. }
                | super::Kind::Command(_)
                | super::Kind::Wait(_)
                | super::Kind::Bide(_)
                | super::Kind::Let { .. }
                | super::Kind::Pull { .. }
                | super::Kind::Alongside { .. } => {}
            }
            path.pop();
        }
    }

    // Swapped once around the whole walk: every question is the spell's room's.
    let player = world.resource::<Cwd>().0;
    let Some(room) = at else {
        return Vec::new();
    };
    world.insert_resource(Cwd(room));
    let mut out = Vec::new();
    walk(world, body, pc, vars, &mut Vec::new(), &mut out);
    world.insert_resource(Cwd(player));
    out
}
