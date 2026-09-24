//! A spell in flight: what it is doing, and what it does next.
//!
//! DESIGN.md §8's execution model, and its failure taxonomy's title — *"scripts
//! always log and never halt."*

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::Prose;
use crate::parser::{Mode, Resolution, Verb, analyse};
use crate::session::Scrollback;
use crate::tick::Tick;
use crate::tower::{self, Cwd, Name, NodeId};

use super::block::{Blocked, would_block};

/// Steps one spell may execute in one tick.
///
/// §8's *"execution budget — the orb's attention"*. At one, a spell costs a
/// tick per step, so a shorter spell is a faster spell — the lever §11.5 wants;
/// at four, a tight spell and a sloppy one were the same. Everything counts as a
/// step, or block-heavy spells would be free.
///
/// The floor, not the number: [`budget`] is what the orb can do now. A `const`
/// because a default has to exist before any world does.
pub const SCRIPT_BUDGET: usize = 1;

/// How many steps a spell may take this tick.
///
/// Not a constant, because a step costs a tick: the budget *is* the speed of
/// every piece of automation, and §11.5 wants the weave to sell it.
///
/// It reads `Taken`, which stays empty in this version — every Mastery node
/// ships as a marker, so this answers [`SCRIPT_BUDGET`] today.
///
/// Additive, not a maximum: two nodes granting a step each give three, because
/// a tier is *one of* its siblings and `max` would refund the second choice.
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
/// Waiting is normal; waiting for ever is not — §8's taxonomy is titled
/// *"scripts always log and never halt"*.
///
/// Generous on purpose: longer than any single §10.1 stage, so a legitimate wait
/// never trips it.
pub const PATIENCE: u64 = 120;

/// The longest a single `bide` will hold, however long it was told to.
///
/// A clamp rather than a refusal. `bide until` once bought four billion ticks of
/// silence on a step that never reaches [`PATIENCE`]; that form is a complaint
/// now, but a very large literal produces the same failure — a spell stopped
/// dead and looking finished.
///
/// An hour, matching `MAX_MEDITATE`, so a bide that hits it is visibly a mistake
/// rather than mysteriously slow. Not [`PATIENCE`] — a bide is blocked on
/// nothing and a long one is legal.
pub const LONGEST_BIDE: u32 = 3600;

/// How deep a part may call a part.
///
/// Separate from [`MAX_DEPTH`], which bounds `invoke` — each level there is a
/// whole second spell with its own budget, where this bounds descents inside one
/// spell.
///
/// The budget is not the guard (§8): at one step a tick a runaway recursion does
/// not hang the game, it grows the save by a [`Descent`] a second until nothing
/// can read it. Eight rather than three, because the number only has to be past
/// what a person would write on purpose.
pub const MAX_PARTS: usize = 8;

/// How many cursors one spell may have running at once.
///
/// [`MAX_PARTS`]'s argument one level out, and a strand is the more expensive
/// thing: `alongside` inside a `repeat` forks one a lap, each carrying its own
/// `pc`, `loops`, `vars` and stack of descents.
///
/// Four rather than eight, because a strand also multiplies what the spell
/// *does* per tick — each spends its own budget. Two is the shape the language
/// was built for; four leaves room for a pipeline of three.
pub const MAX_STRANDS: usize = 4;

/// How deep `invoke` may nest.
///
/// §8 fixes this at 3: exhausting the budget makes every subsequent instruction
/// *Budget starved*, which logs at high verbosity only, so all automation would
/// stop silently. A depth limit makes runaway recursion loud.
pub const MAX_DEPTH: u8 = 3;

/// The spell whose instruction is running, for anything that instruction casts.
///
/// Set around one instruction and cleared after. `invoke` reads it to tell a
/// spell's call from a player's, which the intent cannot say: both arrive
/// through the same dispatch (§13). `None` means a player typed the line.
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct Caller(pub Option<Casting>);

/// What a nested cast inherits from the run that asked for it.
///
/// A struct rather than the bare depth it started as: a second fact travels the
/// same road, and two resources set beside each other are two things that must
/// be cleared together.
#[derive(Debug, Clone, Copy)]
pub struct Casting {
    /// How many `invoke`s deep the caller already is.
    pub depth: u8,
    /// Whether the caller runs without the player standing there.
    pub unattended: bool,
}

