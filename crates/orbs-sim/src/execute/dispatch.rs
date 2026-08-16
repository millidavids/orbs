//! Which verb runs what, and the two records every verb can need.
//!
//! Registration and dispatch only. The bodies live beside their concern —
//! [`pipeline`](super::pipeline) for §10.1's brewing loop,
//! [`navigate`](super::navigate) for §7's places,
//! [`files`](super::files) for §3's log — because this file is the one every
//! phase must edit, and a merge between a brewing change and a log change should
//! not conflict for a reason that is not semantic.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::parser::{Intent, Verb};
use crate::rng::Rngs;
use crate::session::{Pending, Queued, Scrollback, Skip};
use crate::tick::Tick;
use crate::tower;

use super::{files, navigate, pipeline, recall, scribe};

/// The name the scrollback answers to.
///
/// §3: unlogged output is forbidden, so the record stream *is* the log. Giving
/// it a filename is not a debugging affordance — it is the same object the
/// player will `peruse` and pipe once the filesystem exists, reachable early.
pub const LOG: &str = "orb.log";

/// The most ticks one `meditate` may pass.
///
/// An hour of world time. A cap exists because the loop runs inside a single
/// `step()`: at ~microseconds a tick this is imperceptible, and an uncapped
/// count typed by a curious player should not be able to stall a frame.
pub const MAX_MEDITATE: u64 = 3600;

/// Run everything the player queued since the last tick.
///
/// Exclusive: a command reads and writes whatever its domain touches, and
/// enumerating that as system parameters now would be a guess about domains that
/// do not exist yet.
pub fn run_pending(world: &mut World) {
    let queued = world.resource_mut::<Pending>().drain();
    for item in queued {
        match item {
            Queued::Command(intent) => execute(&intent, world),
            // A save is not a command — no verb ran, and no `Intent` describes
            // it — but it lands on the same boundary and in the same order.
            Queued::Write { name, lines } => {
                scribe::write(world, &name, &lines);
            }
            // A tester's door, absent from a release build entirely.
            #[cfg(debug_assertions)]
            Queued::Spawn(order) => super::debug::run(world, &order),
        }
    }
}

/// Run one resolved command, from wherever the caller is standing.
///
/// **The script runner's door into the same dispatch a typed line takes.** §13
/// is explicit that if the live game and the CLI harness diverged *"we would not
/// find out until Phase 9"*, and a script with its own copy of any verb is that
/// divergence with an extra step. It does not go through
/// [`Pending`](crate::session::Pending): that queue is drained by the `commands`
/// schedule which runs **before** the one the runner is in, so a script routed
/// through it would manage exactly one instruction per tick whatever its budget
/// said.
pub fn execute_one(intent: &Intent, world: &mut World) {
    execute(intent, world);
}

fn execute(intent: &Intent, world: &mut World) {
    match intent.verb {
        Verb::Attend => navigate::attend(intent, world),
        Verb::Survey => navigate::survey(intent, world),
        Verb::Meditate => meditate(intent, world),
        Verb::Status => status(world),
        Verb::Unfurl => super::unfurl::unfurl(world),
        Verb::Weave => super::weave::weave(world),
        Verb::Peruse => files::peruse(intent, world),
        Verb::Sift => files::sift(intent, world),
        Verb::Verify => files::verify(intent, world),
        Verb::Research => super::research::research(intent, world),
        Verb::Follow => super::research::follow(intent, world),
        Verb::Wander => super::wander::wander(world),
        Verb::Move => pipeline::carry(intent, world),
        Verb::Wield => pipeline::wield(intent, world),
        // §10.1's per-instrument verbs. One arm, because the instrument is found
        // from the verb rather than named — see `pipeline::operate`.
        Verb::Grind | Verb::Digest | Verb::Mix | Verb::Distil | Verb::Kindle => {
            pipeline::operate(intent, world);
        }
        Verb::Stop => pipeline::stop(intent, world),
        Verb::Empty => pipeline::empty(intent, world),
        Verb::Recall => recall::recall(intent, world),
        Verb::Scribe => scribe::scribe(intent, world),
        Verb::Invoke => tower::spell::invoke(intent, world),
        Verb::Bind => tower::spell::bind(intent, world),
        Verb::Purge => pipeline::purge(intent, world),
        _ => acknowledge(intent.verb, world),
    }
}

