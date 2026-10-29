//! A spell in flight: what it is doing, and what it does next.
//!
//! §8's execution model, under its failure taxonomy's rule: *"scripts always
//! log and never halt."*

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::Prose;
use crate::parser::{Mode, Resolution, Verb, analyse};
use crate::session::Scrollback;
use crate::tick::Tick;
use crate::tower::{self, Cwd, Name, NodeId};

use super::block::{Blocked, would_block};

/// Steps one spell may execute in one tick, before the weave adds any.
///
/// One, so a step costs a tick and a shorter spell is a faster one — the lever
/// §11.5 wants. Everything counts as a step, or block-heavy spells would be free.
pub const SCRIPT_BUDGET: usize = 1;

/// How many steps a spell may take this tick: [`SCRIPT_BUDGET`] plus what the
/// weave grants.
///
/// Additive, not a maximum: a tier is *one of* its siblings, and `max` would
/// refund the second choice.
#[must_use]
pub fn budget(world: &World) -> usize {
    let extra: usize = world.get_resource::<tower::Taken>().map_or(0, |taken| {
        taken.ids().iter().filter_map(|id| steps_granted(id)).sum()
    });
    SCRIPT_BUDGET.saturating_add(extra)
}

use tower::steps_granted;

/// How long a blocked instruction waits before it is called a failure.
///
/// Longer than any single §10.1 stage, so a legitimate wait never trips it.
pub const PATIENCE: u64 = 120;

/// The longest a single `bide` will hold, however long it was told to.
///
/// A clamp, not a refusal: a huge literal otherwise stops a spell dead looking
/// finished. An hour, matching `MAX_MEDITATE`, so hitting it reads as a mistake.
pub const LONGEST_BIDE: u32 = 3600;

/// How deep a part may call a part, inside one spell.
///
/// The budget is not the guard (§8): a runaway recursion at one step a tick
/// grows the save by a [`Descent`] a second rather than hanging. Eight is past
/// what anyone writes on purpose.
pub const MAX_PARTS: usize = 8;

/// How many cursors one spell may have running at once.
///
/// Lower than [`MAX_PARTS`] because each strand spends its own budget, and
/// `alongside` inside a `repeat` forks one a lap. Four leaves room for a
/// pipeline of three.
pub const MAX_STRANDS: usize = 4;

/// How deep `invoke` may nest.
///
/// §8 fixes it at 3: a starved budget logs at high verbosity only, so without a
/// depth limit runaway recursion would stop all automation silently.
pub const MAX_DEPTH: u8 = 3;

/// The spell whose instruction is running, for anything that instruction casts.
///
/// `invoke` reads it to tell a spell's call from a player's; both arrive through
/// the same dispatch (§13). `None` means a player typed the line.
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct Caller(pub Option<Casting>);

/// What a nested cast inherits from the run that asked for it.
#[derive(Debug, Clone, Copy)]
pub struct Casting {
    /// How many `invoke`s deep the caller already is.
    pub depth: u8,
    /// Whether the caller runs without the player standing there.
    pub unattended: bool,
    /// Where the player is standing while the line runs: `Cwd` is the spell's
    /// room for its length. What the rail's news latch asks (§19, *the tower
    /// speaks up*).
    pub player: Entity,
}