/// A spell the orb is working through.
#[derive(Component, Debug, Clone)]
pub struct Running {
    /// Which spell. A [`NodeId`], not an `Entity` — `tower::node` is explicit
    /// that an `Entity` is *"meaningless across a save or a rebuilt world"*, and
    /// a half-executed spell is state a save has to carry.
    pub spell: NodeId,
    /// The program, derived from the spell's text when it was cast.
    ///
    /// A view, never the truth: `Held` stays canonical because §8's hot-reload
    /// is line-anchored and §8.1's sabotage surface is *"a line reordered"*.
    pub program: super::Program,
    /// Where in the program execution has reached.
    ///
    /// A *path* rather than a line number: `[0]` is the first step, `[2, 1]` the
    /// second step inside the third. Blocks made a single `pc` insufficient.
    pub pc: Vec<usize>,
    /// Each open block, outermost first — what it is and what it has left.
    ///
    /// §8 requires in-flight state be serialisable, and a save with a `pc` and
    /// no counts would resume every enclosing loop from its first turn. The
    /// block's *kind* too, because walking out of an `if` pops one more path
    /// element than walking out of a `repeat` — see [`Loop`](super::Loop).
    pub loops: Vec<super::Loop>,
    /// How far through the record stream this spell has read.
    ///
    /// A *sequence number*, not an index — see `Records::sequence`. A `wait`
    /// looks only at what arrived after this, which lets a spell blocked for
    /// thirty ticks still see everything that happened in them.
    pub seen: u64,
    /// How many `invoke`s deep this is.
    pub depth: u8,
    /// Whether this run survives the player leaving the domain it runs in.
    ///
    /// Not the same question as `Bound`: a bound spell that `invoke`s another
    /// gives the child a `Running` and no `Bound`, so asking about the component
    /// ended the child the moment the player walked out.
    ///
    /// Set at cast from the caller ([`Casting`]): a binding is unattended, a
    /// standing recast is, and anything either casts inherits it. What a
    /// *player* invokes is attended, and so is everything it invokes.
    pub unattended: bool,
    /// The domain this spell runs in.
    ///
    /// Fixed, not walked: a spell is written for a domain and `attend` inside
    /// one is refused. The *swap* is still necessary — verb bodies read `Cwd`
    /// directly, so the domain is installed around each instruction.
    pub at: NodeId,
    /// When the current instruction first found itself blocked.
    pub waiting_since: Option<Tick>,
    /// How many ticks the running `bide` was told to spend.
    ///
    /// `Some` says the bide has started, which is the whole of what it carries
    /// now that the count is a literal again. Cleared and saved with
    /// `waiting_since`, the other half of the same instruction.
    pub biding: Option<u32>,
    /// Lines this casting has already complained about a missing name on.
    ///
    /// Once per line per cast: a question inside a `repeat` is asked every turn,
    /// and a name the tower cannot place is wrong on the four hundredth turn
    /// exactly as on the first. Cleared at cast and when the text changes under
    /// it, so a name that goes missing later is still heard about.
    ///
    /// A `Vec` rather than a set because its order is part of a deterministic
    /// session.
    pub said: Vec<usize>,
    /// What each name the spell has bound stands for.
    ///
    /// `set best to north` puts one here; `for each way` rebinds `way` at the
    /// top of every pass. A value is a *name*, already resolved against the
    /// room — see [`Kind::Let`](super::Kind::Let).
    ///
    /// A `BTreeMap` because this travels to a save, and rows that moved between
    /// two runs of one seed would fail the lockstep test. Kept across a
    /// mid-flight edit: a store rebuilt on every save would empty an accumulator
    /// half way through the loop filling it.
    pub vars: std::collections::BTreeMap<String, String>,
    /// Which part's body the current frame is walking, or `None` for the
    /// spell's own.
    ///
    /// A *name*, resolved through `program::tree` at every step, so a definition
    /// that moves while the spell runs is still the same part (§8).
    pub part: Option<String>,
    /// The callers waiting for the current frame to return, outermost first.
    ///
    /// A stack of frames, not a second `pc`: a path addresses one tree and a
    /// part is a different tree, so `gathering()` inside a `repeat` inside
    /// `gathering` needs the caller's path *and* its open blocks kept whole.
    pub stack: Vec<Descent>,
    /// Every cursor this spell has, including the one currently swapped into the
    /// fields above.
    ///
    /// Never empty while the spell runs. A cast builds one, `alongside` appends,
    /// a cursor that runs off the end is removed, and the spell ends when the
    /// last goes.
    pub strands: Vec<Strand>,
    /// Whether the cursor now swapped in has run off the end of its outermost
    /// frame.
    ///
    /// A flag rather than `finish` called from inside the step loop, because
    /// running out is a fact about a *cursor* and ending one about the *spell*:
    /// a producer that returns no longer takes its consumer down with it.
    pub spent: bool,
}

/// One cursor: where a spell is, and everything private to being there.
///
/// The split is *per-position* against *per-spell*. `seen` is here: two cursors
/// sharing one would have cursor A satisfying a wait move cursor B past events
/// B never saw. What stays on [`Running`] is what a spell has one of however
/// many places it is in at once — which spell, its `program`, its `invoke`
/// depth, whether it survives the player leaving, the room, and `said`.
///
/// `Strand` rather than `Cursor` because `bind_cursors` already means the
/// counters a `for each` walks with. [`Descent`] is different again: a suspended
/// frame *inside* a strand.
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
/// Not the obvious word: the usual one is among the four layout names
/// `tests/boundaries.rs` forbids under `orbs-sim/src`. A call stack really is a
/// stack of descents.
///
/// `vars` is here, reversing the decision to share them (§19): a part takes
/// arguments now, so `for each way` inside a part no longer rebinds the caller's
/// `way` and a part cannot reach a name it was not given.
///
/// `spell` is not here: a spell is contained to one `.spell` file, so every
/// descent belongs to the spell that opened it (§19). Reaching into another
/// spell's text is `invoke`, a second [`Running`] with its own budget.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Descent {
    /// The body this frame was walking — `None` for the spell's own.
    pub part: Option<String>,
    /// Where in it, pointing at the call that suspended it.
    pub pc: Vec<usize>,
    /// Its open blocks, which the callee must not disturb.
    pub loops: Vec<super::Loop>,
    /// Its bindings, which the callee neither sees nor may disturb.
    ///
    /// Taken from the caller on the way in and put back on the way out, so a
    /// part's `let` cannot outlive it and the caller's accumulator survives a
    /// call that happens to use the same name.
    pub vars: std::collections::BTreeMap<String, String>,
}

