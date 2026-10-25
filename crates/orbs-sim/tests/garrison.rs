//! The tower's standing garrison (§19, *Supply*), driven through the real game.
//!
//! Troops left the arsenal. The tower holds a garrison of its own: a most
//! nothing passes, a floor the bailey keeps without a menagerie, and desertion
//! between. A held beast refills it; `defend` levies all of it.
//!
//! `debug_circle` and `debug_siege` are the doors in, so these are debug-only.
#![cfg(debug_assertions)]

mod common;

use common::{ever_said, reloaded, run, said};
use orbs_render::{FieldName, Value};
use orbs_sim::tower::siege::{self, GARRISON_FLOOR, GARRISON_MOST, Garrison, Siege};
use orbs_sim::{Save, Sim};

fn garrison(sim: &Sim) -> Garrison {
    *sim.world().resource::<Garrison>()
}

fn set(sim: &mut Sim, standing: u32, reserve: u32) {
    sim.world_mut()
        .resource_mut::<Garrison>()
        .restore(standing, reserve);
}

/// One beast held at the circle, from wherever the player stands.
fn hold(sim: &mut Sim) {
    for line in ["attend menagerie", "summon", "debug_circle", "summon"] {
        run(sim, line);
    }
}

fn siege(sim: &mut Sim) -> Option<Siege> {
    let world = sim.world_mut();
    world.query::<&Siege>().iter(world).next().cloned()
}

/// Win the siege that stands: one round from won, then the round.
fn win(sim: &mut Sim) {
    run(sim, "attend bailey");
    run(sim, "debug_siege");
    for _ in 0..20 {
        if siege(sim).is_some_and(|siege| !siege.running()) {
            return;
        }
        run(sim, "hold");
    }
    panic!("the siege never ended: {:?}", said(sim));
}

#[test]
fn a_new_tower_stands_at_its_floor() {
    let sim = Sim::new(3);
    assert_eq!(garrison(&sim).standing(), GARRISON_FLOOR);
    assert!(sim.holds_reading("menagerie", "roster", "spears"));
    assert!(!sim.holds_reading("menagerie", "roster", "full"));
}

#[test]
fn the_garrison_never_exceeds_its_most() {
    let mut sim = Sim::new(3);
    set(&mut sim, GARRISON_MOST - 2, 0);
    hold(&mut sim);
    assert_eq!(garrison(&sim).home(), GARRISON_MOST, "{:?}", said(&sim));
    assert!(
        ever_said(
            &sim,
            "2 join the garrison, and 2 are turned away at its most"
        ),
        "{:?}",
        said(&sim)
    );
    assert!(sim.holds_reading("menagerie", "roster", "full"));
    hold(&mut sim);
    assert_eq!(garrison(&sim).home(), GARRISON_MOST);
}

#[test]
fn a_full_garrison_turns_bodies_away_and_still_counts_the_run() {
    let mut sim = Sim::new(3);
    set(&mut sim, GARRISON_MOST, 0);
    let before = sim.experience();
    hold(&mut sim);
    assert_eq!(garrison(&sim).home(), GARRISON_MOST);
    assert!(
        ever_said(&sim, "the garrison is at its most"),
        "{:?}",
        said(&sim)
    );
    assert_eq!(
        sim.tally("event:figure"),
        1,
        "a turned-away hold went uncounted"
    );
    assert!(sim.experience() > before, "a turned-away hold paid nothing");
}

#[test]
fn desertion_stops_at_the_floor_and_pauses_in_a_siege() {
    let mut sim = Sim::new(3);
    set(&mut sim, GARRISON_MOST, 0);
    sim.step_n(siege::DESERT_EVERY * 2);
    assert_eq!(garrison(&sim).standing(), GARRISON_MOST - 2);
    sim.step_n(siege::DESERT_EVERY * 20);
    assert_eq!(
        garrison(&sim).standing(),
        GARRISON_FLOOR,
        "desertion passed the floor"
    );

    // Mid-siege nobody leaves, however long the player takes.
    run(&mut sim, "attend bailey");
    run(&mut sim, "defend");
    set(&mut sim, 9, 0);
    sim.step_n(siege::DESERT_EVERY * 5);
    assert_eq!(
        garrison(&sim).standing(),
        9,
        "bodies deserted a running siege"
    );
}

#[test]
fn desertion_is_the_same_however_the_ticks_are_taken() {
    let lived = |collapsed: bool| {
        let mut sim = Sim::new(5);
        set(&mut sim, GARRISON_MOST, 0);
        if collapsed {
            run(&mut sim, "meditate 1000");
        } else {
            sim.step_n(1001);
        }
        garrison(&sim).standing()
    };
    assert_eq!(lived(true), lived(false));
}

