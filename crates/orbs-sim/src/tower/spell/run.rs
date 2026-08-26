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
/// §8's *"execution budget — the orb's attention"*, and at **one** it stops being
/// only a runaway guard and becomes a mechanic: a spell costs one tick per step,
/// so **a shorter spell is a faster spell**. Two lines that do what three did is
/// a real advantage, and a `repeat` whose body could have been tightened is paid
/// for every turn.
///
/// That is the lever §11.5 wants and four did not give: at four, the difference
/// between a tight spell and a sloppy one disappeared inside a single tick, and
/// the only thing the budget bounded was catastrophe.
///
/// Everything counts as a step — a command, checking a `wait`, entering a
/// `repeat`, asking an `if`. Counting only *commands* would make block-heavy
/// spells free, which is the opposite of the incentive.
///
/// **The floor, not the number.** It is what the orb can do untrained, and
/// [`budget`] is what it can do now — the weave raises it. This stays a `const`
/// because a default has to exist before any world does: `Taken` is a resource,
/// and half the tests here build a program without one.
pub const SCRIPT_BUDGET: usize = 1;

/// How many steps a spell may take this tick.
///
/// # Why this stopped being a constant
///
/// A step costs a tick, so the budget *is* the speed of every piece of
/// automation in the game — and §11.5 wants the weave to sell something, while
/// §8's own note says at one step the budget is *"a mechanic"* rather than a
/// guard. A number that is both the mechanic and the reward has to be readable
/// from the tree, and a `const` cannot be.
///
/// **It reads `Taken`, which is empty and stays empty in this version.** Every
/// Mastery node ships as a marker, so this answers [`SCRIPT_BUDGET`] today and
/// the wiring is what is being built — making the tree takeable is the weave
/// phase's item, not this one. The nodes are authored in `progression.toml` and
/// on screen, so a player at 24 experience can see what the choice will be.
///
/// **Additive, and deliberately not a maximum.** Two nodes granting a step each
/// give three, because a tier is *one of* its siblings — a player who takes the
/// step node in two tiers has spent both choices on speed, and reading it as
/// `max` would silently refund the second.
#[must_use]
pub fn budget(world: &World) -> usize {
    let extra: usize = world.get_resource::<tower::Taken>().map_or(0, |taken| {
        taken.ids().iter().filter_map(|id| steps_granted(id)).sum()
    });
    SCRIPT_BUDGET.saturating_add(extra)
}

/// What one Mastery node adds to the budget, if that is what it is for.
///
/// **The id is the contract**, exactly as `progression.toml` says: *"ids are
/// decisions, not prose — they are what a taken node is stored as."* So the
/// grant is derived from the id rather than from a second table that could
/// disagree with the one the screen draws.
fn steps_granted(id: &str) -> Option<usize> {
    id.strip_prefix("steps_")?.parse().ok()
}

/// How long a blocked instruction waits before it is called a failure.
///
/// **Waiting is normal; waiting for ever is not.** A spell that grinds and then
/// siphons blocks for the whole grind, which is right. A spell waiting on an
/// instrument nothing will ever free has stopped — and §8's taxonomy is titled
/// *"scripts always log and never halt"*, so an unbounded silent yield would be
/// the one thing it forbids, arriving through the mechanism that makes waiting
/// quiet.
///
/// Generous on purpose: longer than any single §10.1 stage, so a legitimate wait
/// never trips it.
pub const PATIENCE: u64 = 120;

/// How deep a part may call a part.
///
/// **Separate from [`MAX_DEPTH`], because the two guard different things.** That
/// one bounds `invoke`, where each level is a whole second spell with its own
/// budget and its own record attribution; this bounds a stack of descents inside
/// one spell, which costs a [`Descent`] each and nothing else.
///
/// The budget is not the guard here either, and for §8's stated reason: at one
/// step a tick a runaway recursion does not hang the game, it grows the save by
/// a descent a second until nothing can read it. So it is bounded, and loudly —
/// the same argument `MAX_DEPTH` makes, arrived at from the other side.
///
/// Eight rather than three: a part cannot take an argument, so recursion here is
/// a shape nobody has a use for yet, and the number only has to be past what a
/// person would write on purpose.
pub const MAX_PARTS: usize = 8;

/// How deep `invoke` may nest.
///
/// §8 fixes this at 3 and argues why the budget alone is not a sufficient
/// recursion guard: exhausting it makes every subsequent instruction *Budget
/// starved*, which logs at high verbosity only, so all automation would stop
/// **silently**. Depth-limiting makes runaway recursion loud and diagnosable.
pub const MAX_DEPTH: u8 = 3;

