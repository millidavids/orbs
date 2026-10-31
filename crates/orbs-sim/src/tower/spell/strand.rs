//! One cursor's turn: walking the program, block by block, until its budget is
//! spent or it blocks.

use bevy_ecs::prelude::*;
use orbs_render::Role;

use crate::tower::{self, Cwd, Name};

use super::guard::{advance_pc, asked_where_the_spell_is};
use super::line::{Progress, bide, pull, run_line, wait_for};
use super::parts::{called, forked, returned};
use super::run::{Caller, Casting, Running, budget, dragged, node_of, walking};
use super::saying::{already_said, ended, finish, say_failure, say_missing};

/// Run up to `budget` instructions of the cursor swapped in now.
///
/// The budget is read once: one that grew while being spent is the shape a loop
/// guard must never have.
pub(super) fn step_strand(world: &mut World, entity: Entity) {
    let allowance = dragged(world, entity, budget(world));
    for _ in 0..allowance {
        // Entry and lap both arrive here, which makes this the one writer of a
        // `for each`'s cursor — see [`bind_cursors`].
        bind_cursors(world, entity);
        let Some(state) = world.get::<Running>(entity).cloned() else {
            return;
        };

        let Some(step) = super::program::at(walking(&state), &state.pc).cloned() else {
            // Off the end of a frame: a part returns to its caller.
            if returned(world, entity) {
                continue;
            }
            // Off the end of a cursor, which ends the spell only if it was the
            // last — see `ended`.
            if let Some(mut running) = world.get_mut::<Running>(entity) {
                running.spent = true;
            }
            ended(world, entity, &state);
            return;
        };

        // A definition is stepped past where it stands; running it would do the
        // work twice for anyone who also called it.
        if matches!(step.kind, super::Kind::Part { .. }) {
            advance_pc(world, entity);
            continue;
        }

        if let super::Kind::Call { name, args } = &step.kind {
            called(world, entity, &state, step.line, name, args);
            continue;
        }

        if let super::Kind::Alongside { name, args } = &step.kind {
            forked(world, entity, &state, step.line, name, args);
            continue;
        }

        // Entering a block spends a step, and that is the guard: a `repeat`
        // whose body spends nothing would otherwise loop inside one tick.
        if let super::Kind::Repeat { times, until, body } = &step.kind {
            // Asked on the way in too, so `until` with the answer already yes
            // runs zero times — and in the spell's own room, as an `if` is.
            if let Some(condition) = until {
                let at = node_of(world, state.at);
                let condition = standing_for(&state.vars, condition);
                let (answer, missing) = asked_where_the_spell_is(world, at, &condition);
                if !matches!(answer, Some(false)) {
                    if answer.is_none() {
                        say_missing(world, entity, &state, step.line, &missing);
                    }
                    advance_pc(world, entity);
                    continue;
                }
            }
            // Stepped past, not into: an empty body leaves a path `at` cannot
            // resolve, and `repeat 0` would otherwise run once — §8.1's shape,
            // where an enemy zeroing a count bought an execution.
            if body.is_empty() || *times == Some(0) {
                advance_pc(world, entity);
                continue;
            }
            if let Some(mut running) = world.get_mut::<Running>(entity) {
                let Running { pc, loops, .. } = &mut *running;
                super::program::enter(pc, loops, *times);
            }
            continue;
        }

        // A branch is always entered, even an empty `else`: `step_past` walks
        // in and out correctly, and a special case would be a second exit.
        if let super::Kind::If {
            condition,
            body,
            otherwise,
        } = &step.kind
        {
            // The spell's own domain, for the length of the question.
            let player = world.resource::<Cwd>().0;
            let asked = node_of(world, state.at).and_then(|at| {
                world.insert_resource(Cwd(at));
                let asked = condition
                    .as_ref()
                    .map(|condition| standing_for(&state.vars, condition))
                    .map(|condition| super::watch::holds(world, &condition));
                world.insert_resource(Cwd(player));
                asked
            });
            let (answer, missing) = asked.unwrap_or((None, Vec::new()));

            // `None` named a place the tower does not have — §8's *Referent
            // missing* — and is said once per line per cast.
            if !missing.is_empty() && !already_said(world, entity, step.line) {
                say_failure(
                    world,
                    &state,
                    "spell_nowhere",
                    &missing.join(", "),
                    Role::Danger,
                );
            }

            // Neither half: falling through to `else` looked exactly like an
            // inverted condition. A question nobody can answer decides nothing.
            let Some(holds) = answer else {
                advance_pc(world, entity);
                continue;
            };
            let half = if holds { body } else { otherwise };
            if half.is_empty() {
                advance_pc(world, entity);
            } else if let Some(mut running) = world.get_mut::<Running>(entity) {
                let Running { pc, loops, .. } = &mut *running;
                super::program::enter_branch(pc, loops, holds);
            }
            continue;
        }

        // Entering costs a step, as a `repeat` does.
        if let super::Kind::Each { group, body } = &step.kind {
            let members = node_of(world, state.at)
                .map(|room| tower::group_at(world, room, group).len())
                .unwrap_or_default();
            // Stepped past rather than into, as `repeat 0` is.
            if members == 0 || body.is_empty() {
                advance_pc(world, entity);
                continue;
            }
            if let Some(mut running) = world.get_mut::<Running>(entity) {
                let Running { pc, loops, .. } = &mut *running;
                super::program::enter_each(pc, loops);
            }
            continue;
        }

        // A binding costs a step (§8), or a spell of nothing but `set` would
        // loop inside one tick.
        if let super::Kind::Let { name, value } = &step.kind {
            let stood_for = substituted(&state.vars, value);
            if let Some(mut running) = world.get_mut::<Running>(entity) {
                running.vars.insert(name.clone(), stood_for);
            }
            advance_pc(world, entity);
            continue;
        }

        // A `let` whose value comes out of the world, yielding while the satchel
        // is bare.
        if let super::Kind::Pull { name, from } = &step.kind {
            let from = substituted(&state.vars, from);
            if pull(world, entity, &state, name, &from) == Progress::Blocked {
                return;
            }
            continue;
        }

        if let super::Kind::Wait(wanted) = &step.kind {
            let wanted = substituted(&state.vars, wanted);
            if wait_for(world, entity, &state, &wanted) == Progress::Blocked {
                return;
            }
            continue;
        }

        // The runner holds how far a bide has gone, as it holds a `repeat`'s
        // laps: a countdown in the compiled step would leave the next cast at 0.
        if let super::Kind::Bide(delay) = &step.kind {
            if bide(world, entity, &state, *delay) == Progress::Blocked {
                return;
            }
            continue;
        }

        let super::Kind::Command(line) = step.kind else {
            continue;
        };
        // `follow way` reaches the dispatch as `follow north`.
        let line = substituted(&state.vars, &line);

        // The spell's own position for one instruction, then the player's back —
        // strictly nested, so nothing outside observes the swap.
        let player = world.resource::<Cwd>().0;
        let Some(at) = node_of(world, state.at) else {
            finish(world, entity, &state);
            return;
        };
        world.insert_resource(Cwd(at));
        world.insert_resource(Caller(Some(Casting {
            depth: state.depth,
            unattended: state.unattended,
            player,
        })));

        let outcome = run_line(world, entity, &state, &line);

        world.insert_resource(Cwd(player));
        world.insert_resource(Caller(None));

        if outcome == Progress::Blocked {
            return;
        }
    }
}