#[test]
fn defend_musters_the_whole_standing_garrison() {
    let mut sim = Sim::new(3);
    set(&mut sim, 9, 0);
    run(&mut sim, "attend bailey");
    run(&mut sim, "defend");
    let fight = siege(&mut sim).expect("defend opened a siege");
    assert_eq!(fight.garrison.count, 9);
    assert_eq!(fight.levied, 9);
    assert_eq!(garrison(&sim).standing(), 0, "the levy left bodies at home");
}

#[test]
fn a_hold_mid_siege_waits_in_reserve_and_joins_at_settle() {
    let mut sim = Sim::new(3);
    run(&mut sim, "attend bailey");
    run(&mut sim, "defend");
    hold(&mut sim);
    assert!(ever_said(&sim, "wait in reserve"), "{:?}", said(&sim));
    assert_eq!(garrison(&sim).reserve(), 4);
    let line = siege(&mut sim).map(|fight| fight.garrison.count);
    assert_eq!(
        line,
        Some(GARRISON_FLOOR),
        "the reserve reached the fighting line"
    );

    win(&mut sim);
    let home = garrison(&sim);
    assert_eq!(home.reserve(), 0);
    assert!(home.standing() >= GARRISON_FLOOR);
    assert!(home.standing() <= GARRISON_MOST);
}

#[test]
fn survivors_return_but_refills_do_not() {
    let mut sim = Sim::new(3);
    run(&mut sim, "attend bailey");
    run(&mut sim, "defend");
    // Four scrolls flow at the slowest rate: one body a round.
    run(&mut sim, "debug_spawn verdant-scroll 4");
    run(&mut sim, "wield verdant-scroll");
    let line = siege(&mut sim).map(|fight| fight.garrison.count);
    assert_eq!(line, Some(GARRISON_FLOOR + 1), "{:?}", said(&sim));

    win(&mut sim);
    assert_eq!(
        garrison(&sim).standing(),
        GARRISON_FLOOR,
        "a refill came home as standing garrison"
    );
}

#[test]
fn a_hold_mid_siege_never_fills_the_reserve_past_what_the_line_leaves() {
    // The levied line comes home too. A reserve filled to the most was clamped
    // away at settle after the player was told the bodies were waiting.
    let mut sim = Sim::new(3);
    set(&mut sim, 8, 0);
    run(&mut sim, "attend bailey");
    run(&mut sim, "defend");
    hold(&mut sim);
    assert_eq!(garrison(&sim).reserve(), GARRISON_MOST - 8);
    assert!(
        ever_said(&sim, "wait in reserve, and 2 are turned away"),
        "{:?}",
        said(&sim)
    );

    // A full line leaves no room at all, and says so.
    let mut full = Sim::new(3);
    set(&mut full, GARRISON_MOST, 0);
    run(&mut full, "attend bailey");
    run(&mut full, "defend");
    hold(&mut full);
    assert_eq!(garrison(&full).reserve(), 0);
    assert!(ever_said(&full, "turned away"), "{:?}", said(&full));
}

#[test]
fn a_refilled_casualty_comes_home_as_a_healed_one_would() {
    // Levy nine and lose four. A heal brings the four back and so does a
    // refill. Subtracting the refill made a golem cost bodies a heal kept.
    let home = |refill: bool| {
        let mut sim = Sim::new(3);
        set(&mut sim, 9, 0);
        run(&mut sim, "attend bailey");
        run(&mut sim, "defend");
        let (survivors, levied) = {
            let world = sim.world_mut();
            let mut sieges = world.query::<&mut Siege>();
            let mut fight = sieges.iter_mut(world).next().expect("a siege");
            fight.garrison.count = 5;
            fight.garrison.vigour = 5 * siege::VIGOUR;
            if refill {
                assert_eq!(fight.reinforce(4), 4);
            } else {
                fight.heal(4 * siege::VIGOUR);
            }
            (fight.garrison.count, fight.levied)
        };
        siege::disband(sim.world_mut(), survivors, levied);
        garrison(&sim).standing()
    };
    assert_eq!(home(false), 9, "a healed line came home short");
    assert_eq!(home(true), home(false), "a refill cost what a heal kept");
}

#[test]
fn a_fallen_garrison_is_reassigned_its_floor() {
    let mut sim = Sim::new(3);
    set(&mut sim, 0, 0);
    siege::disband(sim.world_mut(), 0, 9);
    assert_eq!(garrison(&sim).standing(), GARRISON_FLOOR);
}

