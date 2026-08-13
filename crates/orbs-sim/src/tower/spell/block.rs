//! Whether an instruction can proceed, asked **without** refusing it.
//!
//! # The distinction this file exists for
//!
//! A player who types `grind sage` at a busy mortar should be told so — that is
//! `tower::work::refuse_busy`, and §5.0's *"repairing the rats occupies the
//! laboratory pane for its duration"* only bites if the refusal says so.
//!
//! A **script** reaching the same instruction should *wait*. It is not a
//! mistake; it is the next line of a recipe arriving before the previous one
//! finished. Running it through the refusal path would emit one record per tick
//! for the whole duration of the run it is waiting on — and a spell that grinds
//! then siphons would fill the log with dozens of identical complaints while
//! doing exactly the right thing.
//!
//! So the predicate and the refusal are separated, and this is the predicate.
//!
//! # Both tiers, because the second is the one that fires
//!
//! `tower::work::begin` refuses on two grounds: the instrument being busy, and
//! the **tower-wide** production slot being taken
//! (`in_flight().len() >= CAPACITY`, and `CAPACITY` is 1). Only the first had a
//! separable predicate; the second emitted its `work_busy` record inline.
//!
//! At capacity 1 the second is the one a script hits, for every multi-instrument
//! spell — which is every §10.1 brew, and therefore the whole point. Checking
//! only the instrument would let those instructions through to `begin`, which
//! logs at the player once per tick: exactly the noise this exists to prevent.

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
    /// The tower's production slot is taken by something else.
    ///
    /// §9's fourth invariant reserves it for an in-flight action's whole
    /// duration, and §11.5 opens at capacity 1.
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
/// **Only duration-actions can block.** `attend`, `survey`, `peruse` and the
/// rest neither start work nor need the slot, so asking about them would make a
/// script that reads a log wait behind a brew for no reason.
#[must_use]
pub fn would_block(world: &mut World, intent: &Intent) -> Option<Blocked> {
    // Every instrument this instruction touches has to be free. **The question
    // is "would this be refused", not "does this start work"** — the first
    // version of this asked the narrower one and `siphon` fell through it, so a
    // spell that ground and then siphoned was told *"the mortar_and_pestle is
    // still at work"* as though it had typed the line itself. `move`, `empty`,
    // `purge` and `siphon` all refuse on a busy instrument without beginning
    // anything, and each was a way for a correct spell to be shouted at.
    for at in touches(world, intent) {
        if let Some(why) = tower::busy(world, at) {
            return Some(Blocked::Instrument {
                name: name_of(world, at),
                doing: why.label().to_owned(),
            });
        }
    }

    // The tower-wide slot, for the verbs that actually begin a run. **This is
    // the tier that fires** for a multi-instrument spell, and it has no
    // separable predicate of its own in `work::slot` — `begin` tests it inline
    // and emits its refusal at the same time.
    if begins_work(intent.verb)
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

/// Whether this verb takes the tower's production slot.
///
/// The verbs that reach `work::begin`. `siphon`, `move` and `empty` are
/// deliberately absent: they need an instrument to be *idle*, but they take no
/// slot and finish within the tick — §9's triage band exists so short work still
/// runs during a brew.
const fn begins_work(verb: Verb) -> bool {
    verb.is_operation() || matches!(verb, Verb::Wield | Verb::Research | Verb::Purge)
}

/// Every instrument `intent` needs to find idle, where the script is standing.
///
/// §10.1's per-instrument verbs name the *material*, not the tool — the tool is
/// what the verb means — so that instrument is found from its
/// [`Operation`](crate::tower::Operation) component. **The same derivation
/// `execute::pipeline::operate` uses**, extracted rather than copied: the first
/// copy exists specifically to end the name-string dispatch six sites were
/// doing, and a second copy would restart it.
///
/// A `move` names two places and needs **both** free — §10.1's lock covers
/// taking as much as putting, and `pipeline::carry` refuses on either.
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
    // **`stop` is not here, and must not be.** The list is "verbs a busy
    // instrument would refuse", and `stop` is the one verb whose whole purpose
    // is the busy case — `pipeline::stop` never refuses on `Working`. Listing it
    // made a scripted `stop` wait out the very run it was cancelling, burn
    // PATIENCE, report `spell_gave_up`, and then fire on an idle tool as a
    // no-op. A spell could not call anything off.
    if !matches!(
        intent.verb,
        Verb::Wield | Verb::Empty | Verb::Purge | Verb::Move
    ) {
        return Vec::new();
    }

    // Whatever places it named, by leaf — a `Place` argument resolves to a full
    // path while nodes carry only their last segment.
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