/// Work every running spell forward.
///
/// Ordered before `tower::finish` in the schedule, so a spell sees the world as
/// the previous tick left it rather than racing the completion of the run it is
/// waiting on.
pub fn advance(world: &mut World) {
    // Ordered by `NodeId`, never by a query's iteration order: adding or
    // removing `Running` moves an entity between tables, and two spells'
    // instructions must interleave the same way on every run from a seed.
    let mut running: Vec<(Entity, NodeId)> = world
        .query::<(Entity, &Running)>()
        .iter(world)
        .map(|(entity, running)| (entity, running.spell))
        .collect();
    running.sort_unstable_by_key(|(_, spell)| *spell);

    for (entity, _) in running {
        // An invocation needs you standing there: the domain is fixed at cast
        // and the player's position was never read again, so walking out and
        // leaving one running was free (§19).
        //
        // A bound spell survives it, and so does anything it casts — see
        // [`Running::unattended`]. Asking about the `Bound` *component* killed a
        // held spell's nested `invoke`, since only the parent wears it.
        if !world
            .get::<Running>(entity)
            .is_some_and(|state| state.unattended)
            && left_it(world, entity)
        {
            continue;
        }
        // Everything this spell emits is marked as its doing, set once here
        // rather than at the emit sites — a spell's output *is* what the
        // ordinary commands emit, so every future one would have to remember.
        //
        // The transcript then draws what the player did and the log keeps
        // everything (`FieldName::Spell`); a `repeat` pushing records every few
        // ticks scrolled the player's own last line off in seconds.
        let named = world
            .get::<Running>(entity)
            .map(|state| spell_name(world, state));
        set_attribution(world, named.as_deref());
        step_one(world, entity);
        // Cleared unconditionally: a spell left credited would take the player's
        // own next line with it, and the transcript would stop showing them their
        // own typing.
        set_attribution(world, None);
    }
}

/// Whether the player has walked out on this invocation, ending it if so.
///
/// Compared by domain, not by node: `attend alembic` stands the player at a
/// fixture *inside* the laboratory, so comparing entities would stop an
/// invocation every time its owner leaned over an instrument.
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
/// `alongside` forks a second cursor (§8, [`Strand`]). They step in `strands`
/// order, each spending its whole budget before the next begins — batch rather
/// than round-robin, which is a determinism rule rather than a preference: it is
/// the rule [`advance`] uses one level up, and the two are identical at budget 1,
/// so nothing written today would pin them. The pin is
/// `two_cursors_interleave_the_same_way_at_two_steps_a_tick`.
///
/// A cursor that runs off the end is removed and the rest carry on. One that
/// blocks yields only itself — see [`step_strand`].
fn step_one(world: &mut World, entity: Entity) {
    let mut index = 0;
    loop {
        // Re-read every lap, because stepping changes the count: a spent strand
        // is removed here and `alongside` appends one, so a length taken before
        // the loop would step a strand that had gone or miss one that arrived.
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
        // `finish` removes the component, so there may be nothing left to put
        // back. Checked before the swap rather than inside it, because "the
        // spell ended" and "this strand ended" are different answers.
        if world.get::<Running>(entity).is_none() {
            return;
        }
        if swap_out(world, entity, index) {
            // The strand went, so the next is at this index. `remove`, never
            // `swap_remove`: reordering live cursors would change how they
            // interleave and break replay for a spell that outlives a fork.
            continue;
        }
        index += 1;
    }
}

/// Move `strands[index]` into the fields the runner walks.
///
/// A swap rather than an index everywhere, because the ~110 places that touch
/// `pc`, `loops`, `vars`, `part`, `stack`, `seen`, `waiting_since` and `biding`
/// would each have to name a cursor — one silent failure mode per site. So the
/// active cursor lives in `Running`'s own fields and the others are parked
/// beside it, which is `Cwd`'s idiom one level down.
///
/// The cost: `Running`'s cursor fields mean *the cursor currently stepping*,
/// unambiguous only inside [`step_one`]. Anything reading them from outside gets
/// whichever strand was put back last — [`line_of`]'s problem, not this one's.
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
/// Returns whether the strand ended, which is what tells [`step_one`] not to
/// advance its index.
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
/// §8.1's *trigger-clock* surface: a spell whose schedule has been dragged gets
/// fewer instructions a tick, so it falls behind the world it was written
/// against without a single line of it being wrong — the subtlest of the four,
/// since it reads perfectly and only `verify` finds it.
///
/// It skips whole ticks rather than shaving the budget, because the shipped
/// `SCRIPT_BUDGET` is 1: `allowance - drag` floored at one returned *one* for
/// every drag value. So a dragged spell runs on one tick in every `drag + 1`,
/// keeping §8's *"scripts always log and never halt"* by periodicity.
fn dragged(world: &World, entity: Entity, allowance: usize) -> usize {
    // `entity` is the spell's own node, because `invoke` inserts `Running` onto
    // it. `Retimed` rides the node rather than the `Running`, where `Bound`
    // sits: a `Running` is rebuilt every lap, so sabotage hung on one would be
    // repaired by the spell running off the end.
    let Some(drag) = world
        .get::<super::super::Retimed>(entity)
        .map(|retimed| retimed.drag)
        .filter(|drag| *drag > 0)
    else {
        return allowance;
    };
    // The tick decides, so this is a pure function of replayed state and takes
    // no draw of its own.
    let now = world.resource::<crate::tick::Tick>().get();
    if now.is_multiple_of(drag + 1) {
        allowance
    } else {
        0
    }
}

