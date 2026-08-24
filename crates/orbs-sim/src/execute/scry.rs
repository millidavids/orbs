//! `probe` and `dial` — the lens's two words (DESIGN.md §10).
//!
//! The shape is the archive's, one room over: `probe` opens a reading the way
//! `research` opens a maze, the sockets and sigils publish readings the way the
//! four compass bearings do, and a spell's `if` resolves against them at cast
//! because [`tower::ward::readings`] is registered unconditionally in the scene.
//!
//! **The module is still `scry`**, because that is what the domain is called
//! (§10) even though the word did not survive the naming sweep — `scr` reaches
//! `scribe`, and `tests/naming.rs` allows no exemption. `probe` opens and
//! presses in one word, which is `grind`'s move-and-wield idiom.
//!
//! What is different is that **`probe` holds the tower's production slot** and
//! `research` does not. §19 refuses the slot to a maze because a solve is
//! hundreds of ticks and a solver holding it would starve every other spell into
//! `spell_gave_up`; a press is twelve ticks and gives the slot back between
//! presses, so the lens can honour ROADMAP's stated scarcity — *"a read is not a
//! brew"* — where the archive could not.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::Prose;
use crate::parser::{Intent, Verb};
use crate::rng::{RngStream, Rngs};
use crate::session::Scrollback;
use crate::tower::{
    self, Cwd,
    ward::{self, Ward},
};

/// `dial <socket> <sigil>` — turn one dial of the aperture.
///
/// **Free, and instantly.** A dial is not work; §10.1's cost model puts the
/// price on the run, and here the run is the press. It is also what lets a
/// player change three sockets and press once, which is the shape a deduction
/// actually takes.
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
    // **Bare means *try something else here*.** The sigil slot is optional so a
    // script can turn a dial without naming what to turn it to — see
    // `SOCKET_AND_SIGIL` and `Ward::advance`. A player types the sigil; a spell,
    // having no variables, cannot.
    let Some(sigil) = named.next() else {
        advance(world, prism, crate::parser::leaf(socket));
        return;
    };
    // **Both slots are `NounKind::Place`**, so `seat laboratory nitre` parses
    // and has to be refused here — the same shape `research::named` already has
    // for `follow`. Naming what was wrong beats a generic refusal: a player who
    // typed the sigil and the socket the wrong way round is one word from right.
    // **The leaf, for every sentence below.** A resolved place arrives as its
    // full path, and prose that echoes it reads *"the /tower/lens/second socket
    // turns to /tower/lens/borax"* — the tower's internals in a line meant for a
    // player. `path_of` exists for when the path *is* the answer, which this is
    // not.
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

    // **Published either way, because `seat` marks either way.** A dial aimed at
    // a socket bumps its tally whether or not anything moved — that tally is what
    // a stateless ladder walks — so returning before this left `survey first`
    // reporting a stale count and a spell reading the pre-dial value. It was
    // masked while the player stood in the lens, because the next press
    // republished and caught up; `publish` taking the prism is what removes that
    // mask, so this had to be fixed with it.
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
/// **This is what makes a solver writable at all.** §8's language has no
/// variables, so a spell cannot name the sigil a socket has not tried; bare, the
/// ward picks the next one. The success line names what it chose, because a
/// player watching a spell work needs to see *what* it tried, and a script that
/// could not name it still wants the transcript to.
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

    // **One outcome now: it turns.** This used to be a three-way match, because
    // `advance` took the first sigil the socket had not *tried since the last
    // gain* — so it could find none left (`dial_spent`) or land on the one
    // already there (`dial_held`). Both were symptoms of the ward keeping a
    // per-socket candidate list, which is the player's bookkeeping and is gone.
    //
    // A cyclic step always moves, so the only thing to say is which sigil it
    // moved to. `dial_spent` has no reachable path left.
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
/// **Two acts in one word**, which is §19's per-instrument idiom: `grind sage`
/// is a `move` and a `wield`, and this is finding a far orb and pressing it. It
/// costs nothing in clarity because the aperture opens on a fixed figure, so the
/// first press means the same thing every time and there is nothing to dial
/// before it. It also gives a solver spell a one-word body.
///
/// **Instant, and it takes no slot.** A press used to schedule twelve ticks of
/// work on the prism through the ordinary machinery; it now answers on the tick it
/// is typed, like `dial`. The whole domain is therefore free of the tower's
/// production pool — see [`land`] for what that changed and what it did not.
///
/// The answer still goes through [`land`], which is where a press has always been
/// applied. What is gone is the wait in front of it, not the step.
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

    // **Applied here rather than routed through `work::land`.** With no duration
    // there is nothing for the slot machinery to hold, and scheduling zero ticks
    // would still take the production slot for a tick — which is the cost this
    // change is removing, arriving by the back door.
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
        // **No `Detail`.** The record's fields are drawn after its sentence, and
        // the astray count is already *in* the sentence — carrying it twice read
        // as `the ward is level 2`, a number with nothing attached to it. Rule 4
        // wants the facts in fields, but not the ones the prose has already
        // spent.
        answered(world, &message, u64::from(aligned), Some(shift), None);
        return;
    }

    // The seal is open. The reading is over, so the ward goes and the readings
    // go with it — the same order `research::tread` uses, and for the same
    // reason: leaving it would keep stale readings on the sockets for ever.
    world.entity_mut(prism).remove::<Ward>();
    publish(world, prism);

    let earned = yield_of(world, spent);
    let message = world
        .resource::<Prose>()
        .line("probe_broken", &[("quantity", &spent.to_string())]);
    answered(world, &message, earned, None, None);
    // After the sentence about the work, for the reason `transmute` gives: the
    // seal opens, *and then* the orb can hold another spell.
    tower::credit(world, earned);
    spill(world);
}