/// A spell the orb is working through.
#[derive(Component, Debug, Clone)]
pub struct Running {
    /// Which spell — a [`NodeId`], because an `Entity` means nothing across a
    /// save, and a half-run spell is state a save carries.
    pub spell: NodeId,
    /// The program, derived from the spell's text when it was cast.
    ///
    /// A view, never the truth: `Held` stays canonical because §8's hot-reload
    /// is line-anchored and §8.1's sabotage is *"a line reordered"*.
    pub program: super::Program,
    /// Where execution has reached, as a path: `[2, 1]` is the second step
    /// inside the third.
    pub pc: Vec<usize>,
    /// Each open block, outermost first — what it is and what it has left.
    ///
    /// Saved, or a resumed spell would restart every enclosing loop. The kind
    /// too: leaving an `if` pops one more path element than leaving a `repeat`.
    pub loops: Vec<super::Loop>,
    /// How far through the record stream this spell has read, as a sequence
    /// number, so a `wait` blocked for thirty ticks still sees all of them.
    pub seen: u64,
    /// How many `invoke`s deep this is.
    pub depth: u8,
    /// Whether this run survives the player leaving its domain.
    ///
    /// Not `Bound`: a bound spell's `invoke`d child has a `Running` and no
    /// `Bound`. Set at cast from the [`Casting`]; what a player invokes, and
    /// everything that invokes, is attended.
    pub unattended: bool,
    /// The domain this spell runs in, fixed at cast; it is installed as `Cwd`
    /// around each instruction because verb bodies read `Cwd`.
    pub at: NodeId,
    /// When the current instruction first found itself blocked.
    pub waiting_since: Option<Tick>,
    /// How many ticks the running `bide` was told to spend; `Some` says it has
    /// started. Saved with `waiting_since`, the other half of it.
    pub biding: Option<u32>,
    /// Lines this casting has already complained about a missing name on.
    ///
    /// Once per line per cast, so a question inside a `repeat` is not said every
    /// turn. Cleared at cast and when the text changes. A `Vec` because its order
    /// is part of a deterministic session.
    pub said: Vec<usize>,
    /// What each name the spell has bound stands for — a resolved name, see
    /// [`Kind::Let`](super::Kind::Let).
    ///
    /// A `BTreeMap` because it is saved and must not reorder between runs. Kept
    /// across an edit, or an accumulator would empty half way through its loop.
    pub vars: std::collections::BTreeMap<String, String>,
    /// Which part's body the current frame walks, or `None` for the spell's own.
    ///
    /// A name resolved at every step, so a definition that moves while the spell
    /// runs is still the same part (§8).
    pub part: Option<String>,
    /// The callers waiting for the current frame to return, outermost first.
    ///
    /// Whole frames, not a second `pc`: a part is a different tree, so a call
    /// inside a `repeat` inside the same part needs its caller's path and blocks.
    pub stack: Vec<Descent>,
    /// Every cursor this spell has, including the one swapped into the fields
    /// above. The spell ends when the last one runs off the end.
    pub strands: Vec<Strand>,
    /// Whether the cursor now swapped in has run off its outermost frame.
    ///
    /// A flag, because running out ends a cursor and only the last one ends the
    /// spell: a producer that returns must not take its consumer down.
    pub spent: bool,
    /// What a cursor yielded on this turn, if one did. Cleared at the top of
    /// each turn, so it is never stale — see [`Hold`](super::Hold).
    pub held: Option<super::Hold>,
}

/// One cursor: where a spell is, and everything private to being there.
///
/// Per-position state lives here; [`Running`] keeps what a spell has one of
/// however many places it is at once. `seen` is per-cursor, or one cursor's
/// wait would move another past events it never saw. A [`Descent`] is a
/// suspended frame *inside* a strand.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Strand {
    /// Where this cursor is — see [`Running::pc`].
    pub pc: Vec<usize>,
    /// Its open blocks — see [`Running::loops`].
    pub loops: Vec<super::Loop>,
    /// How far it has read the record stream — see [`Running::seen`].
    pub seen: u64,
    /// When its current instruction first blocked — see
    /// [`Running::waiting_since`].
    pub waiting_since: Option<Tick>,
    /// How long its running `bide` is for — see [`Running::biding`].
    pub biding: Option<u32>,
    /// What it has bound — see [`Running::vars`].
    pub vars: std::collections::BTreeMap<String, String>,
    /// Which part's body it walks — see [`Running::part`].
    pub part: Option<String>,
    /// Its suspended callers — see [`Running::stack`].
    pub stack: Vec<Descent>,
}

impl Running {
    /// Make `strand` the cursor the runner walks.
    fn take_up(&mut self, strand: &Strand) {
        self.pc.clone_from(&strand.pc);
        self.loops.clone_from(&strand.loops);
        self.seen = strand.seen;
        self.waiting_since = strand.waiting_since;
        self.biding = strand.biding;
        self.vars.clone_from(&strand.vars);
        self.part.clone_from(&strand.part);
        self.stack.clone_from(&strand.stack);
    }

    /// The cursor the runner has been walking, to park.
    fn lay_down(&self) -> Strand {
        Strand {
            pc: self.pc.clone(),
            loops: self.loops.clone(),
            seen: self.seen,
            waiting_since: self.waiting_since,
            biding: self.biding,
            vars: self.vars.clone(),
            part: self.part.clone(),
            stack: self.stack.clone(),
        }
    }
}

