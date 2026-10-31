//! What a spell may do beside a brew: everything that takes no production slot.
//!
//! A spell holds on a slot it would take (`block::begins_work`), so a verb
//! listed there wrongly waits out the brew and gives up at `PATIENCE`. Typed,
//! each of these runs during a brew; scripted, it has to as well.

#![cfg(debug_assertions)]

mod common;

use common::{ever_said, run, said};
use orbs_sim::Sim;

/// A distillation holding the tower's one slot, and the player still in the
/// laboratory.
fn beside_a_brew(seed: u64) -> Sim {
    let mut sim = Sim::new(seed);
    run(&mut sim, "attend laboratory");
    run(&mut sim, "kindle charcoal");
    run(&mut sim, "debug_spawn clarified-draught 1");
    run(&mut sim, "distil clarified-draught");
    assert!(
        sim.working().is_some(),
        "the brew that should hold the slot did not start: {:?}",
        said(&sim),
    );
    sim
}

#[test]
fn a_spells_press_does_not_wait_on_a_brew() {
    let mut sim = beside_a_brew(3);
    run(&mut sim, "attend lens");
    sim.write_spell("pressing", &["probe".to_owned(), "probe".to_owned()]);
    sim.step();
    run(&mut sim, "invoke pressing");
    sim.step_n(4);

    assert!(
        !ever_said(&sim, "pressing.spell waits"),
        "a spell's press waited on the brew: {:?}",
        said(&sim),
    );
    assert!(
        ever_said(&sim, "the seal is"),
        "the spell's second press was never answered: {:?}",
        said(&sim),
    );
}

#[test]
fn a_spells_kindle_does_not_wait_on_a_brew() {
    let mut sim = beside_a_brew(3);
    run(&mut sim, "debug_spawn charcoal 1");
    sim.write_spell("stoking", &["kindle charcoal".to_owned()]);
    sim.step();
    run(&mut sim, "invoke stoking");
    sim.step_n(3);

    assert!(
        !ever_said(&sim, "stoking.spell waits"),
        "a spell's kindle waited on the brew: {:?}",
        said(&sim),
    );
}

#[test]
fn a_spells_opening_does_not_wait_on_a_brew() {
    let mut sim = beside_a_brew(3);
    run(&mut sim, "attend archive");
    sim.write_spell("reading", &["research".to_owned()]);
    sim.step();
    run(&mut sim, "invoke reading");
    sim.step_n(3);

    assert!(
        ever_said(&sim, "shelves that do not end"),
        "a spell's research waited on the brew: {:?}",
        said(&sim),
    );
}

#[test]
fn a_spells_purge_takes_the_triage_slot_not_the_brews() {
    let mut sim = beside_a_brew(3);
    sim.write_spell("scouring", &["purge mortar_and_pestle".to_owned()]);
    sim.step();
    run(&mut sim, "invoke scouring");
    sim.step_n(3);

    assert!(
        ever_said(&sim, "scouring the mortar_and_pestle"),
        "a spell's purge waited on the brew: {:?}",
        said(&sim),
    );
}
