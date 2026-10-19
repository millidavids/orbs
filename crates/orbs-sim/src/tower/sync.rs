//! Keeping a node's reading children in step with what it should say.
//!
//! The rule `stocktake` set: write only on a change, so an idle tick issues no
//! `NodeId` and a watched hour and a `meditate`-collapsed one number the tower
//! alike. One body for every system that publishes readings, so a fix to it
//! lands once.

use bevy_ecs::prelude::*;

use super::node::{Name, children_of};
use super::stock::Stock;

/// Make `node`'s children named in `family` say exactly `want`, in order: a
/// word with a count is a counted reading, one without a plain one.
///
/// Children outside `family` are left alone. When what is said already matches,
/// nothing is touched; otherwise every child in `family` goes and `want` is
/// raised afresh, in order.
pub fn sync_readings(
    world: &mut World,
    node: Entity,
    family: &[&str],
    want: &[(&str, Option<u32>)],
) {
    let said: Vec<(Entity, String, Option<u32>)> = children_of(world, node)
        .into_iter()
        .filter_map(|child| {
            let name = world.get::<Name>(child)?.0.clone();
            family.contains(&name.as_str()).then(|| {
                let count = match world.get::<Stock>(child) {
                    Some(Stock::Counted(count)) => Some(*count),
                    _ => None,
                };
                (child, name, count)
            })
        })
        .collect();
    let matches = said.len() == want.len()
        && said
            .iter()
            .zip(want)
            .all(|((_, name, count), (word, wanted))| name == word && count == wanted);
    if matches {
        return;
    }
    for (stale, _, _) in said {
        world.entity_mut(stale).despawn();
    }
    for (word, count) in want {
        match count {
            Some(count) => {
                super::raise_count(world, node, word, *count);
            }
            None => {
                super::raise_reading(world, node, word);
            }
        }
    }
}
