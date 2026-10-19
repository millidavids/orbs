//! The circle's second use: demons (§19, *Supply*), driven through the game.
//!
//! A bare `summon` that holds refills the standing garrison. `summon golem`
//! binds the same hold into the arsenal as a batch of golems, which `deploy`
//! opens on the wall like any supply.
//!
//! `debug_circle` is the door in, so these are debug-only.
#![cfg(debug_assertions)]

use orbs_render::{FieldName, Value};
use orbs_sim::tower::siege::{GARRISON_FLOOR, Garrison};
use orbs_sim::{Save, Sim, tower};

fn run(sim: &mut Sim, line: &str) {
    sim.submit(line);
    sim.step();
}

fn said(sim: &Sim) -> Vec<String> {
    sim.scrollback()
        .records()
        .iter()
        .filter_map(|record| match record.field(FieldName::Message) {
            Some(Value::Text(text)) => Some(text.to_owned()),
            _ => None,
        })
        .collect()
}

fn ever_said(sim: &Sim, needle: &str) -> bool {
    said(sim).iter().any(|line| line.contains(needle))
}

fn held(sim: &Sim, named: &str) -> u32 {
    tower::supply::held_in_arsenal(sim.world(), named)
}

/// A beast called in and held at the circle, as `demon` or bare.
fn hold(sim: &mut Sim, demon: Option<&str>) {
    run(sim, "attend menagerie");
    run(sim, "summon");
    run(sim, "debug_circle");
    run(
        sim,
        demon
            .map_or_else(|| "summon".to_owned(), |demon| format!("summon {demon}"))
            .as_str(),
    );
}

#[test]
fn a_hold_can_be_bound_as_a_demon_instead_of_bodies() {
    let mut sim = Sim::new(3);
    hold(&mut sim, Some("golem"));
    assert!(
        ever_said(&sim, "4 golem bound to the arsenal"),
        "{:?}",
        said(&sim)
    );
    assert_eq!(held(&sim, "golem"), 4);
    assert_eq!(
        sim.world().resource::<Garrison>().home(),
        GARRISON_FLOOR,
        "a bound hold also refilled the garrison"
    );
}

#[test]
fn a_demon_bound_with_no_arsenal_is_not_announced_or_counted() {
    // The sentence counted four golems that went nowhere, and the stores and
    // renown were paid for a making that reached no shelf.
    let mut sim = Sim::new(3);
    let arsenal = tower::keep(sim.world()).expect("an open tower has an arsenal");
    sim.world_mut().entity_mut(arsenal).remove::<tower::Keep>();
    hold(&mut sim, Some("golem"));
    assert!(ever_said(&sim, "the circle holds"), "{:?}", said(&sim));
    assert!(ever_said(&sim, "no golem is bound"), "{:?}", said(&sim));
    assert!(!ever_said(&sim, "bound to the arsenal"), "{:?}", said(&sim));
    assert_eq!(sim.tally("made:golem"), 0);
}

#[test]
fn a_demon_hold_is_counted_once_as_a_run() {
    let mut sim = Sim::new(3);
    let before = sim.experience();
    hold(&mut sim, Some("familiar"));
    assert_eq!(sim.tally("event:figure"), 1);
    assert_eq!(sim.tally("made:familiar"), 1);
    assert_eq!(sim.tally("made:troop"), 0, "a bound hold counted as bodies");
    assert!(sim.experience() > before, "a bound hold paid nothing");
}

#[test]
fn a_lesser_circle_binds_only_bodies_and_costs_no_call() {
    // A sealed tower at the menagerie's door: lesser beasts, and a golem the
    // test opens by hand, since no station opens one before the whole circle.
    let mut sim = Sim::sealed(3);
    run(&mut sim, "debug_reach lens_1");
    tower::opened::open(sim.world_mut(), &tower::opened::demon_key("golem"));
    run(&mut sim, "attend menagerie");
    run(&mut sim, "summon");
    run(&mut sim, "summon golem");
    assert!(ever_said(&sim, "binds bodies only"), "{:?}", said(&sim));
    let calls = sim.beast().map(tower::circle::Beast::calls);
    assert_eq!(calls, Some(0), "the refusal spent a call");
}

#[test]
fn a_demon_not_yet_opened_cannot_be_bound() {
    let mut sim = Sim::sealed(3);
    run(&mut sim, "debug_reach lens_1");
    run(&mut sim, "attend menagerie");
    run(&mut sim, "summon");
    run(&mut sim, "debug_circle");
    run(&mut sim, "summon golem");
    assert_eq!(held(&sim, "golem"), 0, "an unopened demon was bound");
}

#[test]
fn a_demon_is_deployed_as_a_supply() {
    let mut sim = Sim::new(3);
    run(&mut sim, "attend bailey");
    run(&mut sim, "defend");
    let full = {
        let world = sim.world_mut();
        world
            .query::<&tower::Siege>()
            .iter(world)
            .next()
            .map(tower::Siege::full)
            .expect("a siege")
    };
    run(&mut sim, "debug_spawn effigy 6");
    run(&mut sim, "deploy effigy");
    assert!(ever_said(&sim, "the effigy flows"), "{:?}", said(&sim));
    let world = sim.world_mut();
    let after = world
        .query::<&tower::Siege>()
        .iter(world)
        .next()
        .map(tower::Siege::full)
        .expect("a siege");
    assert_eq!(after, full + 3);
}

#[test]
fn every_demon_has_a_home_a_tint_and_a_page() {
    let sim = Sim::new(3);
    let spendables = orbs_sim::content::Spendables::builtin();
    let materials = orbs_sim::content::Materials::builtin();
    let prose = orbs_sim::content::Prose::builtin();
    let arsenal = tower::keep(sim.world());
    for demon in spendables.demons() {
        assert_eq!(
            tower::home(sim.world(), demon),
            arsenal,
            "{demon} has no home"
        );
        assert!(materials.names().contains(&demon), "{demon} has no tint");
        assert!(prose.has(&format!("recall_{demon}")), "{demon} has no page");
        assert_eq!(
            spendables.verb_for(demon),
            Some(orbs_sim::parser::Verb::Deploy)
        );
    }
}

#[test]
fn an_open_towers_older_document_gains_its_demons() {
    let sim = Sim::new(3);
    let mut save = sim.snapshot();
    if let Some(opened) = save.progress.opened.as_mut() {
        opened.retain(|key| !key.starts_with("demon:"));
    }
    save.world.format = 18;
    let text = save.to_toml().expect("a save renders");
    let loaded = Sim::restored(&Save::from_toml(&text).expect("a format-18 save opens"));
    for demon in orbs_sim::content::Spendables::builtin().demons() {
        assert!(
            loaded
                .world()
                .resource::<tower::Opened>()
                .has(&tower::opened::demon_key(demon)),
            "an open tower lost {demon}"
        );
    }
}
