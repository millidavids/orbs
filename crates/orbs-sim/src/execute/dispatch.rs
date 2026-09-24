//! Which verb runs what, and the two records every verb can need.
//!
//! Registration and dispatch only. The bodies live beside their concern —
//! [`pipeline`] for §10.1's brewing loop, [`navigate`] for §7's places,
//! [`files`] for §3's log — because this is the file every phase must edit.

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
/// §3: unlogged output is forbidden, so the record stream *is* the log. The
/// filename is not a debugging affordance — it is the same object the player
/// will `peruse` and pipe.
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
/// enumerating that as system parameters would be a guess about domains that do
/// not exist yet.
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
/// The script runner's door into the same dispatch a typed line takes: a
/// script with its own copy of any verb is §13's live-game/harness divergence.
/// Not through [`Pending`], which the `commands` schedule drains *before* the
/// one the runner is in — a script routed that way would manage one instruction
/// per tick whatever its budget said.
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
        Verb::Status => status(world),
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
        Verb::Summon => super::summon::summon(world, divined),
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
/// Beside `execute` rather than on [`Verb`] because this *is* that function's
/// dispatch read as data: the parser knows the whole §6.1 vocabulary, while
/// what the world can act on is a fact about this module. Written out rather
/// than probed, because a match arm is not data;
/// `a_dark_verb_only_acknowledges_and_a_live_one_does_not` catches the drift.
///
/// §15's scaffold tutorial reads it: naming a verb that only acknowledges walks
/// a tester into a dead end.
///
/// Live is not the same as available — [`is_gated`] is the second question.
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
/// One filter, shared: the boot report and `recall`'s overview list the same
/// thing, and two copies of *live, ungated, in scope* is two chances for them
/// to disagree.
///
/// Three exclusions, because a word that can only refuse is worse than a word
/// that is absent:
///
/// - not [`is_live`]: nobody has built it, so it acknowledges and does nothing.
/// - [`is_gated`]: it works and the tower has not earned it, like `bind` at
///   concentration 0.
/// - not offered by the [`Scene`](crate::parser::Scene): a per-instrument verb
///   whose instrument is elsewhere. At the tower root this is the
///   `!is_operation()` the boot report used to hardcode, so generalising it
///   changed no list.
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
/// Two filters, both the spell's rather than the player's. [`offered`] reads
/// the scene of the room the player stands in; a spell is written *for* a
/// domain and runs there however far away it is edited from, so the scene is
/// built for that domain. The second is `may_issue` — nine verbs are refused
/// inside a spell, and a listing offering them is worse than a short one.
///
/// An unknown domain answers with the verbs that need no fixture: those work
/// anywhere, and inventing a room's vocabulary would be a guide making things
/// up.
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
/// [`expect`](crate::parser::expect) against the scene of the domain the
/// *file* belongs to, not the room the player is standing in — a laboratory
/// spell offers `grind` however far away it is being edited from.
///
/// The scene is built here rather than handed out, because `scene_at` rebuilds
/// every recipe, topic and node and is not something a painter should reach
/// for. An unknown domain answers against an empty scene, which offers the
/// language's own words and no verbs.
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
/// [`is_live`]'s companion: the difference between *"nobody built this"* and
/// *"you have not earned it"*. Both keep a word off §15's scaffold list.
///
/// A question about the world, so not `const`: the gate moves, and the boot
/// report is written before a player has earned anything.
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

/// Report what the world currently is.
///
/// Every value here belongs to a subsystem that otherwise has no way of being
/// seen from the game — the tick and seed are the determinism spine.
fn status(world: &mut World) {
    let tick = world.resource::<Tick>().get();
    let seed = world.resource::<Rngs>().master_seed();
    let logged = world.resource::<Scrollback>().records().len();
    let queued = world.resource::<Pending>().len();
    // §11.5's two progression numbers, and the only place both are readable.
    // `concentration` is derived from `experience`: a player looking at 12
    // wants to know what 16 buys.
    let earned = world.resource::<tower::Experience>().get();
    let held = tower::concentration(world);
    // §11.5's second number, and the only one that can fall. One buys
    // capability and stays bought; the other is standing and can be lost.
    let known = world.resource::<tower::Renown>().get();
    // §11.5's mana, and its ceiling — which the Ley Line's `pool` and `floor`
    // raise. Nothing else on screen said the pool's size outside a siege board.
    let pool = u64::from(world.resource::<tower::Quintessence>().get());
    let ceiling = u64::from(tower::ceiling(world));

    // Read before the scrollback is borrowed, and one walk of `Running` rather
    // than a second — `tower::running_spells` is what the rail folds.
    let casting = tower::running_spells(world);

    let mut scrollback = world.resource_mut::<Scrollback>();
    let rows = scrollback.records_mut();
    for (name, value) in [
        ("tick", tick),
        ("seed", seed),
        ("experience", earned),
        ("renown", known),
        ("concentration", quantity(held)),
        ("quintessence", pool),
        ("ceiling", ceiling),
        ("logged", quantity(logged)),
        ("queued", quantity(queued)),
    ] {
        rows.push(RecordKind::Status)
            .text(FieldName::Name, name)
            .count(FieldName::Quantity, value)
            .finish();
    }

    // The full answer behind the rail's `+n`, which until now pointed at
    // nothing. The rail is the glance and this is the answer.
    //
    // A section, not more `Status` rows — the reading column above is guarded
    // by a *shape* test (two or more records, each a name and a numeric
    // quantity), so a row carrying a spell's room would break its alignment.
    // Absent when nothing runs, because a section that is usually a bare rule
    // teaches the eye to skip it.
    if casting.is_empty() {
        return;
    }
    rows.push(RecordKind::Section)
        .text(FieldName::Kind, CASTING)
        .finish();
    for one in casting {
        // `Detail` is what makes this a described listing rather than a tiled
        // one (`record/view.rs` decides on the field's presence): tiled, a run
        // of names reads `tending  threading` with nothing saying where either
        // is.
        let where_it_is = if one.cursors > 1 {
            format!("{}, on {} cursors", one.domain, one.cursors)
        } else {
            one.domain.clone()
        };
        rows.push(RecordKind::Entry)
            .text(FieldName::Name, &one.spell)
            .text(FieldName::Detail, &where_it_is)
            .finish();
    }
}

/// The heading `status` puts over what is running.
///
/// A table entry beside `Verb::canonical`, not authored prose — it is the same
/// class of thing as a noun-kind label, which is what every other section
/// heading in a listing is.
const CASTING: &str = "casting";

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