/// [`dragged`], reachable from a test. The doc lives on `dragged` itself.
#[cfg(test)]
pub(crate) fn dragged_for_test(world: &World, entity: Entity, allowance: usize) -> usize {
    dragged(world, entity, allowance)
}

/// Run up to [`budget`] instructions of the cursor that is currently swapped in.
///
/// Read once, before the first step. A node cannot be taken mid-tick, and if it
/// ever could, a budget that grew while it was being spent is the shape a loop
/// guard must never have.
fn step_strand(world: &mut World, entity: Entity) {
    let allowance = dragged(world, entity, budget(world));
    for _ in 0..allowance {
        // Before the state is read, so every step sees its cursors. Entry and
        // lap both arrive here, which makes this the one writer — see
        // [`bind_cursors`].
        bind_cursors(world, entity);
        let Some(state) = world.get::<Running>(entity).cloned() else {
            return;
        };

        let Some(step) = super::program::at(walking(&state), &state.pc).cloned() else {
            // Off the end of a *frame*, which is not the end of the spell: a
            // part that has run out returns to its caller, and only the
            // outermost frame finishes anything.
            if returned(world, entity) {
                continue;
            }
            // This *cursor* is done, which is not the spell. Finishing here
            // would have a producer running out and taking its consumer down
            // mid-pull; [`ended`] calls `finish` when the last has gone.
            if let Some(mut running) = world.get_mut::<Running>(entity) {
                running.spent = true;
            }
            ended(world, entity, &state);
            return;
        };

        // A definition is stepped past where it stands. Reaching one is
        // ordinary — a spell is read top to bottom — and running it here would
        // do the work twice for anyone who also called it.
        if matches!(step.kind, super::Kind::Part { .. }) {
            advance_pc(world, entity);
            continue;
        }

        // A call suspends this frame and opens one on the part.
        if let super::Kind::Call { name, args } = &step.kind {
            called(world, entity, &state, step.line, name, args);
            continue;
        }

        // A fork starts the part as a cursor of its own and steps past.
        if let super::Kind::Alongside { name, args } = &step.kind {
            forked(world, entity, &state, step.line, name, args);
            continue;
        }

        // Entering a block spends a budget step, which looks wasteful and is the
        // guard: a `repeat` whose body never spends any — empty, or every step
        // an already-satisfied `wait` — would be an unbounded loop inside one
        // tick. The budget is all that stands between a typo and a hang.
        if let super::Kind::Repeat { times, until, body } = &step.kind {
            // The guard is asked on the way in as well as the way out, which is
            // the difference between a guard and a do-while: `repeat until the
            // stacks is idle` with the stacks already shut must run zero times.
            // An unanswerable question stops it too — see `guard_answers`.
            //
            // Asked in the spell's own room, as an `if` is: `holds` finds a
            // place through `Cwd`, so answering against wherever the *player*
            // stands made a bound spell see nothing and stop.
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
            // An empty body is stepped past, not into: descending puts the path
            // somewhere `at` cannot resolve, which the runner reads as the end
            // of the spell. `repeat 0` too — the turn counter was consulted only
            // on the way *out*, so the body ran once, which is also §8.1's shape
            // where an enemy zeroing a count bought one execution.
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

        // An `if` asks the world a question and takes one half or the other. A
        // branch is always entered, even an empty `else` — `step_past` already
        // walks in and straight out correctly, and special-casing it here would
        // be a second exit to keep in step with the first.
        if let super::Kind::If {
            condition,
            body,
            otherwise,
        } = &step.kind
        {
            // The spell's own domain, for the length of the question. `Cwd` is
            // how a place is found, exactly as it is for a command.
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

            // `None` is a third answer and has to be said out loud: the question
            // named a place the tower does not have — §8's *Referent missing*,
            // not the answer being no.
            //
            // Once per line per cast, not per evaluation: inside a `repeat` a
            // single bad name emitted a Danger record every tick. `said` is
            // cleared at cast and when the text changes, so a fix is heard about.
            if !missing.is_empty() && !already_said(world, entity, step.line) {
                say_failure(
                    world,
                    &state,
                    "spell_nowhere",
                    &missing.join(", "),
                    Role::Danger,
                );
            }

            // Neither half, when nobody can answer. Falling through to `else`
            // had a spell with one bad name take the same branch for ever,
            // looking exactly like an inverted condition. A question the orb
            // cannot answer decides nothing.
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

        // A set is walked, and its cursor bound before the body runs. Entering
        // costs a budget step, as a `repeat` does: a `for each` over an empty
        // set that cost nothing would be a free lap.
        if let super::Kind::Each { group, body } = &step.kind {
            let members = node_of(world, state.at)
                .map(|room| tower::group_at(world, room, group).len())
                .unwrap_or_default();
            // Nothing to walk, or nothing to do with it. Stepped past rather
            // than into, as `repeat 0` and an empty body are: descending puts
            // the path somewhere `at` cannot resolve.
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

        // A binding costs a step, like everything else (§8). It reads no world
        // and takes no slot, but a free line would make a spell of nothing but
        // `set` an unbounded loop inside one tick.
        if let super::Kind::Let { name, value } = &step.kind {
            let stood_for = substituted(&state.vars, value);
            if let Some(mut running) = world.get_mut::<Running>(entity) {
                running.vars.insert(name.clone(), stood_for);
            }
            advance_pc(world, entity);
            continue;
        }

        // A `pull` is a `let` whose value comes out of the world, so it costs a
        // step for the same reason and yields instead of binding when the
        // satchel is bare.
        if let super::Kind::Pull { name, from } = &step.kind {
            let from = substituted(&state.vars, from);
            if pull(world, entity, &state, name, &from) == Progress::Blocked {
                return;
            }
            continue;
        }

        // A `wait` reads the world rather than acting on it, so it costs no
        // position swap and no dispatch.
        if let super::Kind::Wait(wanted) = &step.kind {
            let wanted = substituted(&state.vars, wanted);
            if wait_for(world, entity, &state, &wanted) == Progress::Blocked {
                return;
            }
            continue;
        }

        // A `bide` spends the rest of the tick and nothing else. It reads no
        // world and issues no command, so it cannot fail — and it deliberately
        // does not consult `PATIENCE`, because a bide is blocked on nothing.
        //
        // The *runner* holds how far through it is, as a `repeat`'s laps are
        // held: a program is compiled once and cast many times, so a countdown
        // written into the step would leave the second cast biding zero.
        if let super::Kind::Bide(delay) = &step.kind {
            if bide(world, entity, &state, *delay) == Progress::Blocked {
                return;
            }
            continue;
        }

        let super::Kind::Command(line) = step.kind else {
            continue;
        };
        // Bound names stand for what they hold before the parser sees the line:
        // `follow way` has to reach the dispatch as `follow north`, word-wise
        // so a variable called `n` cannot rewrite the middle of `north`.
        let line = substituted(&state.vars, &line);

        // The spell's own position, swapped in for the length of one instruction
        // and swapped back before anything else can see it.
        //
        // A save/restore rather than a position threaded through every verb
        // body, which would grow a once-used parameter on a dozen verbs. This is
        // strictly nested and `scene::rebuild` runs in a later pass, so nothing
        // outside observes the swap.
        let player = world.resource::<Cwd>().0;
        let Some(at) = node_of(world, state.at) else {
            finish(world, entity, &state);
            return;
        };
        world.insert_resource(Cwd(at));
        world.insert_resource(Caller(Some(Casting {
            depth: state.depth,
            unattended: state.unattended,
        })));

        let outcome = run_line(world, entity, &state, &line);

        // The player's position, back. Nothing is read out of the swap: the
        // spell's domain is fixed, so where the instruction left `Cwd` is not a
        // fact worth keeping. It was, while `attend` walked.
        world.insert_resource(Cwd(player));
        world.insert_resource(Caller(None));

        if outcome == Progress::Blocked {
            return;
        }
    }
}

/// `text` with every bound name replaced by what it stands for.
///
/// Word by word, never as a substring: a variable called `n` substituted
/// textually would rewrite `north` into `<value>orth`.
///
/// Case-folded on the way in, because `set` lowercases the name it binds. Not
/// recursive: a name standing for another name would be a chain nobody wrote, so
/// `set best to way` resolves `way` once, where the line runs — see
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
/// Through `Condition::rename`, which is the same walk `compile` uses to fix
/// names against the room — so a variable reaches `holds` looking exactly like a
/// name the player typed, and nothing downstream needs to know variables exist.
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
/// Refreshed here rather than written once on entry, because a cursor moves on
/// every lap and the lap happens inside
/// [`step_past`](super::program::step_past), which knows nothing about sets.
/// Binding on entry alone would leave `way` holding the first member for the
/// whole loop.
///
/// One place, at the top of every step, from the loop stack: idempotent and
/// correct for entry and lap alike. Two writers for one binding is how the two
/// exits from a block came to disagree — see [`Loop`](super::Loop).
///
/// The stack is walked against the path because the elements each descent costs
/// differ: a `repeat` and a `for each` one, a branch of an `if` two.
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
/// Only what arrived since the cursor: reading records after the `seen` mark
/// lets a spell blocked for thirty ticks still see everything that happened in
/// them, and stops the second turn of a loop returning instantly on the first
/// turn's event. A `wait` waits for something *new*.
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
        // Everything up to here has been accounted for, so the next `wait` —
        // this one on the loop's next turn — starts from a clean mark.
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

    // Waiting is normal; waiting for ever is not — §8's *"scripts always log
    // and never halt"*.
    if tick.get().saturating_sub(since.get()) >= PATIENCE {
        say_failure(world, state, "spell_gave_up", wanted, Role::Danger);
        if let Some(mut running) = world.get_mut::<Running>(entity) {
            running.waiting_since = None;
            running.seen = now;
        }
        advance_pc(world, entity);
        return Progress::Done;
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
        // The prompt's answer in a spell's voice: nothing is missing, a verb was
        // handed words it cannot use. See `compile::reading`.
        Resolution::TakesNothing { extra, .. } => {
            say_failure(world, state, "spell_takes_nothing", &extra, Role::Danger);
            advance_pc(world, entity);
            return Progress::Done;
        }
        // §8's *Referent missing*: the line named something that is not there
        // any more, or is not there from here. Skips, logs, continues.
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

    // Through the *same* dispatch a typed line takes. A script must not get a
    // second implementation of any verb, or the live game and the balance
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
/// `waiting_since` carries the countdown rather than a second counter beside it:
/// a second answer to when-did-this-step-begin is the shape §19 records going
/// wrong most. It also makes a bide survive a save for free, which §8 requires.
///
/// It never reaches `PATIENCE`, deliberately: a spell counting to three is doing
/// what it was written to do, so this says nothing on the transcript.
fn bide(world: &mut World, entity: Entity, state: &Running, delay: u32) -> Progress {
    let now = *world.resource::<Tick>();
    // Stamped once, on the tick the bide begins, and held. The pair has to be
    // latched together even now the count is a literal, because
    // `waiting_since` alone cannot say whether the bide has started.
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
    // `>=`, so `bide 1` spends one whole tick and `bide 0` none — the count is
    // ticks *elapsed*. `ticks - 1` because completing spends a tick of its own,
    // so a bide that finished on the tick its count ran out put the next
    // instruction a tick late. `bide n` means "the next line runs n ticks from
    // here", which makes `bide 0` and `bide 1` the same line at one step a tick.
    //
    // Still clamped: only a very large literal reaches here now that `bide
    // <reading>` no longer compiles, and the failure looks the same either way.
    if now.get().saturating_sub(since.get()) >= u64::from(ticks.saturating_sub(1)) {
        if let Some(mut running) = world.get_mut::<Running>(entity) {
            running.waiting_since = None;
            running.biding = None;
        }
        advance_pc(world, entity);
        return Progress::Done;
    }
    Progress::Blocked
}

/// Hold the spell where it is, saying so once.
fn wait(world: &mut World, entity: Entity, state: &Running, blocked: &Blocked) -> Progress {
    let now = *world.resource::<Tick>();
    let since = match state.waiting_since {
        Some(since) => since,
        None => {
            // The first tick of a wait is the one worth a line; a record per
            // tick would bury the log under a spell behaving correctly.
            say_blocked(world, state, blocked);
            if let Some(mut running) = world.get_mut::<Running>(entity) {
                running.waiting_since = Some(now);
            }
            now
        }
    };

    if now.get().saturating_sub(since.get()) >= PATIENCE {
        // Waiting has become halting, which §8 forbids — so this is where a
        // wait stops being quiet.
        say_failure(world, state, "spell_gave_up", blocked.name(), Role::Danger);
        if let Some(mut running) = world.get_mut::<Running>(entity) {
            running.waiting_since = None;
        }
        advance_pc(world, entity);
        return Progress::Done;
    }
    Progress::Blocked
}

/// Whether a script may issue this verb at all.
///
/// Three of these are hazards rather than nonsense:
///
/// - `meditate` writes `Skip`, which `Sim::step` drains in a while-loop, so a
///   scripted `meditate 3600` runs an hour of world time inside one `step()`.
/// - `scribe` opens the editor, so the *player's* next keystrokes land in a
///   spell they did not open.
/// - `undo` is command-anchored (§6) and has no meaning from a script.
///
/// `execute::is_live` is the wrong instrument: it answers *"does this verb
/// work"*, a different question that happens to overlap today.
#[must_use]
pub const fn may_issue(verb: Verb) -> bool {
    !matches!(
        verb,
        // A spell does not walk: it is written *for* a domain, and allowing
        // `attend` would put back the walking position that makes
        // canonicalising an `if` undecidable.
        Verb::Attend
            | Verb::Meditate
            | Verb::Scribe
            | Verb::Undo
            | Verb::Bind
            // `unfurl` takes the keyboard as `scribe` does — it hands the
            // transcript the keys and pages the view back, and inside a
            // `repeat` it re-seizes faster than Escape can give it back.
            | Verb::Unfurl
            // `weave` opens a whole screen, the same objection with more of the
            // window behind it: `repeat 100 / weave` is a soft-lock.
            | Verb::Weave
            // `wander` draws no screen but makes the prompt dead, so Escape is
            // the only way out — and a spell re-taking the arrows every lap
            // would race the player for the one key that ends it.
            //
            // `follow` is deliberately not here: walking the maze is the point
            // of automating the archive. Nor are `summon` and `limn` — one opens
            // a puzzle and takes no keys, the other is the act itself.
            | Verb::Wander
            // And it may not end the session. `quit` reached the vocabulary,
            // `Verb::ALL` and `dispatch::execute` with this list the one place
            // it was missed, so a spell could raise `Quitting` — and a bound
            // spell re-casts off the end, closing the game on the orb's clock.
            | Verb::Quit
            // And it may not open the menu, which takes the pane and the keys:
            // the keyboard-seizing argument again, and a bound spell re-casting
            // would put it back up every lap.
            | Verb::Menu
    )
}

/// Which line of the file execution is on, or `None` once it has run off the end.
///
/// The step's own line, carried from parsing. Deriving it from the path cannot
/// work: the program is a tree and the file a list, comments are not steps, and
/// a step two blocks deep is three path elements. §8.1 wants the culprit named,
/// and *"line 3"* about the wrong line is worse than nothing.
///
/// `Option`, not `0`: a spell whose last line has run keeps `Running` until the
/// tick tidies it, and a sentinel put the marker on line zero, which no file has.
#[must_use]
pub fn line_of(state: &Running) -> Option<u64> {
    super::program::at(walking(state), &state.pc).map(|step| u64::try_from(step.line).unwrap_or(0))
}

/// Hand the current frame back to whoever called it.
///
/// Returns whether there was one. `false` means the outermost frame has run out,
/// which is the only thing that ends a spell.
///
/// The caller resumes pointing at its own call and is stepped past it here
/// rather than on the way in: §8 requires in-flight state to be serialisable at
/// *every* tick boundary, and a `pc` past the call is a position the save could
/// not explain.
fn returned(world: &mut World, entity: Entity) -> bool {
    // Scoped, not `drop`ped: the borrow has to end before `advance_pc` takes
    // the world again, and dropping a `Mut<'_, _>` is a no-op clippy objects to.
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
        // The part's own store goes with it. A `let` inside a part is the
        // part's, and an argument bound on the way in must not be readable by
        // the line after the call.
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
/// Three refusals, all loud: a part the spell does not define is a missing name,
/// said once per line per cast (`Running::said`); a call handing over the wrong
/// number of names cannot be made; a stack past [`MAX_PARTS`] is runaway
/// recursion. Each steps past rather than halting (§8).
///
/// The arity check is here as well as in `compile` because §8 hot-reloads a
/// spell's text under it: a player who adds a parameter while the spell runs
/// leaves every call one short, and binding what arrived would let the body ask
/// about a name standing for itself.
fn called(
    world: &mut World,
    entity: Entity,
    state: &Running,
    line: usize,
    name: &str,
    args: &[String],
) {
    let Some(params) = super::program::signature(state.program.body(), name) else {
        // Compilation already complains about this, so reaching it means the
        // definition went away *while the spell ran* — §8's hot-reload, arriving
        // at the one line that cannot survive it.
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
    // Resolved in the caller's store, before that store is put away: an
    // argument is a name, so what travels is what the caller has it standing
    // for. One level, `substituted`'s rule. `program::arguments` already
    // lowercases every slot as it parses.
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
            // Taken, not cloned: the callee opens with a store of its own, so
            // leaving the caller's behind would be the old shared shape with a
            // copy on the stack that nothing reads.
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
/// [`called`] is the sibling and most of this is its body. Three differences:
///
/// - The caller is not suspended: no [`Descent`] is pushed, so the line after
///   the fork runs on the caller's next step.
/// - The new cursor has an empty stack, because nothing waits for it. Running
///   off the end ends the strand and no more — see [`ended`].
/// - It is bounded by [`MAX_STRANDS`] rather than [`MAX_PARTS`]: how many
///   cursors there are against one cursor's depth.
///
/// The strand is appended and [`step_one`] re-reads the count each lap, so a
/// fork made part-way through this tick *is* stepped once more — `invoke`'s
/// door, where the new thing gets its budget from the point it exists.
fn forked(
    world: &mut World,
    entity: Entity,
    state: &Running,
    line: usize,
    name: &str,
    args: &[String],
) {
    // `pull`'s gate, for `pull`'s reason — the complaint at cast is the report
    // and this is what stops the line.
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
    // Resolved in the forking cursor's store, at the moment of the fork.
    // `called`'s rule, and it has to be: the caller goes on running, so a name
    // resolved later would be read against a store that had moved (§19).
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
        // `seen` is carried over, not started at nought: a strand opening at
        // zero would have its first `wait` satisfied by every event of the run
        // so far, all at once.
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
/// Empty when the part it names has gone, which is a real state: §8 hot-reloads
/// a spell's text, so a player may delete a definition while a frame is inside
/// it. An empty block reads as *off the end* at the next step.
fn walking(state: &Running) -> &super::Block {
    static NOTHING: super::Block = Vec::new();
    super::program::tree(state.program.body(), state.part.as_deref()).unwrap_or(&NOTHING)
}

/// Whether this casting has already complained about `line`, marking it said.
///
/// See [`Running::said`]. Marking here rather than at the call site keeps "have
/// we said it" and "we have now" from drifting apart.
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
/// The program is cloned out first because `step_past` needs it while `Running`
/// is borrowed mutably and both live on the same entity — one clone per
/// instruction, against threading a borrow through the whole runner.
fn advance_pc(world: &mut World, entity: Entity) {
    let Some((program, part)) = world
        .get::<Running>(entity)
        .map(|state| (state.program.clone(), state.part.clone()))
    else {
        return;
    };
    // The frame's own tree, cloned with the program for the same borrow reason.
    // An absent part is an empty block, which `step_past` walks straight off the
    // end of — see [`walking`].
    let body = super::program::tree(program.body(), part.as_deref())
        .cloned()
        .unwrap_or_default();
    // Every guard answered before `Running` is borrowed mutably: `holds` wants
    // the world and `step_past` wants `&mut Running`, both on this entity. Only
    // the loops the path is *inside*, and all in the spell's own room — see
    // `guard_answers`.
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
                // A `for each` goes round while the set has a member after the
                // one just finished. No entry means no set — where a group the
                // room does not have ends up — so the loop stops rather than
                // walking nothing for ever.
                (super::Loop::Each(index), Some((_, Continues::Members(many)))) => {
                    index + 1 < *many
                }
                (super::Loop::Each(_), _) => false,
                // An unguarded `repeat` is absent from the list and that reads as
                // yes, so it costs no world read at all.
                (_, Some((_, Continues::Guard(more)))) => *more,
                (_, _) => true,
            }
        };
        if !super::program::step_past(&body, pc, loops, again) {
            // Off the end. `at` will return `None` next time round and the
            // spell finishes there, so there is one place that ends a spell.
            pc.clear();
        }
    }
}

/// Answer `condition` in the *spell's own room*, not the player's.
///
/// `watch::holds` finds a place through `Cwd`, so a question answered wherever
/// the player happens to stand is about the wrong room. `Kind::If` always
/// swapped; `until` did not, which made a bound spell see nothing and stop.
///
/// One function rather than the swap written twice, because two expressions of
/// one rule is how they came to disagree.
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
/// It looks in the room the *spell* stands in, never the player's: a satchel
/// exists in every domain under one name, so a lookup against `Cwd` would have a
/// bound producer fill one queue while its consumer drained another, in silence.
/// §19 records this confusion three times; `state.at` is the answer every time.
///
/// Yielding is not waiting: an empty satchel returns [`Progress::Blocked`]
/// without touching `waiting_since`, so it never reaches [`PATIENCE`] and never
/// latches `‼` on the rail. A consumer caught up with its producer is the
/// ordinary state of a working pipeline. What bounds it is the work running out.
///
/// A satchel that is not there *is* a fault, said once: `pull note from
/// mortar_and_pestle` names something real that holds no queue, which is §8's
/// *Referent missing* rather than a wait, so it goes through [`say_missing`]
/// and steps past.
fn pull(world: &mut World, entity: Entity, state: &Running, name: &str, from: &str) -> Progress {
    let line = super::program::at(walking(state), &state.pc).map_or(0, |step| step.line);
    // Asked here as well as at cast, which is `may_issue`'s rule: a boundary
    // with one guard is one a future caster can walk around. Silent, because
    // `compile::check_learned` has said it once per cast already.
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
        return Progress::Blocked;
    };

    if let Some(mut running) = world.get_mut::<Running>(entity) {
        running.vars.insert(name.to_owned(), taken);
    }
    advance_pc(world, entity);
    Progress::Done
}

/// Name what a guard could not place, once per line per cast.
///
/// §8.1's *"the culprit is never anonymous"*. A guard that stops on an
/// unanswerable question is the one failure mode with nothing on screen
/// explaining it: the spell simply does not run.
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
///
/// Two shapes, because the two loops end for different reasons: a `repeat until`
/// ends when the world says so and a `for each` when it runs out of members, so
/// the walker owes the first an answer and the second a count. The runner
/// compares, because only it knows which member the loop is on.
#[derive(Debug, Clone, Copy)]
enum Continues {
    /// A `repeat until`: whether the loop may take another turn.
    Guard(bool),
    /// A `for each`: how many members its set has, now.
    Members(u32),
}

/// Whether each guarded loop may take another turn, by path.
///
/// Three answers folded into two, and the fold is a decision. `until X`
/// continues while X is *not* satisfied, so `Some(false)` goes round again and
/// `Some(true)` stops; an unanswerable question — §8's *Referent missing* —
/// stops too.
///
/// That is the opposite of `if`, where an unreadable question declines to act.
/// Declining to act is safe; a `repeat` that declined to *stop* would run for
/// ever on a question nobody can answer.
///
/// A loop with no `until` is absent from this list and [`advance_pc`] reads that
/// as yes, so an unguarded `repeat` costs no world read at all.
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
                    // Only a loop the path is inside can be unwound, and such a
                    // loop is always an ancestor of the current `pc`. Asking
                    // the rest would put questions to the world about loops
                    // that are not running.
                    if let Some(condition) = until
                        && pc.starts_with(path)
                    {
                        let more = matches!(super::watch::holds(world, condition).0, Some(false));
                        out.push((path.clone(), Continues::Guard(more)));
                    }
                    walk(world, body, pc, path, out);
                }
                // The set is measured, not the guard asked: a `for each` goes
                // round while the set has a member left, so what the walker
                // owes the runner is a count. Same short-circuit as above.
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
                    // Both halves, and the path elements a branch costs:
                    // `enter_branch` pushes the half *and* the step, so a loop
                    // nested in an `if` is two elements deeper than its index
                    // suggests — matching what `step_past` pops.
                    for (half, block) in [body, otherwise].into_iter().enumerate() {
                        path.push(half);
                        walk(world, block, pc, path, out);
                        path.pop();
                    }
                }
                // A definition is not walked into, the reason `at` refuses to
                // descend: its loops belong to another frame, so walking in
                // would ask about a `repeat` inside a part nobody has called,
                // against a path this frame's `pc` can never hold.
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

    // The room swapped once around the whole walk, not per guard: every question
    // below is answered where the *spell* stands, for `Kind::If`'s reason — a
    // bound spell runs while the player is in another room by design.
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

/// The spell has run out of lines.
///
/// Silent for a held spell, because it has not finished: `bind::stand` casts it
/// again next tick, so *"tending.spell is finished"* would be contradicted a
/// tick later. A binding ends when the player says `stop`.
///
/// One cursor has run out; end the spell if it was the last. That is
/// `alongside`'s termination rule — a `repeat` with no guard in a forked part
/// keeps the whole spell alive.
fn ended(world: &mut World, entity: Entity, state: &Running) {
    let last = world
        .get::<Running>(entity)
        .is_none_or(|running| running.strands.len() <= 1);
    if last {
        finish(world, entity, state);
    }
}

fn finish(world: &mut World, entity: Entity, state: &Running) {
    let name = spell_name(world, state);
    if let Some(mut bound) = world.get_mut::<super::Bound>(entity) {
        // What this lap has complained about, kept for the next — see
        // [`Bound::said`]. The component outlives the run, so the rationing has
        // to outlive it too or it is no rationing at all.
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

    // Remove the component, never despawn the entity: `Running` is worn by the
    // spell node itself, so despawning would delete the spell from the grimoire
    // and the next `invoke` would fuzzy-match to some other spell.
    world.entity_mut(entity).remove::<Running>();
}

/// What the spell is called, for a record.
///
/// §8.1: *"the culprit is never anonymous."* Every line this module emits
/// carries the spell and the line number.
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
    // The rail's fault mark is raised here and nowhere else, because this is the
    // one place that knows a spell failed *and* which room it was working in.
    // §8 makes a broken spell log rather than halt, and §9's minimised half is
    // for noticing without going to read that log.
    //
    // `Role::Danger` only: a `Cost` here is a spell waiting its turn for the
    // production slot, which happens constantly and is not a fault.
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
