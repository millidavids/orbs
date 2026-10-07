//! Whose hand did the work, and the ledger that says so (DESIGN.md §19,
//! *number go up*).

use orbs_render::{FieldName, RecordKind, Value};
use orbs_sim::Sim;

/// Submit each line and give the world a tick to do it.
fn run(sim: &mut Sim, lines: &[&str]) {
    for line in lines {
        sim.submit(line);
        sim.step();
    }
}

/// Grind one sage by hand, to completion.
fn grind(sim: &mut Sim) {
    run(sim, &["attend laboratory", "grind sage"]);
    sim.step_n(12);
}

/// The headings `status` drew last.
fn sections(sim: &Sim) -> Vec<String> {
    sim.scrollback()
        .records()
        .iter()
        .filter(|record| record.kind() == RecordKind::Section)
        .filter_map(|record| match record.field(FieldName::Kind) {
            Some(Value::Text(text)) => Some(text.to_owned()),
            _ => None,
        })
        .collect()
}

#[test]
fn the_players_own_work_is_never_a_spells() {
    let mut sim = Sim::new(1);
    grind(&mut sim);
    assert_eq!(
        sim.tally("at:mortar_and_pestle"),
        1,
        "the grind did not land"
    );
    assert_eq!(sim.tally("spell:at:mortar_and_pestle"), 0);
    assert_eq!(sim.tally("spell"), 0);
}

#[cfg(debug_assertions)]
#[test]
fn a_spells_work_is_counted_twice_and_reaches_a_deed_once() {
    let mut sim = Sim::new(1);
    run(
        &mut sim,
        &[
            "attend archive",
            "debug_spawn fragment 8",
            "debug_spell assembling",
            "invoke assembling",
        ],
    );
    sim.step_n(120);

    let scrolls = sim.tally("scroll");
    assert!(scrolls >= 1, "the spell assembled nothing");
    assert_eq!(
        sim.tally("spell:scroll"),
        scrolls,
        "a spell's scroll went uncounted"
    );
    assert!(sim.tally("spell") >= scrolls);
    // The deed reads `scroll`, and the twin is another key: one count each.
    assert_eq!(sim.tally("at:lectern"), scrolls);
    run(&mut sim, &["status"]);
    assert!(
        sections(&sim).contains(&"ledger".to_owned()),
        "a scroll copied drew no ledger",
    );

    // ...and the player's own work straight after, with the spell still bound
    // to its loop, is the player's.
    grind(&mut sim);
    assert_eq!(sim.tally("at:mortar_and_pestle"), 1);
    assert_eq!(
        sim.tally("spell:at:mortar_and_pestle"),
        0,
        "the spell's hand leaked onto the player's grind",
    );
}

#[test]
fn a_tower_that_has_done_nothing_counted_keeps_no_ledger() {
    // A section that is usually a bare rule teaches the eye to skip it. A grind
    // makes no potion, so no row counts it.
    let mut sim = Sim::new(1);
    grind(&mut sim);
    run(&mut sim, &["status"]);
    assert!(!sections(&sim).contains(&"ledger".to_owned()));
}
