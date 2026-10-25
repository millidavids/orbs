//! Every reading the bailey publishes — the half the scripting rests on (§5.1).
//!
//! A reading is published only while it is true: `spell::watch` answers `is
//! empty` by asking for children, so an always-counted area could never be
//! empty. The sanctum's `potency` follows the same rule.

use bevy_ecs::prelude::*;

// Only [`refresh`] resolves the rampart by `Cwd`, and that is `debug_siege`'s.
#[cfg(debug_assertions)]
use super::shared::fixture;
use crate::execute::readings::{clear, reading, room_of};
use crate::parser::Verb;
use crate::tower::{self, siege, siege::Siege};

/// Republish every reading the bailey publishes.
///
/// Takes the rampart, never `Cwd`, the lens's rule.
pub(crate) fn publish(world: &mut World, rampart: Entity) {
    let Some(room) = room_of(world, rampart) else {
        return;
    };
    let siege = world.get::<Siege>(rampart).cloned();
    // The tower's pool, not the siege's: quintessence is shared with the forge,
    // so the coffer reports what the whole tower holds and a charm laid
    // mid-siege shows here as dice you can no longer pledge.
    let pool = world.resource::<tower::Quintessence>().get();

    clear(world, rampart);
    if let Some(siege) = &siege {
        tower::raise_count(world, rampart, siege::TURNS, siege.turns);
        // The reading a `repeat until` needs: `routed` asks about one band, and
        // the shipped solver asked it of the *enemy*, which is never true when
        // the garrison broke — so a lost siege left the loop spinning.
        if !siege.running() {
            tower::raise_reading(world, rampart, siege::LIFTED);
        }
        // The telegraphed intent, as a word a spell can ask for. This is the
        // whole of what "show the odds before the commitment" costs.
        tower::raise_reading(world, rampart, siege.intent.word());
    }

    // The four areas, the half the scripting rests on: a solver deciding *where
    // to pledge* asks these and nothing else.
    //
    //   if the buckler is empty        — nothing behind it yet
    //   if the line has 2 pledged      — how many dice are on it
    //   if the sortie has 12 ceiling   — the best it could come to
    //   if the buckler has wasted      — the intent will throw it away
    //
    // Published only when something is pledged, apart from `wasted`.
    for area in siege::Area::ALL {
        let Some(node) = reading(world, room, area.word()) else {
            continue;
        };
        clear(world, node);
        let Some(siege) = &siege else { continue };
        if !siege.running() {
            continue;
        }

        if !siege.pledged(area).is_empty() {
            let (_, high) = siege.range(area);
            tower::raise_count(world, node, siege::CEILING, high);
        }
        // The exception: a solver wants *not here* before it pledges.
        if !area.answers(siege.intent) {
            tower::raise_reading(world, node, siege::MOOT);
        }
    }

    // The coffer names the dice free *and affordable*, so `if the coffer has
    // d20` is one natural positive. The comparison is strict and excludes the
    // die you can exactly afford; the world derives the word, as the maze's
    // `spoil` does. The pool rides the coffer too.
    if let Some(coffer) = reading(world, room, siege::COFFER) {
        clear(world, coffer);
        if let Some(siege) = &siege
            && siege.running()
        {
            for die in siege.affordable(pool) {
                tower::raise_reading(world, coffer, die.word());
            }
            // Only while there is any, which is `potency`'s rule and
            // `raise_count`'s contract: a nought count makes the coffer
            // permanently non-empty and kills `if the coffer is empty`.
            if pool > 0 {
                tower::raise_count(world, coffer, siege::QUINTESSENCE, pool);
            }
        }
    }

    for name in [siege::GARRISON, siege::ENEMY] {
        let Some(node) = reading(world, room, name) else {
            continue;
        };
        clear(world, node);
        let Some(siege) = &siege else { continue };
        let band = if name == siege::GARRISON {
            siege.garrison
        } else {
            siege.enemy
        };

        // Only while the band is standing — load-bearing rather than tidy, for
        // the module header's reason.
        if band.count > 0 {
            tower::raise_count(world, node, siege::SPEARS, band.count);
            tower::raise_count(world, node, siege::METTLE, band.vigour);
        } else {
            tower::raise_reading(world, node, siege::ROUTED);
        }

        // The odds as a number a spell can weigh, composed without drawing
        // (§5.1). Only while the band stands: a routed side's aim is
        // meaningless, not nought.
        if band.count > 0 {
            let odds = if name == siege::GARRISON {
                siege.garrison_roll().chance()
            } else {
                siege.enemy_roll().chance()
            };
            if odds > 0 {
                tower::raise_count(world, node, siege::AIM, odds);
            }
        }

        if name == siege::GARRISON {
            if siege.few() {
                tower::raise_reading(world, node, siege::FEW);
            }
            if siege.hurt() {
                tower::raise_reading(world, node, siege::HURT);
            }
        } else {
            // Only while some are standing, which it was not: this sat outside
            // the branch above and published `foes = 0` for a routed enemy,
            // against `raise_count`'s own "never called with nought".
            if band.count > 0 {
                tower::raise_count(world, node, siege::FOES, band.count);
            }
            if siege.outnumbered() {
                tower::raise_reading(world, node, siege::OUTNUMBERED);
            }
            if siege.massed() {
                tower::raise_reading(world, node, siege::MASSED);
            }
        }
    }
}

/// Republish from wherever the player is standing, for the typed path.
///
/// `debug_siege`'s alone, and gated as it is.
#[cfg(debug_assertions)]
pub(crate) fn refresh(world: &mut World) {
    let Some(rampart) = fixture(world) else {
        return;
    };
    publish(world, rampart);
}

/// What each die costs, on the die's own node.
///
/// Raised once at construction: a price is the die's, not a fight's, and an
/// absent reading is nought, which an affordability guard reads as free — the
/// sanctum's `integrity` defect. `for each die` walks these.
pub(crate) fn publish_dice(world: &mut World) {
    // Walked for by operation, never through `Cwd`, which is the root at
    // construction — `pylon`'s lookup.
    let Some(rampart) = world.iter_entities().find_map(|entity| {
        entity
            .get::<crate::tower::Operation>()
            .is_some_and(|operation| operation.0 == Verb::Defend)
            .then(|| entity.id())
    }) else {
        return;
    };
    let Some(room) = room_of(world, rampart) else {
        return;
    };
    for die in siege::POOL {
        // Named rather than skipped: a die in `POOL` with no node is one a
        // player can pledge and no spell can price, which
        // `every_die_the_pool_holds_has_a_node` now fails the build over.
        if let Some(node) = reading(world, room, die.word()) {
            clear(world, node);
            tower::raise_count(world, node, siege::QUINTESSENCE, siege::cost_of(die));
        }
    }
}
