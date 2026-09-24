//! The athanor: shared heat, and the one instrument that takes no Focus.
//!
//! §10.1. Four instruments consume the tower's production pool and §11.5's
//! capacity tops out at four; the athanor is infrastructure, not a stage.
//!
//! An interval, not a countdown, so in-flight state stays idempotent under a
//! `meditate` of hundreds of ticks (§19). Fuel is stored as when it was lit and
//! how much was in it, and `remaining = ticks - (now - lit_at)`. It is also why
//! the rate cannot vary with load: charging more for two heated instruments
//! would make fuel an integration over history.
//!
//! It burns whether or not anything is mounted, so idle lit time is waste and
//! you want both heated stages inside one lighting — a window at 1 Hz, never a
//! reflex. That is also §8's automation argument made mechanical: a script that
//! ends its loop with `stop athanor` wastes nothing.
//!
//! Heat is checked when work starts and the run then completes. Pausing would
//! be a countdown by another name; spoiling would cost progress, against
//! §11.5's "never ruinous, only slower". The batching pressure comes from the
//! burn being time-based, not from spoilage.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use super::node::Name;
use crate::content::{Fuels, Prose};
use crate::parser::{NounKind, Verb};
use crate::session::Scrollback;
use crate::tick::Tick;

/// The name of the instrument that gives heat.
pub const ATHANOR: &str = "athanor";

/// A fire that is burning.
///
/// Not [`Working`](super::Working): the athanor consumes no Focus, so nothing
/// that counts the production pool may see it.
#[derive(Component, Debug, Clone, Copy)]
pub struct Burning {
    /// When it was lit.
    pub lit_at: Tick,
    /// How much fuel it held at that moment, in ticks.
    pub ticks: u64,
}

impl Burning {
    /// Ticks of fuel left at `now`.
    #[must_use]
    pub const fn remaining(&self, now: Tick) -> u64 {
        let elapsed = now.get().saturating_sub(self.lit_at.get());
        self.ticks.saturating_sub(elapsed)
    }

    /// Fuel left and fuel total, for a meter.
    ///
    /// Remaining over total, where
    /// [`Working::progress`](super::Working::progress) reports elapsed over
    /// total, so a meter drawn from this drains rather than fills. Same
    /// fraction, shrinking numerator — the render side needs to know nothing.
    #[must_use]
    pub const fn fuel(&self, now: Tick) -> (u64, u64) {
        (self.remaining(now), self.ticks)
    }
}

/// Fuel that has been banked but is not alight.
///
/// What `stop athanor` preserves. Without it damping burns everything left and
/// the automation lesson above is a story rather than a mechanic.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Banked(pub u64);

/// Fuel that is banked but not alight, in ticks.
#[must_use]
pub fn banked(world: &World, athanor: Entity) -> u64 {
    world.get::<Banked>(athanor).map_or(0, |banked| banked.0)
}

/// The fire's state, if there is one.
#[must_use]
pub fn burning(world: &World, athanor: Entity) -> Option<Burning> {
    world.get::<Burning>(athanor).copied()
}

/// Whether the athanor is alight *now*.
///
/// What the heated instruments ask before they will start.
#[must_use]
pub fn lit(world: &mut World, athanor: Entity) -> bool {
    let now = *world.resource::<Tick>();
    world
        .get::<Burning>(athanor)
        .is_some_and(|fire| fire.remaining(now) > 0)
}

/// The heat source among the current room's fixtures, if there is one.
///
/// The marker, not the name, and a fixture, not any child: `siphon` deposits
/// products at `cwd`, so matching on `Name == ATHANOR` could find a reagent
/// called `athanor` on the floor and ask *it* whether it was alight.
#[must_use]
pub fn find(world: &mut World, cwd: Entity) -> Option<Entity> {
    super::node::children_of(world, cwd)
        .into_iter()
        .find(|node| {
            world.get::<super::Fixture>(*node).is_some()
                && world.get::<super::HeatSource>(*node).is_some()
        })
}

