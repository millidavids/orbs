//! `muster` and `haul` — the sanctum's two words (DESIGN.md §10).
//!
//! The shape is the lens's, one room over: `muster` draws a course up the way
//! `probe` opens a ward, the three stations publish readings the way the four
//! sockets do, and a spell's `if` resolves against them at cast because
//! [`tower::pylon::readings`] is registered unconditionally in the scene.
//!
//! **Neither word takes the production slot**, which is `probe`'s decision
//! rather than `research`'s. §19 refuses the slot to a maze because a solve is
//! hundreds of ticks and a solver holding it would starve every other spell into
//! `spell_gave_up`; a haul is instant and a course is a hundred of them, so the
//! same argument applies with more force. What this domain costs is **time** —
//! one tick per haul through the queue, and 127 of them on a neglected tower.
//!
//! So a bound `holding` runs *beside* a full brewing loop rather than instead of
//! one, which is what §19 calls additive rather than competing. §5.0's slot is
//! contested in Phase 10, not here.
//!
//! # What a completed course is worth
//!
//! Two things, and the order they are said in matters: the barrier goes back up
//! (`tower::mend`), and then the orb earns. That is `land`'s order in the lens —
//! the work, and then what the work bought.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use super::readings::{clear, reading, room_of};
use crate::content::Prose;
use crate::parser::{Intent, Verb};
use crate::session::Scrollback;
use crate::tower::{
    self,
    pylon::{self, Course, Refused},
};

/// The pylon's name, which is where a sanctum record is filed.
const PYLON: &str = "pylon";

/// `muster` — draw a fresh course of wards up out of the wellspring.
///
/// Refuses where there is no pylon and where a course is already drawn; both
/// name the way forward, which is what keeps the verb off `is_gated`.
pub(super) fn muster(world: &mut World) {
    let Some(pylon) = fixture(world) else {
        say(world, Verb::Muster, "muster_nowhere", &[], Role::Cost);
        return;
    };
    if world.get::<Course>(pylon).is_some() {
        say(world, Verb::Muster, "muster_already", &[], Role::Cost);
        return;
    }

    // **The domain's one draw, and it is here rather than in a system.** A
    // course's height comes from the integrity deficit plus a jitter, so
    // `RngStream::Battlements` advances once per `muster` and never on a tick
    // nobody asked for — which is what lets `tower::erode` be appended to the
    // schedule without shifting any existing replay.
    let deficit = world.resource::<tower::Integrity>().deficit();
    let height = tower::height_for(world, deficit);
    world.entity_mut(pylon).insert(Course::new(height));
    publish(world, pylon);

    say(
        world,
        Verb::Muster,
        "muster_opens",
        &[("quantity", &height.to_string())],
        Role::Success,
    );
}

/// `haul <from> <to>` — carry the topmost ward from one station to another.
///
/// **Directional, and it refuses.** Between any two stations exactly one haul is
/// legal, so a symmetric word would have been unambiguous — and it would leave a
/// spell with nothing to read. The refusal is what makes `potency` worth
/// publishing.
pub(super) fn haul(intent: &Intent, world: &mut World) {
    let Some(pylon) = fixture(world) else {
        say(world, Verb::Haul, "muster_nowhere", &[], Role::Cost);
        return;
    };
    if world.get::<Course>(pylon).is_none() {
        say(world, Verb::Haul, "haul_unopened", &[], Role::Cost);
        return;
    }

    let mut named = intent.arguments.iter().map(|argument| &argument.value);
    let (Some(from), Some(to)) = (named.next(), named.next()) else {
        say(world, Verb::Haul, "haul_incomplete", &[], Role::Cost);
        return;
    };
    // **The leaf, for every sentence below.** A resolved place arrives as its
    // full path, and prose that echoed it would read *"the ward goes to
    // /tower/sanctum/barrier"* — the tower's internals in a line meant for a
    // player. The same reason `scry::dial` strips them.
    let from = crate::parser::leaf(from).to_owned();
    let to = crate::parser::leaf(to).to_owned();

    // **Both slots are `NounKind::Place`**, so `haul laboratory barrier` parses
    // and has to be refused here. Naming which of the two was wrong beats a
    // generic refusal: a player who typed one station and one instrument is one
    // word from right.
    // **Matched rather than re-asked.** The refusal arm used to call
    // `station_of(&from)` a *third* time to work out which name had failed,
    // where the tuple it is standing in already knows — two answers to one
    // question, free to drift apart from the pattern above them.
    let (here, there) = match (pylon::station_of(&from), pylon::station_of(&to)) {
        (Some(here), Some(there)) => (here, there),
        (None, _) => return refuse_station(world, &from),
        _ => return refuse_station(world, &to),
    };

    let moved = world
        .get_mut::<Course>(pylon)
        .expect("the course was there a moment ago")
        .haul(here, there);

    let size = match moved {
        Ok(size) => size,
        Err(refused) => {
            // **Nothing was published**, because nothing moved: unlike a dial,
            // which marks a socket whether or not it turns, a refused haul
            // leaves the course exactly as it was. Republishing would be
            // harmless and would also be a lie about something having happened.
            let key = match refused {
                Refused::Same => "haul_same_station",
                Refused::Bare => "haul_bare",
                Refused::Greater => "haul_greater",
            };
            say(
                world,
                Verb::Haul,
                key,
                &[("name", &from), ("detail", &to)],
                Role::Cost,
            );
            return;
        }
    };

    publish(world, pylon);
    say(
        world,
        Verb::Haul,
        "haul_moved",
        &[
            ("quantity", &size.to_string()),
            ("name", &from),
            ("detail", &to),
        ],
        Role::Success,
    );

    if world.get::<Course>(pylon).is_some_and(Course::solved) {
        finish(world, pylon);
    }
}

