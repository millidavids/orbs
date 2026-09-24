//! `muster` and `haul` — the sanctum's two words (DESIGN.md §10).
//!
//! The shape is the lens's, one room over: `muster` draws a course up the way
//! `probe` opens a ward, the three stations publish readings the way the four
//! sockets do, and a spell's `if` resolves against them at cast because
//! [`tower::pylon::readings`] is registered unconditionally in the scene.
//!
//! Neither word takes the production slot, for `probe`'s reason (§19): a solver
//! holding it would starve every other spell into `spell_gave_up`. What this
//! domain costs is time — one tick per haul, 127 of them on a neglected tower —
//! so a bound `holding` runs *beside* a full brewing loop, additive rather than
//! competing.
//!
//! A completed course does two things, in this order: the barrier goes back up
//! (`tower::mend`), and then the orb earns — `land`'s order in the lens.

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

    // The domain's one draw, here rather than in a system:
    // `RngStream::Battlements` advances once per `muster` and never on a tick
    // nobody asked for, so `tower::erode` can join the schedule without
    // shifting a replay.
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
/// Directional, and it refuses. A symmetric word would be unambiguous and would
/// leave a spell nothing to read; the refusal is what makes `potency` worth
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
    // The leaf, for every sentence below: a resolved place arrives as a full
    // path, and prose echoing it shows a player the tower's internals. Same
    // reason `scry::dial` strips them.
    let from = crate::parser::leaf(from).to_owned();
    let to = crate::parser::leaf(to).to_owned();

    // Both slots are `NounKind::Place`, so `haul laboratory barrier` parses and
    // has to be refused here. Matched rather than re-asked: the refusal arm
    // used to call `station_of` a third time, where the tuple already knows.
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
            // Nothing published, because nothing moved — unlike a dial, which
            // marks a socket whether or not it turns, a refused haul leaves the
            // course exactly as it was.
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
/// Naming the wrong half beats a generic refusal: a player who typed one
/// station and one instrument is one word from right.
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
/// The `Course` is removed first, as `scry::land` does: a solver loops `repeat
/// until the pylon is idle`, and a course left standing one more tick would let
/// it haul into a puzzle it had already won.
fn finish(world: &mut World, pylon: Entity) {
    let Some(course) = world.get::<Course>(pylon) else {
        return;
    };
    let (height, hauls) = (course.height(), course.hauls());

    world.entity_mut(pylon).remove::<Course>();
    // Mended before published: `publish` writes the `integrity` reading from
    // the resource, so the other order left `survey pylon` saying nought while
    // the rail, reading the resource directly, said fifty-six.
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
        // `At` is what a spell reads to know the pylon has finished; `Source`
        // files the line in `sanctum.log`. The archive shipped without the
        // latter and had an empty log from the day it was built (§19).
        .text(FieldName::At, PYLON)
        .text(FieldName::Source, PYLON)
        .role(Role::Success)
        .finish();

    // After the sentence about the work, for the reason `transmute` gives: the
    // barrier goes back up, *and then* the orb can hold another spell.
    tower::done(world, &tower::Work::at(PYLON), earned);
}

/// Publish everything a course has to say, as readings a spell can ask for.
///
/// Takes the pylon rather than reading `Cwd` — the bug the lens paid for: a
/// bound solver working while the player stands elsewhere would find no pylon
/// and leave every reading frozen at the last haul.
///
/// Clears and re-raises rather than diffing, as `research::refresh` does. Four
/// nodes at most, and a stale `potency` is worse than none.
pub(crate) fn publish(world: &mut World, pylon: Entity) {
    let Some(room) = room_of(world, pylon) else {
        return;
    };
    let course = world.get::<Course>(pylon).cloned();

    clear(world, pylon);
    // `integrity` is published whether or not a course is drawn: it is about
    // the tower rather than the puzzle, and a spell reads it to decide whether
    // to muster at all.
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
        // Only while the station holds a ward: `spell::watch` answers `is
        // empty` by asking whether a node has children, so a station always
        // carrying a `potency` could never be empty and the first two rungs of
        // every solver would be dead.
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