/// Whether the orb does this verb's work yet, or only says it heard.
///
/// Lives beside `execute` rather than on [`Verb`] because this *is* the dispatch
/// in that function, read as data — the parser knows the whole §6.1 vocabulary
/// and should keep knowing it, while what the world can currently act on is a
/// fact about this module and changes as phases land.
///
/// Written out rather than probed, because a match arm is not data. The
/// disagreement it invites is caught by
/// `a_dark_verb_only_acknowledges_and_a_live_one_does_not`, which drives all
/// sixteen through a real [`Sim`](crate::Sim) and fails the moment the two drift
/// apart.
///
/// # Why anything reads this
///
/// §15's scaffold tutorial names the vocabulary so the gate measures the parser
/// rather than a tester's guesswork. Naming a verb that only acknowledges would
/// walk that tester straight into a **dead end** — the metric §15 calls more
/// important than the raw resolution rate — so the report names what works, and
/// this is where "works" is written down.
///
/// One of the six was worse than a dead end. With no scripts in scope `bind`'s
/// only slot was unfillable, and the documented `find`/`bind` collision in the
/// parser's vocabulary table then handed the line to `sift` unopposed: `bind
/// night_watch` searched the session log and reported success. That is fixed by
/// there being scripts to name, not by this list.
///
/// # Live is not the same as available
///
/// [`is_gated`] is the second question, and `bind` is the first verb to need it:
/// it works, and it refuses until the tower has earned somewhere to put a spell.
/// The scaffold asks both, because a word that can only refuse is the dead end
/// this list exists to keep off it.
#[must_use]
pub const fn is_live(verb: Verb) -> bool {
    matches!(
        verb,
        Verb::Attend
            | Verb::Survey
            | Verb::Meditate
            | Verb::Status
            | Verb::Peruse
            | Verb::Sift
            | Verb::Move
            | Verb::Wield
            | Verb::Grind
            | Verb::Digest
            | Verb::Mix
            | Verb::Distil
            | Verb::Kindle
            | Verb::Stop
            | Verb::Empty
            | Verb::Recall
            | Verb::Scribe
            | Verb::Invoke
            // **Live, and gated.** Working and being *available* are different
            // questions now: `bind` does everything it will ever do, and refuses
            // until the tower has earned the concentration to hold a spell. See
            // [`is_gated`], which is what keeps it off the boot report until it
            // can do something.
            | Verb::Bind
            | Verb::Research
            | Verb::Purge
            | Verb::Verify
            // **The one word whose whole reason for existing is being found.**
            // A verb that makes long output readable, left off the list a cold
            // launch teaches from, would be exactly the affordance-nobody-can-
            // discover problem it was added to solve — one level up.
            | Verb::Unfurl
            // **Live and never gated**, unlike `bind` below. At experience 0 it
            // shows the first threshold named and nothing taken, which is the
            // onboarding value rather than a dead end — a new player learns what
            // the work is *for*. There is no total at which it can only refuse.
            | Verb::Weave
            | Verb::Follow
            // Live, and it refuses in two states rather than being gated by
            // one: there are no stacks here, or none open yet. Both
            // name the way forward, so neither is the dead end this list
            // exists to keep off the scaffold.
            | Verb::Wander
    )
}

/// Every verb worth offering where the player is standing.
///
/// **One filter, shared.** The boot report and `recall`'s overview list the same
/// thing for the same reasons, and two copies of *live, ungated, in scope* is two
/// chances for the tutorial a player reads at launch to disagree with the manual
/// they ask for a minute later.
///
/// Three exclusions, each of them the same rule one step further in — a word
/// that can only refuse is worse than a word that is absent:
///
/// - not [`is_live`]: nobody has built it, so it acknowledges and does nothing.
/// - [`is_gated`]: it works and the tower has not earned it, like `bind` at
///   concentration 0.
/// - not offered by the [`Scene`](crate::parser::Scene): a per-instrument verb
///   whose instrument is elsewhere. At the tower root this is exactly the
///   `!is_operation()` the boot report used to hardcode, because an empty scene
///   offers no operations — so generalising it changed no list.
#[must_use]
pub fn offered(world: &World) -> Vec<Verb> {
    let scene = world.resource::<crate::parser::Scene>();
    Verb::ALL
        .into_iter()
        .filter(|verb| is_live(*verb) && !is_gated(*verb, world) && scene.offers(*verb))
        .collect()
}

