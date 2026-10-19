//! A bare hold's bodies, brought home to the garrison (§19, *Supply*).

use bevy_ecs::prelude::*;

use crate::content::Prose;
use crate::tower;

/// Send `troops` to the standing garrison, or to its reserve mid-siege, never
/// past its most, and say where they went and how many were turned away.
pub(super) fn enlist(world: &mut World, troops: u32) -> String {
    let fighting = tower::siege::besieged(world);
    let (joined, turned) = tower::siege::enlist(world, troops);
    let prose = world.resource::<Prose>();
    let counts = [
        ("count", joined.to_string()),
        ("quantity", turned.to_string()),
    ];
    let counts: Vec<(&str, &str)> = counts
        .iter()
        .map(|(field, value)| (*field, value.as_str()))
        .collect();
    match (joined, turned, fighting) {
        (0, _, _) => prose.line("summon_garrison_full", &[]),
        (_, 0, true) => prose.counted("summon_reserve", joined),
        (_, _, true) => prose.line("summon_reserve_some", &counts),
        (_, 0, false) => prose.counted("summon_garrison", joined),
        (_, _, false) => prose.line("summon_garrison_some", &counts),
    }
}
