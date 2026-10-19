//! What a run costs beyond its inputs (§19, `0.17.1`).
//!
//! One recipe field, and stillness is the only recipe that sets it:
//! `quintessence`, taken when the run starts. `most`, how many the tower might
//! hold, is retired (§19, *Supply*): a supply's threshold is the price now.

use bevy_ecs::prelude::*;

use crate::tower;

/// Why a run may not start, as a prose key and the fields it fills — or nothing.
///
/// Two different facts, so two sentences: the tower could never hold that much
/// quintessence as it stands, and it merely does not hold enough yet. Only the
/// second is answered by waiting.
pub(super) fn refusal(
    world: &World,
    cost: u32,
) -> Option<(&'static str, Vec<(&'static str, String)>)> {
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