/// Whether `verb` works but is not available *yet*.
///
/// The companion to [`is_live`], and the difference between *"nobody built
/// this"* and *"you have not earned it"*. Both keep a word off §15's scaffold
/// list and for the same reason — a tutorial that names a verb which can only
/// refuse spends the gate's most important metric, the dead-end rate, on
/// something the player cannot act on.
///
/// **A question about the world, so not `const`.** The gate moves: `bind` is
/// unavailable at concentration 0 and available for ever after, and the boot
/// report is written before a player has earned anything.
#[must_use]
pub fn is_gated(verb: Verb, world: &World) -> bool {
    match verb {
        // §11.5's turn: the tower is worked entirely by hand until the orb has
        // somewhere to put a spell.
        Verb::Bind => tower::concentration(world) == 0,
        _ => false,
    }
}

/// Let time pass.
fn meditate(intent: &Intent, world: &mut World) {
    let count = intent
        .arguments
        .first()
        .and_then(|argument| argument.value.parse::<u64>().ok())
        .unwrap_or(1)
        .min(MAX_MEDITATE);

    world.resource_mut::<Skip>().request(count);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Meditate.canonical())
        .count(FieldName::Remaining, count)
        .finish();
}

/// Report what the world currently is.
///
/// Every value here belongs to a subsystem that until now had no way of being
/// seen from the game: the tick and seed are the determinism spine, and drawing
/// them as a table is the record model doing its job in the binary rather than
/// in an example.
fn status(world: &mut World) {
    let tick = world.resource::<Tick>().get();
    let seed = world.resource::<Rngs>().master_seed();
    let logged = world.resource::<Scrollback>().records().len();
    let queued = world.resource::<Pending>().len();
    // §11.5's two progression numbers, and the only place they are *both*
    // readable. `concentration` is derived from `experience`, so the pair is one
    // fact twice — which is the point of showing them together: a player looking
    // at 12 wants to know what 16 buys.
    let earned = world.resource::<tower::Experience>().get();
    let held = tower::concentration(world);

    let mut scrollback = world.resource_mut::<Scrollback>();
    let rows = scrollback.records_mut();
    for (name, value) in [
        ("tick", tick),
        ("seed", seed),
        ("experience", earned),
        ("concentration", quantity(held)),
        ("logged", quantity(logged)),
        ("queued", quantity(queued)),
    ] {
        rows.push(RecordKind::Status)
            .text(FieldName::Name, name)
            .count(FieldName::Quantity, value)
            .finish();
    }
}

/// A count as a record value, saturating rather than wrapping.
fn quantity(count: usize) -> u64 {
    u64::try_from(count).unwrap_or(u64::MAX)
}

/// The parser named something that has since stopped existing.
///
/// §8's failure taxonomy calls this a missing referent, and it is reachable
/// because resolution happens on submit while execution happens on the next
/// tick — enough of a gap for one command to remove what the next one names.
pub(super) fn missing(verb: Verb, target: &str, world: &mut World) {
    // The fields carry the facts (rule 4); the sentence is authored (rule 6).
    // Without it this drew as two bare values — `mix sage` — which names what
    // was wanted and never says it was a refusal. That is the same defect
    // `work_busy` was written to fix, on the commonest refusal in the game.
    let message = world
        .resource::<crate::content::Prose>()
        .line("missing_target", &[("path", target)]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, verb.canonical())
        .text(FieldName::Path, target)
        .text(FieldName::Message, &message)
        .role(Role::Danger)
        .finish();
}

/// The orb understood, and has nothing to do about it yet.
pub(super) fn acknowledge(verb: Verb, world: &mut World) {
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, verb.canonical())
        .role(Role::Success)
        .finish();
}