/// `text` with every bound name replaced by what it stands for.
///
/// Word by word, so a variable called `n` cannot rewrite `north`. Case-folded,
/// because `set` lowercases what it binds. One level only — see
/// [`Kind::Let`](super::Kind::Let).
fn substituted(vars: &std::collections::BTreeMap<String, String>, text: &str) -> String {
    if vars.is_empty() {
        return text.to_owned();
    }
    text.split_whitespace()
        .map(|word| {
            vars.get(&word.to_lowercase())
                .map_or(word, String::as_str)
                .to_owned()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// A question with every bound name replaced by what it stands for.
///
/// Through `Condition::rename`, the walk `compile` uses, so a variable reaches
/// `holds` looking exactly like a name the player typed.
pub(super) fn standing_for(
    vars: &std::collections::BTreeMap<String, String>,
    condition: &crate::parser::Condition,
) -> crate::parser::Condition {
    if vars.is_empty() {
        return condition.clone();
    }
    let mut copy = condition.clone();
    copy.rename(&mut |_, name| vars.get(&name.to_lowercase()).cloned());
    copy
}

/// Point every open `for each`'s cursor at the member it is on.
///
/// At the top of every step, from the loop stack, because the lap happens in
/// [`step_past`](super::program::step_past), which knows nothing of sets. One
/// writer for entry and lap alike. The path is walked against the stack because
/// a branch costs two elements and a loop one.
fn bind_cursors(world: &mut World, entity: Entity) {
    let Some(state) = world.get::<Running>(entity).cloned() else {
        return;
    };
    let Some(room) = node_of(world, state.at) else {
        return;
    };

    let mut bound: Vec<(String, String)> = Vec::new();
    let mut consumed = 0usize;
    for open in &state.loops {
        if consumed >= state.pc.len() {
            break;
        }
        if let super::Loop::Each(index) = open
            && let Some(step) = super::program::at(walking(&state), &state.pc[..=consumed])
            && let super::Kind::Each { group, .. } = &step.kind
            && let Some(member) = tower::group_at(world, room, group)
                .get(*index as usize)
                .and_then(|node| world.get::<Name>(*node))
        {
            bound.push((group.clone(), member.0.clone()));
        }
        consumed += if matches!(open, super::Loop::Branch) {
            2
        } else {
            1
        };
    }

    if let Some(mut running) = world.get_mut::<Running>(entity) {
        for (group, member) in bound {
            running.vars.insert(group, member);
        }
    }
}
