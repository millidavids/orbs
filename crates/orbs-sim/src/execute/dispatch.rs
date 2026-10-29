//! Which verb runs what, and the two records every verb can need.
//!
//! Registration and dispatch only. The bodies live beside their concern —
//! [`pipeline`] for §10.1's brewing loop, [`navigate`] for §7's places,
//! [`files`] for §3's log — because this is the file every phase must edit.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::parser::{Intent, Verb};
use crate::session::{Pending, Queued, Scrollback, Skip};
use crate::tower;

use super::{files, navigate, pipeline, recall, scribe};

/// The name the scrollback answers to.
///
/// The record stream *is* the log (§3), and this is the file the player
/// `peruse`s.
pub const LOG: &str = "orb.log";

/// The most ticks one `meditate` may pass.
///
/// An hour of world time, capped because the loop runs inside one `step()`.
pub const MAX_MEDITATE: u64 = 3600;

/// Run everything the player queued since the last tick.
///
/// Exclusive, since a command touches whatever its domain does.
pub fn run_pending(world: &mut World) {
    let queued = world.resource_mut::<Pending>().drain();
    for item in queued {
        match item {
            Queued::Command(intent) => execute(&intent, world, false),
            Queued::Divined(intent) => execute(&intent, world, true),
            // A save is not a command — no verb ran, and no `Intent` describes
            // it — but it lands on the same boundary and in the same order.
            Queued::Write {
                name,
                lines,
                read,
                by,
            } => {
                scribe::write(world, &name, &lines, &read, by);
            }
            Queued::Take(id) => super::weave::grant(world, &id),
            // A tester's door, absent from a release build entirely.
            #[cfg(debug_assertions)]
            Queued::Spawn(order) => super::debug::run(world, &order),
        }
    }
}

/// Run one resolved command, from wherever the caller is standing.
///
/// The script runner's door into the same dispatch a typed line takes (§13).
/// Not through [`Pending`], drained before the runner's schedule, which would
/// allow one instruction a tick whatever the budget.
pub fn execute_one(intent: &Intent, world: &mut World) {
    execute(intent, world, false);
}

/// Run one command. `divined` is whether the augury read it rather than the orb
/// (§6); a spell's line never is, because the player saw its reading when it
/// was written.
fn execute(intent: &Intent, world: &mut World, divined: bool) {
    // Any other word answers *no* to a pending `quit`: the question lasts one
    // line, so a `quit` thought better of cannot end a session three commands
    // later. `quit` itself is the *yes* — see `quit::Quitting`.
    if intent.verb != Verb::Quit {
        world.resource_mut::<super::quit::Quitting>().never_mind();
    }
    match intent.verb {
        Verb::Attend => navigate::attend(intent, world),
        Verb::Survey => navigate::survey(intent, world),
        Verb::Meditate => meditate(intent, world),
        Verb::Status => super::status::status(intent, world),
        Verb::Unfurl => super::unfurl::unfurl(world),
        Verb::Quit => super::quit::quit(world),
        Verb::Menu => super::quit::menu(world),
        Verb::Weave => super::weave::weave(world),
        Verb::Peruse => files::peruse(intent, world),
        Verb::Sift => files::sift(intent, world),
        Verb::Verify => files::verify(intent, world),
        Verb::Research => super::research::research(intent, world),
        Verb::Follow => super::research::follow(intent, world),
        Verb::Wander => super::wander::wander(world),
        Verb::Probe => super::scry::probe(world),
        Verb::Dial => super::scry::dial(intent, world),
        Verb::Muster => super::muster::muster(world),
        Verb::Haul => super::muster::haul(intent, world),
        Verb::Summon => super::summon::summon(intent, world, divined),
        Verb::Limn => super::summon::limn(intent, world),
        Verb::Queue => super::queue::queue(intent, world),
        Verb::Defend => super::defend::defend(world),
        Verb::Deploy => super::defend::deploy(intent, world),
        Verb::Quaff => super::defend::quaff(intent, world),
        Verb::Hold => super::defend::hold(world),
        Verb::Pledge => super::defend::pledge(intent, world),
        Verb::Petition => super::defend::petition(world),
        Verb::Imbue => super::imbue::imbue(world, intent),
        Verb::Snap => super::imbue::snap(world, intent),
        Verb::Anneal => super::imbue::anneal(world),
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
/// `execute`'s dispatch read as data, so it lives here, not on [`Verb`];
/// `a_dark_verb_only_acknowledges_and_a_live_one_does_not` catches drift.
/// Live is not available — [`is_gated`] is the second question.
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
            // Live, and gated: `bind` does everything it will ever do, and
            // refuses until the tower has earned the concentration to hold a
            // spell. [`is_gated`] keeps it off the boot report until then.
            | Verb::Bind
            | Verb::Research
            | Verb::Purge
            | Verb::Verify
            // A verb that makes long output readable, left off the list a cold
            // launch teaches from, is itself the undiscoverable affordance it
            // was added to solve.
            | Verb::Unfurl
            // The way out has to be offered everywhere: a player who cannot
            // find how to stop is stuck in the game rather than in a room.
            | Verb::Quit
            // Beside `quit`, and not the same word: the menu is where a game is
            // chosen, so a player who cannot reach it is stuck with the one
            // they are in.
            | Verb::Menu
            // Live and never gated, unlike `bind`: at experience 0 it
            // shows the first threshold named and nothing taken, which teaches
            // a new player what the work is *for*.
            | Verb::Weave
            | Verb::Follow
            // Live, and it refuses in two states rather than being gated by
            // one: no stacks here, or none open yet. Both name the way forward.
            | Verb::Wander
            // The lens's two. Each refuses in voice where it cannot work and
            // names the way forward, which is this list's test rather than
            // "does it always succeed".
            | Verb::Probe
            | Verb::Dial
            // The sanctum's two, on the same reading. `haul`'s interesting
            // refusal — a greater ward onto a lesser — *is* the puzzle.
            | Verb::Muster
            | Verb::Haul
            // The menagerie's two, on the same reading. A balk is not a
            // refusal — it is the circle answering wrongly, which is the
            // puzzle, exactly as a refused haul is.
            | Verb::Summon
            | Verb::Limn
            // The bailey's five, on the same reading. A round that goes badly
            // is not a refusal — it is the siege going badly.
            | Verb::Defend
            | Verb::Deploy
            | Verb::Quaff
            | Verb::Hold
            | Verb::Pledge
            // ...and `petition`. Being unable to afford it is a price, not a
            // dead end, and the sentence names both numbers.
            | Verb::Petition
            // The forge's three. A lattice that does not light is the puzzle
            // going badly, not a refusal.
            | Verb::Imbue
            | Verb::Snap
            | Verb::Anneal
            // The satchel's push. Full is a producer that has outrun its
            // consumer, which is the pipeline saying something true about
            // itself.
            | Verb::Queue
    )
}