/// One caller, waiting for the part it called to finish.
///
/// Not the usual word, which `tests/boundaries.rs` forbids under `orbs-sim/src`.
/// It carries `vars` because a part takes arguments and cannot reach a name it
/// was not given; it carries no `spell`, because a spell is one file (§19).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Descent {
    /// The body this frame was walking — `None` for the spell's own.
    pub part: Option<String>,
    /// Where in it, pointing at the call that suspended it.
    pub pc: Vec<usize>,
    /// Its open blocks, which the callee must not disturb.
    pub loops: Vec<super::Loop>,
    /// Its bindings, put away for the call and back after it, so a part's `let`
    /// cannot outlive it or clobber the caller's.
    pub vars: std::collections::BTreeMap<String, String>,
}

/// Work every running spell forward.
///
/// Before `tower::finish`, so a spell sees the world as the last tick left it
/// rather than racing the run it is waiting on.
pub fn advance(world: &mut World) {
    // By `NodeId`, never query order: adding or removing `Running` moves an
    // entity between tables, and replay needs one interleaving.
    let mut running: Vec<(Entity, NodeId)> = world
        .query::<(Entity, &Running)>()
        .iter(world)
        .map(|(entity, running)| (entity, running.spell))
        .collect();
    running.sort_unstable_by_key(|(_, spell)| *spell);

    for (entity, _) in running {
        // An invocation needs you standing there (§19); a bound spell, and
        // anything it casts, does not — see [`Running::unattended`].
        if !world
            .get::<Running>(entity)
            .is_some_and(|state| state.unattended)
            && left_it(world, entity)
        {
            continue;
        }
        // Credited here rather than at every emit site, so the transcript draws
        // what the player did and the log keeps everything (`FieldName::Spell`).
        let named = world
            .get::<Running>(entity)
            .map(|state| spell_name(world, state));
        set_attribution(world, named.as_deref());
        step_one(world, entity);
        // Always cleared, or the player's own next line is credited to a spell.
        set_attribution(world, None);
    }
}

/// Whether the player has walked out on this invocation, ending it if so.
///
/// By domain, not node: `attend alembic` stands inside the laboratory, and
/// leaning over an instrument must not stop a spell.
fn left_it(world: &mut World, entity: Entity) -> bool {
    let Some(state) = world.get::<Running>(entity) else {
        return false;
    };
    let at = state.at;
    let Some(home) = node_of(world, at) else {
        return false;
    };
    let cwd = world.resource::<Cwd>().0;
    if tower::domain_of(world, cwd).unwrap_or(cwd) == home {
        return false;
    }

    let named = world
        .get::<Running>(entity)
        .map(|state| spell_name(world, state))
        .unwrap_or_default();
    world.entity_mut(entity).remove::<Running>();
    let message = world
        .resource::<Prose>()
        .line("spell_unattended", &[("name", &named)]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Invoke.canonical())
        .text(FieldName::Path, &named)
        .text(FieldName::Message, &message)
        .role(Role::Cost)
        .finish();
    true
}

/// Credit everything pushed from now on to `spell`, or to nobody.
fn set_attribution(world: &mut World, spell: Option<&str>) {
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .attribute(spell);
}

/// Step every cursor this spell has, each with its own budget.
///
/// In `strands` order, each spending its whole budget before the next — batch,
/// as [`advance`] is one level up, for replay. Pinned by
/// `two_cursors_interleave_the_same_way_at_two_steps_a_tick`. A cursor that
/// runs off the end is removed; one that blocks yields only itself.
fn step_one(world: &mut World, entity: Entity) {
    // Only on a turn the spell takes: a retimed one skips ticks, and clearing
    // its hold on those flickered the rail between the hold and the tool.
    if dragged(world, entity, budget(world)) > 0
        && let Some(mut running) = world.get_mut::<Running>(entity)
    {
        running.held = None;
    }
    let mut index = 0;
    loop {
        // Re-read every lap: a spent strand is removed and `alongside` appends.
        let Some(count) = world
            .get::<Running>(entity)
            .map(|state| state.strands.len())
        else {
            return;
        };
        if index >= count {
            return;
        }
        swap_in(world, entity, index);
        step_strand(world, entity);
        // `finish` removes the component: the spell ended, not just this strand.
        if world.get::<Running>(entity).is_none() {
            return;
        }
        if swap_out(world, entity, index) {
            // The next strand is at this index now. `remove`, never
            // `swap_remove`: reordering cursors would change how they interleave.
            continue;
        }
        index += 1;
    }
}

