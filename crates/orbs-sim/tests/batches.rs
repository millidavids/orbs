//! A run makes a batch, and the tower counts the run (§19, Supply).
//!
//! Potions are spent by volume, so a distillation yields several. Experience,
//! renown and the stores were tuned on runs, and none of them may notice.
//!
//! `debug_spawn` is the door in, so these are debug-only.
#![cfg(debug_assertions)]

mod common;

use bevy_ecs::prelude::Entity;
use common::run;
use orbs_sim::{Sim, tower};

fn alembic(sim: &mut Sim) -> Entity {
    let world = sim.world_mut();
    world
        .query::<(Entity, &tower::Name)>()
        .iter(world)
        .find(|(_, name)| name.0 == "alembic")
        .map(|(alembic, _)| alembic)
        .expect("the laboratory has an alembic")
}

/// One clarity distilled, the alembic charmed `fruitful` or not.
fn distilled(fruitful: bool) -> Sim {
    let mut sim = Sim::new(3);
    for line in [
        "attend laboratory",
        "kindle charcoal",
        "debug_spawn clarified-draught",
    ] {
        run(&mut sim, line);
    }
    if fruitful {
        let at = alembic(&mut sim);
        let mut charmed = tower::Charmed::default();
        charmed.lay(tower::Charm {
            kind: tower::charm::Kind::Fruitful,
            from: orbs_sim::Tick::new(0),
            ticks: 1_000_000,
        });
        sim.world_mut().entity_mut(at).insert(charmed);
    }
    run(&mut sim, "distil clarified-draught");
    run(&mut sim, "meditate 60");
    sim
}

fn held(sim: &mut Sim, named: &str) -> u32 {
    let at = alembic(sim);
    tower::holdings(sim.world(), at)
        .into_iter()
        .find(|(name, _)| name == named)
        .map_or(0, |(_, held)| held)
}

#[test]
fn a_run_yields_what_its_recipe_authors_and_is_counted_once() {
    let mut sim = distilled(false);
    assert_eq!(held(&mut sim, "clarity"), 3, "a distillation is a batch");
    assert_eq!(held(&mut sim, "phlegm"), 1, "the byproduct is not");

    // One run, however many it made.
    assert_eq!(sim.tally("potion"), 1);
    assert_eq!(sim.tally("made:clarity"), 1);
    let now = sim.world().resource::<orbs_sim::Tick>().get();
    assert_eq!(
        sim.world().resource::<tower::Stores>().rate("clarity", now),
        1,
        "the stores counted the batch rather than the run"
    );
}

#[test]
fn a_fruitful_run_yields_one_more_than_authored() {
    let mut sim = distilled(true);
    assert_eq!(held(&mut sim, "clarity"), 4);
    assert_eq!(held(&mut sim, "phlegm"), 1, "fruitful doubled the leavings");
    assert_eq!(sim.tally("potion"), 1);
}
