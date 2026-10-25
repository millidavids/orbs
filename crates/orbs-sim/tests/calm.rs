//! A scroll's calm supply (§19, *Supply*), driven through the game: no scroll
//! does anything alone, an open supply draws on the clock whether or not
//! anything uses it, and a siege pauses it.
//!
//! `debug_spawn` stocks the arsenal, so these are debug-only.
#![cfg(debug_assertions)]

mod common;

use common::{ever_said, run, said};
use orbs_sim::Sim;
use orbs_sim::tower::{self, supply::Scope, supply::Supplies};

const QUICK: &str = "quickening-scroll";

fn held(sim: &Sim) -> u32 {
    tower::supply::held_in_arsenal(sim.world(), QUICK)
}

fn quickened(sim: &Sim) -> Option<u64> {
    let world = sim.world();
    let shelf = tower::home(world, "sage").expect("sage has a shelf");
    tower::quickened_by(world, shelf)
}

/// Standing in the laboratory with `count` quickening scrolls, the supply opened.
fn opened(seed: u64, count: u32) -> Sim {
    let mut sim = Sim::new(seed);
    run(&mut sim, "attend laboratory");
    run(&mut sim, &format!("debug_spawn {QUICK} {count}"));
    run(&mut sim, &format!("wield {QUICK}"));
    sim
}

#[test]
fn one_scroll_short_of_the_slowest_rate_quickens_nothing() {
    let sim = opened(1, 3);
    assert_eq!(
        quickened(&sim),
        None,
        "three scrolls quickened the laboratory"
    );
    assert_eq!(held(&sim), 3, "the refusal took a scroll");
    assert!(ever_said(&sim, "flows from 4"), "{:?}", said(&sim));
}

#[test]
fn opening_draws_at_once_and_then_on_the_clock() {
    let mut sim = opened(1, 8);
    assert_eq!(held(&sim), 7, "the word did not draw");
    assert_eq!(quickened(&sim), Some(2));
    run(&mut sim, "meditate 299");
    assert_eq!(held(&sim), 6, "no draw 300 ticks after the word");
}

#[test]
fn the_word_again_raises_it_a_rate() {
    // As on the wall: the raise pays only the difference from what this draw
    // took, and the clock keeps counting from the first word.
    let mut sim = opened(1, 12);
    run(&mut sim, &format!("wield {QUICK}"));
    assert_eq!(
        quickened(&sim),
        Some(3),
        "the raise did not quicken by a third"
    );
    assert_eq!(held(&sim), 10, "the raise drew more than the difference");
    assert!(
        ever_said(&sim, "unrolls faster, 2 every 300"),
        "{:?}",
        said(&sim)
    );
    run(&mut sim, "meditate 298");
    assert_eq!(
        held(&sim),
        8,
        "the next draw was not on the first word's clock"
    );
}

#[test]
fn draining_steps_down_then_rolls_it_up() {
    // 11, then the raise: 9 at the faster rate. It draws 2 to 7, then 7 is
    // under 8 and it steps down. Four draws more at the slowest, 6 to 3, and 3
    // cannot run.
    let mut sim = opened(1, 11);
    run(&mut sim, &format!("wield {QUICK}"));
    assert_eq!(held(&sim), 9);
    run(&mut sim, "meditate 598");
    assert!(ever_said(&sim, "slows to 1 every 300"), "{:?}", said(&sim));
    assert_eq!(quickened(&sim), Some(2));
    run(&mut sim, "meditate 1500");
    assert!(ever_said(&sim, "runs dry"), "{:?}", said(&sim));
    assert!(sim.world().resource::<Supplies>().get(QUICK).is_none());
    assert_eq!(held(&sim), 3, "the last three are held, unusable");
}

#[test]
fn a_scroll_left_in_the_cabinet_is_named_where_it_waits() {
    // A supply draws from the arsenal alone, and `empty lectern` puts its
    // scrolls in the archive's cabinet. The refusal said there were none, with
    // the scrolls listed in the same room.
    let mut sim = Sim::new(1);
    for line in [
        "attend archive",
        "debug_spawn fragment 4 lectern",
        "wield lectern",
        "meditate 200",
        "empty lectern",
    ] {
        run(&mut sim, line);
    }
    for scroll in ["gleaning-scroll", QUICK, "verdant-scroll"] {
        run(&mut sim, &format!("wield {scroll}"));
    }
    assert!(
        ever_said(&sim, "is in the cabinet. move it to the arsenal"),
        "{:?}",
        said(&sim)
    );
}

#[test]
fn no_run_starts_in_the_gap_between_windows() {
    // Each window lasts a draw's interval and one tick more, so it overlaps the
    // next. Checked on every tick, across several draws.
    let mut sim = opened(1, 20);
    for tick in 0..1_000 {
        assert!(
            quickened(&sim).is_some(),
            "a gap at tick {tick} after the word"
        );
        sim.step();
    }
}

