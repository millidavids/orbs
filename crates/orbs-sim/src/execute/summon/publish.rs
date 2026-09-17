//! What the circle says about itself, as readings a spell can ask for.

use bevy_ecs::prelude::*;

use super::super::readings::{clear, reading, room_of};
use crate::tower::{
    self,
    circle::{self, Beast, Glyph},
};

/// Publish everything the circle has to say.
///
/// **Takes the circle rather than reading `Cwd`**, which is the lens's recorded
/// bug avoided by construction: whoever calls this has already found the circle,
/// and a publisher that looked again could look in the wrong room.
///
/// Clears and re-raises rather than diffing, as every domain does.
pub(crate) fn publish(world: &mut World, at: Entity) {
    let Some(room) = room_of(world, at) else {
        return;
    };
    let fervour = world
        .get::<Beast>(at)
        .map(|beast| beast.shape().temper().fervour());

    clear(world, at);
    if let Some(fervour) = fervour {
        // **Never nought**, because a temper lit on every row or none is refused
        // when the table is built, and no lesser temper is either — so the
        // circle having no children means exactly *no beast*, which is what
        // `is empty` answers.
        tower::raise_count(world, at, circle::FERVOUR, fervour);
    }

    for glyph in Glyph::ALL {
        publish_glyph(world, room, at, glyph);
    }
}

/// Republish one glyph — what a `limn` changed, and nothing else.
///
/// **Not [`publish`]**, which clears and re-raises the circle's `fervour` and all
/// three glyphs. A limn moves one glyph and never the temper, and a bound
/// `taming` limns up to 216 times a beast: republishing everything despawned and
/// re-raised four readings each time to say three of them again with new ids,
/// which is the churn `call` was changed to stop.
pub(crate) fn republish_glyph(world: &mut World, at: Entity, glyph: Glyph) {
    if let Some(room) = room_of(world, at) {
        publish_glyph(world, room, at, glyph);
    }
}

fn publish_glyph(world: &mut World, room: Entity, at: Entity, glyph: Glyph) {
    let Some(node) = reading(world, room, glyph.word()) else {
        return;
    };
    // **What it is limned with, and nothing else** — the lens's socket carrying
    // its sigil. Never whether it is *right*: a verdict on a glyph would be the
    // orb solving the circle, and §19 records the lens paying for exactly that.
    // **A dark glyph says nothing**: at a lesser circle the outer two are not
    // part of it, so a spell asking what they are limned with hears nothing
    // rather than the opening.
    let limned = world
        .get::<Beast>(at)
        .filter(|beast| beast.shape().lights(glyph))
        .map(|beast| beast.humour(glyph).word());
    clear(world, node);
    if let Some(word) = limned {
        tower::raise_reading(world, node, word);
    }
}

/// Publish the circle where the player is standing — for a tester's shortcut,
/// which changes the glyphs without going through `limn`.
///
/// **Debug-only, at its definition**, as `defend`'s is: `debug_circle` is its one
/// caller, so a release build would find it unused.
#[cfg(debug_assertions)]
pub(crate) fn refresh(world: &mut World) {
    if let Some(at) = super::verbs::fixture(world) {
        publish(world, at);
    }
}