/// Light the athanor on whatever fuel is in it, plus anything banked.
///
/// Returns whether it took.
pub fn kindle(world: &mut World, athanor: Entity) -> bool {
    let now = *world.resource::<Tick>();
    if lit(world, athanor) {
        report(world, "kindle_lit", Role::Cost, "lit", None);
        return false;
    }

    // Every burnable thing in it goes on at once. Fuel is committed when the
    // match is struck, which makes lighting a decision rather than a formality.
    let held = super::node::children_of(world, athanor);
    let mut ticks = world.get::<Banked>(athanor).map_or(0, |banked| banked.0);
    let mut spent = Vec::new();
    // Ash already owed from fuel burnt before a damping. Starting empty meant a
    // relighting found no fuel and overwrote the debt with nothing.
    let mut ash = world
        .get::<Ash>(athanor)
        .map(|owed| owed.0.clone())
        .unwrap_or_default();
    // `fuel` lengthens what is lit now, never what was banked — that was
    // lengthened at its first lighting, and twice is one charcoal paid twice.
    let longer = super::grant::fuel_percent(world);
    for node in held {
        let Some(name) = world.get::<Name>(node).map(|name| name.0.clone()) else {
            continue;
        };
        if let Some(fuel) = world.resource::<Fuels>().get(&name) {
            // Per unit, not per pile: N charcoal is one node with `Counted(N)`,
            // and crediting one unit's ticks while burning the node destroyed
            // the other N-1. Endless fuel burns one unit and stays endless.
            let units = match world.get::<super::Stock>(node) {
                Some(super::Stock::Counted(count)) => *count,
                _ => 1,
            };
            for _ in 0..units {
                ticks = ticks.saturating_add(fuel.ticks.saturating_mul(100 + longer) / 100);
                ash.push(fuel.leaves.clone());
            }
            spent.push((name, units));
        }
    }

    if ticks == 0 {
        report(world, "kindle_cold", Role::Cost, "cold", None);
        return false;
    }

    let spent_fuel = !spent.is_empty();
    for (name, units) in spent {
        super::stock::take(world, athanor, &name, units);
    }
    // The ash it will leave is decided now and held until burn-out, so a
    // collapsed burn spawns what a watched one would, in the same order —
    // insertion order is the parse (§19).
    world
        .entity_mut(athanor)
        .insert(Burning { lit_at: now, ticks })
        .insert(Ash(ash))
        .remove::<Banked>();

    // Relighting on banked fuel says so, or damping reads as throwing fuel
    // away.
    let key = if spent_fuel {
        "kindle_done"
    } else {
        "kindle_banked"
    };
    report(world, key, Role::Success, "lit", Some(ticks));
    true
}

/// What a burn will leave when it is spent.
#[derive(Component, Debug, Clone)]
pub struct Ash(pub Vec<String>);

/// Damp the fire, banking what has not burnt.
///
/// Returns whether there was anything to damp.
pub fn damp(world: &mut World, athanor: Entity) -> bool {
    let now = *world.resource::<Tick>();
    let Some(fire) = world.get::<Burning>(athanor).copied() else {
        report(world, "damp_idle", Role::Normal, "idle", None);
        return false;
    };

    let left = fire.remaining(now);

    // A fire with nothing left has gone out, not been damped. `Ash` is landed
    // only by `burn`, which queries live `Burning` — so removing it here on the
    // tick the fuel hit zero orphaned the debt and the ash never appeared.
    if left == 0 {
        spend(world, athanor);
        return false;
    }

    world.entity_mut(athanor).remove::<Burning>();
    world.entity_mut(athanor).insert(Banked(left));
    report(world, "damp_done", Role::Success, "damped", Some(left));
    true
}

/// Put out a fire that has run out, landing what it owes.
///
/// The one place a burn ends, so [`burn`] and [`damp`]-at-zero cannot disagree
/// about what it leaves behind.
fn spend(world: &mut World, athanor: Entity) {
    world.entity_mut(athanor).remove::<Burning>();
    let leavings = world
        .get::<Ash>(athanor)
        .map(|ash| ash.0.clone())
        .unwrap_or_default();
    world.entity_mut(athanor).remove::<Ash>();

    for leaving in leavings {
        // Merged, so a second burning adds to the ash rather than standing a
        // second pile under the same name.
        super::stock::give(world, athanor, &leaving, NounKind::Reagent, 1);
    }

    let message = world.resource::<Prose>().line("athanor_out", &[]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, ATHANOR)
        .text(FieldName::At, ATHANOR)
        .text(FieldName::State, "cold")
        .text(FieldName::Message, &message)
        .role(Role::Cost)
        .finish();
}

/// Spend any fire whose fuel has run out.
///
/// Runs in the sim schedule beside [`finish`](super::finish). Ash lands once,
/// at burn-out: spawning per tick issues ids into `Children` at a rate that
/// depends on how the ticks were consumed, and insertion order is the parse
/// (§19).
pub fn burn(world: &mut World) {
    let now = *world.resource::<Tick>();
    let spent: Vec<Entity> = world
        .query::<(Entity, &Burning)>()
        .iter(world)
        .filter(|(_, fire)| fire.remaining(now) == 0)
        .map(|(entity, _)| entity)
        .collect();

    for athanor in spent {
        spend(world, athanor);
    }
}