/// The spell whose instruction is running, for anything that instruction casts.
///
/// Set around one instruction and cleared after, exactly as the script's
/// position is. `invoke` reads it to know whether *it* is being called by a
/// spell or typed by a player, which the intent alone cannot say: both arrive
/// through the same dispatch, which is the point (§13).
///
/// **`None` means a player typed the line themselves.**
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct Caller(pub Option<Casting>);

/// What a nested cast inherits from the run that asked for it.
///
/// A struct rather than the bare depth it started as, because a second fact
/// turned out to travel the same road and a second resource set beside the
/// first is two things that must be cleared together — the drift this module
/// already refuses for `Cwd`.
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
    /// Which spell. **A [`NodeId`], not an `Entity`** — `tower::node` is
    /// explicit that an `Entity` is *"meaningless across a save or a rebuilt
    /// world"*, and a half-executed spell is state a save has to carry.
    pub spell: NodeId,
    /// The program, derived from the spell's text when it was cast.
    ///
    /// **A view, never the truth.** `Held` stays canonical because §8's
    /// hot-reload is line-anchored and §8.1's sabotage surface is *"a line
    /// reordered"* — an enemy mutates the text, and the program is whatever the
    /// text then means. Re-derived on every cast.
    pub program: super::Program,
    /// Where in the program execution has reached.
    ///
    /// A **path** rather than a line number: `[0]` is the first step, `[2, 1]`
    /// is the second step inside the third. Blocks made a single `pc`
    /// insufficient the moment they arrived.
    pub pc: Vec<usize>,
    /// Each open block, outermost first — what it is and what it has left.
    ///
    /// §8 requires in-flight state be serialisable, and a spell suspended inside
    /// a loop is exactly that: a save with a `pc` and no counts would resume
    /// every enclosing loop from its first turn. It records the block's *kind*
    /// as well, because walking out of an `if` pops one more path element than
    /// walking out of a `repeat` — see [`Loop`](super::Loop).
    pub loops: Vec<super::Loop>,
    /// How far through the record stream this spell has read.
    ///
    /// A **sequence number**, not an index — see `Records::sequence`. A `wait`
    /// looks only at what arrived after this, which is what lets a spell blocked
    /// for thirty ticks still see everything that happened in them.
    pub seen: u64,
    /// How many `invoke`s deep this is.
    pub depth: u8,
    /// Whether this run survives the player leaving the domain it runs in.
    ///
    /// **Not the same question as `Bound`**, and conflating them was a bug: a
    /// bound spell that `invoke`s another gives its child a `Running` and no
    /// `Bound` — the child is not held, it is a step of something that is — so
    /// asking about the component ended the child the moment the player walked
    /// out, in exactly the walk-away case `bind` exists to sell.
    ///
    /// Set at cast from the caller ([`Casting`]): a binding is unattended, a
    /// standing recast is, and anything either of them casts inherits it. What
    /// a *player* invokes is attended, and so is everything it invokes — or the
    /// child would outlive the parent the player's own departure just ended.
    pub unattended: bool,
    /// The domain this spell runs in.
    ///
    /// **Fixed, not walked.** A spell is written for a domain and works there;
    /// `attend` inside one is refused. What remains necessary is the *swap* —
    /// verb bodies read `Cwd` directly (`siphon` puts what it collects where you
    /// stand), so the domain is installed around each instruction and the
    /// player's own position put back. Removing the walk did not remove that.
    pub at: NodeId,
    /// When the current instruction first found itself blocked.
    pub waiting_since: Option<Tick>,
    /// Lines this casting has already complained about a missing name on.
    ///
    /// **Once per line per cast.** A question inside a `repeat` is asked every
    /// turn, and a name the tower cannot place is wrong on the first turn in
    /// exactly the way it is wrong on the four hundredth — so saying it once is
    /// the report, and saying it every time is a fault of its own. Cleared when
    /// the spell is cast and when its text changes under it, so a name that goes
    /// missing *later* is still heard about.
    ///
    /// A `Vec` rather than a set because it holds one entry per broken line of
    /// one spell, and its order is part of a deterministic session.
    pub said: Vec<usize>,
    /// What each name the spell has bound stands for.
    ///
    /// `set best to north` puts one here; `for each way` rebinds `way` at the
    /// top of every pass. A value is a **name**, already resolved against the
    /// room — see [`Kind::Let`](super::Kind::Let).
    ///
    /// # Ordered, and that is not decoration
    ///
    /// A `BTreeMap` rather than a `HashMap`: this travels to a save, and a
    /// document whose rows moved between two runs of one seed would fail the
    /// lockstep test that pins the snapshot as a complete description.
    ///
    /// # Cleared at cast, kept across a mid-flight edit
    ///
    /// The same answer `pc` and `loops` get, and for the reason `scribe` gives
    /// for them: *"a diff that guesses wrong moves a running spell to a line the
    /// player did not point it at."* A store rebuilt on every save would empty
    /// an accumulator half way through the loop that was filling it.
    pub vars: std::collections::BTreeMap<String, String>,
    /// Which part's body the current frame is walking, or `None` for the
    /// spell's own.
    ///
    /// A **name**, resolved through `program::tree` at every step, so a
    /// definition that moves while the spell runs is still the same part (§8's
    /// hot-reload is line-anchored and this is the same argument one level up).
    pub part: Option<String>,
    /// The callers waiting for the current frame to return, outermost first.
    ///
    /// **A stack of frames, not a second `pc`.** A path addresses one tree, and
    /// a part is a different tree — so `gathering()` inside a `repeat` inside
    /// `gathering` needs the caller's path *and* its open blocks kept whole
    /// while the callee walks its own.
    pub stack: Vec<Descent>,
}

