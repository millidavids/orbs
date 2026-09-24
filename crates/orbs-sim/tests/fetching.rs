//! An instrument's own verb fetches what it needs — §10.1's collapse, pinned.
//!
//! §19 records that an instrument's own verb collapses `move` and `wield`:
//! `grind sage` *is* both. What it does not say, and nothing asserted, is where
//! the fetch may reach from — the whole difference between a loop with `move`
//! in it and one without.
//!
//! `pipeline::reachable` is the rule: every instrument in the room that is not
//! busy, in raise order, and then the store. So a stage can take its input
//! straight out of the instrument that made it.
//!
//! Not a line in the balance harness: `BY_HAND` does brew a clarity with no
//! `move`, but it `empty`s the mortar first, so the ground-sage comes from the
//! shelf — it would go on passing with the tool-to-tool fetch removed.
//!
//! `move` stays. It is the only way finished work leaves the room that made it,
//! and the only way to reach an instrument's `charged` state, which three
//! animation See-it lines need. What is wrong is teaching it as part of the
//! brewing loop (§19).

use orbs_render::{FieldName, Value};
use orbs_sim::Sim;

fn run(sim: &mut Sim, line: &str) {
    sim.submit(line);
    sim.step();
}

/// Every `Source` the orb has reported, which is where a fetch came *from*.
fn sources(sim: &Sim) -> Vec<String> {
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

#[test]
fn a_stage_takes_its_input_straight_out_of_the_instrument_before_it() {
    // The half `BY_HAND` cannot see: no `empty` and no `move`, so the
    // ground-sage is still in the mortar beside the husks and `digest` has to
    // reach in and take the one it needs.
    let mut sim = Sim::new(1);
    for line in ["attend laboratory", "kindle charcoal", "grind sage"] {
        run(&mut sim, line);
    }
    sim.step_n(9);

    let before = sim.scrollback().records().len();
    run(&mut sim, "digest ground-sage");

    let said = sources(&sim);
    let moved = said
        .iter()
        .skip_while(|_| false)
        .filter(|line| line.contains("ground-sage"))
        .find(|line| line.contains("balneum_mariae"))
        .unwrap_or_else(|| panic!("the bath never took the ground-sage: {said:?}"));
    assert!(
        moved.contains("mortar_and_pestle"),
        "the fetch did not come from the mortar, so a tool-to-tool reach has gone: \
         {moved:?}",
    );

    // ...and the husks stayed behind. A fetch that emptied the tool would be
    // `empty` wearing another name, and would take the byproduct with it.
    run(&mut sim, "survey mortar_and_pestle");
    let listed: Vec<String> = sim
        .scrollback()
        .records()
        .iter()
        .skip(before)
        .filter_map(|record| record.field(FieldName::Name))
        .filter_map(|value| match value {
            Value::Text(text) => Some(text.to_owned()),
            _ => None,
        })
        .collect();
    assert!(
        listed.iter().any(|name| name == "husks"),
        "the fetch took the byproduct too: {listed:?}",
    );
}

#[test]
fn a_whole_brew_needs_no_move() {
    // §11.5's anchor, reached without the word once. `empty` is a separate
    // question: it clears the *byproduct* the last stage left so the mortar can
    // take a second load, and a brew without it stalls at `grind rock-salt`.
    let mut sim = Sim::new(1);
    for line in [
        "attend laboratory",
        "kindle charcoal",
        "grind sage",
        "meditate 9",
        "digest ground-sage",
        "meditate 14",
        "empty mortar_and_pestle",
        "grind rock-salt",
        "meditate 9",
        "mix sage-tincture with ground-salt",
        "meditate 12",
        "distil clarified-draught",
        "meditate 60",
    ] {
        run(&mut sim, line);
    }

    assert_eq!(
        sim.experience(),
        16,
        "a clarity no longer brews without `move`: {:?}",
        sources(&sim),
    );
}

#[test]
fn move_is_still_the_only_way_finished_work_leaves_the_room() {
    // The reason the verb stays: nothing else carries between domains.
    //
    // §10.1's chain brewed properly, with no debug door in it. It used to open
    // with `debug_spawn clarified-draught`, which is `cfg(debug_assertions)`, so
    // under `--release` the line was unresolvable and this passed judgement on a
    // world that was never built.
    let mut sim = Sim::new(1);
    for line in [
        "attend laboratory",
        "kindle charcoal",
        "grind sage",
        "meditate 9",
        "empty mortar_and_pestle",
        "digest ground-sage",
        "meditate 14",
        "grind rock-salt",
        "meditate 9",
        "empty mortar_and_pestle",
        "mix sage-tincture with ground-salt",
        "meditate 12",
        "distil clarified-draught",
    ] {
        run(&mut sim, line);
    }
    sim.step_n(60);
    run(&mut sim, "empty alembic");
    run(&mut sim, "move clarity to arsenal");

    run(&mut sim, "attend archive");
    let before = sim.scrollback().records().len();
    run(&mut sim, "survey arsenal");
    let listed: Vec<String> = sim
        .scrollback()
        .records()
        .iter()
        .skip(before)
        .filter_map(|record| record.field(FieldName::Name))
        .filter_map(|value| match value {
            Value::Text(text) => Some(text.to_owned()),
            _ => None,
        })
        .collect();
    assert!(
        listed.iter().any(|name| name == "clarity"),
        "the potion never reached the arsenal: {listed:?}",
    );
}
