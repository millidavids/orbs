//! `probe` and `dial` — the lens's two words (DESIGN.md §10).
//!
//! The archive's shape, one room over: `probe` opens a reading as `research`
//! opens a maze, and a spell's `if` resolves against the readings at cast
//! because [`tower::ward::readings`] is registered unconditionally.
//!
//! Still named `scry` after the domain (§10) though the word lost the naming
//! sweep — `scr` reaches `scribe`. `probe` opens and presses in one word.
//!
//! Like `research`, `probe` takes no production slot — `ward::PRESS_TICKS` is
//! nought, so a press answers on the tick it is typed and a bound solver runs
//! beside a full brewing loop. It was twelve ticks and that scarcity is
//! withdrawn (§19).

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use super::readings::{clear, fixture, reading, room_of};
use crate::content::Prose;
use crate::parser::{Intent, Verb};
use crate::rng::{RngStream, Rngs};
use crate::session::Scrollback;
use crate::tower::{
    self,
    ward::{self, Ward},
};

/// `dial <socket> <sigil>` — turn one dial of the aperture.
///
/// Free and instant. §10.1 prices the run and here the run is the press, so a
/// player can change three sockets and press once.
pub(super) fn dial(intent: &Intent, world: &mut World) {
    let Some(prism) = fixture(world, Verb::Probe) else {
        say(world, Verb::Dial, "scry_nowhere", &[], Role::Cost);
        return;
    };
    if world.get::<Ward>(prism).is_none() {
        say(world, Verb::Dial, "dial_unopened", &[], Role::Cost);
        return;
    }

    let mut named = intent.arguments.iter().map(|argument| &argument.value);
    let Some(socket) = named.next() else {
        say(world, Verb::Dial, "dial_incomplete", &[], Role::Cost);
        return;
    };
    // Bare means *try something else here*: a spell has no variables, so it
    // cannot name the sigil. See `SOCKET_AND_SIGIL` and `Ward::advance`.
    let Some(sigil) = named.next() else {
        advance(world, prism, crate::parser::leaf(socket));
        return;
    };
    // Both slots are `NounKind::Place`, so `seat laboratory nitre` parses and
    // has to be refused here by name — a player who swapped the two words is
    // one word from right.
    //
    // The leaf, for every sentence below: a resolved place arrives as its full
    // path, and prose echoing it puts the tower's internals in a player's line.
    let socket = crate::parser::leaf(socket).to_owned();
    let sigil = crate::parser::leaf(sigil).to_owned();
    let (socket, sigil) = (socket.as_str(), sigil.as_str());

    let (Some(socket_at), Some(sigil_at)) = (ward::socket_of(socket), ward::sigil_of(sigil)) else {
        let key = if ward::socket_of(socket).is_none() {
            "dial_not_a_socket"
        } else {
            "dial_not_a_sigil"
        };
        say(
            world,
            Verb::Dial,
            key,
            &[("name", socket), ("detail", sigil)],
            Role::Cost,
        );
        return;
    };

    let moved = world
        .get_mut::<Ward>(prism)
        .is_some_and(|mut ward| ward.seat(socket_at, sigil_at));

    // Published either way, because `seat` marks either way: a dial bumps the
    // socket's tally whether or not anything moved, so returning early left a
    // spell reading the pre-dial value.
    publish(world, prism);

    if !moved {
        // Either it is already there or the socket has settled. Both are states
        // the player can see, and both are refusals rather than faults.
        say(
            world,
            Verb::Dial,
            "dial_held",
            &[("name", socket)],
            Role::Cost,
        );
        return;
    }
    say(
        world,
        Verb::Dial,
        "dial_done",
        &[("name", socket), ("detail", sigil)],
        Role::Success,
    );
}

