//! `summon` and `sing` — the menagerie's two words (DESIGN.md §10).
//!
//! The shape is the sanctum's one room over: `summon` opens a figure the way
//! `muster` draws a course, the circle publishes readings the way the pylon
//! does, and a spell's `if` resolves against them at cast because
//! [`tower::chant`]'s readings are registered unconditionally in the scene.
//!
//! **Neither word takes the production slot**, and here that is not a
//! preference. A `sing` that queued behind a brew would arrive after the
//! syllable it was answering had already landed — not a wait but a guaranteed
//! miss. `spell::block::begins_work` carries the exemption, and §19 records it
//! being *forgotten* for the sanctum and what that cost.
//!
//! # The one clock in the tower
//!
//! A syllable lands on a tick and singing the right one on that tick strikes it.
//! §10.1's *"never a reflex"* is struck for this domain and this domain only
//! (§19); every other room still answers to what the player chooses rather than
//! to when they act.
//!
//! **This module still knows nothing finer than a tick.** A real-time press
//! carries a sub-tick `phase` for grading, and it is graded and dropped — never
//! stored, never in a component, never in the save. `orbs-sim` gains no time
//! source, which is rules 1 and 3 together.
//!
//! # What a finished chant is worth
//!
//! Two things, in this order: the circle yields troops, and then the orb earns.
//! That is `muster`'s order and the lens's — the work, and then what the work
//! bought. A collapsed one wears the barrier instead, which is the only cost
//! this domain has and the reason attempting one is free.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use super::readings::{clear, reading, room_of};
use crate::content::Prose;
use crate::parser::{Intent, Verb};
use crate::session::Scrollback;
use crate::tower::{
    self,
    chant::{self, Chant, Strike, Syllable},
};

/// The circle's name, which is where a menagerie record is filed.
const CIRCLE: &str = "circle";

/// What a troop is called on a shelf.
pub const TROOP: &str = "troop";

/// `summon` — draw a fresh figure up at the circle.
///
/// Refuses where there is no circle and where a chant is already running; both
/// name the way forward, which is what keeps the verb off `is_gated`.
pub(super) fn summon(world: &mut World) {
    let Some(circle) = fixture(world) else {
        say(world, Verb::Summon, "summon_nowhere", &[], Role::Cost);
        return;
    };
    if world.get::<Chant>(circle).is_some() {
        say(world, Verb::Summon, "summon_already", &[], Role::Cost);
        return;
    }

    // **The domain's one draw, and it is here rather than in a system.** A
    // figure is rolled once per `summon`, so `RngStream::Menagerie` advances
    // only when somebody asks — which is what lets the per-tick systems be
    // appended to the schedule without shifting any existing replay.
    let figure = {
        let mut rngs = world.resource_mut::<crate::rng::Rngs>();
        Chant::draw(&mut rngs)
    };
    world.entity_mut(circle).insert(figure);
    // The tick a figure gathers on is not one anybody could have sung — see
    // [`Answered`].
    world.resource_mut::<Answered>().0 = true;
    publish(world, circle);

    say(
        world,
        Verb::Summon,
        "summon_opens",
        &[("quantity", &chant::LENGTH.to_string())],
        Role::Success,
    );
}

/// `sing <syllable>` — answer the syllable at the aperture.
///
/// The typed path. A real-time press reaches [`strike`] directly instead,
/// carrying a phase; both go through the same body below so a hand-sung chant
/// and a keyed one cannot disagree about what a strike is worth.
pub(super) fn sing(intent: &Intent, world: &mut World) {
    let Some(circle) = fixture(world) else {
        say(world, Verb::Sing, "summon_nowhere", &[], Role::Cost);
        return;
    };
    if world.get::<Chant>(circle).is_none() {
        say(world, Verb::Sing, "sing_unopened", &[], Role::Cost);
        return;
    }

    let Some(named) = intent.arguments.first().map(|one| &one.value) else {
        say(world, Verb::Sing, "sing_incomplete", &[], Role::Cost);
        return;
    };
    // **The leaf, for the same reason `haul` takes one.** A resolved place
    // arrives as its full path, and prose echoing it would read *"you sing
    // /tower/menagerie/skyward"* — the tower's internals in a line meant for a
    // player.
    let word = crate::parser::leaf(named).to_owned();
    let Some(syllable) = Syllable::from_word(&word) else {
        say(
            world,
            Verb::Sing,
            "sing_not_a_syllable",
            &[("name", &word)],
            Role::Cost,
        );
        return;
    };

    strike(world, circle, syllable);
}

