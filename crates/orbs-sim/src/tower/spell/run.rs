//! A spell in flight: what it is holding, and the turn each one takes.
//!
//! §8's execution model, under its failure taxonomy's rule: *"scripts always
//! log and never halt."* A cursor's walk is `strand`, an instruction `line`.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::Prose;
use crate::parser::Verb;
use crate::session::Scrollback;
use crate::tick::Tick;
use crate::tower::{self, Cwd, NodeId};

use super::saying::spell_name;
use super::strand::step_strand;

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
    // The tower's spell is homed at the tower, which every room is in: it
    // needs the player only somewhere in the tower, not in the grimoire.
    if super::whole::is_tower(world, at) && super::whole::inside(world, cwd) {
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
pub(super) fn dragged(world: &World, entity: Entity, allowance: usize) -> usize {
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

/// Which line of the file execution is on, or `None` once it has run off the end.
///
/// The step's own line from parsing: the program is a tree and the file a list,
/// so a path cannot say. `None`, not `0` — no file has a line zero.
#[must_use]
pub fn line_of(state: &Running) -> Option<u64> {
    super::program::at(walking(state), &state.pc).map(|step| u64::try_from(step.line).unwrap_or(0))
}

/// The block the current frame is walking.
///
/// Empty when its part has been deleted under it, which reads as *off the end*.
pub(super) fn walking(state: &Running) -> &super::Block {
    static NOTHING: super::Block = Vec::new();
    super::program::tree(state.program.body(), state.part.as_deref()).unwrap_or(&NOTHING)
}

/// The entity a stable identity names.
pub(super) fn node_of(world: &World, id: NodeId) -> Option<Entity> {
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
