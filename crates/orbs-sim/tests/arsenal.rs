//! The arsenal: finished work, and the one room reachable from every other.
//!
//! §7 scopes naming to where you are standing, and that held while nothing a
//! domain made was wanted anywhere else. A finished thing breaks it — and
//! **nothing in the tower could be carried between rooms at all**, because
//! `carry`'s destination lookup wants a `Fixture` child of `cwd` and a domain is
//! neither.
//!
//! The exemption this makes is narrow and `tower::keep` states it. What this file
//! holds is the part that is easy to get wrong: **nameable is not enough**. Every
//! lookup that can now name what is in here has to reach it, or a word resolves
//! at full confidence and then reports "no such thing" — §15's dead end, arriving
//! through the affordance meant to remove one.

use orbs_render::{FieldName, Value};
use orbs_sim::Sim;

/// A potion in the arsenal, brewed and carried the way a player would.
///
/// **Distilled rather than spawned into place.** `debug_spawn` would put a
/// `clarity` on the shelf without it ever having been made, which is the state
/// under test in reverse: what has to work is *carrying finished work out of the
/// room that finished it*.
fn with_a_potion() -> Sim {
    let mut sim = Sim::new(1);
    for line in [
        "attend laboratory",
        "kindle charcoal",
        "debug_spawn clarified-draught",
        "distil clarified-draught",
    ] {
        sim.submit(line);
        sim.step();
    }
    sim.step_n(60);
    sim.submit("empty alembic");
    sim.step();
    sim.submit("move clarity to arsenal");
    sim.step();
    sim
}

/// Every message the orb has said.
fn messages(sim: &Sim) -> Vec<String> {
    sim.scrollback()
        .records()
        .iter()
        .filter_map(|record| record.field(FieldName::Message))
        .filter_map(|value| match value {
            Value::Text(text) => Some(text.to_owned()),
            _ => None,
        })
        .collect()
}

/// Whether the orb's last answer was *"there is no ... within reach"*.
///
/// **The shape of the failure this whole file is about.** A verb that can name a
/// thing and cannot reach it does not crash and does not refuse in character; it
/// says the thing is not there, which is the one answer that is false.
fn said_missing(sim: &Sim, since: usize) -> bool {
    messages(sim)
        .into_iter()
        .skip(since)
        .any(|line| line.contains("within reach"))
}

/// What **work** is in the arsenal right now, by name.
///
/// The log is not work. Every domain is raised with one, so a room's own
/// furniture is always in there and counting it would make an empty arsenal read
/// as holding something.
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
    // **This was broken for as long as there have been potions.** `move`'s first
    // slot was `NounKind::Reagent` and `produce::transmute` gives a `potion =
    // true` output `NounKind::Essence`, so `move clarity ...` could not fill a
    // required slot — silently, because `empty` turns an instrument out
    // wholesale and never asks what kind anything is, so the one route that
    // mattered inside the laboratory worked.
    let sim = with_a_potion();
    assert_eq!(kept(&sim), ["clarity"], "the potion never left the shelf");
}

#[test]
fn the_arsenal_takes_finished_work_and_refuses_stock() {
    // The rule that stops this becoming a second dispensary. Asked of the
    // **kind**, never of the name: telling finished work from stock by name
    // would mean the tower deciding which reagents are waste, which §10.1
    // refuses outright.
    let mut sim = with_a_potion();
    let before = messages(&sim).len();
    sim.submit("move sage to arsenal");
    sim.step();

    assert_eq!(kept(&sim), ["clarity"], "the arsenal took a reagent");
    assert!(
        !said_missing(&sim, before),
        "the sage was reported absent rather than refused: {:?}",
        messages(&sim),
    );
    // §6 forbids a bare error, so the refusal has to say where sage *does*
    // belong rather than only that it does not belong here.
    assert!(
        messages(&sim)
            .into_iter()
            .skip(before)
            .any(|line| line.contains("shelf")),
        "the refusal did not say where a reagent goes: {:?}",
        messages(&sim),
    );
}

#[test]
fn every_verb_that_can_name_a_kept_thing_can_reach_it() {
    // **The trap the whole exemption turns on.** Registering the arsenal's
    // contents in the scene makes them nameable everywhere — and `purge` and
    // `verify` take `NounKind::Any`, so they will now *resolve* on a potion from
    // any room. Both searched `cwd`'s children and nothing else, so both would
    // have resolved at full confidence and then reported the potion absent.
    //
    // Driven per verb, not asserted on `accepts`: what has to be true is the
    // answer the player gets.
    for verb in ["verify clarity", "peruse arsenal.log", "purge clarity"] {
        let mut sim = with_a_potion();
        sim.submit("attend archive");
        sim.step();
        let before = messages(&sim).len();
        sim.submit(verb);
        sim.step_n(6);
        assert!(
            !said_missing(&sim, before),
            "`{verb}` named what it could not reach: {:?}",
            messages(&sim),
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
    // `reachable` puts the arsenal **last**, so a reagent in the room always
    // outranks one carried — which is what makes adding it unable to change what
    // any existing command picks up. Checked with the same name in both places.
    let mut sim = with_a_potion();
    sim.submit("attend laboratory");
    sim.step();
    // A second clarity, on the shelf this time.
    sim.submit("debug_spawn clarity");
    sim.step();

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
    // §19 records the archive's log being empty from the day it was built,
    // because every completion set none of the three fields `files::in_domain`
    // matches on. A domain whose log is always blank is a `peruse` that looks
    // broken, and it is invisible until somebody tries it.
    let mut sim = with_a_potion();
    sim.submit("attend archive");
    sim.step();
    let before = messages(&sim).len();
    sim.submit("peruse arsenal.log");
    sim.step();

    let read: Vec<String> = messages(&sim).into_iter().skip(before).collect();
    assert!(
        read.iter().any(|line| line.contains("arsenal")),
        "arsenal.log is empty: {read:?}",
    );
}

#[test]
fn the_arsenal_refuses_to_be_unmade() {
    // It holds everything the player has finished, so §7's guard matters more
    // here than anywhere. It comes free from being a top-level branch, and this
    // is what says so — a future `Role` that forgot to raise it as one would
    // make `purge arsenal` destroy a session's work.
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
    // **Pillar 3's half of the feature, and it is not obvious.** A spell is
    // written *for* a domain and `may_issue` forbids it `attend`ing, so it can
    // only name what is in scope where it runs. Finished work that lived in the
    // room that made it could never be used by a spell running anywhere else —
    // so the arsenal is what makes automation able to touch a potion at all.
    let mut sim = with_a_potion();
    sim.submit("attend archive");
    sim.step();
    sim.write_spell("checking", &["verify clarity".to_owned()]);
    sim.step();
    let before = messages(&sim).len();
    sim.submit("invoke checking");
    sim.step_n(6);

    assert!(
        !said_missing(&sim, before),
        "a spell could not reach the arsenal: {:?}",
        messages(&sim),
    );
    assert!(
        sim.scrollback()
            .records()
            .iter()
            .any(|record| record.field(FieldName::Source) == Some(Value::Text("clarity"))),
        "the spell never verified the potion",
    );
}