/// Every verb worth offering where the player is standing.
///
/// One filter for the boot report and `recall`'s overview: [`is_live`], not
/// [`is_gated`], and offered by the [`Scene`](crate::parser::Scene). A word
/// that can only refuse is worse than one that is absent.
#[must_use]
pub fn offered(world: &World) -> Vec<Verb> {
    let scene = world.resource::<crate::parser::Scene>();
    Verb::ALL
        .into_iter()
        .filter(|verb| is_live(*verb) && !is_gated(*verb, world) && scene.offers(*verb))
        .collect()
}

/// The verbs a spell written for `domain` may actually use.
///
/// The scene is built for `domain`, not the player's room, since a spell runs
/// where it is written for; and `may_issue` filters what a spell may say. An
/// unknown domain answers with the verbs that need no fixture.
#[must_use]
pub fn spell_vocabulary(world: &World, domain: &str) -> Vec<Verb> {
    let scene = super::navigate::find_domain(world, domain)
        .map(|at| crate::tower::scene_at(world, at))
        .unwrap_or_default();
    Verb::ALL
        .into_iter()
        .filter(|verb| is_live(*verb) && crate::tower::spell::may_issue(*verb))
        .filter(|verb| scene.offers(*verb))
        .collect()
}

/// What may come next in a line of a spell written for `domain`.
///
/// [`expect`](crate::parser::expect) against the file's domain, not the
/// player's room. Built here, since `scene_at` is no painter's to call; an
/// unknown domain answers against an empty scene.
#[must_use]
pub fn spell_expect(
    world: &World,
    domain: &str,
    line: &str,
    caret: usize,
    open: &[crate::parser::SpellWord],
) -> crate::parser::Expectation {
    let at = super::navigate::find_domain(world, domain);
    let scene = at
        .map(|node| crate::tower::scene_at(world, node))
        .unwrap_or_default();
    // The same `groups_at` `recall scripting` prints, so the guide and the
    // manual name the same sets. From the scene it would offer the room's
    // contents, which is a different question.
    let sets = at.map(|node| crate::tower::groups_at(world, node));
    crate::parser::expect(
        line,
        caret,
        &crate::parser::Situation {
            scene: &scene,
            spell: true,
            prompt_open: false,
            open,
            sets: sets.as_deref().unwrap_or_default(),
        },
    )
}

/// Whether `verb` works but is not available *yet*.
///
/// [`is_live`]'s companion: *you have not earned it*, where that is *nobody
/// built this*. A question about the world, so not `const`.
#[must_use]
pub fn is_gated(verb: Verb, world: &World) -> bool {
    match verb {
        // §11.5's turn: the tower is worked entirely by hand until the orb has
        // somewhere to put a spell.
        Verb::Bind => tower::concentration(world) == 0,
        // §8's channel is bought at the loom (`satchel_1`), so it is off every
        // listing until then, exactly as `bind` is.
        Verb::Queue => !tower::holds(world, tower::Grant::Satchel),
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

/// The parser named something that has since stopped existing.
///
/// §8's missing referent, reachable because resolution is on submit and
/// execution on the next tick.
pub(super) fn missing(verb: Verb, target: &str, world: &mut World) {
    // The fields carry the facts (rule 4); the sentence is authored (rule 6).
    // Without it this drew as two bare values — `mix sage` — which never says
    // it was a refusal.
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
