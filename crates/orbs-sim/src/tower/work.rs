//! Actions that take time.
//!
//! DESIGN.md §5.0: *"Issuing an action is free; **actions take time to
//! complete.**"* Ticks are world time, not currency — there is no pool to spend
//! and a fast typist gains nothing over a slow one. The scarcity is **duration**,
//! and it is expressed as concurrency: how many things you can have in flight at
//! once.
//!
//! That is why this is the economy. §5.0 again: *"Attention is therefore the real
//! scarcity, and it is expressed as concurrency … which means the economy and the
//! focus system are the same system rather than two bolted together."*
//!
//! # One production slot
//!
//! §11.5 starts the player at **multiplex capacity 1**, and §9's fourth
//! invariant reserves that slot for an in-flight manual action for its whole
//! duration. So brewing occupies the tower, not merely the alembic: start a
//! decoction and you are not also deciphering. That is the trade the whole focus
//! track is built on, and giving each domain its own slot would delete it while
//! being *more* code — a counter per domain where the design needs one.
//!
//! §9 also grants a per-pane **triage** slot, so short work still runs during a
//! production action. Phase 0 has one triage verb, `purge`, and it is instant.
//!
//! # Stored as an interval, not a countdown
//!
//! §8 requires in-flight actions be *"first-class serialisable entities with
//! start and completion ticks"*. Comparing against the clock rather than
//! decrementing a counter is idempotent, survives `meditate` running hundreds of
//! ticks inside one `step`, serialises without interpretation, and makes the
//! progress fraction a pure function of the tick.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use super::node::{Name, Nameable, NodeId, NodeIds, Protected};
use crate::parser::{NounKind, Verb};
use crate::session::Scrollback;
use crate::tick::Tick;

/// How long a decoction takes.
///
/// §11.5 puts production actions at 3–10 minutes, which at 1 Hz is 180–600
/// ticks. **Phase 0 deliberately runs shorter**: §15's gate is a *fifteen-minute*
/// scripted scenario, and one design-faithful brew would eat a fifth to two
/// thirds of a tester's entire session — measuring their patience rather than
/// the parser.
///
/// So the slice sits at §11.5's routine end. The shipped numbers arrive with the
/// balance harness in Phase 1, which is what §11.5 says sweeps them; this is the
/// first constant it will touch.
pub const DECOCT_TICKS: u64 = 20;

/// How long a decipherment takes. Shorter than a brew — §10 makes archive the
/// domain played most, and the one the player returns to between other work.
pub const DIVINE_TICKS: u64 = 12;

/// Work in progress.
#[derive(Component, Debug, Clone, Copy)]
pub struct Working {
    /// What is being done.
    pub verb: Verb,
    /// What it is being done to.
    pub subject: NodeId,
    /// When it started.
    pub started: Tick,
    /// When it lands.
    pub ends: Tick,
}

impl Working {
    /// Ticks elapsed and ticks total, for a meter.
    #[must_use]
    pub fn progress(&self, now: Tick) -> (u64, u64) {
        let total = self.ends.get().saturating_sub(self.started.get());
        let done = now.get().saturating_sub(self.started.get()).min(total);
        (done, total)
    }

    /// Whether `now` has reached the end.
    #[must_use]
    pub const fn finished(&self, now: Tick) -> bool {
        now.get() >= self.ends.get()
    }
}

/// How many production actions may be in flight at once.
///
/// §11.5's opening multiplex capacity. Raising it is a **progression unlock**,
/// not a tuning knob — §9 keeps panes and capacity as separate unlocks that
/// "must not be conflated".
pub const CAPACITY: usize = 1;

/// Whether the tower's production slot is free, and what holds it.
#[must_use]
pub fn occupied(world: &mut World) -> Option<(Verb, Entity)> {
    world
        .query::<(Entity, &Working)>()
        .iter(world)
        .map(|(entity, working)| (working.verb, entity))
        .next()
}

/// Begin work at `place`, on `subject`.
///
/// Refuses when the production slot is taken, naming what holds it — §5.0's
/// *"repairing the rats occupies the alembic pane for its duration, during which
/// you are not brewing"* only bites if the refusal says so.
pub fn begin(world: &mut World, place: Entity, verb: Verb, subject: NodeId, ticks: u64) -> bool {
    if let Some((holder, at)) = occupied(world) {
        let name = world
            .get::<Name>(at)
            .map_or_else(String::new, |name| name.0.clone());
        world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Completion)
            .text(FieldName::Name, verb.canonical())
            .text(FieldName::State, holder.canonical())
            .text(FieldName::Source, &name)
            .role(Role::Cost)
            .finish();
        return false;
    }

    let now = *world.resource::<Tick>();
    world.entity_mut(place).insert(Working {
        verb,
        subject,
        started: now,
        ends: Tick::new(now.get().saturating_add(ticks)),
    });
    true
}