/// `dial <socket>` with no sigil — turn it one step round the six.
///
/// What makes a solver writable: §8's language has no variables, so a spell
/// cannot name the sigil a socket has not tried. The success line names what
/// the ward chose, so a player watching can see what was tried.
fn advance(world: &mut World, prism: Entity, socket: &str) {
    let Some(at) = ward::socket_of(socket) else {
        say(
            world,
            Verb::Dial,
            "dial_not_a_socket",
            &[("name", socket), ("detail", "")],
            Role::Cost,
        );
        return;
    };

    // A cyclic step always moves, so the only thing to say is which sigil it
    // moved to. The three-way match went with the per-socket candidate list,
    // and `dial_spent` has no reachable path left.
    let moved = world
        .get_mut::<Ward>(prism)
        .is_some_and(|mut ward| ward.advance(at));
    let sigil = world
        .get::<Ward>(prism)
        .map_or(0, |ward| ward.aperture_at(at));
    publish(world, prism);

    if moved {
        say(
            world,
            Verb::Dial,
            "dial_done",
            &[("name", socket), ("detail", ward::SIGILS[sigil])],
            Role::Success,
        );
    } else {
        say(
            world,
            Verb::Dial,
            "dial_held",
            &[("name", socket)],
            Role::Cost,
        );
    }
}

/// `probe` — press the aperture against the ward, opening a reading if none is.
///
/// Two acts in one word, §19's per-instrument idiom: finding a far orb and
/// pressing it. The aperture opens on a fixed figure, so the first press means
/// the same thing every time and a solver spell has a one-word body.
///
/// Instant, and it takes no slot: a press answers on the tick it is typed, like
/// `dial`. The answer still goes through [`land`]; what is gone is the wait in
/// front of it, not the step.
pub(super) fn probe(world: &mut World) {
    let Some(prism) = fixture(world, Verb::Probe) else {
        say(world, Verb::Probe, "scry_nowhere", &[], Role::Cost);
        return;
    };

    if world.get::<Ward>(prism).is_none() {
        let ward = {
            let mut rngs = world.resource_mut::<Rngs>();
            Ward::new(&mut rngs)
        };
        world.entity_mut(prism).insert(ward);
        refresh(world);
        say(world, Verb::Probe, "scry_opens", &[], Role::Success);
    }

    // Applied here rather than routed through `work::land`: scheduling zero
    // ticks would still take the production slot for a tick.
    land(world, prism);
}

/// A press has finished: ask the ward, publish, and see whether the seal broke.
///
/// Called from `work::land`, which is where every completed run lands — a
/// separate per-tick system would be a second clock over the same event.
pub fn land(world: &mut World, prism: Entity) {
    let Some(mut ward) = world.get_mut::<Ward>(prism) else {
        return;
    };
    ward.press();
    let (aligned, astray) = ward.last();
    let broken = ward.broken();
    let spent = ward.spent();
    let shift = ward.shift().map_or("level", ward::Shift::word);

    // The prism we were handed, never the one under `Cwd` — see `publish`.
    publish(world, prism);

    if !broken {
        let message = world.resource::<Prose>().line(
            "probe_answered",
            &[
                ("quantity", &aligned.to_string()),
                ("detail", &astray.to_string()),
                ("state", shift),
            ],
        );
        // No `Detail`: the astray count is already in the sentence, and
        // carrying it twice read as `the ward is level 2`.
        answered(world, &message, u64::from(aligned), Some(shift), None);
        return;
    }

    // The seal is open, so the ward goes and the readings with it: leaving them
    // would keep stale readings on the sockets for ever.
    world.entity_mut(prism).remove::<Ward>();
    publish(world, prism);

    let earned = yield_of(world, spent);
    let message = world
        .resource::<Prose>()
        .line("probe_broken", &[("quantity", &spent.to_string())]);
    answered(world, &message, earned, None, None);
    // After the sentence about the work, for the reason `transmute` gives: the
    // seal opens, *and then* the orb can hold another spell.
    tower::done(world, &tower::Work::at(PRISM), earned);
    spill(world);
}

/// How many lines of the far wizard's log a broken seal gives up.
const SPILL: usize = 12;

