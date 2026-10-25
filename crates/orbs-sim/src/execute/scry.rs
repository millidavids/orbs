//! `probe` and `dial` — the lens's two words (DESIGN.md §10).
//!
//! The archive's shape: `probe` opens a reading as `research` opens a maze,
//! and [`tower::ward::readings`] is registered unconditionally so a spell's
//! `if` resolves at cast. Named after the domain; the verb `scry` lost to
//! `scribe`. No production slot, so a solver runs beside a brew (§19).

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
    // Both slots are `Place`, so a wrong word is refused here by name. The
    // leaf for every sentence below, never the full path.
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
/// What makes a solver writable, since a spell cannot name a sigil it has not
/// tried. The line names what was chosen.
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
/// Two acts in one word (§19). The aperture opens on a fixed figure, so a
/// solver has a one-word body. Instant and slotless; the answer still goes
/// through [`land`].
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
        publish(world, prism);
        say(world, Verb::Probe, "scry_opens", &[], Role::Success);
    }

    // Applied here rather than routed through `work::land`: scheduling zero
    // ticks would still take the production slot for a tick.
    land(world, prism);
}

/// A press has finished: ask the ward, publish, and see whether the seal broke.
///
/// Called from `work::land`, where every completed run lands, so there is one
/// clock.
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
        answered(world, &message, u64::from(aligned), Some(shift));
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
    answered(world, &message, earned, None);
    // After the sentence about the work, for the reason `transmute` gives: the
    // seal opens, *and then* the orb can hold another spell.
    tower::done(world, &tower::Work::at(PRISM), earned);
    spill(world);
}

/// How many lines of the far wizard's log a broken seal gives up.
const SPILL: usize = 12;

/// The far orb's log, spilling into yours. Flavour, not required reading.
///
/// Quiet, to `lens.log`; the transcript gets one sentence (§19). The shapes
/// are authored and the nouns from `Recipes`, so no line names an operation
/// that cannot happen. Sometimes a recipe (`tower::learned`), whose sentence
/// is drawn.
fn spill(world: &mut World) {
    for line in stolen(world, SPILL) {
        overheard(world, &line);
    }

    let Some(found) = tower::discover(world) else {
        return;
    };

    // The recipe as the far wizard wrote it, then said out loud: a discovery is
    // worth interrupting a player for.
    for line in super::recall::route_lines(world, &found) {
        overheard(world, &line);
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

/// One line of the far orb's log: quiet, so it reaches `lens.log` and not the
/// transcript.
fn overheard(world: &mut World, line: &str) {
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        // `Message`, not `Entry`: an `Entry` speaks as a `TableRow` and draws
        // every field, so a stolen line came out wrapped in the orb's own
        // bookkeeping — `probe mix phlegm prism`.
        .push(RecordKind::Message)
        .text(FieldName::Name, Verb::Probe.canonical())
        .text(FieldName::Message, line)
        .text(FieldName::Source, PRISM)
        .quiet()
        .finish();
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
            .map(|_| rand::RngExt::random_range(rng, 0..steps.len()))
            .collect()
    };

    picks
        .into_iter()
        .filter_map(|pick| {
            let (instrument, input) = steps.get(pick)?;
            // Another wizard's instrument, by name: no entity to read an
            // `Operation` off, so the tower's own table says what it runs.
            let verb = tower::operation_of(instrument)
                .unwrap_or(Verb::Wield)
                .canonical();
            Some(world.resource::<Prose>().line(
                "spill_step",
                &[("name", verb), ("detail", input), ("at", instrument)],
            ))
        })
        .collect()
}

/// What a solve pays, given how many presses it took — `tower::worth_within_par`,
/// which the menagerie's circle shares.
fn yield_of(world: &World, spent: u32) -> u64 {
    tower::worth_within_par(world, PRISM, spent, ward::PAR)
}

/// The prism's name, which is where a lens record is filed.
const PRISM: &str = "prism";

/// Publish a named prism's readings, whoever is standing where.
///
/// Takes the entity, never `Cwd`: a press lands outside `spell::run`'s domain
/// swap, so a bound solver would publish nothing.
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
    super::readings::filed(world, PRISM, verb, key, args, role);
}

/// A press's own record — what the ward answered, filed where it happened.
///
/// `Source` and `At` file it in `lens.log` (§19); `At` is also how a spell
/// knows the prism has finished.
fn answered(world: &mut World, message: &str, quantity: u64, state: Option<&str>) {
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
    builder.finish();
}