/// How many lines of the far wizard's log a broken seal gives up.
const SPILL: usize = 12;

/// The far orb's log, spilling into yours.
///
/// # It is flavour, and it is not required reading
///
/// A dozen lines of somebody else's laboratory — a grind, a digestion, an
/// instrument turned out. **Quiet**, so they go to `lens.log` and not to the
/// transcript: twelve lines per solve would push the player's own last command
/// off screen in seconds, which is the same argument §19 makes for a spell's
/// output going to the log. What the transcript gets is one sentence saying how
/// many there were.
///
/// # Generated from real content, so it stays true
///
/// The shapes are authored (`spill_*` in `prose.toml`, rule 6) and the nouns are
/// drawn from `Recipes` — so a line always names an instrument that exists doing
/// something it can actually do. A template filled from a free-for-all of names
/// would print `digest sage`, which is a plausible-looking line for an operation
/// the bath cannot perform, and a player who tried it would learn the wrong
/// thing about their own tower.
///
/// # And now and then it is a recipe
///
/// The roll is in `tower::learned`. When it lands, the log spells the thing out
/// and the orb writes it down — *"most likely this will be automated, not
/// requiring you to read the recipe manually"* — so the sentence about it is
/// drawn while the working is not.
fn spill(world: &mut World) {
    for line in stolen(world, SPILL) {
        // **Quiet, but filed.** `Records::drawn` skips it, `lens.log` keeps it —
        // and `Source` is what makes the second true. §19 records the archive
        // shipping without it and having an empty log from the day it was built.
        world
            .resource_mut::<Scrollback>()
            .records_mut()
            // **`Message`, not `Entry`.** An `Entry` speaks as a `TableRow`,
            // which `Record::is_prose` excludes — so every field draws, and a
            // stolen line came out as `probe mix phlegm prism`: the orb's own
            // bookkeeping wrapped around somebody else's log entry. A `Message`
            // draws its sentence alone and keeps the fields for `sift`.
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

    // The recipe, written into the log as the far wizard wrote it — and then
    // said out loud, because a discovery is the one thing here worth
    // interrupting a player for.
    for line in super::recall::route_lines(world, &found) {
        world
            .resource_mut::<Scrollback>()
            .records_mut()
            // **`Message`, not `Entry`.** An `Entry` speaks as a `TableRow`,
            // which `Record::is_prose` excludes — so every field draws, and a
            // stolen line came out as `probe mix phlegm prism`: the orb's own
            // bookkeeping wrapped around somebody else's log entry. A `Message`
            // draws its sentence alone and keeps the fields for `sift`.
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
        // **No `Detail`.** A record carrying prose draws its message *and* its
        // detail — `Detail` is secondary prose subordinate to the message — so
        // the name printed in front of the sentence written to say it: *"dreaming
        // and a way to make dreaming"*. The same trap `recall`'s route steps
        // record falling into. What was found is on `Origin`, which is a fact for
        // `sift` and not a second sentence.
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
/// Every line names a real `(instrument, verb, material)` triple, drawn from the
/// recipe table — see [`spill`] for why a free-for-all would teach the player
/// something false about their own tower.
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
/// Read off the `Operation` the tower gives it rather than matched on a name —
/// §19 records `craft_of` paying twice for the hardcoded-name pattern. An
/// instrument with no verb of its own is `wield`ed, which is what the lectern
/// already is.
fn verb_for(instrument: &str) -> &'static str {
    match instrument {
        "mortar_and_pestle" => Verb::Grind.canonical(),
        "balneum_mariae" => Verb::Digest.canonical(),
        "flask_and_rod" => Verb::Mix.canonical(),
        "alembic" => Verb::Distil.canonical(),
        _ => Verb::Wield.canonical(),
    }
}

/// What a solve pays, given how many presses it took.
///
/// **Full at or under par, less beyond it, and never nothing.** The halving
/// curve a first draft proposed is withdrawn: measured, a blind ladder needs
/// ~23 presses against a player's ~4, and halving would land it on the floor of
/// 1 — while the ladder *already* pays 5.5× the ticks, so a yield penalty
/// double-counts and drives automated scrying below the archive's maze. The tick
/// cost does the real work; this only rewards playing well.
fn yield_of(world: &World, spent: u32) -> u64 {
    let full = tower::worth(world, PRISM);
    if spent <= ward::PAR {
        full
    } else {
        full.saturating_mul(3) / 4
    }
}

/// The prism's name, which is where a lens record is filed.
const PRISM: &str = "prism";

/// Publish everything the ward has to say, as readings a spell can ask for.
///
/// Clears and re-raises rather than diffing, exactly as `research::refresh`
/// does: a node holding a stale `settled` is worse than one holding nothing, and
/// the sockets are six children at most.
pub(crate) fn refresh(world: &mut World) {
    let Some(prism) = fixture(world, Verb::Probe) else {
        return;
    };
    publish(world, prism);
}

/// Publish a named prism's readings, whoever is standing where.
///
/// **`land` must not resolve the prism from `Cwd`, and this is why it takes the
/// entity.** A press finishes in the tick schedule (`work::land`), which is
/// *outside* `spell::run`'s domain swap — so a **bound** solver working while the
/// player is in the laboratory found no prism under `Cwd`, published nothing, and
/// left every reading frozen at the last `dial`: a socket that had just settled
/// still read `loose`, the ladder dialled it, `seat` refused, and the rung fired
/// for ever.
///
/// It also left the reading children behind when a seal broke, which is exactly
/// the *"stale readings on the sockets for ever"* the removal order exists to
/// prevent. The one test that binds a solver never left the lens, so nothing
/// caught it.
pub(crate) fn publish(world: &mut World, prism: Entity) {
    let Some(room) = room_of(world, prism) else {
        return;
    };
    let ward = world.get::<Ward>(prism).cloned();

    clear(world, prism);
    if let Some(ward) = &ward
        && ward.pressed()
    {
        // **The counts are still shown, and are no longer askable.** They reach
        // the player as *record fields* — `survey prism` prints them and so does
        // the sheet — which is the game: Mastermind shows you the pegs for every
        // guess. What a spell may ask for is `readings()`, and that is now only
        // the two deltas, so `if the prism has aligned` no longer resolves.
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
        // **What is in it, and nothing else.** A socket used to publish
        // `settled`/`loose` and two tallies beside this — a verdict on the
        // position and a count of what had been tried there. Both were the orb
        // keeping the player's notes; what is left is the player reading their
        // own dial back (§19).
        if let Some(sigil) = ward.seated(index) {
            tower::raise_reading(world, node, sigil);
        }
    }
}

fn clear(world: &mut World, node: Entity) {
    for held in tower::children_of(world, node) {
        world.entity_mut(held).despawn();
    }
}

/// A lens fixture, by the operation it carries.
///
/// **Where the player is standing**, like `research::stacks`: the readings and
/// the picture both follow the room, so neither can outrun the other by
/// following the player out of it.
fn fixture(world: &World, verb: Verb) -> Option<Entity> {
    let cwd = world.resource::<Cwd>().0;
    beside_node(world, cwd, verb)
}

fn beside_node(world: &World, parent: Entity, verb: Verb) -> Option<Entity> {
    tower::children_of(world, parent).into_iter().find(|node| {
        world
            .get::<tower::Operation>(*node)
            .is_some_and(|operation| operation.0 == verb)
    })
}

/// One of the sockets or sigils, by name, among `room`'s own fixtures.
///
/// **Takes the room rather than reading `Cwd`**, for the reason [`publish`]
/// gives: a press lands in the tick schedule, where the player may be standing
/// anywhere.
fn reading(world: &World, room: Entity, wanted: &str) -> Option<Entity> {
    tower::children_of(world, room).into_iter().find(|node| {
        world.get::<tower::Reading>(*node).is_some()
            && world
                .get::<tower::Name>(*node)
                .is_some_and(|name| name.0 == wanted)
    })
}

/// The room a fixture stands in.
fn room_of(world: &World, node: Entity) -> Option<Entity> {
    world
        .get::<bevy_ecs::hierarchy::ChildOf>(node)
        .map(bevy_ecs::hierarchy::ChildOf::parent)
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
/// `Source` and `At` are what make `lens.log` a log: §3 keeps one stream and a
/// domain log is that stream filtered by where each line happened. The archive
/// shipped without them and had an empty log from the day it was built (§19), so
/// every record in this module sets them.
///
/// `At` additionally is **what a spell reads to know the prism has finished** —
/// the same field `land` sets for every other instrument.
fn answered(
    world: &mut World,
    message: &str,
    quantity: u64,
    state: Option<&str>,
    detail: Option<&str>,
) {
    // **The `Mut` is bound before the chain.** `resource_mut` returns a guard,
    // and a builder held past the end of the statement that created it outlives
    // the borrow it came from — which is why every other emitter in the tower
    // chains straight through to `finish()`. This one cannot, because `state` is
    // conditional.
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