/// Say what just happened to the fire, and how much fuel it has.
///
/// The number is not decoration: banked fuel is a component rather than a node
/// to `survey`, so without it in the sentence a player who damps sees an empty
/// athanor and concludes their charcoal is gone. Reported as exactly that.
fn report(world: &mut World, key: &str, role: Role, state: &str, fuel: Option<u64>) {
    let ticks = fuel.map(|fuel| fuel.to_string()).unwrap_or_default();
    let message = world.resource::<Prose>().line(key, &[("left", &ticks)]);
    let mut scrollback = world.resource_mut::<Scrollback>();
    let mut record = scrollback
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, ATHANOR)
        .text(FieldName::At, ATHANOR)
        .text(FieldName::State, state)
        .text(FieldName::Message, &message)
        .role(role);
    if let Some(fuel) = fuel {
        record = record.count(FieldName::Remaining, fuel);
    }
    record.finish();
}

/// The refusal a heated instrument gives when the fire is out.
pub fn refuse_cold(world: &mut World, instrument: &str) {
    let message = world
        .resource::<Prose>()
        .line("wield_cold", &[("name", instrument)]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Wield.canonical())
        .text(FieldName::Path, instrument)
        .text(FieldName::State, "cold")
        .text(FieldName::Message, &message)
        .role(Role::Cost)
        .finish();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Sim;

    /// Everything where the player is standing.
    fn here(sim: &mut Sim) -> Vec<String> {
        let cwd = sim.world_mut().resource::<super::super::Cwd>().0;
        super::super::node::children_of(sim.world_mut(), cwd)
            .into_iter()
            .filter_map(|node| sim.world_mut().get::<Name>(node).map(|name| name.0.clone()))
            .collect()
    }

    /// How long one charcoal burns, read from the content rather than repeated.
    ///
    /// Three tests broke the day the burn was slowed, each having `40` written
    /// into it. A test that pins a tuning number fails on tuning.
    fn charcoal_ticks() -> u64 {
        Fuels::builtin()
            .get("charcoal")
            .expect("charcoal burns")
            .ticks
    }

    /// A sim standing in the laboratory with the athanor alight.
    fn burning() -> Sim {
        let mut sim = Sim::new(1);
        sim.submit("attend laboratory");
        sim.step();
        sim.submit("move charcoal to athanor");
        sim.step();
        sim.submit("wield athanor");
        sim.step();
        sim
    }

    #[test]
    fn fuel_is_a_function_of_the_tick_not_a_countdown() {
        // A `meditate` of hundreds of ticks inside one `step()` must leave the
        // fire exactly where being watched would (§19).
        let fire = Burning {
            lit_at: Tick::new(10),
            ticks: 40,
        };
        assert_eq!(fire.remaining(Tick::new(10)), 40);
        assert_eq!(fire.remaining(Tick::new(30)), 20);
        assert_eq!(fire.remaining(Tick::new(50)), 0);
        // Past the end it floors rather than wrapping.
        assert_eq!(fire.remaining(Tick::new(9_999)), 0);
    }

    #[test]
    fn the_fuel_meter_reads_as_remaining_so_the_bar_drains() {
        // Rule 2 from this side: reporting the remainder rather than the
        // elapsed is all that makes the bar drain, and nothing downstream
        // knows it is different.
        let fire = Burning {
            lit_at: Tick::new(0),
            ticks: 40,
        };
        assert_eq!(fire.fuel(Tick::new(0)), (40, 40));
        assert_eq!(fire.fuel(Tick::new(30)), (10, 40));
    }

    #[test]
    fn lighting_consumes_the_fuel_and_burns_out_leaving_ash() {
        let mut sim = burning();
        sim.submit("attend athanor");
        sim.step();
        assert!(
            !here(&mut sim).contains(&"charcoal".to_owned()),
            "the charcoal was not committed to the fire"
        );

        sim.submit(&format!("meditate {}", charcoal_ticks() + 10));
        sim.step();
        assert!(
            here(&mut sim).contains(&"ash".to_owned()),
            "a spent fire left nothing"
        );
    }

    #[test]
    fn the_athanor_takes_no_focus_slot() {
        // §11.5's capacity tops out at four because the athanor is
        // infrastructure. A fire that took a slot would make it five.
        let mut sim = burning();
        assert!(
            sim.working().is_none(),
            "the fire is holding the tower's production slot"
        );

        sim.submit("move sage to mortar_and_pestle");
        sim.step();
        sim.submit("wield mortar_and_pestle");
        sim.step();
        assert!(
            sim.working().is_some(),
            "the fire blocked an instrument from starting"
        );
    }

    #[test]
    fn damping_banks_what_has_not_burnt() {
        // What makes `stop athanor` worth writing at the end of a script loop,
        // and so §8's "automation beats doing it by hand" mechanical.
        let mut sim = burning();
        sim.step_n(20);
        sim.submit("stop athanor");
        sim.step();

        let cwd = sim.world_mut().resource::<super::super::Cwd>().0;
        let athanor = find(sim.world_mut(), cwd).expect("the athanor is here");
        let banked = sim
            .world_mut()
            .get::<Banked>(athanor)
            .map_or(0, |banked| banked.0);
        assert!(banked > 0, "damping burnt the remainder anyway");
        assert!(
            banked < charcoal_ticks(),
            "damping banked fuel that had already gone"
        );
    }

    #[test]
    fn ash_survives_a_damping_and_arrives_when_the_fire_finally_dies() {
        // Relighting on banked fuel found no new fuel and overwrote the ash
        // already owed, so a fire damped once left nothing behind.
        let mut sim = burning();
        sim.step_n(20);
        sim.submit("stop athanor");
        sim.step();
        sim.submit("wield athanor");
        sim.step();
        sim.submit(&format!("meditate {}", charcoal_ticks() + 10));
        sim.step();

        sim.submit("attend athanor");
        sim.step();
        assert!(
            here(&mut sim).contains(&"ash".to_owned()),
            "the ash owed from before the damping never arrived"
        );
    }

    /// Everything the orb has said this session.
    fn said(sim: &Sim) -> Vec<String> {
        sim.scrollback()
            .records()
            .iter()
            .map(|record| record.to_line())
            .collect()
    }

    #[test]
    fn damping_says_how_much_fuel_keeps() {
        // Reported as "stopping early got rid of the charcoal". The mechanic
        // was right and the feedback absent: banked fuel is a component, so
        // `survey athanor` shows an empty instrument.
        let mut sim = burning();
        sim.step_n(10);
        sim.submit("stop athanor");
        sim.step();

        assert!(
            said(&sim)
                .iter()
                .any(|line| line.contains("ticks of fuel keep")),
            "damping did not say what it kept: {:?}",
            said(&sim)
        );
    }

    #[test]
    fn relighting_on_banked_fuel_says_so() {
        // "The athanor takes light" after a damping gives no sign that what was
        // saved came back.
        let mut sim = burning();
        sim.step_n(10);
        sim.submit("stop athanor");
        sim.step();
        sim.submit("wield athanor");
        sim.step();

        assert!(
            said(&sim).iter().any(|line| line.contains("banked fuel")),
            "relighting did not mention the banked fuel: {:?}",
            said(&sim)
        );
    }

    #[test]
    fn light_the_athanor_lights_it_rather_than_listing_it() {
        // `light` was unclaimed and scored 600 against `list`, so it resolved
        // to `survey`. Claimed, the exact match wins.
        let mut sim = Sim::new(1);
        sim.submit("attend laboratory");
        sim.step();
        sim.submit("move charcoal to athanor");
        sim.step();
        sim.submit("light athanor");
        sim.step();

        let cwd = sim.world_mut().resource::<super::super::Cwd>().0;
        let athanor = find(sim.world_mut(), cwd).expect("the athanor is here");
        assert!(
            lit(sim.world_mut(), athanor),
            "`light athanor` did not light it: {:?}",
            said(&sim)
        );
    }

    #[test]
    fn a_heated_instrument_refuses_while_the_fire_is_out() {
        let mut sim = Sim::new(1);
        sim.submit("attend laboratory");
        sim.step();
        sim.submit("move sage to mortar_and_pestle");
        sim.step();
        sim.submit("wield mortar_and_pestle");
        sim.step();
        sim.step_n(20);
        sim.submit("move ground-sage to balneum_mariae");
        sim.step();

        sim.submit("wield balneum_mariae");
        sim.step();
        assert!(sim.working().is_none(), "a cold bath started anyway");

        sim.submit("move charcoal to athanor");
        sim.step();
        sim.submit("wield athanor");
        sim.step();
        sim.submit("wield balneum_mariae");
        sim.step();
        assert!(sim.working().is_some(), "a lit athanor did not help");
    }

    #[test]
    fn a_run_already_started_completes_even_if_the_fire_dies_under_it() {
        // §10.1: heat is checked at the start and the run then completes.
        let mut sim = Sim::new(1);
        sim.submit("attend laboratory");
        sim.step();
        sim.submit("move sage to mortar_and_pestle");
        sim.step();
        sim.submit("wield mortar_and_pestle");
        sim.step();
        sim.step_n(20);
        sim.submit("move ground-sage to balneum_mariae");
        sim.step();
        sim.submit("move charcoal to athanor");
        sim.step();
        sim.submit("wield athanor");
        sim.step();
        sim.submit("wield balneum_mariae");
        sim.step();

        // Damp it the instant the bath is under way.
        sim.submit("stop athanor");
        sim.step();
        sim.step_n(30);

        sim.submit("attend balneum_mariae");
        sim.step();
        assert!(
            here(&mut sim).contains(&"sage-tincture".to_owned()),
            "the run was lost when the fire went out"
        );
    }
}