#[test]
fn a_calm_supply_drains_the_same_however_the_ticks_are_taken() {
    let mut watched = opened(2, 10);
    let mut skipped = opened(2, 10);
    for _ in 0..1_000 {
        watched.step();
    }
    run(&mut skipped, "meditate 999");
    assert_eq!(held(&watched), held(&skipped));
    assert_eq!(
        watched.world().resource::<Supplies>(),
        skipped.world().resource::<Supplies>()
    );
    assert_eq!(quickened(&watched), quickened(&skipped));
}

#[test]
fn a_calm_supply_pauses_while_besieged_and_cannot_open_in_one() {
    let mut sim = opened(3, 10);
    run(&mut sim, "attend bailey");
    run(&mut sim, "defend");
    let before = held(&sim);
    run(&mut sim, "meditate 900");
    assert_eq!(held(&sim), before, "a calm supply drew during a siege");

    // The siege takes `wield` while it runs, and one supply an item.
    run(&mut sim, &format!("wield {QUICK}"));
    assert!(
        ever_said(&sim, "already flows elsewhere"),
        "{:?}",
        said(&sim)
    );

    // A scroll not open in a room opens on the wall instead.
    run(&mut sim, "debug_spawn gleaning-scroll 4");
    run(&mut sim, "wield gleaning-scroll");
    let scope = sim
        .world()
        .resource::<Supplies>()
        .get("gleaning-scroll")
        .map(|open| open.scope);
    assert_eq!(scope, Some(Scope::Siege), "a calm supply opened in a siege");
}

#[test]
fn a_calm_supply_cannot_open_in_a_siege_from_another_room() {
    // The wall takes `wield` only in the bailey, so from the laboratory a
    // siege used to reach the calm supply and quicken the room mid-fight.
    let mut sim = Sim::new(3);
    run(&mut sim, "attend bailey");
    run(&mut sim, "defend");
    run(&mut sim, &format!("debug_spawn {QUICK} 8"));
    run(&mut sim, "attend laboratory");
    run(&mut sim, &format!("wield {QUICK}"));
    assert_eq!(quickened(&sim), None, "a calm supply opened mid-siege");
    assert_eq!(held(&sim), 8, "a scroll was taken mid-siege");
    assert!(
        ever_said(&sim, "while the wall is fighting"),
        "{:?}",
        said(&sim)
    );
}

#[test]
fn a_stale_supply_says_it_is_stale_not_dry() {
    // Thirty held and none made lately: the supply stops because the stores
    // are out, and the line says to make more rather than that too few are left.
    let mut sim = opened(5, 30);
    run(&mut sim, "meditate 2100");
    assert!(
        sim.world().resource::<Supplies>().get(QUICK).is_none(),
        "a spent supply went on drawing"
    );
    assert!(ever_said(&sim, "stores are out"), "{:?}", said(&sim));
    assert!(
        !ever_said(&sim, "too few are left"),
        "a full arsenal was told it ran dry: {:?}",
        said(&sim)
    );
}

#[test]
fn stop_rolls_a_calm_supply_up() {
    let mut sim = opened(4, 8);
    run(&mut sim, &format!("stop {QUICK}"));
    assert!(
        ever_said(&sim, "you roll the quickening-scroll up"),
        "{:?}",
        said(&sim)
    );
    assert!(sim.world().resource::<Supplies>().get(QUICK).is_none());
    run(&mut sim, "meditate 600");
    assert_eq!(held(&sim), 7, "a stopped supply went on drawing");
}

#[test]
fn stopping_and_reopening_before_the_next_draw_takes_nothing_twice() {
    // The wall's rule in a room: what a draw took stands, and the word again
    // opens it as it stood until the next draw would have come.
    let mut sim = opened(4, 8);
    run(&mut sim, &format!("stop {QUICK}"));
    run(&mut sim, &format!("wield {QUICK}"));
    assert_eq!(held(&sim), 7, "reopening took a second ration");
    // On the first word's clock.
    run(&mut sim, "meditate 297");
    assert_eq!(held(&sim), 6, "the reopened supply lost its clock");

    // Stopped past where its next draw would have come, it opens afresh.
    run(&mut sim, &format!("stop {QUICK}"));
    run(&mut sim, "meditate 300");
    run(&mut sim, &format!("wield {QUICK}"));
    assert_eq!(held(&sim), 5, "a new period opened for nothing");
}

#[test]
fn a_spent_store_cannot_open() {
    let mut sim = Sim::new(5);
    run(&mut sim, "attend laboratory");
    run(&mut sim, &format!("debug_spawn {QUICK} 8"));
    run(&mut sim, "meditate 1900");
    run(&mut sim, &format!("wield {QUICK}"));
    assert_eq!(quickened(&sim), None);
    assert!(ever_said(&sim, "stores are out"), "{:?}", said(&sim));
}

#[test]
fn the_lectern_makes_two_of_a_scroll() {
    let mut sim = Sim::new(6);
    run(&mut sim, "attend archive");
    run(&mut sim, "debug_spawn fragment 4 lectern");
    run(&mut sim, "wield lectern");
    sim.step_n(25);
    assert!(ever_said(&sim, "yields 2"), "{:?}", said(&sim));
}
