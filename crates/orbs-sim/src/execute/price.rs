//! What a run costs beyond its inputs (§19, `0.17.1`).
//!
//! Two recipe fields, and stillness is the only recipe that sets either:
//! `quintessence`, taken when the run starts, and `most`, how many of the
//! product the tower may hold at once.

use bevy_ecs::prelude::*;

use crate::content::Recipes;
use crate::tower;

/// Why a run may not start, as a prose key and the fields it fills — or nothing.
///
/// Three different facts, so three sentences: the tower already has one, the
/// tower could never hold that much quintessence as it stands, and it merely
/// does not hold enough yet. Only the last is answered by waiting.
pub(super) fn refusal(
    world: &mut World,
    cost: u32,
    most: Option<u32>,
    made: Option<&str>,
) -> Option<(&'static str, Vec<(&'static str, String)>)> {
    if let (Some(most), Some(made)) = (most, made)
        && held_anywhere(world, made).saturating_add(brewing(world, made)) >= most
    {
        return Some(("wield_most", vec![("kind", made.to_owned())]));
    }
    // A worn barrier lowers the ceiling, and the pool never regenerates past
    // it: "you hold 12" would send a player to wait for what cannot arrive.
    let ceiling = tower::ceiling(world);
    if cost > ceiling {
        return Some((
            "wield_beyond",
            vec![
                ("kind", cost.to_string()),
                ("quantity", ceiling.to_string()),
            ],
        ));
    }
    let held = world.resource::<tower::Quintessence>().get();
    if cost > held {
        return Some((
            "wield_short",
            vec![("kind", cost.to_string()), ("quantity", held.to_string())],
        ));
    }
    None
}

/// Take a run's quintessence. Called only once `begin` has agreed, because it
/// can still refuse for the slot and a refusal must cost nothing.
pub(super) fn charge(world: &mut World, cost: u32) {
    if cost > 0 {
        world.resource_mut::<tower::Quintessence>().spend(cost);
    }
}

/// How many of `name` the tower holds, wherever it is — the arsenal, a shelf,
/// or an instrument waiting to be emptied.
fn held_anywhere(world: &mut World, name: &str) -> u32 {
    world
        .query::<(&tower::Name, &tower::Stock)>()
        .iter(world)
        .filter(|(held, _)| held.0 == name)
        .map(|(_, stock)| match stock {
            tower::Stock::Counted(units) => *units,
            tower::Stock::Endless => u32::MAX,
        })
        .fold(0, u32::saturating_add)
}

/// How many runs in flight will make `name`.
///
/// A brew has no product node until it lands, so stock alone would let a second
/// instrument start one beside it. One alembic and its busy lock prevent that
/// today; this is what holds when there are two.
fn brewing(world: &mut World, name: &str) -> u32 {
    let working: Vec<(Entity, String)> = world
        .query_filtered::<(Entity, &tower::Name), With<tower::Working>>()
        .iter(world)
        .map(|(at, instrument)| (at, instrument.0.clone()))
        .collect();
    let known = tower::known(world);
    let recipes = world.resource::<Recipes>();
    let count = working
        .into_iter()
        .filter(|(at, instrument)| {
            let holding = tower::holdings(world, *at);
            recipes
                .matching(instrument, &holding, &known)
                .is_some_and(|recipe| recipe.outputs().contains(&name))
        })
        .count();
    u32::try_from(count).unwrap_or(u32::MAX)
}
