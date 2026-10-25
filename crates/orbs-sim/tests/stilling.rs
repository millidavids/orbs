//! Stillness's price, driven through the real laboratory (§19, `0.17.1`).
//!
//! A potion that ends a siege is only fair if it is dear: a scroll from the
//! archive, ten minutes of the alembic, and most of the tower's quintessence.
//! Each is asserted here, with a slot refusal that must not charge. Its old
//! one-held cap is retired (§19, *Supply*): it brews three a run and goes stale
//! like anything else.
//!
//! `debug_spawn` is the door in, so these are debug-only.
#![cfg(debug_assertions)]

mod common;

use common::{ever_said, run, said};
use orbs_sim::{Sim, tower};

fn quintessence(sim: &Sim) -> u32 {
    sim.world().resource::<tower::Quintessence>().get()
}

/// Leave exactly `held` quintessence in the tower.
fn hold_quintessence(sim: &mut Sim, held: u32) {
    let now = quintessence(sim);
    assert!(now >= held, "the tower holds only {now}, wanted {held}");
    sim.world_mut()
        .resource_mut::<tower::Quintessence>()
        .spend(now - held);
}

/// A laboratory with the fire lit and a stilling-draught on the shelf.
fn ready(seed: u64) -> Sim {
    let mut sim = Sim::new(seed);
    run(&mut sim, "attend laboratory");
    run(&mut sim, "kindle charcoal");
    run(&mut sim, "debug_spawn stilling-draught 1");
    sim
}

fn stillness_held(sim: &mut Sim) -> u32 {
    let world = sim.world_mut();
    world
        .query::<(&tower::Name, &tower::Stock)>()
        .iter(world)
        .filter(|(name, _)| name.0 == "stillness")
        .map(|(_, stock)| match stock {
            tower::Stock::Counted(units) => *units,
            tower::Stock::Endless => u32::MAX,
        })
        .sum()
}

#[test]
fn a_short_tower_cannot_start_stillness_and_spends_nothing() {
    let mut sim = ready(1);
    hold_quintessence(&mut sim, 19);
    run(&mut sim, "distil stilling-draught");
    assert!(
        ever_said(&sim, "quintessence, and you hold 19"),
        "{:?}",
        said(&sim)
    );
    assert_eq!(quintessence(&sim), 19, "a refusal cost quintessence");
}

#[test]
fn starting_stillness_takes_twenty_quintessence() {
    let mut sim = ready(1);
    hold_quintessence(&mut sim, 20);
    run(&mut sim, "distil stilling-draught");
    assert_eq!(quintessence(&sim), 0, "{:?}", said(&sim));

    // Ten minutes, and the alembic hands over three.
    run(&mut sim, "meditate 600");
    run(&mut sim, "empty alembic");
    assert_eq!(stillness_held(&mut sim), 3, "{:?}", said(&sim));
}

#[test]
fn one_brew_of_stillness_can_be_drunk() {
    // A brew makes what drinking one needs held (§19, *Supply*), so the
    // dearest thing the laboratory makes is never stranded short of a use.
    let mut sim = ready(1);
    hold_quintessence(&mut sim, 20);
    run(&mut sim, "distil stilling-draught");
    run(&mut sim, "meditate 600");
    run(&mut sim, "empty alembic");
    run(&mut sim, "move stillness to arsenal");
    run(&mut sim, "attend bailey");
    run(&mut sim, "defend");
    run(&mut sim, "quaff stillness");
    assert!(
        ever_said(&sim, "a stillness falls along the wall"),
        "{:?}",
        said(&sim)
    );
}

#[test]
fn a_slot_refusal_costs_nothing() {
    // The slot is tower-wide and one wide: a grind holds it, so the alembic is
    // refused by `begin` — and quintessence is only taken once `begin` agrees.
    let mut sim = ready(1);
    hold_quintessence(&mut sim, 20);
    run(&mut sim, "grind sage");
    run(&mut sim, "distil stilling-draught");
    assert!(
        ever_said(&sim, "busy"),
        "the slot did not refuse: {:?}",
        said(&sim)
    );
    assert_eq!(quintessence(&sim), 20, "a slot refusal cost quintessence");
}

#[test]
fn stillness_may_be_brewed_while_some_is_held() {
    // `most` is retired (§19, *Supply*): the threshold is the price now.
    let mut sim = ready(1);
    run(&mut sim, "debug_spawn stillness 1");
    hold_quintessence(&mut sim, 20);
    run(&mut sim, "distil stilling-draught");
    assert_eq!(quintessence(&sim), 0, "{:?}", said(&sim));
}

#[test]
fn a_fruitful_alembic_yields_one_more_stillness() {
    let mut sim = ready(1);
    hold_quintessence(&mut sim, 20);
    let world = sim.world_mut();
    let alembic = world
        .query::<(bevy_ecs::prelude::Entity, &tower::Name)>()
        .iter(world)
        .find(|(_, name)| name.0 == "alembic")
        .map(|(alembic, _)| alembic)
        .expect("the laboratory has an alembic");
    let mut charmed = tower::Charmed::default();
    charmed.lay(tower::Charm {
        kind: tower::charm::Kind::Fruitful,
        from: orbs_sim::Tick::new(0),
        ticks: 1_000_000,
    });
    world.entity_mut(alembic).insert(charmed);

    run(&mut sim, "distil stilling-draught");
    run(&mut sim, "meditate 600");
    assert_eq!(
        stillness_held(&mut sim),
        4,
        "fruitful did not add one to the batch: {:?}",
        said(&sim)
    );
}

