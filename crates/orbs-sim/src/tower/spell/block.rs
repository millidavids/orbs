//! Whether an instruction can proceed, asked without refusing it.
//!
//! A player at a busy mortar is told so (`tower::work::refuse_busy`); a script
//! should wait, or the refusal would log once per tick for the whole run. So the
//! predicate and the refusal are separate, and this is the predicate.
//!
//! Both tiers, the instrument and the tower's one production slot: at capacity
//! 1 the slot is what every multi-instrument brew hits.

use bevy_ecs::prelude::*;

use crate::parser::{Intent, Verb};
use crate::tower::{self, Cwd};

/// Why an instruction cannot run yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Blocked {
    /// The instrument this instruction needs is mid-run.
    Instrument {
        /// What it is called, for the log.
        name: String,
        /// What it is doing, as `tower::Busy` words it.
        doing: String,
    },
    /// The tower's production slot is taken by something else (§9, §11.5).
    Slot {
        /// Where the work holding it is happening.
        name: String,
        /// What is being done there, as the verb's participle.
        doing: String,
    },
}

impl Blocked {
    /// What is holding this up, for a record.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Instrument { name, .. } | Self::Slot { name, .. } => name,
        }
    }

    /// What that thing is doing.
    #[must_use]
    pub fn doing(&self) -> &str {
        match self {
            Self::Instrument { doing, .. } | Self::Slot { doing, .. } => doing,
        }
    }
}

/// Whether `intent` would have to wait, asked from where the script is standing.
///
/// Only duration-actions block: a script reading a log must not wait behind a
/// brew.
#[must_use]
pub fn would_block(world: &mut World, intent: &Intent) -> Option<Blocked> {
    // "Would this be refused", not "does this start work": `move`, `empty`,
    // `purge` and `siphon` refuse on a busy instrument without beginning
    // anything.
    for at in touches(world, intent) {
        if let Some(why) = tower::busy(world, at) {
            return Some(Blocked::Instrument {
                name: name_of(world, at),
                doing: why.label().to_owned(),
            });
        }
    }

    // `begin` tests the slot inline as it refuses, so there is no separable
    // predicate to call.
    if begins_work(intent)
        && tower::in_flight(world).len() >= tower::CAPACITY
        && let Some((doing, at)) = tower::occupied(world)
    {
        return Some(Blocked::Slot {
            name: name_of(world, at),
            doing: doing.participle().to_owned(),
        });
    }

    None
}

/// Whether this instruction takes the tower's production slot.
///
/// The verbs that reach `work::begin`. It reads the argument: `wield` charging
/// an instrument is a run and spending a scroll is not, and a scripted quicken
/// once waited out the brew it was written to hurry. Through
/// [`execute::spending`](crate::execute::spending), as `pipeline::wield` asks.
fn begins_work(intent: &Intent) -> bool {
    if crate::execute::spending(intent).is_some() {
        return false;
    }
    // Operations that schedule nothing: scoped to an instrument, but instant.
    // Left in, a bound solver waits on a slot it never takes and gives up at
    // `PATIENCE` — §19 records the sanctum's costing exactly that.
    if matches!(
        intent.verb,
        Verb::Dial | Verb::Muster | Verb::Haul | Verb::Summon | Verb::Limn
    ) {
        return false;
    }
    intent.verb.is_operation() || matches!(intent.verb, Verb::Wield | Verb::Research | Verb::Purge)
}

/// Every instrument `intent` needs to find idle, where the script is standing.
///
/// A §10.1 verb names the material, so its instrument is found by
/// [`Operation`](crate::tower::Operation), as `execute::pipeline::operate` finds
/// it. A `move` needs both its places free.
fn touches(world: &World, intent: &Intent) -> Vec<Entity> {
    let cwd = world.resource::<Cwd>().0;
    let here = tower::children_of(world, cwd);

    if intent.verb.is_operation() {
        return here
            .into_iter()
            .filter(|node| {
                world
                    .get::<tower::Operation>(*node)
                    .is_some_and(|operation| operation.0 == intent.verb)
            })
            .collect();
    }
    // Never `stop`, whose purpose is the busy case: listed, a spell could not
    // call anything off.
    if !matches!(
        intent.verb,
        Verb::Wield | Verb::Empty | Verb::Purge | Verb::Move
    ) {
        return Vec::new();
    }

    // By leaf: a `Place` argument is a full path, a node only its last segment.
    intent
        .arguments
        .iter()
        .filter(|argument| argument.kind == crate::parser::NounKind::Place)
        .filter_map(|argument| {
            let leaf = crate::parser::leaf(&argument.value).to_owned();
            here.iter().copied().find(|node| {
                world
                    .get::<tower::Name>(*node)
                    .is_some_and(|name| name.0 == leaf)
            })
        })
        .collect()
}

fn name_of(world: &World, at: Entity) -> String {
    world
        .get::<tower::Name>(at)
        .map_or_else(String::new, |name| name.0.clone())
}