#[test]
fn the_bailey_stands_without_the_menagerie() {
    // A sealed game can arm the wall before the menagerie opens.
    let mut sim = Sim::sealed(3);
    run(&mut sim, "debug_reach sanctum_1");
    run(&mut sim, "attend bailey");
    run(&mut sim, "defend");
    let line = siege(&mut sim).map(|fight| fight.garrison.count);
    assert_eq!(line, Some(GARRISON_FLOOR), "{:?}", said(&sim));
}

#[test]
fn a_standing_garrison_travels_in_the_save() {
    let mut sim = Sim::new(3);
    set(&mut sim, 9, 0);
    let loaded = reloaded(&sim.snapshot());
    assert_eq!(garrison(&loaded).standing(), 9);
}

/// A format-16 document with troops in the arsenal: each was two bodies on the
/// wall, so that is what comes home — to the reserve if a siege is open.
#[test]
fn an_older_documents_troops_join_the_garrison() {
    let older = |fighting: bool, troops: u32| {
        let mut sim = Sim::new(3);
        run(&mut sim, &format!("debug_spawn clarity {troops}"));
        if fighting {
            run(&mut sim, "attend bailey");
            run(&mut sim, "defend");
        }
        let mut save = sim.snapshot();
        // A held troop, as format 16 wrote one: a counted node in the arsenal.
        let mut troop = save
            .nodes
            .iter()
            .find(|node| node.path.ends_with("/arsenal/clarity"))
            .cloned()
            .expect("the arsenal holds clarity");
        troop.path = troop.path.replace("/clarity", "/troop");
        save.nodes.push(troop);
        save.progress.garrison = None;
        save.world.format = 16;
        let text = save.to_toml().expect("a save renders");
        Sim::restored(&Save::from_toml(&text).expect("a format-16 save opens"))
    };

    let calm = older(false, 2);
    assert_eq!(garrison(&calm).standing(), GARRISON_FLOOR + 4);
    assert!(
        !calm.holds_reading("arsenal", "troop", "fresh"),
        "the troop node survived the migration"
    );

    let fighting = older(true, 2);
    assert_eq!(garrison(&fighting).standing(), 0);
    assert_eq!(garrison(&fighting).reserve(), 4);

    // Ten bodies mid-siege: the reserve fills only what the levied line
    // leaves, as `enlist` would, so nothing is told it waits and then lost.
    let mut crowded = older(true, 5);
    assert_eq!(garrison(&crowded).reserve(), GARRISON_MOST - GARRISON_FLOOR);
    assert_eq!(siege::held(crowded.world_mut()), GARRISON_MOST);
}

/// Every named count `status` printed, newest last.
fn status_rows(sim: &mut Sim) -> Vec<(String, u64)> {
    run(sim, "status");
    sim.scrollback()
        .records()
        .iter()
        .filter_map(|record| {
            let name = match record.field(FieldName::Name) {
                Some(Value::Text(text)) => text.to_owned(),
                _ => return None,
            };
            match record.field(FieldName::Quantity) {
                Some(Value::Count(count)) => Some((name, count)),
                _ => None,
            }
        })
        .collect()
}

#[test]
fn status_says_the_garrison_and_its_most() {
    let mut sim = Sim::new(3);
    let rows = status_rows(&mut sim);
    assert!(rows.contains(&("garrison".to_owned(), u64::from(GARRISON_FLOOR))));
    assert!(rows.contains(&("garrison most".to_owned(), u64::from(GARRISON_MOST))));
}

#[test]
fn the_roster_and_status_count_the_line_on_the_wall() {
    // The levy left nought at home, so mid-siege the roster dropped `full` and
    // `status` said nought, while a hold was turned away as full.
    let mut sim = Sim::new(3);
    set(&mut sim, GARRISON_MOST, 0);
    sim.step();
    assert!(sim.holds_reading("menagerie", "roster", siege::FULL));
    run(&mut sim, "attend bailey");
    run(&mut sim, "defend");
    sim.step();
    assert!(
        sim.holds_reading("menagerie", "roster", siege::FULL),
        "the levied line stopped counting"
    );
    let garrison = status_rows(&mut sim)
        .into_iter()
        .rev()
        .find(|(name, _)| name == "garrison")
        .map(|(_, count)| count);
    assert_eq!(garrison, Some(u64::from(GARRISON_MOST)));

    hold(&mut sim);
    assert!(
        ever_said(&sim, "turned away"),
        "the roster said full and the hold disagreed: {:?}",
        said(&sim)
    );
}
