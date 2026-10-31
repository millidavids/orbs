//! What the tower's spell sees: every open room, and anything in one by name
//! (§19, *The tower's spell*). A room spell still sees only its own room.

#![cfg(debug_assertions)]

mod common;

use common::{run, run_all, said};
use orbs_sim::Sim;

/// `question` asked by a spell written where the player now stands, invoked by
/// hand: `Some` for the half it took, `None` for neither, an unanswerable
/// question. Two `wait`s, which speak at once and change nothing.
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
    run(sim, "invoke asking");
    sim.step_n(3);
    let heard: Vec<String> = said(sim).into_iter().skip(before).collect();
    run(sim, "stop asking");
    match (
        heard.iter().any(|line| line.contains("yesmark")),
        heard.iter().any(|line| line.contains("nomark")),
    ) {
        (true, false) => Some(true),
        (false, true) => Some(false),
        (false, false) => None,
        (true, true) => panic!("both halves ran for {question:?}"),
    }
}

/// `question`, asked by the tower's spell: the player walks to `/tower` to
/// write it, and stays there.
fn from_the_tower(sim: &mut Sim, question: &str) -> Option<bool> {
    run(sim, "attend tower");
    asks(sim, question)
}

/// A tower at concentration 2, the player at `/tower`.
fn tower() -> Sim {
    let mut sim = Sim::new(11);
    run_all(&mut sim, &["debug_ley 96", "attend tower"]);
    sim
}

#[test]
fn a_room_is_working_only_while_a_run_is_in_hand() {
    let mut sim = tower();
    let idle = "the laboratory is idle";
    assert_eq!(from_the_tower(&mut sim, idle), Some(true));

    // A lit fire alone is no work: a hearth alight is not a job.
    run_all(&mut sim, &["attend laboratory", "kindle charcoal"]);
    assert_eq!(from_the_tower(&mut sim, idle), Some(true));

    run_all(&mut sim, &["attend laboratory", "grind sage"]);
    assert_eq!(from_the_tower(&mut sim, idle), Some(false));

    // Ready, waiting to be emptied, is a stall, not work.
    sim.step_n(12);
    assert_eq!(from_the_tower(&mut sim, idle), Some(true));
}

#[test]
fn a_puzzle_room_is_working_while_its_puzzle_is_open() {
    let mut sim = tower();
    assert_eq!(from_the_tower(&mut sim, "the lens is working"), Some(false));
    run_all(&mut sim, &["attend lens", "probe"]);
    assert_eq!(from_the_tower(&mut sim, "the lens is working"), Some(true));
}

#[test]
fn a_tower_spell_reaches_any_instrument_by_name() {
    let mut sim = tower();
    assert_eq!(from_the_tower(&mut sim, "the alembic is idle"), Some(true));
    assert_eq!(
        from_the_tower(&mut sim, "the dispensary has sage"),
        Some(true)
    );
    run_all(&mut sim, &["attend laboratory", "grind sage"]);
    assert_eq!(
        from_the_tower(&mut sim, "the mortar_and_pestle is working"),
        Some(true)
    );
}

#[test]
fn a_satchel_is_nobodys_from_the_tower() {
    // Every room has one under the one name, so the tower cannot mean any.
    let mut sim = tower();
    run(&mut sim, "debug_take satchel_1");
    assert_eq!(from_the_tower(&mut sim, "the satchel is empty"), None);
}

#[test]
fn a_room_spell_still_sees_only_its_room() {
    let mut sim = tower();
    run(&mut sim, "attend laboratory");
    assert_eq!(asks(&mut sim, "the alembic is idle"), Some(true));
    assert_eq!(asks(&mut sim, "the prism is idle"), None);
    assert_eq!(asks(&mut sim, "the lens is idle"), None);
}

#[test]
fn a_tower_spell_keeps_an_idle_room_busy() {
    let mut sim = tower();
    let lines: Vec<String> = [
        "repeat",
        "    if the lens is idle and breaking is idle",
        "        invoke breaking",
        "    end",
        "    bide 5",
        "end",
    ]
    .iter()
    .map(|line| (*line).to_owned())
    .collect();
    sim.write_spell("looking", &lines);
    sim.step();
    run_all(&mut sim, &["bind looking", "attend grimoire"]);
    sim.step_n(300);
    let row = sim.ledger_of("lens").expect("the lens keeps a row");
    assert!(
        row.by_spell > 0,
        "no ward broken by a spell: {row:?} {:?}",
        said(&sim)
    );
}

#[test]
fn a_tower_spell_cannot_see_into_a_sealed_room() {
    // The same question in an open tower, so the answer below is the seal and
    // not the name.
    let mut open = tower();
    assert_eq!(from_the_tower(&mut open, "the circle is idle"), Some(true));
    assert_eq!(
        from_the_tower(&mut open, "the menagerie is idle"),
        Some(true)
    );

    let mut sim = Sim::sealed(11);
    run_all(&mut sim, &["debug_ley 96", "attend tower"]);
    // The menagerie opens late on the mastery lines, not the ley. The room as
    // well as what is in it: the tower's children are the rooms, sealed too.
    assert_eq!(from_the_tower(&mut sim, "the circle is idle"), None);
    assert_eq!(from_the_tower(&mut sim, "the menagerie is idle"), None);
    assert_eq!(
        from_the_tower(&mut sim, "the alembic is idle"),
        Some(true),
        "the open laboratory went out of sight too"
    );
}

#[test]
fn a_spell_outranks_a_thing_of_its_name() {
    // `compile` reads `charcoal` as the spell; the run must too, or the
    // charcoal on a shelf answers idle for ever and the guard never waits.
    let mut sim = tower();
    assert_eq!(
        from_the_tower(&mut sim, "charcoal is working"),
        Some(false),
        "nothing called charcoal answers from the tower, so this proves nothing"
    );

    run(&mut sim, "attend laboratory");
    let lines = ["repeat", "bide 5", "end"].map(str::to_owned);
    sim.write_spell("charcoal", &lines);
    sim.step();
    run(&mut sim, "bind charcoal");
    assert_eq!(
        from_the_tower(&mut sim, "charcoal is working"),
        Some(true),
        "{:?}",
        said(&sim)
    );
}