/// The far orb's log, spilling into yours. Flavour, not required reading.
///
/// Quiet, so the dozen lines go to `lens.log` and not the transcript: twelve
/// lines per solve would push the player's own last command off screen (§19).
/// The transcript gets one sentence saying how many there were.
///
/// The shapes are authored (`spill_*` in `prose.toml`, rule 6) and the nouns
/// come from `Recipes`, so a line never names an operation an instrument cannot
/// perform — `digest sage` would teach a player something false about their own
/// tower.
///
/// Now and then it is a recipe, rolled in `tower::learned`: the log spells it
/// out and the orb writes it down, so the sentence about it is drawn while the
/// working is not.
fn spill(world: &mut World) {
    for line in stolen(world, SPILL) {
        // Quiet, but filed: `Records::drawn` skips it and `lens.log` keeps it,
        // which is what `Source` makes true (§19).
        world
            .resource_mut::<Scrollback>()
            .records_mut()
            // `Message`, not `Entry`: an `Entry` speaks as a `TableRow` and
            // draws every field, so a stolen line came out wrapped in the orb's
            // own bookkeeping — `probe mix phlegm prism`.
            .push(RecordKind::Message)
            .text(FieldName::Name, Verb::Probe.canonical())
            .text(FieldName::Message, &line)
            .text(FieldName::Source, PRISM)
            .quiet()
            .finish();
    }

    let Some(found) = tower::discover(world) else {
        return;
    };

    // The recipe as the far wizard wrote it, then said out loud: a discovery is
    // worth interrupting a player for.
    for line in super::recall::route_lines(world, &found) {
        world
            .resource_mut::<Scrollback>()
            .records_mut()
            // `Message`, not `Entry`: an `Entry` speaks as a `TableRow` and
            // draws every field, so a stolen line came out wrapped in the orb's
            // own bookkeeping — `probe mix phlegm prism`.
            .push(RecordKind::Message)
            .text(FieldName::Name, Verb::Probe.canonical())
            .text(FieldName::Message, &line)
            .text(FieldName::Source, PRISM)
            .quiet()
            .finish();
    }

    let message = world
        .resource::<Prose>()
        .line("probe_found", &[("name", &found)]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Probe.canonical())
        // No `Detail`: a record draws its message *and* its detail, so the name
        // printed in front of the sentence — *"dreaming and a way to make
        // dreaming"*. What was found goes on `Origin`, a fact for `sift`.
        .text(FieldName::Origin, &found)
        .text(FieldName::Source, PRISM)
        .text(FieldName::Message, &message)
        .role(Role::Success)
        .finish();

    // The rail's green mark, so a player in another room is told (§9).
    tower::mark(world, LENS, tower::Mark::News);
}

/// The room, for the rail's mark.
const LENS: &str = "lens";

/// `count` lines of somebody else's laboratory log.
///
/// Every line names a real `(instrument, verb, material)` triple from the
/// recipe table — see [`spill`].
fn stolen(world: &mut World, count: usize) -> Vec<String> {
    let steps: Vec<(String, String)> = {
        let recipes = world.resource::<crate::content::Recipes>();
        recipes
            .instruments()
            .into_iter()
            .flat_map(|instrument| {
                recipes
                    .for_instrument(instrument)
                    .iter()
                    .filter_map(move |recipe| {
                        let input = recipe.inputs().first().copied()?;
                        Some((instrument.to_owned(), input.to_owned()))
                    })
            })
            .collect()
    };
    if steps.is_empty() {
        return Vec::new();
    }

    let picks: Vec<usize> = {
        let mut rngs = world.resource_mut::<Rngs>();
        let rng = rngs.stream(RngStream::Lens);
        (0..count)
            .map(|_| rand::Rng::random_range(rng, 0..steps.len()))
            .collect()
    };

    picks
        .into_iter()
        .filter_map(|pick| {
            let (instrument, input) = steps.get(pick)?;
            let verb = verb_for(instrument);
            Some(world.resource::<Prose>().line(
                "spill_step",
                &[("name", verb), ("detail", input), ("at", instrument)],
            ))
        })
        .collect()
}

/// The word another wizard would have typed at this instrument.
///
/// Matched on the name, which is the one place in the tower that has to be:
/// [`stolen`] builds its steps from the recipe table, so an instrument here is
/// a content string describing somebody else's laboratory rather than a node of
/// this one. There is no entity to read an `Operation` off.
///
/// The cost is the hazard `tower::panel::craft_of` avoids by reading the
/// component: rename an instrument in `recipes.toml` and it falls through to
/// `wield` here, silently, in a line of flavour nobody diffs. An instrument
/// with no verb of its own is `wield`ed anyway, like the lectern, so the
/// fallthrough and the correct answer look identical.
fn verb_for(instrument: &str) -> &'static str {
    match instrument {
        "mortar_and_pestle" => Verb::Grind.canonical(),
        "balneum_mariae" => Verb::Digest.canonical(),
        "flask_and_rod" => Verb::Mix.canonical(),
        "alembic" => Verb::Distil.canonical(),
        _ => Verb::Wield.canonical(),
    }
}

