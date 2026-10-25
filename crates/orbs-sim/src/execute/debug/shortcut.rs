//! The words that leave a puzzle one move from finished, and `debug_swap`.

use bevy_ecs::prelude::*;

use super::door::nothing_to_give;
use super::word::{CIRCLE, COURSE, SIEGE};
use crate::tower::{self, puzzle::Open};

/// Substitute a reagent on a shelf where the player is standing.
///
/// The first pile by name, so a dump reaches the same one every time
/// (`tower::node`).
pub(super) fn swap(world: &mut World) {
    let cwd = world.resource::<tower::Cwd>().0;
    let mut piles: Vec<(String, Entity)> = tower::children_of(world, cwd)
        .into_iter()
        .flat_map(|place| tower::children_of(world, place))
        .filter(|node| world.get::<tower::Stock>(*node).is_some())
        .filter_map(|node| {
            world
                .get::<tower::Name>(node)
                .map(|name| (name.0.clone(), node))
        })
        .collect();
    piles.sort_unstable();

    if let Some((name, node)) = piles.first() {
        let claimed = format!("{name}-");
        tower::substitute(world, *node, &claimed);
    }
}

/// Hand the open ward's answer to the aperture, so the next press breaks it.
pub(super) fn ward(world: &mut World) {
    if let Some(node) = tower::here_with::<tower::Ward>(world)
        && let Some(mut ward) = world.get_mut::<tower::Ward>(node)
    {
        ward.give_away();
    }
}

/// Stack the standing course but its smallest ward, one haul from finished.
pub(super) fn course(world: &mut World) {
    if let Some(node) = tower::here_with::<tower::Course>(world)
        && let Some(mut course) = world.get_mut::<tower::Course>(node)
    {
        course.give_away();
        // Republished, unlike `debug_ward`: a solver reads the `potency` this
        // moved to choose the next haul.
        crate::execute::refresh(world, Open::Course);
        return;
    }
    nothing_to_give(world, COURSE, "muster_nothing_to_give");
}

/// Limn the circle to one of the waiting beast's solutions, so the next
/// `summon` holds it.
pub(super) fn circle(world: &mut World) {
    if let Some(node) = tower::here_with::<tower::circle::Beast>(world)
        && let Some(mut beast) = world.get_mut::<tower::circle::Beast>(node)
        && let Some(solution) = beast.solution()
    {
        for (glyph, humour) in tower::circle::Glyph::ALL.into_iter().zip(solution) {
            beast.limn(glyph, humour);
        }
        // Republished, for `debug_course`'s reason: a spell reads the glyphs'
        // readings next, and stale they still name the opening.
        crate::execute::refresh(world, Open::Beast);
        return;
    }
    nothing_to_give(world, CIRCLE, "summon_nothing_to_give");
}

/// Thin the enemy to one, so the next `hold` is the round that ends it.
pub(super) fn siege(world: &mut World) {
    if let Some(node) = tower::here_with::<tower::Siege>(world)
        && let Some(mut siege) = world.get_mut::<tower::Siege>(node)
        && siege.running()
    {
        siege.give_away();
        // Republished, as for `debug_course`: a decision tree reads `foes`.
        crate::execute::refresh_rampart(world);
        return;
    }
    nothing_to_give(world, SIEGE, "defend_nothing_to_give");
}