/// One caller, waiting for the part it called to finish.
///
/// # Not the obvious word, and the same reason `Nesting` is not
///
/// The word for this everywhere else in computing is one of the four layout
/// names `tests/boundaries.rs` forbids anywhere under `orbs-sim/src` — rule 2,
/// matched by *substring* so the guard is unarguable rather than clever. The
/// parser's stack entry already pays this toll; this is the second, and a call
/// stack really is a stack of descents.
///
/// # `vars` is here, and that is the decision — reversed once
///
/// The roadmap's shape for this was `(spell, pc, loops, vars)`, and it was built
/// **without** the `vars`: variables were shared, on the argument that *"a part
/// takes no arguments, so a private store would leave it with no way to be told
/// anything at all"* (§19). A part takes arguments now, which removes that
/// premise rather than overruling it — so the store is per-frame, and the
/// parameters are what fills it.
///
/// What this bought, in the order it matters:
///
/// - **A call says what it hands over, at the call.** `between(wellspring,
///   near)` reads as a sentence. The three `let` pairs it replaced were four
///   lines of ceremony per call and named nothing at the point of use.
/// - **`for each way` inside a part no longer rebinds the caller's `way`.**
///   That was the stated cost of sharing and it is simply gone.
/// - **A part cannot reach a name it was not given**, so reading one is a local
///   act: its parameters and its own `let`s are all there is.
///
/// The cost, stated as plainly as the old one was: a part has **no** access to
/// the caller's bindings, so anything it needs must be passed. For a language
/// whose programs fit on a screen that is the cheaper rule to teach — *what
/// goes in the brackets is what it can see* — and it is the one a reader can
/// check by looking at one line.
///
/// `spell` is not here, and that one is **settled**: a spell is contained to a
/// single `.spell` file, so every descent belongs to the spell that opened it
/// and there is nothing for the field to say (§19). A spell reaching into
/// another spell's text is what `invoke` is for, and an `invoke` is a second
/// [`Running`] with its own budget rather than a descent — which is the
/// distinction that keeps this struct one spell wide.
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
    // **Ordered by `NodeId`, never by a query's iteration order.** `tower::node`
    // records archetype order as a bug that changes what a phrase resolves to
    // with no test catching it, and adding or removing `Running` moves an entity
    // between tables. Two spells' instructions must interleave the same way on
    // every run from a seed.
    let mut running: Vec<(Entity, NodeId)> = world
        .query::<(Entity, &Running)>()
        .iter(world)
        .map(|(entity, running)| (entity, running.spell))
        .collect();
    running.sort_unstable_by_key(|(_, spell)| *spell);

    for (entity, _) in running {
        // **An invocation needs you standing there, and this is what makes that
        // true.** §19 has always said an invoked spell *"needs you standing
        // there"*, and nothing enforced it: the domain is fixed at cast and the
        // player's position was never read again, so walking out and leaving one
        // running was free. That left `bind` with nothing to sell — the one
        // thing §8 says it adds was already there for nothing.
        //
        // A **bound** spell is exactly the one that survives this, which is the
        // whole of what concentration buys — and so is anything a bound spell
        // casts. See [`Running::unattended`]: asking about the `Bound`
        // *component* here killed a held spell's nested `invoke` on the tick the
        // player walked out, because only the parent wears the component.
        if !world
            .get::<Running>(entity)
            .is_some_and(|state| state.unattended)
            && left_it(world, entity)
        {
            continue;
        }
        // **Everything this spell emits is marked as its doing**, set once here
        // rather than at the emit sites — a spell's output *is* what the ordinary
        // commands emit, so there is nothing at those sites to change and every
        // future one would have had to remember.
        //
        // The transcript then draws what the player did and the log keeps
        // everything (`FieldName::Spell`). A `repeat` loop pushes several records
        // every few ticks for as long as it runs, and the player's own last line
        // was scrolling off in seconds.
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
/// **Compared by domain, not by node.** `attend alembic` stands the player at a
/// fixture *inside* the laboratory, and a spell running there is one they are
/// watching — asking whether the two entities are equal would stop an invocation
/// every time its owner leaned over an instrument.
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

/// Run up to [`budget`] instructions of the spell on `entity`.
///
/// **Read once, before the first step.** A node cannot be taken mid-tick, so
/// re-reading it per step would be a resource lookup for an answer that cannot
/// change — and if it ever could, a budget that grew while it was being spent is
/// the shape a loop guard must never have.
fn step_one(world: &mut World, entity: Entity) {
    let allowance = budget(world);
    for _ in 0..allowance {
        // **Before the state is read, so every step sees its cursors.** Entry
        // and lap both arrive here, which is what makes this the one writer —
        // see [`bind_cursors`].
        bind_cursors(world, entity);
        let Some(state) = world.get::<Running>(entity).cloned() else {
            return;
        };

        let Some(step) = super::program::at(walking(&state), &state.pc).cloned() else {
            // **Off the end of a *frame*, which is not the end of the spell.**
            // A part that has run out returns to whoever called it; only the
            // outermost frame running out finishes anything. This is the one
            // place a frame is popped, exactly as `finish` is the one place a
            // spell ends.
            if returned(world, entity) {
                continue;
            }
            finish(world, entity, &state);
            return;
        };

        // A **definition** is stepped past where it stands. Reaching one is
        // ordinary — a spell is read top to bottom and its parts are written
        // among its lines — and running it here would do the work twice for
        // anyone who also called it.
        if matches!(step.kind, super::Kind::Part { .. }) {
            advance_pc(world, entity);
            continue;
        }

        // A **call** suspends this frame and opens one on the part.
        if let super::Kind::Call { name, args } = &step.kind {
            called(world, entity, &state, step.line, name, args);
            continue;
        }

        // Entering a block **spends a budget step**, which looks wasteful and is
        // the guard: a `repeat` whose body never spends any — an empty one, or
        // one whose every step is a `wait` that is already satisfied — would
        // otherwise be an unbounded loop inside a single tick, and the game
        // would stop. The budget is the only thing standing between a player's
        // typo and a hang.
        if let super::Kind::Repeat { times, until, body } = &step.kind {
            // **The guard is asked on the way in as well as the way out**, which
            // is Autonauts' rule and the difference between a guard and a
            // do-while: `repeat until the stacks is idle` with the stacks already
            // shut must run zero times, because the first pass is where a spell
            // does damage. An unanswerable question stops it too — see
            // `guard_answers` for why that is the opposite of `if`'s rule.
            //
            // **Asked in the spell's own room, exactly as an `if` is.** `holds`
            // finds a place through `Cwd`, so answering a guard against wherever
            // the *player* happens to be standing made a **bound** spell — which
            // by design runs while they are elsewhere — see nothing, answer
            // `None`, and stop. `dev_spells.toml`'s `threading` is bound and its
            // bound is `repeat until the stacks is idle`, so the flagship case
            // was the broken one.
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
            // An empty body is stepped **past**, not into. Descending into one
            // puts the path somewhere `at` cannot resolve, which the runner
            // reads as the end of the spell — so `repeat 2 / end / survey` ended
            // before the survey rather than after it.
            // **`repeat 0` is stepped past too.** The turn counter was only
            // consulted on the way *out* (`left > 1`), so a block was always
            // entered once before anything asked whether it should run at all —
            // `repeat 0` did its body exactly once. That is also §8.1's sabotage
            // shape: an enemy zeroing a count still bought one execution.
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

        // An `if` asks the world a question and takes one half or the other.
        // **A branch is always entered**, even when the answer is no and the
        // `else` is empty — walking into an empty block and straight out of it
        // is what `step_past` already does correctly, and special-casing the
        // empty half here would be a second exit to keep in step with the first.
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

            // **`None` is a third answer, and it has to be said out loud.** The
            // question named a place the tower does not have — §8's *Referent
            // missing*, not the answer being no.
            //
            // **Once per line per cast.** It was once per *evaluation*, which is
            // right for a spell that asks a question and stops and wrong for one
            // that asks it inside a `repeat`: a single bad name emitted a Danger
            // record every tick for as long as the spell ran. What a player needs
            // is to be told, not to be told again. `said` is cleared when the
            // spell is cast and when its text changes, so a fix is heard about.
            if !missing.is_empty() && !already_said(world, entity, step.line) {
                say_failure(
                    world,
                    &state,
                    "spell_nowhere",
                    &missing.join(", "),
                    Role::Danger,
                );
            }

            // **Neither half, when nobody can answer.** It used to fall through
            // to `else`, which is worse than useless: a spell with one bad name
            // took the same branch for ever and looked exactly like a condition
            // someone had inverted — §19's own words for the last bug here. A
            // question the orb cannot answer decides nothing.
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

        // **A set is walked, and its cursor is bound before the body runs.**
        // Entering is one budget step, exactly as a `repeat` is and for the same
        // reason: a `for each` over an empty set that cost nothing would be a
        // free lap, and the budget is what stands between a spell and a hang.
        if let super::Kind::Each { group, body } = &step.kind {
            let members = node_of(world, state.at)
                .map(|room| tower::group_at(world, room, group).len())
                .unwrap_or_default();
            // Nothing to walk, or nothing to do with it. Both are stepped
            // **past** rather than into, which is the answer `repeat 0` and an
            // empty body already get — descending would put the path somewhere
            // `at` cannot resolve, which the runner reads as the end of the
            // spell.
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

        // **A binding costs a step, like everything else** (§8: *"everything
        // counts as a step"*). It reads no world and takes no slot, but a line
        // that were free would make a spell of nothing but `set` an unbounded
        // loop inside one tick.
        if let super::Kind::Let { name, value } = &step.kind {
            let stood_for = substituted(&state.vars, value);
            if let Some(mut running) = world.get_mut::<Running>(entity) {
                running.vars.insert(name.clone(), stood_for);
            }
            advance_pc(world, entity);
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

        let super::Kind::Command(line) = step.kind else {
            continue;
        };
        // **Bound names stand for what they hold, before the parser sees the
        // line.** `follow way` has to reach the dispatch as `follow north`, and
        // the substitution is word-wise rather than textual so a variable called
        // `n` cannot rewrite the middle of `north`.
        let line = substituted(&state.vars, &line);

        // The spell's own position, swapped in for exactly the length of one
        // instruction and swapped back before anything else can see it.
        //
        // **A save/restore rather than a position threaded through every verb
        // body.** The alternative touches `move`, `wield`, `empty`, `siphon`,
        // `stop`, `purge`, `divine` and all five per-instrument verbs, and every
        // one of them would grow a parameter it uses once. This is strictly
        // nested, and `scene::rebuild` runs in a later pass than this one, so
        // nothing outside observes the swap.
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

        // The player's position, back. **Nothing is read out of the swap** — the
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
/// **Word by word, never as a substring.** A variable called `n` substituted
/// textually would rewrite `north` into `<value>orth`, and a spell whose names
/// silently changed shape is the class of defect this language refuses
/// everywhere else. Splitting on whitespace also means a bound name can only
/// ever replace a whole word, which is what a player writing `follow way` means.
///
/// Case-folded on the way in, because `set` lowercases the name it binds and a
/// player who writes `Way` in the body meant the same cursor.
///
/// Not recursive: a value is a name, and a name that stood for another name
/// would be a chain nobody wrote. `set best to way` resolves `way` **once**,
/// where the line runs — see [`Kind::Let`](super::Kind::Let).
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
/// # Refreshed here rather than written once on entry
///
/// A cursor moves on every lap, and the lap happens inside
/// [`step_past`](super::program::step_past) — which is pure, has no world, and
/// deliberately knows nothing about sets. Binding on entry alone would leave
/// `way` holding the first member for the whole loop.
///
/// So it is done in **one** place, at the top of every step, from the loop stack
/// itself: idempotent, cheap, and correct for entry and lap alike. Two writers
/// for one binding is how the two exits from a block came to disagree, which
/// [`Loop`](super::Loop) already records.
///
/// # Walking the stack against the path
///
/// `loops` records one entry per descent, and the path elements each costs are
/// not the same: a `repeat` and a `for each` cost one, a branch of an `if` costs
/// two (`enter_branch` pushes the half *and* the step). Walking them together is
/// what turns a stack position into the step that opened it.
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
/// # Only what arrived since the cursor
///
/// The spell reads records after its own `seen` mark, which is what lets one
/// blocked for thirty ticks still see everything that happened in them — and
/// what stops the **second** iteration of a loop returning instantly on the
/// first iteration's event. A `wait` waits for something *new*.
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

    // Waiting is normal; waiting for ever is not. §8's taxonomy is titled
    // *"scripts always log and never halt"*, so an unbounded silent wait would
    // be the one thing it forbids.
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
    let Resolution::Resolved { intent, .. } = analyse(line, &scene, Mode::Calm).resolution else {
        // §8's **Referent missing**: the line named something that is not there
        // any more, or is not there from here. Skips, logs, continues.
        say_failure(world, state, "spell_missing", line, Role::Danger);
        advance_pc(world, entity);
        return Progress::Done;
    };

    if !may_issue(intent.verb) {
        say_failure(world, state, "spell_forbidden", line, Role::Danger);
        advance_pc(world, entity);
        return Progress::Done;
    }

    if let Some(blocked) = would_block(world, &intent) {
        return wait(world, entity, state, &blocked);
    }

    // Through the **same** dispatch a typed line takes. A script must not get a
    // second implementation of any verb, or the live game and the balance
    // harness stop being the same game (§13).
    crate::execute::execute_one(&intent, world);
    if let Some(mut running) = world.get_mut::<Running>(entity) {
        running.waiting_since = None;
    }
    advance_pc(world, entity);
    Progress::Done
}

/// Hold the spell where it is, saying so **once**.
fn wait(world: &mut World, entity: Entity, state: &Running, blocked: &Blocked) -> Progress {
    let now = *world.resource::<Tick>();
    let since = match state.waiting_since {
        Some(since) => since,
        None => {
            // The first tick of a wait is the one worth a line. After that the
            // spell is simply doing what a recipe does, and a record per tick
            // would bury the log under a spell behaving correctly.
            say_blocked(world, state, blocked);
            if let Some(mut running) = world.get_mut::<Running>(entity) {
                running.waiting_since = Some(now);
            }
            now
        }
    };

    if now.get().saturating_sub(since.get()) >= PATIENCE {
        // Waiting has become halting. §8's taxonomy is titled *scripts always
        // log and never halt*, so this is where a wait stops being quiet.
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
/// # Three of these are hazards rather than nonsense
///
/// - **`meditate`** writes `Skip`, which `Sim::step` drains in a while-loop. A
///   scripted `meditate 3600` runs an hour of world time inside a single
///   `step()`, with this runner executing on every one of those ticks.
/// - **`scribe`** opens the editor. From inside a script that means the
///   *player's* next keystrokes land in a spell they did not open.
/// - **`undo`** is command-anchored (§6) and has no meaning from a script.
///
/// `execute::is_live` is the wrong instrument for this: it answers *"does this
/// verb work"*, which is a different question that happens to overlap today.
#[must_use]
pub const fn may_issue(verb: Verb) -> bool {
    !matches!(
        verb,
        // A spell does not walk. It is written **for** a domain and works
        // there, so `attend` is not something it can want — and allowing it
        // would put back the walking position that makes canonicalising an
        // `if` undecidable.
        Verb::Attend
            | Verb::Meditate
            | Verb::Scribe
            | Verb::Undo
            | Verb::Bind
            // **`unfurl` takes the keyboard exactly as `scribe` does.** It hands
            // the transcript the keys and pages the view back, so a spell
            // holding one seizes the prompt on the orb's clock — and inside a
            // `repeat` it re-seizes faster than Escape can give it back.
            | Verb::Unfurl
            // And `weave` opens a whole screen, which is the same objection with
            // more of the window behind it. `repeat 100 / weave` is a soft-lock.
            | Verb::Weave
            // `wander` draws no screen at all, and is refused for the same
            // reason all the same: it makes the prompt dead, so Escape is the
            // only way out — and a spell re-taking the arrows every lap would
            // be racing the player for the one key that ends it.
            //
            // **`follow` is deliberately *not* here.** Walking the maze is the
            // whole point of automating the archive; what a spell may not do is
            // decide who is holding the keyboard.
            | Verb::Wander
            // **And it may not end the session.** `quit` was added to the
            // vocabulary, to `Verb::ALL`, to `dispatch::execute` and to the
            // tower's own count, and this list was the one place it was not —
            // so a spell could raise `Quitting`, which is `AppExit::Success`
            // under Bevy and a raw-mode teardown in the terminal. The three
            // above are barred for *seizing the keyboard*; this one closes the
            // game, and a **bound** spell re-casts every time it runs off the
            // end, so it would do so on the orb's clock with nothing the player
            // pressed able to intervene.
            | Verb::Quit
    )
}

/// Which line of the file execution is on, or `None` once it has run off the end.
///
/// **The step's own line**, carried from parsing. Deriving it from the path
/// cannot work: the program is a tree and the file is a list, blank lines and
/// comments are not steps, and a step two blocks deep is three path elements
/// with no arithmetic relating that to a line number. §8.1 wants the culprit
/// named, and a spell that says *"line 3"* about the wrong line is worse than
/// one that says nothing.
///
/// **`Option`, not `0`.** It returned `0` for "nowhere", which the two log sites
/// below never see — they report a failure on the step they are executing. The
/// editor's marker does: a spell whose last line has run keeps `Running` until
/// the tick tidies it up, and a sentinel line number put the marker on a line
/// numbered zero, which no file has.
#[must_use]
pub fn line_of(state: &Running) -> Option<u64> {
    super::program::at(walking(state), &state.pc).map(|step| u64::try_from(step.line).unwrap_or(0))
}

/// Hand the current frame back to whoever called it.
///
/// Returns whether there was one. `false` means the outermost frame has run out,
/// which is the only thing that ends a spell.
///
/// The caller resumes **pointing at its own call**, and is stepped past it here
/// rather than on the way in — a `pc` left pointing past the call would be a
/// position the save could not explain, and §8 requires in-flight state to be
/// serialisable at every tick boundary rather than at most of them.
fn returned(world: &mut World, entity: Entity) -> bool {
    // **Scoped, not `drop`ped.** The borrow has to end before `advance_pc` takes
    // the world again, and a `drop` of a `Mut<'_, _>` is a no-op clippy rightly
    // objects to — the block is what actually releases it.
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
/// **Three refusals, all loud.** A part the spell does not define is a name the
/// orb cannot place, and it is said once per line per cast like every other
/// missing name (`Running::said`). A call that hands over the wrong number of
/// names cannot be made at all. A stack past [`MAX_PARTS`] is runaway recursion,
/// and §8 will not have that stop silently.
///
/// Any of the three **steps past**, never halts: §8's taxonomy is titled
/// *"scripts always log and never halt"*, so a call that cannot be made is a
/// line that did nothing and a spell that carries on.
///
/// # The arity check is here as well as in `compile`, and that is not belt and
/// braces
///
/// §8 hot-reloads a spell's text under it. A player who adds a parameter to a
/// definition while the spell runs leaves every call in the file one short, and
/// the compiled tree the runner is walking is the *old* one until the reload
/// lands. Binding what arrived and leaving the rest empty would let the body ask
/// about a name standing for itself, which resolves against the room and does
/// something — quietly, and not what anyone wrote.
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
    // **Resolved in the caller's store, before that store is put away.** An
    // argument is a name; if the caller has bound it, what travels is what it
    // stands for. One level, which is `substituted`'s rule — a value that stood
    // for another value would be a chain nobody wrote.
    // `program::arguments` lowercases every slot as it parses, so an argument is
    // already normalised and a `to_lowercase` here would allocate for nothing —
    // and, worse, tell the next reader that a `Kind::Call` might hold mixed case.
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
            // **Taken, not cloned.** The callee opens with a store of its own,
            // so leaving the caller's behind would be the old shared shape with
            // a copy on the stack that nothing reads.
            vars: std::mem::take(&mut running.vars),
        };
        running.stack.push(descent);
        running.part = Some(name.to_owned());
        running.pc = vec![0];
        running.vars = params.into_iter().zip(handed).collect();
    }
}

/// The block the current frame is walking.
///
/// **Empty when the part it names has gone**, which is a real state rather than
/// a defect: §8 hot-reloads a spell's text under it, so a player may delete a
/// definition while a frame is inside it. An empty block reads as *off the end*
/// at the next step, which pops the frame and carries on after the call — the
/// gentlest true answer available, and the same one an empty part gives.
fn walking(state: &Running) -> &super::Block {
    static NOTHING: super::Block = Vec::new();
    super::program::tree(state.program.body(), state.part.as_deref()).unwrap_or(&NOTHING)
}

/// Whether this casting has already complained about `line`, marking it said.
///
/// See [`Running::said`]. Marking on the way past rather than at the call site
/// keeps the two halves — "have we said it" and "we have now" — from drifting
/// apart, which is what a separate setter invites.
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
/// is borrowed mutably, and both live on the same entity. One clone per
/// instruction of a structure that is tens of steps at most, against threading a
/// borrow through the whole runner.
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
    // **Every guard answered before `Running` is borrowed mutably.** `holds`
    // wants the world and `step_past` wants `&mut Running`, and both live on this
    // entity — so the questions are asked first and the answers carried in.
    //
    // Only the loops the path is *inside* are asked, and all of them in the
    // spell's own room; `guard_answers` explains both.
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
                // one just finished. **No entry means no set**, which is where a
                // group the room does not have ends up — the loop stops rather
                // than walking nothing for ever.
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

/// Answer `condition` in the **spell's own room**, not the player's.
///
/// `watch::holds` finds a place through `Cwd`, so a question answered against
/// wherever the player happens to be standing is a question about the wrong
/// room. `Kind::If` has always swapped; `until` did not, and that made a **bound**
/// spell — which runs while the player is elsewhere by design — see nothing,
/// answer `None`, and stop. `dev_spells.toml`'s `threading` is bound and its
/// bound is `repeat until the stacks is idle`, so the flagship case was the
/// broken one.
///
/// One function rather than the swap written twice, because two expressions of
/// one rule is how they came to disagree in the first place.
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

/// Name what a guard could not place, once per line per cast.
///
/// §8.1's *"the culprit is never anonymous"*. A guard that stops on an
/// unanswerable question is the one failure mode with nothing on screen
/// explaining it — the spell simply does not run — so it is worth more than the
/// `if` path's version of the same sentence, not less.
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
/// **Two shapes because the two loops end for different reasons**, and folding
/// them into one `bool` would put the arithmetic in the wrong place: a
/// `repeat until` ends when the world says so and a `for each` ends when it runs
/// out of members, so the walker owes the first an answer and the second a
/// count. The runner does the comparison, because only it knows which member the
/// loop is on.
#[derive(Debug, Clone, Copy)]
enum Continues {
    /// A `repeat until`: whether the loop may take another turn.
    Guard(bool),
    /// A `for each`: how many members its set has, now.
    Members(u32),
}

/// Whether each guarded loop may take another turn, by path.
///
/// **Three answers folded into two, and the fold is a decision.** `until X`
/// continues while X is *not* satisfied, so `Some(false)` goes round again and
/// `Some(true)` stops. An **unanswerable** question — §8's *Referent missing*,
/// a place the tower no longer has — also stops.
///
/// That is the opposite of what an `if` does with the same answer, where an
/// unreadable question declines to act. The asymmetry is deliberate: declining
/// to act is safe, while a `repeat` that declined to *stop* would run for ever
/// on a question nobody can answer, which is §19's *"a spell that has stopped
/// describing the world it runs in"* left running instead of caught.
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
                    // **Only a loop the path is inside can be unwound**, and a
                    // loop being unwound is always an ancestor of the current
                    // `pc`. Asking the rest would put questions to the world
                    // about loops that are not running — the *"a read nobody
                    // asked for"* this whole function's short-circuit exists to
                    // avoid, arrived at from the other direction.
                    if let Some(condition) = until
                        && pc.starts_with(path)
                    {
                        let more = matches!(super::watch::holds(world, condition).0, Some(false));
                        out.push((path.clone(), Continues::Guard(more)));
                    }
                    walk(world, body, pc, path, out);
                }
                // **The set is measured, not the guard asked.** A `for each` has
                // no question of its own: it goes round while the set has a
                // member left, so what the walker owes the runner is a count.
                // Same short-circuit as above — only a loop the path is inside
                // can be unwound, and measuring the rest would be a read nobody
                // asked for.
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
                    // **Both halves, and the path elements a branch costs.**
                    // `enter_branch` pushes the half *and* the step, so a loop
                    // nested in an `if` is two elements deeper than its index
                    // suggests — matching what `step_past` pops on the way out.
                    for (half, block) in [body, otherwise].into_iter().enumerate() {
                        path.push(half);
                        walk(world, block, pc, path, out);
                        path.pop();
                    }
                }
                // **A definition is not walked into**, and this is the same
                // reason `at` refuses to descend: its loops belong to a frame
                // that is not this one. Walking in would ask the world about a
                // `repeat` inside a part nobody has called — a read nobody
                // asked for, and against a path this frame's `pc` can never hold.
                super::Kind::Part { .. }
                | super::Kind::Call { .. }
                | super::Kind::Command(_)
                | super::Kind::Wait(_)
                | super::Kind::Let { .. } => {}
            }
            path.pop();
        }
    }

    // **The room swapped once around the whole walk**, not per guard: every
    // question below is answered where the *spell* is standing, for the same
    // reason `Kind::If` swaps — `holds` finds a place through `Cwd`, and a bound
    // spell runs while the player is in another room by design.
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
/// **Silent for a held spell, because it has not finished.** `bind::stand` casts
/// it again on the next tick, so *"tending.spell is finished"* would be a
/// sentence contradicted a tick later, once per lap, for as long as it is held —
/// and with the recast itself already silent, a finish with no beginning reads
/// like the orb letting go. A binding ends when the player says `stop`, which
/// says so in those words.
fn finish(world: &mut World, entity: Entity, state: &Running) {
    let name = spell_name(world, state);
    if let Some(mut bound) = world.get_mut::<super::Bound>(entity) {
        // What this lap has already complained about, kept for the next one —
        // see [`Bound::said`]. The component outlives the run; the rationing has
        // to outlive it with the component or it is no rationing at all.
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

    // **Remove the component, never despawn the entity.** `Running` is worn *by
    // the spell node itself*, so despawning would delete the spell from the
    // grimoire the moment it finished — the file gone, and the next `invoke` of
    // it fuzzy-matching to some other spell entirely. Which is exactly what
    // `invoking_a_running_spell_twice_does_not_start_it_twice` caught: the
    // second `invoke slow` reached `first_light`, because `slow.spell` no longer
    // existed to be named.
    world.entity_mut(entity).remove::<Running>();
}

/// What the spell is called, for a record.
///
/// §8.1: *"sabotage log lines name the affected script and its bind time, so
/// when something does break the culprit is never anonymous."* Every line this
/// module emits carries the spell and the line number for that reason.
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
    // **The rail's fault mark is raised here and nowhere else**, because this is
    // the one place that knows a spell failed *and* which room it was working
    // in. §8 makes a broken spell log rather than halt, so without a mark the
    // only way to find one is to go and read its log — and §9's whole argument
    // for the minimised half is noticing without going.
    //
    // `Role::Danger` only: a `Cost` here is a spell politely waiting its turn
    // for the production slot, which happens constantly and is not a fault.
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
