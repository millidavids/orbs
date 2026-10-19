//! One word on a supply, the same on the wall and in a room (§19, *Supply*).
//!
//! Opening, raising and taking a ration are decided here once, so the bailey
//! and the calm cannot disagree about a refusal or about what a raise costs.

use bevy_ecs::prelude::*;

use super::open::{Open, Scope, Supplies, held_in_arsenal};
use super::rate::{Refusal, may_raise};
use crate::content::Rates;
use crate::tower::{self, Freshness};

/// What a word did to a supply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Word {
    /// Opened at this rate, nothing drawn yet.
    Opened(usize),
    /// Raised to this rate. What the current draw took already counts.
    Raised(usize),
    /// Opened again in the round it was stopped, as it stood then.
    Reopened(usize),
}

impl Word {
    /// The rate it runs at now.
    #[must_use]
    pub const fn at(self) -> usize {
        match self {
            Self::Opened(at) | Self::Raised(at) | Self::Reopened(at) => at,
        }
    }
}

/// Why `name` cannot open in `scope` whatever its rates say: none held, held
/// on a shelf rather than in the arsenal, or open in the other scope.
#[must_use]
pub fn barred(world: &World, name: &str, scope: Scope) -> Option<Refusal> {
    let supplies = world.resource::<Supplies>();
    let open = supplies.get(name).copied();
    if open.is_none() && supplies.stopped(name).is_none() && held_in_arsenal(world, name) == 0 {
        return Some(shelved(world, name).map_or_else(
            || Refusal::bare("spend_none"),
            |shelf| Refusal {
                key: "spend_shelved",
                args: vec![("source", shelf)],
            },
        ));
    }
    open.is_some_and(|open| open.scope != scope)
        .then(|| Refusal::bare("supply_elsewhere"))
}

/// Open `name`'s supply in `scope`, raise it a rate, or say why not.
///
/// What the current draw already took counts toward the next threshold, or a
/// raise would be refused for the ration it had just taken. A supply stopped
/// since its last draw opens again as it stood, so nothing is drawn twice.
///
/// A raise counts from the rate running, which draining can hold below the one
/// chosen, and never lowers the choice.
///
/// # Errors
///
/// The line saying why the rates refused it.
pub fn word(
    world: &mut World,
    name: &str,
    rates: &Rates,
    freshness: Freshness,
    scope: Scope,
) -> Result<Word, Refusal> {
    let supplies = world.resource::<Supplies>();
    let open = supplies.get(name).copied();
    let stopped = supplies
        .stopped(name)
        .copied()
        .filter(|was| was.scope == scope);
    if open.is_none()
        && let Some(was) = stopped
    {
        let mut supplies = world.resource_mut::<Supplies>();
        supplies.reopen(name);
        supplies.open(name, was);
        return Ok(Word::Reopened(was.running));
    }

    let held = held_in_arsenal(world, name);
    let drawn = open.map_or(0, |open| open.drawn);
    let at = may_raise(
        rates,
        held.saturating_add(drawn),
        freshness,
        open.map(|open| open.running),
    )
    .into_rate(held, drawn)?;
    let now = world.resource::<crate::tick::Tick>().get();
    let mut supplies = world.resource_mut::<Supplies>();
    if let Some(open) = supplies.get_mut(name) {
        open.chosen = open.chosen.max(at);
        open.running = at;
        return Ok(Word::Raised(at));
    }
    supplies.open(
        name,
        Open {
            scope,
            chosen: at,
            running: at,
            since: now,
            drawn: 0,
            applied: 0,
        },
    );
    Ok(Word::Opened(at))
}

/// What a ration took, and what the draw had applied before it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ration {
    /// Taken from the arsenal now.
    pub taken: u32,
    /// Applied by this draw before, so an effect adds only the difference.
    pub applied: u32,
}

/// Take what rate `at` draws beyond what this round or period already drew, and
/// mark `name` running there.
///
/// Never lowers what was drawn: a raise pays the difference, and what was paid
/// for stands.
pub fn ration(world: &mut World, name: &str, at: usize, draws: u32) -> Ration {
    let (drawn, applied) = world
        .resource::<Supplies>()
        .get(name)
        .map_or((0, 0), |open| (open.drawn, open.applied));
    let taken = draws.saturating_sub(drawn);
    if let Some(arsenal) = tower::keep(world) {
        tower::take(world, arsenal, name, taken);
    }
    if let Some(open) = world.resource_mut::<Supplies>().get_mut(name) {
        open.drawn = drawn.max(draws);
        open.running = at;
    }
    Ration { taken, applied }
}

/// Record that the current draw has applied `worth`. Never lowers it.
pub fn applied(world: &mut World, name: &str, worth: u32) {
    if let Some(open) = world.resource_mut::<Supplies>().get_mut(name) {
        open.applied = open.applied.max(worth);
    }
}

/// The shelf `name` waits on outside the arsenal, if any: where the player has
/// to move it from. A lectern's scrolls land in the archive's cabinet, and a
/// supply draws from the arsenal alone.
#[must_use]
pub fn shelved(world: &World, name: &str) -> Option<String> {
    let arsenal = tower::keep(world);
    tower::first_in_tree(world, |node| {
        let held = matches!(
            world.get::<tower::Stock>(node),
            Some(tower::Stock::Endless | tower::Stock::Counted(1..))
        );
        if !held
            || world
                .get::<tower::Name>(node)
                .is_none_or(|named| named.0 != name)
        {
            return None;
        }
        let shelf = world.get::<ChildOf>(node)?.parent();
        if Some(shelf) == arsenal {
            return None;
        }
        world.get::<tower::Name>(shelf).map(|named| named.0.clone())
    })
}