/// Move `strands[index]` into the fields the runner walks.
///
/// A swap rather than naming a cursor at every one of ~110 sites, `Cwd`'s idiom
/// one level down. So `Running`'s cursor fields mean *the cursor stepping now*,
/// and are unambiguous only inside [`step_one`].
fn swap_in(world: &mut World, entity: Entity, index: usize) {
    let Some(mut running) = world.get_mut::<Running>(entity) else {
        return;
    };
    let Some(strand) = running.strands.get(index).cloned() else {
        return;
    };
    running.take_up(&strand);
}

/// Put the active cursor back into `strands[index]`, or drop it if it is spent.
///
/// Returns whether the strand ended, so [`step_one`] does not advance its index.
fn swap_out(world: &mut World, entity: Entity, index: usize) -> bool {
    let Some(mut running) = world.get_mut::<Running>(entity) else {
        return false;
    };
    if running.spent {
        running.spent = false;
        if index < running.strands.len() {
            running.strands.remove(index);
        }
        return true;
    }
    let strand = running.lay_down();
    if let Some(slot) = running.strands.get_mut(index) {
        *slot = strand;
    }
    false
}

/// What a retimed spell's budget is worth after the enemy has had it.
///
/// §8.1's trigger-clock surface: the spell reads perfectly and falls behind.
/// Whole ticks are skipped — one in every `drag + 1` runs — because shaving a
/// budget of one leaves one.
fn dragged(world: &World, entity: Entity, allowance: usize) -> usize {
    // `Retimed` rides the spell's node, not its `Running`, which is rebuilt
    // every lap and would repair the sabotage by running off the end.
    let Some(drag) = world
        .get::<super::super::Retimed>(entity)
        .map(|retimed| retimed.drag)
        .filter(|drag| *drag > 0)
    else {
        return allowance;
    };
    // The tick decides, so this is a pure function of replayed state.
    let now = world.resource::<crate::tick::Tick>().get();
    if now.is_multiple_of(drag + 1) {
        allowance
    } else {
        0
    }
}

/// [`dragged`], reachable from a test.
#[cfg(test)]
pub(crate) fn dragged_for_test(world: &World, entity: Entity, allowance: usize) -> usize {
    dragged(world, entity, allowance)
}