/// A request for the arrow keys.
///
/// **Taken rather than read**, which is `Wandering`'s rule: the verb asks once,
/// and a frontend polling a persistent flag would re-open the surface every
/// frame after the player had escaped out of it.
#[derive(Resource, Debug, Default)]
pub struct Chorusing(bool);

impl Chorusing {
    /// Whether a request is waiting.
    #[must_use]
    pub const fn pending(&self) -> bool {
        self.0
    }

    /// Take it, if there is one.
    pub const fn take(&mut self) -> bool {
        let asked = self.0;
        self.0 = false;
        asked
    }
}

/// `chorus` — hand the arrow keys to a running chant.
///
/// Refuses where there is no circle and where nothing is running, both naming
/// the way forward. **A chant must already be open**: unlike `wander`, which can
/// be typed at a maze that is standing there, a figure nobody summoned is a
/// surface with nothing on it.
pub(super) fn chorus(world: &mut World) {
    let Some(circle) = fixture(world) else {
        say(world, Verb::Chorus, "summon_nowhere", &[], Role::Cost);
        return;
    };
    if world.get::<Chant>(circle).is_none() {
        say(world, Verb::Chorus, "sing_unopened", &[], Role::Cost);
        return;
    }
    world.resource_mut::<Chorusing>().0 = true;
    say(world, Verb::Chorus, "chorus_begins", &[], Role::Success);
}

/// Answer the aperture, however the syllable arrived.
///
/// **The one body both paths run**, which is `Sim::walk`'s rule: a hand-walked
/// maze and a spell-walked one cannot disagree about what reaching the exit is
/// worth, and a hand-sung chant and a keyed one cannot disagree here either.
pub(crate) fn strike(world: &mut World, circle: Entity, syllable: Syllable) {
    // **One syllable a tick, and key repeat is why.** `Sim::sing` reaches the
    // world without waiting for a clock, so a frontend looping the frame's
    // events called this once per press — and every call consumes the syllable
    // at the aperture. A finger held on an arrow ate four of them between two
    // ticks and collapsed the figure, with the last three guaranteed `Early`
    // because `consume` had just reset the approach.
    //
    // **The same flag `lapse` reads**, deliberately: *"has the aperture already
    // moved this tick"* is one question, and it was being asked on one side
    // only. A second counter would be a second answer.
    //
    // Checking `event.repeat` in the frontend would not do it — two deliberate
    // presses in one frame are not repeats, and the mapping lives in
    // `orbs-shell` where a frontend's notion of a repeat does not.
    if world.resource::<Answered>().0 {
        return;
    }
    // Read before the component is borrowed, or the two borrows overlap.
    let patient = world.resource::<Patient>().is_set();
    let Some(mut chant) = world.get_mut::<Chant>(circle) else {
        return;
    };
    let Some(outcome) = chant.strike(syllable, patient) else {
        return;
    };
    // **Told before the sentence, because `lapse` runs later in the same tick.**
    // Without this the figure advances twice — once here and once for a lapse
    // that did not happen — and every other syllable is eaten unanswered.
    world.resource_mut::<Answered>().0 = true;
    let key = match outcome {
        Strike::Struck => "sing_struck",
        Strike::Missed => "sing_missed",
        // **Named separately though it costs the same.** A player who sang the
        // right word too soon has made a different mistake from one who sang the
        // wrong word, and telling them apart is the whole of what teaches the
        // timing. §6's contract: a refusal names the way forward.
        Strike::Early => "sing_early",
        // **Unreachable, and settled rather than returned.** `Chant::strike`
        // never answers `Travelling` — only `Chant::travel` does — but the
        // earlier version of this arm bailed out with an early `return`, which
        // would have skipped `settle` if the arm ever became live: a finished or
        // collapsed figure would keep its component, never yield, never wear the
        // barrier, and never republish, with `Answered` already suppressing that
        // tick's lapse. Falling through to `settle` is wrong in no case.
        Strike::Travelling => "sing_early",
    };
    say(
        world,
        Verb::Sing,
        key,
        &[("name", syllable.word())],
        match outcome {
            Strike::Struck => Role::Success,
            _ => Role::Cost,
        },
    );
    settle(world, circle);
}