/// What a solve pays, given how many presses it took — `tower::worth_within_par`,
/// which the menagerie's circle shares.
fn yield_of(world: &World, spent: u32) -> u64 {
    tower::worth_within_par(world, PRISM, spent, ward::PAR)
}

/// The prism's name, which is where a lens record is filed.
const PRISM: &str = "prism";

/// Publish everything the ward has to say, as readings a spell can ask for.
///
/// Clears and re-raises rather than diffing, as `research::refresh` does: a
/// stale `settled` is worse than nothing, and there are six sockets at most.
pub(crate) fn refresh(world: &mut World) {
    let Some(prism) = fixture(world, Verb::Probe) else {
        return;
    };
    publish(world, prism);
}

/// Publish a named prism's readings, whoever is standing where.
///
/// Takes the entity because `land` must not resolve the prism from `Cwd`: a
/// press finishes in the tick schedule, outside `spell::run`'s domain swap, so
/// a bound solver working while the player stood elsewhere published nothing
/// and left every reading frozen at the last `dial`.
pub(crate) fn publish(world: &mut World, prism: Entity) {
    let Some(room) = room_of(world, prism) else {
        return;
    };
    let ward = world.get::<Ward>(prism).cloned();

    clear(world, prism);
    if let Some(ward) = &ward
        && ward.pressed()
    {
        // Shown but not askable: the counts reach the player as record fields,
        // as Mastermind shows the pegs. `readings()` is only the two deltas, so
        // `if the prism has aligned` no longer resolves.
        let (aligned, astray) = ward.last();
        tower::raise_count(world, prism, ward::ALIGNED, aligned);
        tower::raise_count(world, prism, ward::ASTRAY, astray);
        tower::raise_count(world, prism, ward::SPENT, ward.spent());
        // Which way each moved since the last press — the whole of what a
        // codemaker may say, and the whole of what a spell can branch on.
        if let Some(shift) = ward.shift() {
            tower::raise_reading(world, prism, shift.word());
        }
        if let Some(drift) = ward.drift() {
            tower::raise_reading(world, prism, drift.drift_word());
        }
    }

    for (index, name) in ward::SOCKETS.into_iter().enumerate() {
        let Some(node) = reading(world, room, name) else {
            continue;
        };
        clear(world, node);
        let Some(ward) = &ward else { continue };
        // What is in it, and nothing else: `settled`/`loose` and the tallies
        // beside it were the orb keeping the player's notes (§19).
        if let Some(sigil) = ward.seated(index) {
            tower::raise_reading(world, node, sigil);
        }
    }
}

/// One authored line about the lens.
fn say(world: &mut World, verb: Verb, key: &str, args: &[(&str, &str)], role: Role) {
    let message = world.resource::<Prose>().line(key, args);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, verb.canonical())
        .text(FieldName::Message, &message)
        .text(FieldName::Source, PRISM)
        .role(role)
        .finish();
}

/// A press's own record — what the ward answered, filed where it happened.
///
/// `Source` and `At` make `lens.log` a log: §3 keeps one stream, and a domain
/// log is that stream filtered by where each line happened (§19). `At` is also
/// what a spell reads to know the prism has finished.
fn answered(
    world: &mut World,
    message: &str,
    quantity: u64,
    state: Option<&str>,
    detail: Option<&str>,
) {
    // The `Mut` is bound before the chain: `state` is conditional, so the
    // builder cannot chain straight to `finish()` off a temporary guard.
    let mut scrollback = world.resource_mut::<Scrollback>();
    let mut builder = scrollback
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Probe.canonical())
        .text(FieldName::Message, message)
        .count(FieldName::Quantity, quantity)
        .text(FieldName::At, PRISM)
        .text(FieldName::Source, PRISM)
        .role(Role::Success);
    if let Some(state) = state {
        builder = builder.text(FieldName::State, state);
    }
    if let Some(detail) = detail {
        builder = builder.text(FieldName::Detail, detail);
    }
    builder.finish();
}
