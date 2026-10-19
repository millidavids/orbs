//! Drawing the arsenal's standing supplies into the siege (§19, *Supply*).
//!
//! A supply opened on the wall applies at once to the coming round, and after
//! every round that does not end the siege it draws again — at the fastest rate
//! the held count still meets, slower as it drains, shut when it runs dry.
//!
//! Draws take no randomness, and a round's draws apply in one order whatever
//! order they were opened in: the room first, then bodies, then fight, then the
//! dice. A draw that would do nothing is skipped and the supply stays open.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use super::shared::{RAMPART, say};
use crate::content::{Prose, Spendable, Spendables, SupplyKind};
use crate::parser::Verb;
use crate::session::Scrollback;
use crate::tower::supply::{self, Scope, Supplies};
use crate::tower::{self, siege::Siege};

/// Where `kind` comes in a round's draws.
const fn order(kind: SupplyKind) -> u8 {
    match kind {
        SupplyKind::Fortify => 0,
        SupplyKind::Troops => 1,
        SupplyKind::Vigour => 2,
        SupplyKind::Still
        | SupplyKind::Bonus
        | SupplyKind::Advantage
        | SupplyKind::Disadvantage
        | SupplyKind::Upgrade => 3,
    }
}

/// How fresh `named` is to a siege: as at `defend` or now, whichever is better.
pub(super) fn freshness(world: &World, rampart: Entity, named: &str) -> tower::Freshness {
    let now = tower::freshness_of(world, named);
    world
        .get::<Siege>(rampart)
        .and_then(|siege| siege.freshness.get(named).copied())
        .map_or(now, |then| then.better(now))
}

/// Whether a draw of `entry` would change anything this round.
const fn useful(siege: &Siege, entry: &Spendable) -> bool {
    match entry.kind {
        SupplyKind::Vigour => siege.garrison.vigour < siege.full(),
        SupplyKind::Troops => siege.garrison.count < siege.room,
        SupplyKind::Fortify | SupplyKind::Still => true,
        SupplyKind::Bonus
        | SupplyKind::Advantage
        | SupplyKind::Disadvantage
        | SupplyKind::Upgrade => siege.intent.answered(),
    }
}

/// Apply a draw of `entry` at rate `at`, beyond `already` applied this round.
/// Returns what the round has now applied.
fn apply(siege: &mut Siege, named: &str, entry: &Spendable, at: usize, already: u32) -> u32 {
    let Some(rate) = entry.rates.get(at) else {
        return already;
    };
    let worth = entry.worth_at(rate);
    let more = worth.saturating_sub(already);
    match entry.kind {
        SupplyKind::Vigour => siege.heal(more),
        SupplyKind::Troops => {
            siege.reinforce(more);
        }
        SupplyKind::Fortify => siege.fortify(more),
        // Stillness ends the siege where it is opened, and draws nothing here.
        SupplyKind::Still => {}
        SupplyKind::Bonus
        | SupplyKind::Advantage
        | SupplyKind::Disadvantage
        | SupplyKind::Upgrade => {
            if let Some(effect) = entry.effect_at(rate) {
                siege.restage(named, effect);
            }
        }
    }
    worth
}

/// A draw into the log and nowhere else: the pane draws what the player did.
fn log_draw(world: &mut World, named: &str, count: u32) {
    let message = world.resource::<Prose>().line(
        "supply_drawn",
        &[("name", named), ("count", &count.to_string())],
    );
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Hold.canonical())
        .text(FieldName::Message, &message)
        .text(FieldName::Source, RAMPART)
        .count(FieldName::Quantity, u64::from(count))
        .quiet()
        .finish();
}

/// Draw `named` at rate `at` for the coming round, on top of what this round
/// already drew. Returns whether it drew, or was skipped as doing nothing.
pub(super) fn draw_now(world: &mut World, rampart: Entity, named: &str, at: usize) -> bool {
    let Some(entry) = world.resource::<Spendables>().get(named).cloned() else {
        return false;
    };
    draw_entry(world, rampart, named, &entry, at)
}

