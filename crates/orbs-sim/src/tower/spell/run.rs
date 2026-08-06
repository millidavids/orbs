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
pub const SCRIPT_BUDGET: usize = 1;

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

/// How deep `invoke` may nest.
///
/// §8 fixes this at 3 and argues why the budget alone is not a sufficient
/// recursion guard: exhausting it makes every subsequent instruction *Budget
/// starved*, which logs at high verbosity only, so all automation would stop
/// **silently**. Depth-limiting makes runaway recursion loud and diagnosable.
pub const MAX_DEPTH: u8 = 3;

/// How deep the runner currently is, while it is running something.
///
/// Set around one instruction and cleared after, exactly as the script's
/// position is. `invoke` reads it to know whether *it* is
/// being called by a spell or typed by a player, which the intent alone cannot
/// say: both arrive through the same dispatch, which is the point (§13).
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct Depth(pub Option<u8>);

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

/// Credit everything pushed from now on to `spell`, or to nobody.
fn set_attribution(world: &mut World, spell: Option<&str>) {
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .attribute(spell);
}

/// Run up to [`SCRIPT_BUDGET`] instructions of the spell on `entity`.
fn step_one(world: &mut World, entity: Entity) {
    for _ in 0..SCRIPT_BUDGET {
        let Some(state) = world.get::<Running>(entity).cloned() else {
            return;
        };

        let Some(step) = super::program::at(&state.program.body, &state.pc).cloned() else {
            finish(world, entity, &state);
            return;
        };

        // Entering a block **spends a budget step**, which looks wasteful and is
        // the guard: a `repeat` whose body never spends any — an empty one, or
        // one whose every step is a `wait` that is already satisfied — would
        // otherwise be an unbounded loop inside a single tick, and the game
        // would stop. The budget is the only thing standing between a player's
        // typo and a hang.
        if let super::Kind::Repeat { times, body } = &step.kind {
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
            let answer = node_of(world, state.at).and_then(|at| {
                world.insert_resource(Cwd(at));
                let answer = condition
                    .as_ref()
                    .and_then(|condition| super::watch::holds(world, condition));
                world.insert_resource(Cwd(player));
                answer
            });

            // **`None` is a third answer, and it has to be said out loud.** The
            // question named a place the tower does not have — §8's *Referent
            // missing*, not the answer being no. Reported per evaluation, like
            // every other line-level failure: §8's taxonomy is titled *"scripts
            // always log and never halt"*, and this one used to do neither.
            //
            // A condition that could not be *read* is a different fault with its
            // own line, said once when the spell was cast, so it is not repeated.
            if answer.is_none()
                && let Some(condition) = condition.as_ref()
            {
                let missing = condition.place().to_owned();
                say_failure(world, &state, "spell_nowhere", &missing, Role::Danger);
            }

            let holds = answer.unwrap_or(false);
            let half = if holds { body } else { otherwise };
            if half.is_empty() {
                advance_pc(world, entity);
            } else if let Some(mut running) = world.get_mut::<Running>(entity) {
                let Running { pc, loops, .. } = &mut *running;
                super::program::enter_branch(pc, loops, holds);
            }
            continue;
        }

        // A `wait` reads the world rather than acting on it, so it costs no
        // position swap and no dispatch.
        if let super::Kind::Wait(wanted) = &step.kind {
            if wait_for(world, entity, &state, wanted) == Progress::Blocked {
                return;
            }
            continue;
        }

        let super::Kind::Command(line) = step.kind else {
            continue;
        };

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
        world.insert_resource(Depth(Some(state.depth)));

        let outcome = run_line(world, entity, &state, &line);

        // The player's position, back. **Nothing is read out of the swap** — the
        // spell's domain is fixed, so where the instruction left `Cwd` is not a
        // fact worth keeping. It was, while `attend` walked.
        world.insert_resource(Cwd(player));
        world.insert_resource(Depth(None));

        if outcome == Progress::Blocked {
            return;
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
const fn may_issue(verb: Verb) -> bool {
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
    super::program::at(&state.program.body, &state.pc)
        .map(|step| u64::try_from(step.line).unwrap_or(0))
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
    let Some(program) = world
        .get::<Running>(entity)
        .map(|state| state.program.clone())
    else {
        return;
    };
    if let Some(mut running) = world.get_mut::<Running>(entity) {
        let Running { pc, loops, .. } = &mut *running;
        if !super::program::step_past(&program.body, pc, loops) {
            // Off the end. `at` will return `None` next time round and the
            // spell finishes there, so there is one place that ends a spell.
            pc.clear();
        }
    }
}

/// The spell has run out of lines.
fn finish(world: &mut World, entity: Entity, state: &Running) {
    let name = spell_name(world, state);
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