/// Land anything whose time has come.
///
/// §14 announces **completions only** — a duration action finishing is the one
/// event the player did not just cause, and announcing progress instead would be
/// unusable at endgame with ~25 actions in flight.
pub fn finish(world: &mut World) {
    let now = *world.resource::<Tick>();
    let landed: Vec<(Entity, Working)> = world
        .query::<(Entity, &Working)>()
        .iter(world)
        .filter(|(_, working)| working.finished(now))
        .map(|(entity, working)| (entity, *working))
        .collect();

    for (place, working) in landed {
        world.entity_mut(place).remove::<Working>();
        let subject = name_of(world, working.subject);
        world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Completion)
            .text(FieldName::Name, working.verb.canonical())
            .text(FieldName::Detail, &subject)
            .role(Role::Success)
            .finish();

        // §7: *"Alchemical byproduct accumulates in `/alembic` and must be
        // purged manually or by a bound cleanup script."* Waste is an idle
        // mechanic and a nuisance source, and it is what gives `purge` — a
        // destructive verb — something **useful, everyday and scriptable** to
        // destroy rather than a trap to spring.
        if working.verb == Verb::Decoct {
            let id = world.resource_mut::<NodeIds>().issue();
            let residue = world
                .spawn((
                    id,
                    Name(format!("residue-{}", id.get())),
                    Nameable(NounKind::Any),
                ))
                .id();
            world.entity_mut(residue).insert(ChildOf(place));
        }
    }
}

/// Destroy something.
///
/// §7: destruction is *"a tool, not a trap"*. Catastrophic targets refuse in
/// character, which here means a record carrying the target and a danger role —
/// the memorable sentence is composed from those by a content file in Phase 1
/// (rule 6, §12) rather than being written into Rust now.
///
/// Nothing irreplaceable is destructible in Phase 0: byproduct regenerates with
/// the next brew, so `undo` is not load-bearing for §15's gate.
pub fn purge(world: &mut World, target: Entity) {
    let name = world
        .get::<Name>(target)
        .map_or_else(String::new, |name| name.0.clone());

    if world.get::<Protected>(target).is_some() {
        world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Completion)
            .text(FieldName::Name, Verb::Purge.canonical())
            .text(FieldName::Detail, &name)
            .text(FieldName::State, "protected")
            .role(Role::Danger)
            .finish();
        return;
    }

    world.entity_mut(target).despawn();
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Purge.canonical())
        .text(FieldName::Detail, &name)
        .role(Role::Success)
        .finish();
}

/// The name behind a stable id.
fn name_of(world: &mut World, id: NodeId) -> String {
    world
        .query::<(&NodeId, &Name)>()
        .iter(world)
        .find(|(node, _)| **node == id)
        .map_or_else(String::new, |(_, name)| name.0.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Sim;

    #[test]
    fn a_brew_takes_time_and_lands_by_itself() {
        let mut sim = Sim::new(1);
        sim.submit("attend alembic");
        sim.step();
        sim.submit("decoct clarity");
        sim.step();

        assert!(sim.working().is_some(), "nothing went in flight");
        sim.step_n(DECOCT_TICKS - 1);
        assert!(sim.working().is_some(), "it landed early");

        sim.step();
        assert!(sim.working().is_none(), "it never landed");
    }

    #[test]
    fn the_tower_has_one_production_slot_not_one_per_domain() {
        // §11.5 opens at multiplex capacity 1 and §9's invariant 4 reserves it
        // for the whole duration, so brewing occupies the *tower*. A slot per
        // domain would delete the trade the focus track is built on.
        let mut sim = Sim::new(1);
        sim.submit("attend alembic");
        sim.step();
        sim.submit("decoct clarity");
        sim.step();

        sim.submit("attend archive");
        sim.step();
        sim.submit("divine sigil-iv");
        sim.step();

        assert_eq!(
            sim.working().map(|working| working.verb),
            Some(Verb::Decoct),
            "the second action displaced the first",
        );
    }

    #[test]
    fn progress_is_a_function_of_the_clock() {
        // Stored as an interval rather than a countdown, so `meditate` running
        // hundreds of ticks inside one `step` cannot desynchronise it.
        let working = Working {
            verb: Verb::Decoct,
            subject: NodeId::from_raw(0),
            started: Tick::new(10),
            ends: Tick::new(30),
        };
        assert_eq!(working.progress(Tick::new(10)), (0, 20));
        assert_eq!(working.progress(Tick::new(20)), (10, 20));
        assert_eq!(working.progress(Tick::new(30)), (20, 20));
        // Past the end it clamps rather than overrunning the meter.
        assert_eq!(working.progress(Tick::new(99)), (20, 20));
    }

    #[test]
    fn meditating_through_a_brew_lands_it_exactly_once() {
        let mut sim = Sim::new(1);
        sim.submit("attend alembic");
        sim.step();
        sim.submit("decoct clarity");
        sim.step();

        sim.submit(&format!("meditate {}", DECOCT_TICKS * 3));
        sim.step();

        assert!(sim.working().is_none());
        let landed = sim
            .scrollback()
            .records()
            .iter()
            .filter(|record| record.kind() == RecordKind::Completion)
            .filter(|record| record.field(FieldName::Detail).is_some())
            .count();
        assert_eq!(landed, 1, "a completion fired more than once");
    }
}
