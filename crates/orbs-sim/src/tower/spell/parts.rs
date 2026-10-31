//! Parts: calling one, returning from it, and forking one as a second cursor.

use bevy_ecs::prelude::*;
use orbs_render::Role;

use crate::tower;

use super::guard::advance_pc;
use super::run::{Descent, MAX_PARTS, MAX_STRANDS, Running};
use super::saying::{say_failure, say_missing};

/// Hand the current frame back to whoever called it.
///
/// Returns whether there was one; `false` means the outermost frame ran out.
/// The caller is stepped past its call here, not on the way in, so every saved
/// `pc` points at a line the save can explain.
pub(super) fn returned(world: &mut World, entity: Entity) -> bool {
    // Scoped, so the borrow ends before `advance_pc` takes the world again.
    let resumed = {
        let Some(mut running) = world.get_mut::<Running>(entity) else {
            return false;
        };
        let Some(descent) = running.stack.pop() else {
            return false;
        };
        running.part = descent.part;
        running.pc = descent.pc;
        running.loops = descent.loops;
        running.vars = descent.vars;
        true
    };
    if resumed {
        advance_pc(world, entity);
    }
    resumed
}

/// Suspend this frame and open one on `name`, with `args` as its whole store.
///
/// A missing part, the wrong number of names, or a stack past [`MAX_PARTS`] are
/// each said and stepped past (§8). Arity is checked here as well as in
/// `compile` because the text can change under a running spell.
pub(super) fn called(
    world: &mut World,
    entity: Entity,
    state: &Running,
    line: usize,
    name: &str,
    args: &[String],
) {
    let Some(params) = super::program::signature(state.program.body(), name) else {
        // The definition went away while the spell ran.
        say_missing(world, entity, state, line, &[name.to_owned()]);
        advance_pc(world, entity);
        return;
    };
    if params.len() != args.len() {
        say_failure(world, state, "spell_call_arity_now", name, Role::Danger);
        advance_pc(world, entity);
        return;
    }
    if state.stack.len() >= MAX_PARTS {
        say_failure(world, state, "spell_parts_too_deep", name, Role::Danger);
        advance_pc(world, entity);
        return;
    }
    // Resolved in the caller's store before it is put away, one level deep.
    let handed: Vec<String> = args
        .iter()
        .map(|arg| {
            state
                .vars
                .get(arg.as_str())
                .cloned()
                .unwrap_or_else(|| arg.clone())
        })
        .collect();
    if let Some(mut running) = world.get_mut::<Running>(entity) {
        let descent = Descent {
            part: running.part.clone(),
            pc: running.pc.clone(),
            loops: std::mem::take(&mut running.loops),
            vars: std::mem::take(&mut running.vars),
        };
        running.stack.push(descent);
        running.part = Some(name.to_owned());
        running.pc = vec![0];
        running.vars = params.into_iter().zip(handed).collect();
    }
}

/// Start `name` as a second cursor and step past — `alongside gathering()`.
///
/// [`called`] without the suspension: no [`Descent`], an empty stack because
/// nothing waits for it, and bounded by [`MAX_STRANDS`]. `run::step_one`
/// re-reads the count, so a strand forked this tick is stepped this tick.
pub(super) fn forked(
    world: &mut World,
    entity: Entity,
    state: &Running,
    line: usize,
    name: &str,
    args: &[String],
) {
    // Silent: the complaint at cast is the report, and this stops the line.
    if !tower::holds(world, tower::Grant::Cursors) {
        advance_pc(world, entity);
        return;
    }
    let Some(params) = super::program::signature(state.program.body(), name) else {
        say_missing(world, entity, state, line, &[name.to_owned()]);
        advance_pc(world, entity);
        return;
    };
    if params.len() != args.len() {
        say_failure(world, state, "spell_call_arity_now", name, Role::Danger);
        advance_pc(world, entity);
        return;
    }
    if state.strands.len() >= MAX_STRANDS {
        say_failure(world, state, "spell_strands_too_many", name, Role::Danger);
        advance_pc(world, entity);
        return;
    }
    // Resolved at the moment of the fork: the caller runs on, and its store
    // would have moved by any later reading (§19).
    let handed: Vec<String> = args
        .iter()
        .map(|arg| {
            state
                .vars
                .get(arg.as_str())
                .cloned()
                .unwrap_or_else(|| arg.clone())
        })
        .collect();
    if let Some(mut running) = world.get_mut::<Running>(entity) {
        // Carried over: a strand at nought would see every event so far at once.
        let seen = running.seen;
        running.strands.push(super::Strand {
            pc: vec![0],
            part: Some(name.to_owned()),
            vars: params.into_iter().zip(handed).collect(),
            seen,
            ..Default::default()
        });
    }
    advance_pc(world, entity);
}
