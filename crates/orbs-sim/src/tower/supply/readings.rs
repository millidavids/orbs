//! What an arsenal item says about its supply, for a spell to ask (§19,
//! *Supply*): `draw`, the rate it runs at, and `drained`.

use bevy_ecs::prelude::*;

use super::open::Supplies;
use crate::content::Spendables;
use crate::tower;

/// The rate a supply is running at, as a count: one is the slowest.
///
/// `draw`, where the plan said `flow`: that scored 667 against `follow`, the
/// maze's verb, once the sweep could see the arsenal's readings.
pub const DRAW: &str = "draw";

/// Held, but below the slowest rate: nothing can draw it.
pub const DRAINED: &str = "drained";

/// The words an arsenal item answers about its supply.
#[must_use]
pub fn readings() -> Vec<&'static str> {
    vec![DRAW, DRAINED]
}

/// Whether a count in the arsenal moved since this last asked: with a change to
/// [`Supplies`], what the schedule runs [`gauge`] on, so an idle tick walks
/// nothing.
#[must_use]
pub fn restocked(
    changed: Query<&ChildOf, Changed<tower::Stock>>,
    keeps: Query<(), With<tower::Keep>>,
) -> bool {
    changed.iter().any(|parent| keeps.contains(parent.parent()))
}

/// Keep every arsenal item's `draw` and `drained` in step with its supply.
///
/// `stocktake`'s rule: it writes only on a change, so an idle tick issues no
/// `NodeId`, and it touches only its own two words. Called again by every word
/// and draw that moves a supply, so a spell asking twice in one tick reads the
/// first answer's effect.
pub fn gauge(world: &mut World) {
    let Some(arsenal) = tower::keep(world) else {
        return;
    };
    for node in tower::children_of(world, arsenal) {
        if world.get::<tower::Stock>(node).is_none() {
            continue;
        }
        let Some(named) = world.get::<tower::Name>(node).map(|name| name.0.clone()) else {
            continue;
        };
        // A scroll's calm table opens at the same count, which
        // `a_scroll_opens_at_one_count_in_a_room_and_on_the_wall` holds.
        let Some(slowest) = world
            .resource::<Spendables>()
            .get(&named)
            .map(|entry| entry.rates.slowest().opens)
        else {
            continue;
        };
        // The node in hand, not `open::held_in_arsenal`: that walks the tower to find
        // the arsenal again, once per item per tick.
        let held = match world.get::<tower::Stock>(node) {
            Some(tower::Stock::Counted(count)) => *count,
            Some(tower::Stock::Endless) => u32::MAX,
            None => 0,
        };
        let want: Vec<(&str, Option<u32>)> = match world
            .resource::<Supplies>()
            .get(&named)
            .map(|open| open.running)
        {
            Some(running) => vec![(DRAW, u32::try_from(running + 1).ok())],
            None if held > 0 && held < slowest => vec![(DRAINED, None)],
            None => Vec::new(),
        };
        tower::sync_readings(world, node, &readings(), &want);
    }
}