/// Run up to [`budget`] instructions of the cursor swapped in now.
///
/// The budget is read once: one that grew while being spent is the shape a loop
/// guard must never have.
fn step_strand(world: &mut World, entity: Entity) {
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
            // last — see [`ended`].
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
                let (answer, missing) = asked_where_the_spell_is(world, at, condition);
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
fn standing_for(
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

/// What happened to one instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Progress {
    /// It ran, or failed in a way the taxonomy logs and continues past.
    Done,
    /// It cannot proceed yet; the spell keeps its place and tries next tick.
    Blocked,
}

/// Hold until something the spell named happens.
///
/// Only what arrived since the cursor, so a `wait` waits for something *new* and
/// a loop's second turn does not return on the first turn's event.
fn wait_for(world: &mut World, entity: Entity, state: &Running, wanted: &str) -> Progress {
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
pub(super) const fn takes_nothing_key(verb: crate::parser::Verb) -> &'static str {
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

/// Say what a `wait` is waiting on, once.
fn say_waiting(world: &mut World, state: &Running, wanted: &str) {
    let name = spell_name(world, state);
    let message = world
        .resource::<Prose>()
        .line("spell_watching", &[("name", &name), ("source", wanted)]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Invoke.canonical())
        .text(FieldName::Path, &name)
        .text(FieldName::Source, wanted)
        .text(FieldName::Message, &message)
        .role(Role::Cost)
        .finish();
}

/// Execute one line.
fn run_line(world: &mut World, entity: Entity, state: &Running, line: &str) -> Progress {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        advance_pc(world, entity);
        return Progress::Done;
    }

    let scene = tower::scene_at(world, world.resource::<Cwd>().0);
    let intent = match analyse(line, &scene, Mode::Calm).resolution {
        Resolution::Resolved { intent, .. } => intent,
        // A verb handed words it cannot use — see `compile::reading`.
        Resolution::TakesNothing { verb, extra, .. } => {
            say_failure(world, state, takes_nothing_key(verb), &extra, Role::Danger);
            advance_pc(world, entity);
            return Progress::Done;
        }
        // §8's *Referent missing*: skips, logs, continues.
        _ => {
            say_failure(world, state, "spell_missing", line, Role::Danger);
            advance_pc(world, entity);
            return Progress::Done;
        }
    };

    if !may_issue(intent.verb) {
        say_failure(world, state, "spell_forbidden", line, Role::Danger);
        advance_pc(world, entity);
        return Progress::Done;
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

/// Spend `ticks` doing nothing, then move on.
///
/// `waiting_since` carries the countdown rather than a second counter, which
/// also saves it for free. It never reaches `PATIENCE`: a bide is blocked on
/// nothing, so it says nothing.
fn bide(world: &mut World, entity: Entity, state: &Running, delay: u32) -> Progress {
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

/// Whether a script may issue this verb at all.
///
/// Each one refused is a hazard, not nonsense. `execute::is_live` is the wrong
/// instrument: it answers *does this verb work*, which only overlaps.
#[must_use]
pub const fn may_issue(verb: Verb) -> bool {
    !matches!(
        verb,
        // A spell is written *for* a domain; walking would make canonicalising
        // an `if` undecidable.
        Verb::Attend
            // `Sim::step` drains `Skip` in a loop: an hour inside one `step()`.
            | Verb::Meditate
            // Opens the editor, so the player's next keystrokes land in it.
            | Verb::Scribe
            // Command-anchored (§6), with no meaning from a script.
            | Verb::Undo
            | Verb::Bind
            // `unfurl`, `weave` and `wander` take the keyboard; inside a
            // `repeat` they re-seize it faster than Escape gives it back.
            // `follow`, `summon` and `limn` take no keys and stay.
            | Verb::Unfurl
            | Verb::Weave
            | Verb::Wander
            // A bound spell re-casts off the end, so it would close the game on
            // the orb's clock.
            | Verb::Quit
            // Takes the pane and the keys, and a bound spell would reopen it
            // every lap.
            | Verb::Menu
    )
}

/// Which line of the file execution is on, or `None` once it has run off the end.
///
/// The step's own line from parsing: the program is a tree and the file a list,
/// so a path cannot say. `None`, not `0` — no file has a line zero.
#[must_use]
pub fn line_of(state: &Running) -> Option<u64> {
    super::program::at(walking(state), &state.pc).map(|step| u64::try_from(step.line).unwrap_or(0))
}

/// Hand the current frame back to whoever called it.
///
/// Returns whether there was one; `false` means the outermost frame ran out.
/// The caller is stepped past its call here, not on the way in, so every saved
/// `pc` points at a line the save can explain.
fn returned(world: &mut World, entity: Entity) -> bool {
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
fn called(
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
/// nothing waits for it, and bounded by [`MAX_STRANDS`]. [`step_one`] re-reads
/// the count, so a strand forked this tick is stepped this tick.
fn forked(
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

/// The block the current frame is walking.
///
/// Empty when its part has been deleted under it, which reads as *off the end*.
fn walking(state: &Running) -> &super::Block {
    static NOTHING: super::Block = Vec::new();
    super::program::tree(state.program.body(), state.part.as_deref()).unwrap_or(&NOTHING)
}

/// Whether this casting has already complained about `line`, marking it said.
fn already_said(world: &mut World, entity: Entity, line: usize) -> bool {
    let Some(mut running) = world.get_mut::<Running>(entity) else {
        return false;
    };
    if running.said.contains(&line) {
        return true;
    }
    running.said.push(line);
    false
}

/// The entity a stable identity names.
fn node_of(world: &World, id: NodeId) -> Option<Entity> {
    let root = tower::root(world);
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if world.get::<NodeId>(node) == Some(&id) {
            return Some(node);
        }
        stack.extend(tower::children_of(world, node));
    }
    None
}

/// Move past the step just run, closing and repeating blocks as needed.
///
/// The program is cloned out because `step_past` needs it while `Running` is
/// borrowed mutably on the same entity.
fn advance_pc(world: &mut World, entity: Entity) {
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
    let (at, pc) = world
        .get::<Running>(entity)
        .map_or((None, Vec::new()), |state| {
            (node_of(world, state.at), state.pc.clone())
        });
    let guards = guard_answers(world, &body, at, &pc);
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
fn asked_where_the_spell_is(
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

/// Take the oldest name out of a satchel and bind it, or yield until there is
/// one.
///
/// In the spell's room, never `Cwd`: every domain has a satchel under one name,
/// so a producer would fill one queue while its consumer drained another.
/// Yielding is not waiting — it never touches `waiting_since` or reaches
/// [`PATIENCE`]. A thing that holds no satchel is a missing name, said once.
fn pull(world: &mut World, entity: Entity, state: &Running, name: &str, from: &str) -> Progress {
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

/// Name what a guard could not place, once per line per cast.
///
/// A guard stopped by an unanswerable question otherwise leaves nothing on
/// screen: the spell simply does not run.
fn say_missing(
    world: &mut World,
    entity: Entity,
    state: &Running,
    line: usize,
    missing: &[String],
) {
    if missing.is_empty() || already_said(world, entity, line) {
        return;
    }
    say_failure(
        world,
        state,
        "spell_nowhere",
        &missing.join(", "),
        Role::Danger,
    );
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
) -> Vec<(Vec<usize>, Continues)> {
    fn walk(
        world: &World,
        body: &super::Block,
        pc: &[usize],
        path: &mut Vec<usize>,
        out: &mut Vec<(Vec<usize>, Continues)>,
    ) {
        for (index, step) in body.iter().enumerate() {
            path.push(index);
            match &step.kind {
                super::Kind::Repeat { until, body, .. } => {
                    // Only an ancestor of `pc` is running.
                    if let Some(condition) = until
                        && pc.starts_with(path)
                    {
                        let more = matches!(super::watch::holds(world, condition).0, Some(false));
                        out.push((path.clone(), Continues::Guard(more)));
                    }
                    walk(world, body, pc, path, out);
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
                    walk(world, body, pc, path, out);
                }
                super::Kind::If {
                    body, otherwise, ..
                } => {
                    // `enter_branch` pushes the half and the step, so a loop in
                    // an `if` is two elements deeper — matching `step_past`.
                    for (half, block) in [body, otherwise].into_iter().enumerate() {
                        path.push(half);
                        walk(world, block, pc, path, out);
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
    walk(world, body, pc, &mut Vec::new(), &mut out);
    world.insert_resource(Cwd(player));
    out
}

/// One cursor has run out; end the spell if it was the last.
fn ended(world: &mut World, entity: Entity, state: &Running) {
    let last = world
        .get::<Running>(entity)
        .is_none_or(|running| running.strands.len() <= 1);
    if last {
        finish(world, entity, state);
    }
}

/// The spell has run out of lines.
///
/// Silent for a held spell, which `bind::stand` casts again next tick.
fn finish(world: &mut World, entity: Entity, state: &Running) {
    let name = spell_name(world, state);
    if let Some(mut bound) = world.get_mut::<super::Bound>(entity) {
        // The binding outlives the run, so its rationing of complaints must too.
        bound.said = state.said.clone();
        world.entity_mut(entity).remove::<Running>();
        return;
    }
    let message = world
        .resource::<Prose>()
        .line("spell_done", &[("name", &name)]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Invoke.canonical())
        .text(FieldName::Path, &name)
        .text(FieldName::Message, &message)
        .role(Role::Success)
        .finish();

    // Remove, never despawn: `Running` is worn by the spell node itself.
    world.entity_mut(entity).remove::<Running>();
}

/// What the spell is called, for a record — §8.1's culprit is never anonymous.
fn spell_name(world: &World, state: &Running) -> String {
    node_of(world, state.spell)
        .and_then(|node| world.get::<Name>(node))
        .map_or_else(String::new, |name| name.0.clone())
}

fn say_blocked(world: &mut World, state: &Running, blocked: &Blocked) {
    let name = spell_name(world, state);
    let message = world.resource::<Prose>().line(
        "spell_waiting",
        &[
            ("name", &name),
            ("source", blocked.name()),
            ("state", blocked.doing()),
        ],
    );
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Invoke.canonical())
        .text(FieldName::Path, &name)
        .count(FieldName::Quantity, line_of(state).unwrap_or_default())
        .text(FieldName::Source, blocked.name())
        .text(FieldName::Message, &message)
        .role(Role::Cost)
        .finish();
}

fn say_failure(world: &mut World, state: &Running, key: &str, detail: &str, role: Role) {
    // The rail's fault mark is raised here only: the one place that knows a
    // spell failed and which room it was in. `Danger` only — a `Cost` is a
    // spell waiting its turn, which is constant and not a fault.
    if role == Role::Danger {
        crate::tower::mark_fault_at(world, state.at);
    }
    let name = spell_name(world, state);
    let message = world.resource::<Prose>().line(
        key,
        &[
            ("name", &name),
            ("detail", detail),
            ("count", &line_of(state).unwrap_or_default().to_string()),
        ],
    );
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Invoke.canonical())
        .text(FieldName::Path, &name)
        .count(FieldName::Quantity, line_of(state).unwrap_or_default())
        .text(FieldName::Message, &message)
        .role(role)
        .finish();
}
