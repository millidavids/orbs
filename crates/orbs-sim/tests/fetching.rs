//! An instrument's own verb fetches what it needs — §10.1's collapse, pinned.
//!
//! §19 records that giving every instrument its own verb *"collapses the two
//! commands a player types most"*: `grind sage` **is** `move sage to
//! mortar_and_pestle` followed by `wield mortar_and_pestle`. What that entry does
//! not say, and what nothing asserted, is **where the fetch may reach from** — and
//! that is the whole difference between a loop with `move` in it and one without.
//!
//! `pipeline::reachable` is the rule: every instrument in the room that is not
//! busy, in raise order, and then the store. So a stage can take its input
//! straight out of the instrument that made it, and the laboratory's loop never
//! needs the word `move` at all.
//!
//! # Why this is a file and not a line in the balance harness
//!
//! `orbs-balance`'s `BY_HAND` already brews a clarity with no `move` — but it
//! `empty`s the mortar before digesting, so the ground-sage is fetched from the
//! **shelf**. It would go on passing if the tool-to-tool fetch were removed
//! tomorrow, which makes it evidence for the weaker half of the claim only.
//!
//! # `move` is deliberately not being removed
//!
//! It is the only way finished work leaves the room that made it (`move clarity
//! to arsenal`), and the only way to reach an instrument's `charged` state, which
//! three animation See-it lines need because `grind` never rests there. What is
//! wrong is teaching it as part of the brewing loop, and that is a prose change.
//! DESIGN.md §19 records the decision.

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
    // **The half `BY_HAND` cannot see.** No `empty` and no `move`: the ground-sage
    // is still sitting in the mortar with the husks it was left beside, and
    // `digest` has to reach in and take the one it needs.
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
    // §11.5's anchor, reached without the word once. `empty` is still here and is
    // **not** the same question: it clears the *byproduct* the last stage left, so
    // the mortar can take a second load. A brew without it stalls at `grind
    // rock-salt` with "the mortar_and_pestle can do nothing with husks, rock-salt",
    // which is a real gap and a separate one.
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
    // The reason the verb stays. Nothing else carries between domains, and this is
    // what a proposal to retire `move` from the laboratory has to answer.
    // **§10.1's chain, from endless stock, with no debug door in it.** It used to
    // open with `debug_spawn clarified-draught`, which is `cfg(debug_assertions)`
    // — so under `cargo test --release` the line was simply unresolvable, the
    // draught never appeared, and this failed on a world that was never built.
    // The failure said *"the potion never reached the arsenal"* and named
    // nothing about a door, which is what makes the class hard to see.
    //
    // `move` carrying finished work between rooms is shipped behaviour, so the
    // fix is to brew properly rather than to gate the test off the build that
    // ships it.
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