#[test]
fn a_kept_stillness_goes_stale_like_anything_else() {
    // Brewed for a siege rather than kept against the day (§19, *Supply*):
    // past the stores' window it reads `spent` and is refused, and kept.
    let mut sim = Sim::new(11);
    run(&mut sim, "debug_spawn stillness 1");
    sim.step_n(tower::WINDOW + 600);
    run(&mut sim, "attend bailey");
    run(&mut sim, "defend");
    run(&mut sim, "quaff stillness");
    assert!(ever_said(&sim, "stores are out"), "{:?}", said(&sim));
    assert_eq!(stillness_held(&mut sim), 1, "a stale stillness was drunk");
}

#[test]
fn a_thin_store_runs_only_the_slowest_rate() {
    // Every other test spawns a fresh store; this one is made once, by hand,
    // which is the shelf a player who brews once actually has. Twelve held is
    // enough for vigour's second rate, and thin allows only the first (§19,
    // *Supply*): a supply has to be kept up to run fast.
    let mut sim = Sim::new(11);
    run(&mut sim, "attend bailey");
    let arsenal = tower::keep(sim.world()).expect("the tower has an arsenal");
    tower::give(
        sim.world_mut(),
        arsenal,
        "vigour",
        orbs_sim::parser::NounKind::Essence,
        12,
    );
    tower::made(sim.world_mut(), "vigour");
    run(&mut sim, "defend");
    let world = sim.world_mut();
    let full = world
        .query::<&tower::Siege>()
        .iter(world)
        .next()
        .map(tower::Siege::full)
        .expect("a siege");
    run(&mut sim, "quaff vigour");
    run(&mut sim, "quaff vigour");
    let world = sim.world_mut();
    let after = world
        .query::<&tower::Siege>()
        .iter(world)
        .next()
        .map(tower::Siege::full)
        .expect("a siege");
    assert_eq!(after, full + 3, "{:?}", said(&sim));
    assert!(ever_said(&sim, "stores are thin"), "{:?}", said(&sim));
}

/// What an instrument is holding, by name.
fn held_in(sim: &mut Sim, instrument: &str) -> Vec<String> {
    let world = sim.world_mut();
    let at = world
        .query::<(bevy_ecs::prelude::Entity, &tower::Name)>()
        .iter(world)
        .find(|(_, name)| name.0 == instrument)
        .map_or_else(|| panic!("no {instrument}"), |(at, _)| at);
    tower::holdings(sim.world(), at)
        .into_iter()
        .map(|(name, _)| name)
        .collect()
}

/// The whole route, by the hands a player has: the scroll comes out of the
/// arsenal into the flask, and the draught it makes distils into stillness.
#[test]
fn the_scroll_crosses_into_the_flask_and_the_draught_distils() {
    let mut sim = Sim::new(1);
    for line in [
        "attend laboratory",
        "kindle charcoal",
        "debug_spawn quiet-draught 1",
        "debug_spawn gleaning-scroll 1",
        "move quiet-draught to flask_and_rod",
        "move gleaning-scroll to flask_and_rod",
        "wield flask_and_rod",
        "meditate 22",
    ] {
        run(&mut sim, line);
    }
    assert!(
        held_in(&mut sim, "flask_and_rod").contains(&"stilling-draught".to_owned()),
        "the flask made no stilling-draught: {:?}",
        said(&sim),
    );

    for line in [
        "empty flask_and_rod",
        "distil stilling-draught",
        "meditate 600",
        "empty alembic",
    ] {
        run(&mut sim, line);
    }
    assert_eq!(stillness_held(&mut sim), 3, "{:?}", said(&sim));
}

/// A worn barrier can lower the ceiling under the price, and waiting would
/// never fix that — so the refusal says the barrier, not the pool.
#[test]
fn a_ceiling_under_the_price_is_named_as_the_barrier() {
    let mut sim = ready(1);
    tower::wear_by(sim.world_mut(), u32::MAX);
    let ceiling = tower::ceiling(sim.world());
    assert!(
        ceiling < 20,
        "a ruined barrier still holds {ceiling}, so this asks nothing"
    );
    run(&mut sim, "distil stilling-draught");
    assert!(ever_said(&sim, "worn barrier holds"), "{:?}", said(&sim));
}

/// A save from before the scroll stage may hold a quiet-draught in the alembic,
/// mid-brew. It would land on no recipe; format 16 renames it.
#[test]
fn an_older_brew_in_the_alembic_is_not_fouled() {
    let mut sim = Sim::new(1);
    for line in [
        "attend laboratory",
        "debug_spawn quiet-draught 1",
        "move quiet-draught to alembic",
    ] {
        run(&mut sim, line);
    }
    assert!(
        held_in(&mut sim, "alembic").contains(&"quiet-draught".to_owned()),
        "{:?}",
        said(&sim),
    );

    let text = sim.snapshot().to_toml().expect("a save renders").replacen(
        &format!("format = {}", orbs_sim::save::FORMAT),
        "format = 15",
        1,
    );
    let save = orbs_sim::Save::from_toml(&text).expect("a format-15 save opens");
    let mut loaded = Sim::restored(&save);
    assert_eq!(
        held_in(&mut loaded, "alembic"),
        vec!["stilling-draught".to_owned()],
    );
}
