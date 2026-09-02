//! Every reading the bailey publishes — the half the scripting rests on (§5.1).
//!
//! **A reading is published only while it is true, and often only while
//! something is standing.** That is not tidiness: `spell::watch` answers `is
//! empty` by asking whether a node has children, so a band or an area that
//! always carried a count could never be empty, and the first rung of every
//! solver would be dead. The sanctum records the same rule for `potency` and the
//! satchel records it a third time.

use bevy_ecs::prelude::*;

use super::shared::fixture;
use crate::execute::readings::{clear, reading, room_of};
use crate::parser::Verb;
use crate::tower::{self, siege, siege::Siege};

/// Republish every reading the bailey publishes.
///
/// **Takes the rampart rather than reading `Cwd`**, which is the bug the lens
/// paid for and the sanctum recorded: a bound solver working while the player
/// stands in the laboratory would otherwise find no rampart, publish nothing,
/// and leave every reading frozen at the last round.
pub(crate) fn publish(world: &mut World, rampart: Entity) {
    let Some(room) = room_of(world, rampart) else {
        return;
    };
    let siege = world.get::<Siege>(rampart).cloned();
    // **The tower's pool, not the siege's.** Quintessence is shared with the
    // forge now, so what the coffer reports is what the whole tower holds — and
    // a charm laid mid-siege is visible here as dice you can no longer pledge.
    let pool = world.resource::<tower::Quintessence>().get();

    clear(world, rampart);
    if let Some(siege) = &siege {
        tower::raise_count(world, rampart, siege::TURNS, siege.turns);
        // **The reading a `repeat until` needs.** `routed` is a question about
        // one band, and the shipped solver asked it of the *enemy* — which is
        // never true when the garrison is the side that broke, so a lost siege
        // left the loop spinning for ever. This is the question about the siege.
        if !siege.running() {
            tower::raise_reading(world, rampart, siege::LIFTED);
        }
        // The telegraphed intent, as a word a spell can ask for. This is the
        // whole of what "show the odds before the commitment" costs.
        tower::raise_reading(world, rampart, siege.intent.word());
    }

    // **The four areas, and this is the half the scripting rests on.** A solver
    // deciding *where to pledge* asks these and nothing else:
    //
    //   if the buckler is empty        — nothing behind it yet
    //   if the line has 2 pledged      — how many dice are on it
    //   if the sortie has 12 ceiling   — the best it could come to
    //   if the buckler has wasted      — the intent will throw it away
    //
    // **Published only when something is pledged**, apart from `wasted`, for the
    // rule the bands record above: `spell::watch` answers `is empty` by asking
    // whether a node has children, so an area that always carried a count could
    // never be empty and *find the empty area* — the commonest rung a solver
    // wants — would be dead.
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
        // **The warning is published whether or not anything is on it**, which
        // is the one exception to the rule above and the reason it is worth
        // making: a solver wants to know *not to pledge here* before it has,
        // and a reading that only appeared after the mistake would be useless.
        if !area.answers(siege.intent) {
            tower::raise_reading(world, node, siege::MOOT);
        }
    }

    // **The coffer says which dice are free *and affordable*, by name.** A die
    // held but unpayable is deliberately absent, so `if the coffer has d20` means
    // *"I hold it and can pay for it"* — one question, a natural positive, and no
    // grammar at all.
    //
    // The alternative was the numeric comparison, and it is a trap in the
    // positive: `has more quintessence than the d20` is **strict**
    // (`watch.rs:180`), so it excludes the die you can exactly afford, and there
    // is no *at least as many* comparative to reach for. Only the negative
    // spelling is correct, and a mechanic whose natural sentence is a double
    // negative is one §6 would not recognise. This is the maze's `spoil`/`exit`
    // pattern instead: the world derives the word, the spell asks for it.
    //
    // The pool rides the coffer too, so a spell that wants to reason about what
    // is left — rather than merely what it can afford right now — still can.
    if let Some(coffer) = reading(world, room, siege::COFFER) {
        clear(world, coffer);
        if let Some(siege) = &siege
            && siege.running()
        {
            for die in siege.affordable(pool) {
                tower::raise_reading(world, coffer, die.word());
            }
            // **Only while there is any**, which is `potency`'s rule and
            // `raise_count`'s contract: a nought count would make the coffer
            // permanently non-empty and `if the coffer is empty` — *"I can pledge
            // nothing"* — dead.
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

        // **Only while the band is standing**, which is load-bearing rather
        // than tidy: `spell::watch` answers `is empty` by asking whether a node
        // has children, so a band that always carried a count could never be
        // empty and the first rung of every solver would be dead. The sanctum
        // records the same rule for `potency`.
        if band.count > 0 {
            tower::raise_count(world, node, siege::SPEARS, band.count);
            tower::raise_count(world, node, siege::METTLE, band.vigour);
        } else {
            tower::raise_reading(world, node, siege::ROUTED);
        }

        // **The odds, as a number a spell can weigh.** This is the whole of what
        // the hand player has always had and the solver has not: `enemy_roll`
        // and `garrison_roll` compose without drawing, so the chance is knowable
        // before the commitment — §5.1's fairness rule, finally symmetric.
        //
        // Only while the band is standing, for the reason `spears` and `mettle`
        // are: a routed side's aim is not nought, it is meaningless.
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
            // **Only while some are standing**, which it was not: this sat
            // outside the branch above and published `foes = 0` for a routed
            // enemy, against `raise_count`'s own *"never called with nought"*
            // (`build.rs:1160`). A spurious nought on `survey enemy`, and a
            // contract the rest of this file keeps.
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
pub(crate) fn refresh(world: &mut World) {
    let Some(rampart) = fixture(world) else {
        return;
    };
    publish(world, rampart);
}

/// What each die costs, on the die's own node.
///
/// **Raised once, at construction, and never again** — which is the whole
/// correction. A die's price is a fact about the die rather than about a fight,
/// so it does not belong in [`publish`], and putting it there meant it did not
/// exist until the first bailey verb: `survey d20` on a fresh tower answered
/// *"the d20 holds nothing"*, and an absent reading is nought.
///
/// That made the affordability guard every solver ships — `not the coffer has
/// fewer quintessence than the d20` — compare nought against nought and answer
/// **yes** on a tower with no pool. The sanctum's `integrity` is the same defect
/// and the same fix; see `Sim::bare`.
///
/// `for each die` walks these nodes (`build.rs`), so this is also what lets a
/// solver weigh one die against another.
pub(crate) fn publish_dice(world: &mut World) {
    // **Found by walking for the operation, never through `Cwd`.** This runs at
    // construction, where the player is standing at the root — so the ordinary
    // `fixture` answers `None` and the whole bootstrap silently did nothing, the
    // exact shape `publish`'s own doc warns about two functions up. `pylon`'s
    // Cwd-free lookup is the precedent.
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
        // **Named rather than skipped.** A die in `POOL` with no node is a die a
        // player can pledge and no spell can price, which is the silent shape
        // `every_die_the_pool_holds_has_a_node` now fails the build over.
        if let Some(node) = reading(world, room, die.word()) {
            clear(world, node);
            tower::raise_count(world, node, siege::QUINTESSENCE, siege::cost_of(die));
        }
    }
}
