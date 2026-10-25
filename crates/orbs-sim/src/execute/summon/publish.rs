//! What the circle says about itself, as readings a spell can ask for.

use bevy_ecs::prelude::*;

use super::super::readings::{clear, reading, room_of};
use crate::tower::{
    self,
    circle::{self, Beast, Glyph},
};

/// Publish everything the circle has to say.
///
/// Takes the circle, never `Cwd`, the lens's rule. Clears and re-raises, as
/// every domain does.
pub(crate) fn publish(world: &mut World, at: Entity) {
    let Some(room) = room_of(world, at) else {
        return;
    };
    let fervour = world
        .get::<Beast>(at)
        .map(|beast| beast.shape().temper().fervour());

    clear(world, at);
    if let Some(fervour) = fervour {
        // Never nought — the table refuses such tempers — so no children means
        // exactly no beast.
        tower::raise_count(world, at, circle::FERVOUR, fervour);
    }

    for glyph in Glyph::ALL {
        publish_glyph(world, room, at, glyph);
    }
}

/// Republish one glyph — what a `limn` changed, and nothing else.
///
/// Not [`publish`]: a bound `taming` limns up to 216 times a beast, and
/// re-raising four readings each time churns ids.
pub(crate) fn republish_glyph(world: &mut World, at: Entity, glyph: Glyph) {
    if let Some(room) = room_of(world, at) {
        publish_glyph(world, room, at, glyph);
    }
}

fn publish_glyph(world: &mut World, room: Entity, at: Entity, glyph: Glyph) {
    let Some(node) = reading(world, room, glyph.word()) else {
        return;
    };
    // What it is limned with, never whether it is right, which would solve
    // the circle (§19). A dark glyph says nothing.
    let limned = world
        .get::<Beast>(at)
        .filter(|beast| beast.shape().lights(glyph))
        .map(|beast| beast.humour(glyph).word());
    clear(world, node);
    if let Some(word) = limned {
        tower::raise_reading(world, node, word);
    }
}