/// Say that one of `haul`'s two names is not a station, naming which.
///
/// Both slots are `NounKind::Place`, so `haul laboratory barrier` parses and has
/// to be refused here — and naming the wrong half beats a generic refusal, since
/// a player who typed one station and one instrument is one word from right.
fn refuse_station(world: &mut World, wrong: &str) {
    say(
        world,
        Verb::Haul,
        "haul_not_a_station",
        &[("name", wrong)],
        Role::Cost,
    );
}

/// The course is assembled. Take it down, mend the barrier, and pay for the work.
///
/// **The `Course` is removed on the completing tick**, before anything else
/// happens — the same order `scry::land` uses when a seal breaks, and for a
/// sharper reason here: a solver's loop is `repeat until the pylon is idle`, and
/// a course left standing for even one more tick would let the spell issue
/// another haul into a puzzle it had already won.
fn finish(world: &mut World, pylon: Entity) {
    let Some(course) = world.get::<Course>(pylon) else {
        return;
    };
    let (height, hauls) = (course.height(), course.hauls());

    world.entity_mut(pylon).remove::<Course>();
    // **Mended before published, and the order is the whole of it.** `publish`
    // writes the `integrity` reading from the resource, so putting it first left
    // `survey pylon` saying nought while the rail — which reads the resource
    // directly — said fifty-six. Two surfaces disagreeing about the same number
    // on the same tick, and only the one a *spell* reads was wrong.
    let mended = tower::mend(world, height);
    publish(world, pylon);

    let earned = tower::worth(world, PYLON)
        .saturating_mul(u64::try_from(height.saturating_sub(pylon::LEAST) + 1).unwrap_or(1));

    let message = world.resource::<Prose>().line(
        "muster_done",
        &[
            ("quantity", &hauls.to_string()),
            ("detail", &mended.to_string()),
        ],
    );
    let mut scrollback = world.resource_mut::<Scrollback>();
    scrollback
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Muster.canonical())
        .text(FieldName::Message, &message)
        .count(FieldName::Quantity, earned)
        // `At` is **what a spell reads to know the pylon has finished** — the
        // same field `land` sets for every other instrument. `Source` is what
        // files the line in `sanctum.log`; the archive shipped without it and
        // had an empty log from the day it was built (§19).
        .text(FieldName::At, PYLON)
        .text(FieldName::Source, PYLON)
        .role(Role::Success)
        .finish();

    // After the sentence about the work, for the reason `transmute` gives: the
    // barrier goes back up, *and then* the orb can hold another spell.
    tower::credit(world, earned);
}

/// Publish everything a course has to say, as readings a spell can ask for.
///
/// **Takes the pylon rather than reading `Cwd`**, which is the bug the lens paid
/// for and recorded: a bound solver working while the player stands in the
/// laboratory would otherwise find no pylon, publish nothing, and leave every
/// reading frozen at the last haul.
///
/// Clears and re-raises rather than diffing, exactly as `research::refresh`
/// does. Four nodes at most, and a stale `potency` is worse than none.
pub(crate) fn publish(world: &mut World, pylon: Entity) {
    let Some(room) = room_of(world, pylon) else {
        return;
    };
    let course = world.get::<Course>(pylon).cloned();

    clear(world, pylon);
    // **`integrity` is published whether or not a course is drawn**, because the
    // barrier stands whether or not anybody is working on it — it is the one
    // reading here that is about the tower rather than about the puzzle, and a
    // spell asks it to decide whether to muster at all.
    let standing = world.resource::<tower::Integrity>().get();
    tower::raise_count(world, pylon, pylon::INTEGRITY, standing);
    if let Some(course) = &course
        && course.is_odd()
    {
        tower::raise_reading(world, pylon, pylon::ODD);
    }

    for (index, name) in pylon::STATIONS.into_iter().enumerate() {
        let Some(node) = reading(world, room, name) else {
            continue;
        };
        clear(world, node);
        // **Only while the station holds a ward**, and this is load-bearing
        // rather than tidy: `spell::watch` answers `is empty` by asking whether
        // a node has children, so a station that always carried a `potency`
        // could never be empty and the first two rungs of every solver would be
        // dead.
        if let Some(size) = course.as_ref().and_then(|course| course.top(index)) {
            tower::raise_count(
                world,
                node,
                pylon::POTENCY,
                u32::try_from(size).unwrap_or(1),
            );
        }
    }
}

/// Republish from wherever the player is standing, for the typed path.
pub(crate) fn refresh(world: &mut World) {
    let Some(pylon) = fixture(world) else {
        return;
    };
    publish(world, pylon);
}

/// The pylon, where the player is standing.
fn fixture(world: &World) -> Option<Entity> {
    super::readings::fixture(world, Verb::Muster)
}

/// One authored line about the sanctum.
fn say(world: &mut World, verb: Verb, key: &str, args: &[(&str, &str)], role: Role) {
    let message = world.resource::<Prose>().line(key, args);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, verb.canonical())
        .text(FieldName::Message, &message)
        .text(FieldName::Source, PYLON)
        .role(role)
        .finish();
}
