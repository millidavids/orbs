//! The arsenal: finished work, and the one room reachable from every other.
//!
//! `tower::keep`'s exemption to §7's scoping. Nameable is not enough: every
//! lookup that names what is here must reach it, or a word resolves and then
//! finds nothing (§15).

mod common;

use common::{brew_one, run, run_all, said};
use orbs_render::{FieldName, Value};
use orbs_sim::Sim;

/// A potion in the arsenal, brewed and carried the way a player would.
///
/// Distilled rather than spawned into place: `debug_spawn` would put a `clarity`
/// on the shelf without it ever having been made, which is the state under test
/// in reverse.
fn with_a_potion() -> Sim {
    let mut sim = Sim::new(1);
    // §10.1's chain from endless stock, with no debug door, so the file runs
    // in release: the arsenal is a shipped room.
    run_all(&mut sim, &["attend laboratory", "kindle charcoal"]);
    brew_to_the_shelf(&mut sim);
    run(&mut sim, "move clarity to arsenal");

    assert!(
        !kept(&sim).is_empty(),
        "the chain did not reach a potion, so every test in this file proves nothing",
    );
    sim
}

/// §10.1's five stages, once, leaving a `clarity` on the laboratory's shelf.
fn brew_to_the_shelf(sim: &mut Sim) {
    brew_one(sim);
    run(sim, "empty alembic");
}

/// Whether the orb's last answer was *"there is no ... within reach"*.
///
/// The shape of the failure this file is about: a verb that can name a thing and
/// cannot reach it says the thing is not there, which is the one false answer.
fn said_missing(sim: &Sim, since: usize) -> bool {
    said(sim)
        .into_iter()
        .skip(since)
        .any(|line| line.contains("within reach"))
}

/// What *work* is in the arsenal right now, by name.
///
/// The log is not work. Every domain is raised with one, so counting it would
/// make an empty arsenal read as holding something.
fn kept(sim: &Sim) -> Vec<String> {
    let world = sim.world();
    orbs_sim::tower::keeping(world)
        .into_iter()
        .filter(|node| {
            world.get::<orbs_sim::tower::Nameable>(*node).map(|k| k.0)
                != Some(orbs_sim::parser::NounKind::File)
        })
        .filter_map(|node| world.get::<orbs_sim::Name>(node).map(|name| name.0.clone()))
        .collect()
}

#[test]
fn a_finished_potion_can_be_picked_up_at_all() {
    // Broken for as long as there have been potions: `move`'s first slot was
    // `NounKind::Reagent` and a potion is `NounKind::Essence`, so `move clarity
    // ...` could not fill a required slot — silently, because `empty` never asks
    // what kind anything is and that was the route that mattered.
    let sim = with_a_potion();
    assert_eq!(kept(&sim), ["clarity"], "the potion never left the shelf");
}

#[test]
fn a_move_to_the_arsenal_carries_the_whole_batch() {
    // A supply opens on how much the arsenal holds (§19, *Supply*), and a run
    // makes three. Carried one at a time, opening a warding took six words.
    let sim = with_a_potion();
    assert_eq!(
        orbs_sim::tower::supply::held_in_arsenal(sim.world(), "clarity"),
        3
    );
    assert!(
        said(&sim)
            .iter()
            .any(|line| line == "3 clarity: dispensary to arsenal"),
        "{:?}",
        said(&sim)
    );
}

#[cfg(debug_assertions)]
#[test]
fn a_move_into_an_instrument_still_carries_one() {
    // An instrument is charged a unit at a time, and a run spends one.
    let mut sim = Sim::new(1);
    for line in [
        "attend archive",
        "debug_spawn fragment 4",
        "move fragment to lectern",
    ] {
        sim.submit(line);
        sim.step();
    }
    let world = sim.world();
    let lectern =
        orbs_sim::tower::find_by_path(world, "/tower/archive/lectern").expect("a lectern");
    assert_eq!(
        orbs_sim::tower::held(world, lectern, "fragment"),
        Some(orbs_sim::tower::Stock::Counted(1)),
        "{:?}",
        said(&sim)
    );
}

#[test]
fn the_arsenal_takes_finished_work_and_refuses_stock() {
    // The rule that stops this becoming a second dispensary. Asked of the kind,
    // never the name: telling finished work from stock by name would mean the
    // tower deciding which reagents are waste, which §10.1 refuses.
    let mut sim = with_a_potion();
    let before = said(&sim).len();
    sim.submit("move sage to arsenal");
    sim.step();

    assert_eq!(kept(&sim), ["clarity"], "the arsenal took a reagent");
    assert!(
        !said_missing(&sim, before),
        "the sage was reported absent rather than refused: {:?}",
        said(&sim),
    );
    // §6 forbids a bare error, so the refusal says where sage *does* belong.
    assert!(
        said(&sim)
            .into_iter()
            .skip(before)
            .any(|line| line.contains("shelf")),
        "the refusal did not say where a reagent goes: {:?}",
        said(&sim),
    );
}