/// Let the syllable at the aperture land unanswered, and move the figure on.
///
/// The per-tick half: a chant advances whether or not anybody sang. **Appended
/// to the schedule, never inserted**, and it draws nothing from any stream — it
/// is a clock reading, so it cannot perturb a replay. The same rule
/// `tower::settling` has.
pub(crate) fn lapse(world: &mut World) {
    let Some(circle) = open_circle(world) else {
        return;
    };
    // **Only when nothing was sung this tick.** `sing` already advanced the
    // figure, and advancing it twice would eat a syllable nobody was given the
    // chance to answer.
    if world.resource::<Answered>().0 {
        world.resource_mut::<Answered>().0 = false;
        return;
    }
    let patient = world.resource::<Patient>().is_set();
    let Some(mut chant) = world.get_mut::<Chant>(circle) else {
        return;
    };
    // **Republish on every tick, not only when a syllable lands.** `until`
    // counts down while the figure travels, and a solver reads it — so a board
    // and a spell both go stale between landings if this only ran on the last
    // tick of an approach.
    if chant.travel(patient).is_none() {
        return;
    }
    settle(world, circle);
}

/// Whether a chant waits for the singer rather than for the clock.
///
/// **§14's accommodation, and a *rule* rather than a setting** — which is what
/// `orbs_shell::shortcuts` says about every other one of these until Phase 11
/// builds a settings screen. It lives in the sim because it changes what a
/// strike is worth, and a frontend deciding that would be two games.
#[derive(Resource, Debug, Default)]
pub struct Patient(bool);

impl Patient {
    /// Whether the figure is waiting.
    #[must_use]
    pub const fn is_set(&self) -> bool {
        self.0
    }

    /// Flip it, and say what it became.
    pub const fn toggle(&mut self) -> bool {
        self.0 = !self.0;
        self.0
    }
}

/// Whether this tick's syllable has already been dealt with.
///
/// **A flag rather than a tick stamp**, because what [`lapse`] needs to know is
/// *"has the aperture already moved this tick"* and the answer is only ever
/// about the tick in hand. A stamp would be a second representation of the clock
/// in a module whose whole claim is that it has none.
///
/// **`summon` sets it too, and that is not a hack.** The tick a figure gathers
/// on is not a tick anybody could have sung: `summon` executes at the start of
/// it and [`lapse`] runs later in the same schedule, so without this the first
/// syllable is missed before the singer has seen it. It showed as
/// `remaining = 11` on the tick a twelve-syllable figure opened.
#[derive(Resource, Debug, Default)]
pub(crate) struct Answered(pub(crate) bool);

/// Finish the chant if it has run out or fallen apart, and republish either way.
fn settle(world: &mut World, circle: Entity) {
    let Some(chant) = world.get::<Chant>(circle).cloned() else {
        return;
    };
    if chant.has_collapsed() {
        // **Wear before the sentence**, which is `finish`'s rule in the sanctum:
        // the other way round, a collapsed chant says the barrier is whole on
        // the transcript and shows it worn on the rail, on one tick.
        let taken = tower::wear_by(world, chant::WEAR);
        world.entity_mut(circle).remove::<Chant>();
        publish(world, circle);
        say(
            world,
            Verb::Sing,
            "sing_collapsed",
            &[("quantity", &taken.to_string())],
            Role::Danger,
        );
        return;
    }
    if !chant.is_done() {
        publish(world, circle);
        return;
    }

    let troops = chant.troops();
    let (struck, _) = chant.tally();
    world.entity_mut(circle).remove::<Chant>();
    // **Through `tower::home`, never to a named room.** A troop is finished work
    // and keeps itself in the arsenal; asking the rule rather than the room is
    // what stops a sung troop and a `debug_spawn`ed one landing in different
    // places, which §19 records the archive's fragments doing for one change.
    if troops > 0
        && let Some(shelf) = tower::home(world, TROOP)
    {
        tower::give(
            world,
            shelf,
            TROOP,
            crate::parser::NounKind::Essence,
            troops,
        );
    }
    publish(world, circle);
    say(
        world,
        Verb::Sing,
        "sing_finished",
        &[
            ("quantity", &troops.to_string()),
            ("count", &struck.to_string()),
        ],
        Role::Success,
    );
    // The work, and then what the work bought.
    tower::credit(world, u64::from(struck));
}