/// [`draw_now`] with the row already in hand.
fn draw_entry(
    world: &mut World,
    rampart: Entity,
    named: &str,
    entry: &Spendable,
    at: usize,
) -> bool {
    let Some(rate) = entry.rates.get(at).copied() else {
        return false;
    };
    if !world
        .get::<Siege>(rampart)
        .is_some_and(|siege| useful(siege, entry))
    {
        return false;
    }
    let ration = supply::ration(world, named, at, rate.draws);
    let now = {
        let Some(mut siege) = world.get_mut::<Siege>(rampart) else {
            return false;
        };
        apply(&mut siege, named, entry, at, ration.applied)
    };
    supply::applied(world, named, now);
    log_draw(world, named, ration.taken);
    true
}

/// Draw every open supply for the round about to come.
///
/// After a resolved round that did not end the siege. The room a fortify
/// supply held is taken away first and held again by its draw, so closing one
/// shrinks the line on the next round.
pub(super) fn draw_round(world: &mut World, rampart: Entity) {
    // Only the rows that are open, borrowed out once: the whole table is not
    // the round's business.
    let mut open: Vec<(String, Option<Spendable>)> = {
        let spendables = world.resource::<Spendables>();
        world
            .resource::<Supplies>()
            .names(Scope::Siege)
            .into_iter()
            .map(|named| {
                let entry = spendables.get(&named).cloned();
                (named, entry)
            })
            .collect()
    };
    open.sort_by_key(|(_, entry)| entry.as_ref().map_or(u8::MAX, |entry| order(entry.kind)));

    if let Some(mut siege) = world.get_mut::<Siege>(rampart) {
        siege.reset_room();
    }
    world.resource_mut::<Supplies>().new_round();
    for (named, entry) in open {
        let Some(entry) = entry else {
            world.resource_mut::<Supplies>().close(&named);
            continue;
        };
        let held = supply::held_in_arsenal(world, &named);
        let fresh = freshness(world, rampart, &named);
        let Some(was) = world.resource::<Supplies>().get(&named).copied() else {
            continue;
        };
        // Stale or short: the line names which, since the answer to one is
        // making more and to the other holding more.
        let slowed = supply::slowed(&entry.rates, held, fresh, was.chosen);
        let Some(at) = supply::running(&entry.rates, held, fresh, was.chosen) else {
            world.resource_mut::<Supplies>().close(&named);
            let key = supply::ran_low(Scope::Siege, slowed, true);
            say(world, Verb::Hold, key, &[("name", &named)], Role::Cost);
            continue;
        };
        if at < was.running {
            let count = entry.rates.get(at).map_or(0, |rate| rate.draws);
            say(
                world,
                Verb::Hold,
                supply::ran_low(Scope::Siege, slowed, false),
                &[("name", &named), ("count", &count.to_string())],
                Role::Cost,
            );
        }
        if let Some(open) = world.resource_mut::<Supplies>().get_mut(&named) {
            open.running = at;
        }
        draw_entry(world, rampart, &named, &entry, at);
    }
    if let Some(mut siege) = world.get_mut::<Siege>(rampart) {
        siege.clamp_line();
    }
    supply::gauge(world);
}

/// Give back what this round's roll-modifier draws took, for a siege stilled
/// before the round could use them. A heal, a room or a refill has already
/// acted, and stillness's own ration is its price.
///
/// A supply stopped this round counts too: its modifier stood, and stillness
/// throws it away with the rest.
pub(super) fn hand_back(world: &mut World, stilling: &str) {
    let returned: Vec<(String, u32)> = {
        let supplies = world.resource::<Supplies>();
        let spendables = world.resource::<Spendables>();
        supplies
            .iter()
            .chain(supplies.iter_stopped())
            .filter(|(named, open)| {
                *named != stilling
                    && open.scope == Scope::Siege
                    && open.drawn > 0
                    && spendables.get(named).is_some_and(Spendable::modifies_rolls)
            })
            .map(|(named, open)| (named.to_owned(), open.drawn))
            .collect()
    };
    let Some(arsenal) = tower::keep(world) else {
        return;
    };
    for (named, drawn) in returned {
        let kind = tower::kind_of(world, &named);
        tower::give(world, arsenal, &named, kind, drawn);
    }
}
