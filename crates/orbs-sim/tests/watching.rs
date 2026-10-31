//! A spell asking about another spell: `if first_light is running`.
//!
//! A spell is nameable from anywhere, so any spell may ask whether one runs —
//! the question a spell that starts others needs before it starts one twice
//! (§19, *The tower's spell*). Only `is`: a spell holds nothing to count.

mod common;

use common::{run_all, said};
use orbs_sim::Sim;

/// A laboratory with `lines` written as `asking`, and the branch it took said
/// as `yesmark` or `nomark` — two `wait`s, which speak at once and change
/// nothing. `None` is neither half, an unanswerable question.
fn asks(sim: &mut Sim, question: &str) -> Option<bool> {
    let lines = [
        format!("if {question}"),
        "wait for yesmark".to_owned(),
        "else".to_owned(),
        "wait for nomark".to_owned(),
        "end".to_owned(),
    ];
    sim.write_spell("asking", &lines);
    sim.step();
    let before = said(sim).len();
    run_all(sim, &["invoke asking"]);
    sim.step_n(4);
    let said: Vec<String> = said(sim).into_iter().skip(before).collect();
    run_all(sim, &["stop asking"]);
    match (
        said.iter().any(|line| line.contains("yesmark")),
        said.iter().any(|line| line.contains("nomark")),
    ) {
        (true, false) => Some(true),
        (false, true) => Some(false),
        (false, false) => None,
        (true, true) => panic!("both halves ran for {question:?}"),
    }
}

/// A laboratory, a spell `waiting` that holds for a while, and nothing cast.
fn laboratory() -> Sim {
    let mut sim = Sim::new(1);
    run_all(&mut sim, &["attend laboratory"]);
    sim.write_spell("waiting", &["bide 200".to_owned()]);
    sim.step();
    sim
}

#[test]
fn a_spell_can_ask_whether_another_spell_is_running() {
    let mut sim = laboratory();
    assert_eq!(asks(&mut sim, "waiting is running"), Some(false));
    assert_eq!(asks(&mut sim, "waiting is idle"), Some(true));

    run_all(&mut sim, &["invoke waiting"]);
    assert_eq!(asks(&mut sim, "waiting is running"), Some(true));
    assert_eq!(asks(&mut sim, "waiting.spell is working"), Some(true));
    assert_eq!(asks(&mut sim, "not waiting is idle"), Some(true));
}

#[test]
fn a_held_spell_between_laps_is_still_running() {
    // A held spell has no `Running` for the tick between laps, so asking only
    // that would read it idle and start it twice. Bound is working.
    let mut sim = laboratory();
    run_all(&mut sim, &["kindle charcoal"]);
    for _ in 0..2 {
        common::brew_one(&mut sim);
    }
    assert_eq!(sim.concentration(), 1, "{:?}", said(&sim));
    sim.write_spell("lapping", &["survey".to_owned()]);
    sim.step();
    run_all(&mut sim, &["bind lapping"]);

    for _ in 0..4 {
        assert_eq!(asks(&mut sim, "lapping is idle"), Some(false));
    }
}

#[test]
fn a_blank_spell_is_empty() {
    let mut sim = laboratory();
    sim.write_spell("blank", &[String::new()]);
    sim.step();
    assert_eq!(asks(&mut sim, "blank is empty"), Some(true));
    assert_eq!(asks(&mut sim, "waiting is empty"), Some(false));
}

#[test]
fn a_spell_has_nothing_to_count() {
    // `has` keeps today's answer: a spell is no place, and the question names
    // the one the tower lacks rather than quietly answering no.
    let mut sim = laboratory();
    assert_eq!(asks(&mut sim, "waiting has sage"), None);
    assert!(
        said(&sim)
            .iter()
            .any(|line| line.contains("there is no waiting here to ask about")),
        "{:?}",
        said(&sim),
    );
}

#[test]
fn a_question_about_a_spell_reads_cleanly_in_the_editor() {
    let sim = laboratory();
    let lines = vec![
        "invoke waiting".to_owned(),
        "if waiting is running".to_owned(),
        "wait for yes".to_owned(),
        "end".to_owned(),
    ];
    let reading = sim.read_spell("laboratory", &lines);
    assert!(
        reading.iter().all(|line| line.fault.is_none()),
        "{reading:?}"
    );
    // The fair copy's word for it, as `if the mortar is busy` reads.
    assert_eq!(reading[1].heard, "if waiting is working", "{reading:?}");
}

#[test]
fn a_spell_not_written_yet_is_a_place_the_tower_lacks() {
    let mut sim = laboratory();
    assert_eq!(asks(&mut sim, "nonesuch is running"), None);
}

#[test]
fn a_place_beats_a_spell_of_the_same_bare_name() {
    // The athanor is a place in this room; a spell called `athanor` is a spell.
    // The room reads first, so the question asks the fire.
    let mut sim = laboratory();
    sim.write_spell("athanor", &["bide 200".to_owned()]);
    sim.step();
    run_all(&mut sim, &["invoke athanor"]);
    assert_eq!(
        asks(&mut sim, "athanor is working"),
        Some(false),
        "the spell answered for the fire"
    );
}

#[test]
fn waiting_for_a_spell_is_repeat_until_it_is_idle() {
    let mut sim = laboratory();
    sim.write_spell("short", &["bide 3".to_owned()]);
    sim.step();
    sim.write_spell(
        "after",
        &[
            "invoke short".to_owned(),
            "repeat until short is idle".to_owned(),
            "bide 1".to_owned(),
            "end".to_owned(),
            "wait for donemark".to_owned(),
        ],
    );
    sim.step();
    run_all(&mut sim, &["invoke after"]);
    sim.step_n(3);
    assert!(
        !said(&sim).iter().any(|line| line.contains("donemark")),
        "it did not wait for the spell"
    );
    sim.step_n(10);
    assert!(
        said(&sim).iter().any(|line| line.contains("donemark")),
        "it never saw the spell end: {:?}",
        said(&sim),
    );
}

#[test]
fn a_repeat_until_reads_a_bound_name() {
    // `if` stood a bound name for its place; `until` asked the name itself, so a
    // part waiting on its own argument could never stop (§19).
    let mut sim = laboratory();
    sim.write_spell(
        "tending",
        &[
            "part tend(task)".to_owned(),
            "    repeat until task is idle".to_owned(),
            "        bide 1".to_owned(),
            "    end".to_owned(),
            "    wait for tendedmark".to_owned(),
            "end".to_owned(),
            "invoke waiting".to_owned(),
            "tend(waiting)".to_owned(),
        ],
    );
    sim.step();
    run_all(&mut sim, &["invoke tending"]);
    sim.step_n(6);
    // Still in the loop, lines 2–3; stopped, it would be at the `wait` on 5.
    assert!(
        matches!(sim.running_line("tending"), Some(2 | 3)),
        "the loop stopped while the spell it waits on runs, at line {:?}: {:?}",
        sim.running_line("tending"),
        said(&sim),
    );
}