#[test]
fn every_verb_that_can_name_a_kept_thing_can_reach_it() {
    // `purge` and `verify` take `NounKind::Any`, so they must find what the
    // scene names. Driven per verb, on the answer the player gets.
    for verb in ["verify clarity", "peruse arsenal.log", "purge clarity"] {
        let mut sim = with_a_potion();
        sim.submit("attend archive");
        sim.step();
        let before = said(&sim).len();
        sim.submit(verb);
        sim.step_n(6);
        assert!(
            !said_missing(&sim, before),
            "`{verb}` named what it could not reach: {:?}",
            said(&sim),
        );
    }

    // And `purge` did not merely resolve — it acted.
    let mut sim = with_a_potion();
    sim.submit("attend archive");
    sim.step();
    sim.submit("purge clarity");
    sim.step_n(6);
    assert!(kept(&sim).is_empty(), "purge resolved and did nothing");
}

#[test]
fn what_the_arsenal_holds_can_be_carried_back_out() {
    // Half a door is not a door. Getting finished work *in* is `receiver`;
    // getting it out again is `reachable`, and a room you can only fill is a
    // room that eats potions.
    let mut sim = with_a_potion();
    sim.submit("attend laboratory");
    sim.step();
    sim.submit("move clarity to dispensary");
    sim.step();
    assert!(kept(&sim).is_empty(), "the clarity could not be taken back");
}

#[test]
fn the_arsenal_is_not_a_second_dispensary() {
    // `reachable` puts the arsenal last, so a reagent in the room outranks one
    // carried and adding it cannot change what an existing command picks up.
    // Checked with the same name in both places.
    let mut sim = with_a_potion();
    run(&mut sim, "attend laboratory");
    // A second clarity, on the shelf this time — brewed, not spawned, so this
    // test runs in a release build where `debug_spawn` does not exist.
    brew_to_the_shelf(&mut sim);

    sim.submit("move clarity to flask_and_rod");
    sim.step();
    assert_eq!(
        kept(&sim),
        ["clarity"],
        "a bare `move` raided the arsenal while the shelf had one",
    );
}

#[test]
fn the_arsenal_keeps_a_log_that_fills() {
    // The archive's log was empty from the day it was built, because every
    // completion set none of the fields `files::in_domain` matches on (§19) —
    // invisible until somebody tries it.
    let mut sim = with_a_potion();
    sim.submit("attend archive");
    sim.step();
    let before = said(&sim).len();
    sim.submit("peruse arsenal.log");
    sim.step();

    let read: Vec<String> = said(&sim).into_iter().skip(before).collect();
    assert!(
        read.iter().any(|line| line.contains("arsenal")),
        "arsenal.log is empty: {read:?}",
    );
}

#[test]
fn the_arsenal_refuses_to_be_unmade() {
    // It holds everything the player has finished, so §7's guard matters more
    // here than anywhere. It comes free from being a top-level branch; a future
    // `Role` that forgot to raise it as one would make `purge arsenal` destroy
    // a session's work.
    let mut sim = with_a_potion();
    sim.submit("purge arsenal");
    sim.step_n(6);
    assert_eq!(
        kept(&sim),
        ["clarity"],
        "the arsenal was scoured by a verb §7 says must refuse",
    );
}

#[test]
fn a_spell_can_use_what_the_arsenal_holds() {
    // Pillar 3's half of the feature. A spell is written *for* a domain and
    // `may_issue` forbids it `attend`ing, so it can only name what is in scope
    // where it runs — the arsenal is what lets automation touch a potion at all.
    let mut sim = with_a_potion();
    sim.submit("attend archive");
    sim.step();
    sim.write_spell("checking", &["verify clarity".to_owned()]);
    sim.step();
    let before = said(&sim).len();
    sim.submit("invoke checking");
    sim.step_n(6);

    assert!(
        !said_missing(&sim, before),
        "a spell could not reach the arsenal: {:?}",
        said(&sim),
    );
    assert!(
        sim.scrollback()
            .records()
            .iter()
            .any(|record| record.field(FieldName::Source) == Some(Value::Text("clarity"))),
        "the spell never verified the potion",
    );
}