/// Publish everything a chant has to say, as readings a spell can ask for.
///
/// **Takes the circle rather than reading `Cwd`**, which is the bug the lens
/// paid for and recorded: a bound solver working while the player stands in the
/// laboratory would otherwise find no circle, publish nothing, and leave every
/// reading frozen at the last syllable.
///
/// Clears and re-raises rather than diffing, as `research::refresh` does.
pub(crate) fn publish(world: &mut World, circle: Entity) {
    let Some(room) = room_of(world, circle) else {
        return;
    };
    let chant = world.get::<Chant>(circle).cloned();

    clear(world, circle);
    if let Some(chant) = &chant {
        tower::raise_count(
            world,
            circle,
            chant::REMAINING,
            u32::try_from(chant.remaining()).unwrap_or(u32::MAX),
        );
        // **`until` is deliberately not published**, and `chant::readings` says
        // why at length: a spell that can read the clock does not have to count
        // it, and counting is the puzzle. `Chant::until` still answers for the
        // board and for `orbs-balance`; the language cannot ask.
    }

    for one in Syllable::ALL {
        let Some(node) = reading(world, room, one.word()) else {
            continue;
        };
        clear(world, node);
        // **Only the syllable at the aperture carries `next`**, and that is
        // load-bearing rather than tidy — the sanctum's rule for `potency`.
        // `spell::watch` answers `is empty` by asking whether a node has
        // children, so a syllable that always carried a reading could never be
        // empty and a solver's first rung would be dead.
        if chant.as_ref().and_then(Chant::next) == Some(one) {
            tower::raise_reading(world, node, chant::NEXT);
        }
        // **And one behind it**, which is the whole of §8's lookahead. Same
        // rule: only the one syllable carries it, so a lane can still be empty
        // and a solver's `is empty` rung still means something.
        //
        // **A syllable can carry both.** `skyward skyward` is an ordinary chart
        // — repeats are what make the figure unpredictable — and then one lane
        // is the aperture *and* the one behind it. Two separate `if`s rather
        // than an `else`, or a producer reading `onward` would go blind for a
        // lap exactly when the chart repeated.
        if chant.as_ref().and_then(Chant::onward) == Some(one) {
            tower::raise_reading(world, node, chant::ONWARD);
        }
    }
}

// **There is no `refresh` here, where the sanctum has one.** `muster::refresh`
// exists because `erode` writes integrity on a tick and has to republish from
// wherever the player is; every write in this module already goes through
// `settle`, which publishes. A second entry point with no caller would be an
// API nobody had shaped.

/// The circle anywhere in the tower, whether or not the player is standing in it.
pub(crate) fn open_circle(world: &mut World) -> Option<Entity> {
    let mut found = world.query::<(Entity, &tower::Operation)>();
    found
        .iter(world)
        .find(|(_, operation)| operation.0 == Verb::Summon)
        .map(|(entity, _)| entity)
}

/// The circle, where the player is standing.
fn fixture(world: &World) -> Option<Entity> {
    super::readings::fixture(world, Verb::Summon)
}

/// The circle the player is standing over, for the board.
///
/// **Reads `Cwd`, unlike [`open_circle`]**, and the difference is the whole rule
/// the two boards before this one settled: a *picture* follows the room, so it
/// cannot outrun the readings by following the player out of it; a *tick system*
/// must take the entity, because a bound solver runs while the player is
/// anywhere.
pub(crate) fn circle_at(world: &World) -> Option<Entity> {
    fixture(world)
}

/// One authored line about the menagerie.
fn say(world: &mut World, verb: Verb, key: &str, args: &[(&str, &str)], role: Role) {
    let message = world.resource::<Prose>().line(key, args);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, verb.canonical())
        .text(FieldName::Message, &message)
        .text(FieldName::Source, CIRCLE)
        .role(role)
        .finish();
}
