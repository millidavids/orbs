//! Standing supplies on the wall (§19, *Supply*), driven through the real game.
//!
//! No produced item does anything alone. A potion, scroll or demon opens as a
//! supply when the arsenal holds enough for its slowest rate; the same word
//! raises it a rate; `stop` closes it. Each round it draws on its own, slower
//! as it drains, until it runs dry.
//!
//! `debug_spawn` is the door in, so these are debug-only.
#![cfg(debug_assertions)]

mod common;

use common::{ever_said, reloaded, run, said, the_siege};
use orbs_sim::tower::supply::Supplies;
use orbs_sim::{Rngs, Sim, tower};

fn held(sim: &Sim, named: &str) -> u32 {
    tower::supply::held_in_arsenal(sim.world(), named)
}

/// A siege open in the bailey, seed `seed`.
fn besieged(seed: u64) -> Sim {
    let mut sim = Sim::new(seed);
    run(&mut sim, "attend bailey");
    run(&mut sim, "defend");
    sim
}

/// The owner's rule, as a test: one short of the slowest threshold, every
/// authored row is refused and the siege does not move.
#[test]
fn one_item_never_moves_the_siege() {
    let spendables = orbs_sim::content::Spendables::builtin();
    for name in spendables.names() {
        let entry = spendables.get(name).expect("just listed");
        let verb = spendables.verb_for(name).expect("pinned to a verb");
        let short = entry.rates.slowest().opens - 1;
        let mut sim = besieged(11);
        run(&mut sim, "hold");
        run(&mut sim, &format!("debug_spawn {name} {short}"));
        let before = the_siege(&mut sim);
        run(&mut sim, &format!("{} {name}", verb.canonical()));
        assert_eq!(
            the_siege(&mut sim),
            before,
            "{short} {name} moved the siege"
        );
        assert_eq!(held(&sim, name), short, "{short} {name} were taken");
        assert!(
            ever_said(&sim, "it flows from"),
            "{short} {name} was not refused as short: {:?}",
            said(&sim)
        );
    }
}

#[test]
fn opening_applies_at_once_and_each_round_draws_again() {
    let mut sim = besieged(11);
    let full = the_siege(&mut sim).full();
    run(&mut sim, "debug_spawn vigour 12");
    run(&mut sim, "quaff vigour");
    assert!(
        ever_said(&sim, "the vigour flows, 2 a round"),
        "{:?}",
        said(&sim)
    );
    assert_eq!(
        held(&sim, "vigour"),
        10,
        "opening did not draw for the round"
    );
    assert_eq!(the_siege(&mut sim).full(), full + 3);

    run(&mut sim, "hold");
    assert_eq!(held(&sim, "vigour"), 8, "the round did not draw again");
    assert!(sim.world().resource::<Supplies>().get("vigour").is_some());
}

#[test]
fn typing_the_verb_again_raises_one_rate_and_the_fastest_refuses() {
    let mut sim = besieged(11);
    run(&mut sim, "debug_spawn vigour 20");
    run(&mut sim, "quaff vigour");
    run(&mut sim, "quaff vigour");
    assert!(
        ever_said(&sim, "flows faster, 3 a round"),
        "{:?}",
        said(&sim)
    );
    run(&mut sim, "quaff vigour");
    assert!(ever_said(&sim, "as fast as it can"), "{:?}", said(&sim));
}

#[test]
fn a_raise_counts_what_this_round_already_drew_and_applies_the_difference() {
    // Twelve held, two drawn on opening: the raise to twelve still opens, and
    // the room rises by the difference, never by both rates at once.
    let mut sim = besieged(11);
    let full = the_siege(&mut sim).full();
    run(&mut sim, "debug_spawn vigour 12");
    run(&mut sim, "quaff vigour");
    run(&mut sim, "quaff vigour");
    assert!(ever_said(&sim, "flows faster"), "{:?}", said(&sim));
    assert_eq!(the_siege(&mut sim).full(), full + 4, "the raise doubled up");
    assert_eq!(
        held(&sim, "vigour"),
        9,
        "the raise took more than the difference"
    );
}

#[test]
fn draining_steps_down_then_closes() {
    let mut sim = besieged(11);
    run(&mut sim, "debug_spawn vigour 13");
    run(&mut sim, "quaff vigour");
    run(&mut sim, "quaff vigour");
    assert_eq!(held(&sim, "vigour"), 10);

    run(&mut sim, "hold");
    assert!(ever_said(&sim, "slows to 2 a round"), "{:?}", said(&sim));
    assert_eq!(held(&sim, "vigour"), 8);
    for _ in 0..3 {
        if the_siege(&mut sim).running() {
            run(&mut sim, "hold");
        }
    }
    assert!(ever_said(&sim, "runs dry"), "{:?}", said(&sim));
    assert!(sim.world().resource::<Supplies>().get("vigour").is_none());
    assert!(
        held(&sim, "vigour") < 6,
        "a drained supply kept a full shelf"
    );
}

#[test]
fn a_raise_after_a_step_down_counts_from_the_rate_running() {
    // Raised, drained a step, restocked: the word raises from where it runs.
    // Counted from the rate chosen, it said it flowed as fast as it could.
    let mut sim = besieged(11);
    run(&mut sim, "debug_spawn vigour 13");
    run(&mut sim, "quaff vigour");
    run(&mut sim, "quaff vigour");
    run(&mut sim, "hold");
    assert!(ever_said(&sim, "slows to 2 a round"), "{:?}", said(&sim));
    assert_eq!(held(&sim, "vigour"), 8);

    run(&mut sim, "debug_spawn vigour 10");
    run(&mut sim, "quaff vigour");
    assert!(!ever_said(&sim, "as fast as it can"), "{:?}", said(&sim));
    assert_eq!(
        held(&sim, "vigour"),
        17,
        "the raise did not pay the difference"
    );
    let open = sim.world().resource::<Supplies>().get("vigour").copied();
    assert_eq!(open.map(|open| (open.chosen, open.running)), Some((1, 1)));
}

#[test]
fn a_short_raise_says_what_the_arsenal_holds() {
    // Seven, two drawn on opening: five are in the arsenal, and five more
    // reach the faster rate. It said seven held and twelve wanted.
    let mut sim = besieged(11);
    run(&mut sim, "debug_spawn vigour 7");
    run(&mut sim, "quaff vigour");
    run(&mut sim, "quaff vigour");
    assert!(
        ever_said(&sim, "you hold 5 vigour. it flows from 10"),
        "{:?}",
        said(&sim)
    );
    run(&mut sim, "debug_spawn vigour 5");
    run(&mut sim, "quaff vigour");
    assert!(ever_said(&sim, "flows faster"), "{:?}", said(&sim));
}

#[test]
fn fortify_never_accumulates() {
    let mut sim = besieged(11);
    let full = the_siege(&mut sim).full();
    run(&mut sim, "debug_spawn vigour 20");
    run(&mut sim, "quaff vigour");
    for _ in 0..3 {
        if the_siege(&mut sim).running() {
            run(&mut sim, "hold");
        }
    }
    let siege = the_siege(&mut sim);
    if siege.running() {
        assert_eq!(siege.fortified, 3, "the room grew round on round");
        assert!(siege.full() <= full + 3);
    }
}

#[test]
fn closing_fortify_takes_the_room_away() {
    let mut sim = besieged(11);
    let full = the_siege(&mut sim).full();
    run(&mut sim, "debug_spawn vigour 6");
    run(&mut sim, "quaff vigour");
    assert_eq!(the_siege(&mut sim).full(), full + 3);
    run(&mut sim, "stop vigour");
    assert!(ever_said(&sim, "you stop the vigour"), "{:?}", said(&sim));
    run(&mut sim, "hold");
    let siege = the_siege(&mut sim);
    if siege.running() {
        assert_eq!(siege.fortified, 0, "a stopped supply kept its room");
        assert!(siege.garrison.vigour <= siege.full(), "the line overflowed");
    }
}

#[test]
fn stop_closes_a_supply_and_keeps_the_rest() {
    let mut sim = besieged(11);
    run(&mut sim, "debug_spawn vigour 12");
    run(&mut sim, "quaff vigour");
    run(&mut sim, "stop vigour");
    assert_eq!(held(&sim, "vigour"), 10);
    run(&mut sim, "hold");
    assert_eq!(held(&sim, "vigour"), 10, "a stopped supply drew again");
    run(&mut sim, "stop vigour");
    assert!(ever_said(&sim, "is not flowing"), "{:?}", said(&sim));
}

#[test]
fn an_open_supply_does_not_thin_while_the_siege_waits() {
    // Fresh when the enemy arrived, so an hour over one round thins nothing.
    let mut sim = Sim::new(11);
    run(&mut sim, "attend bailey");
    run(&mut sim, "debug_spawn vigour 20");
    run(&mut sim, "defend");
    run(&mut sim, "meditate 3600");
    run(&mut sim, "quaff vigour");
    run(&mut sim, "quaff vigour");
    assert!(ever_said(&sim, "flows faster"), "{:?}", said(&sim));
}

#[test]
fn a_supply_closes_when_the_siege_lifts() {
    let mut sim = besieged(11);
    run(&mut sim, "debug_spawn vigour 20");
    run(&mut sim, "quaff vigour");
    run(&mut sim, "debug_siege");
    for _ in 0..20 {
        if !the_siege(&mut sim).running() {
            break;
        }
        run(&mut sim, "hold");
    }
    assert!(!the_siege(&mut sim).running());
    assert!(
        sim.world().resource::<Supplies>().get("vigour").is_none(),
        "a supply outlived its siege"
    );
}

#[test]
fn draws_take_no_randomness() {
    // The same ticks either way: the calm layer's streams draw on the clock,
    // so only what the supply itself draws may differ, and it draws nothing.
    let positions = |with_supply: bool| {
        let mut sim = besieged(11);
        if with_supply {
            run(&mut sim, "debug_spawn vigour 20");
            run(&mut sim, "quaff vigour");
        } else {
            run(&mut sim, "survey garrison");
            run(&mut sim, "survey garrison");
        }
        run(&mut sim, "hold");
        sim.world().resource::<Rngs>().positions()
    };
    assert_eq!(positions(true), positions(false));
}

#[test]
fn the_round_draws_apply_in_one_order_whatever_order_they_were_opened() {
    // A room and a bonus, which read nothing of each other when they open: an
    // opening applies at once, so only the round's draws are order-free (a
    // room opened before a heal lets the heal fill it, and that is the player's
    // order to choose).
    let fought = |first: &str, second: &str| {
        let mut sim = besieged(11);
        run(&mut sim, "debug_spawn vigour 12");
        run(&mut sim, "debug_spawn warding 12");
        run(&mut sim, &format!("quaff {first}"));
        run(&mut sim, &format!("quaff {second}"));
        run(&mut sim, "hold");
        let siege = the_siege(&mut sim);
        (siege.garrison, siege.fortified, siege.staged)
    };
    assert_eq!(fought("vigour", "warding"), fought("warding", "vigour"));
}

#[test]
fn a_supply_open_in_a_saved_siege_comes_back_open() {
    let mut sim = besieged(11);
    run(&mut sim, "debug_spawn vigour 12");
    run(&mut sim, "quaff vigour");
    let full = the_siege(&mut sim).full();
    let mut loaded = reloaded(&sim.snapshot());
    assert_eq!(
        loaded.world().resource::<Supplies>().get("vigour"),
        sim.world().resource::<Supplies>().get("vigour"),
    );
    assert_eq!(the_siege(&mut loaded).full(), full);
    run(&mut loaded, "hold");
    assert_eq!(held(&loaded, "vigour"), 8, "the loaded supply did not draw");
}

#[test]
fn an_item_says_what_its_supply_is_doing() {
    let mut sim = besieged(11);
    run(&mut sim, "debug_spawn vigour 3");
    run(&mut sim, "debug_spawn mending 12");
    run(&mut sim, "quaff mending");
    sim.step();
    assert!(sim.holds_reading("arsenal", "vigour", tower::supply::DRAINED));
    assert!(sim.holds_reading("arsenal", "mending", tower::supply::DRAW));
    assert!(!sim.holds_reading("arsenal", "vigour", tower::supply::DRAW));
}

#[test]
fn a_supply_whose_stock_was_carried_out_can_still_be_stopped() {
    // One `move` carries the whole batch, and with no node left `stop warding`
    // fuzzed into `warding_off.spell` while the supply stayed open.
    let mut sim = besieged(11);
    run(&mut sim, "debug_spawn warding 12");
    run(&mut sim, "quaff warding");
    run(&mut sim, "attend laboratory");
    run(&mut sim, "move warding to dispensary");
    assert_eq!(held(&sim, "warding"), 0, "{:?}", said(&sim));
    // Back at the wall, where the dispensary's warding is out of reach.
    run(&mut sim, "attend bailey");
    run(&mut sim, "stop warding");
    assert!(ever_said(&sim, "you stop the warding"), "{:?}", said(&sim));
    assert!(sim.world().resource::<Supplies>().get("warding").is_none());
}

#[test]
fn a_spell_asking_twice_in_one_tick_reads_its_first_answer() {
    // With steps to spare, both guards run in one tick. The readings were kept
    // only at the tick's end, so the second saw no draw and raised the supply.
    let mut sim = besieged(11);
    for id in ["steps_1", "steps_2", "steps_3"] {
        run(&mut sim, &format!("debug_take {id}"));
    }
    run(&mut sim, "debug_spawn vigour 20");
    let guarded = ["if the vigour has no draw", "    quaff vigour", "end"];
    let body: Vec<String> = guarded
        .iter()
        .chain(&guarded)
        .map(|line| (*line).to_owned())
        .collect();
    sim.write_spell("tending", &body);
    sim.step();
    run(&mut sim, "invoke tending");
    sim.step_n(5);
    let open = sim.world().resource::<Supplies>().get("vigour").copied();
    assert_eq!(
        open.map(|open| open.chosen),
        Some(0),
        "the second guard raised it: {:?}",
        said(&sim)
    );
}

/// Run `f` on the siege in the bailey.
fn with_siege(sim: &mut Sim, f: impl FnOnce(&mut tower::Siege)) {
    let world = sim.world_mut();
    let mut sieges = world.query::<&mut tower::Siege>();
    let mut fight = sieges.iter_mut(world).next().expect("a siege");
    f(&mut fight);
}

#[test]
fn stopping_and_reopening_in_one_round_draws_no_second_ration() {
    // `stop` forgot what the round had drawn, so a stop and a reopen took and
    // applied a whole ration again, as often as typed.
    let mut sim = besieged(5);
    with_siege(&mut sim, |fight| {
        fight.garrison.vigour = fight.garrison.vigour.saturating_sub(10);
    });
    run(&mut sim, "debug_spawn mending 20");
    let before = the_siege(&mut sim).garrison.vigour;
    for _ in 0..3 {
        run(&mut sim, "quaff mending");
        run(&mut sim, "stop mending");
    }
    assert_eq!(
        the_siege(&mut sim).garrison.vigour - before,
        3,
        "{:?}",
        said(&sim)
    );
    assert_eq!(held(&sim, "mending"), 18, "a second ration was taken");

    // ...and a save between the stop and the reopen remembers it too.
    let mut loaded = reloaded(&sim.snapshot());
    run(&mut loaded, "quaff mending");
    assert_eq!(
        held(&loaded, "mending"),
        18,
        "the reload drew a second ration"
    );
}

#[test]
fn a_supply_that_runs_dry_cannot_reopen_on_last_rounds_ration() {
    // Shut dry at a round's draw, it was remembered as stopped with the ration
    // the round before had taken, so the word reopened it below its threshold
    // for nothing, every round.
    let mut sim = besieged(11);
    run(&mut sim, "debug_spawn vigour 7");
    run(&mut sim, "quaff vigour");
    assert_eq!(held(&sim, "vigour"), 5);
    run(&mut sim, "hold");
    assert!(
        sim.world().resource::<Supplies>().get("vigour").is_none(),
        "five did not run dry: {:?}",
        said(&sim)
    );
    run(&mut sim, "quaff vigour");
    assert!(
        ever_said(&sim, "you hold 5 vigour. it flows from 6"),
        "{:?}",
        said(&sim)
    );
    assert!(
        sim.world().resource::<Supplies>().get("vigour").is_none(),
        "it reopened below its threshold"
    );
}

#[test]
fn reopening_after_a_raise_keeps_the_rate_the_round_paid_for() {
    // A reopening restarted at the slowest rate and forgot the raise's
    // ration, so raising again took a third item for a rate already paid.
    let mut sim = besieged(11);
    run(&mut sim, "debug_spawn vigour 20");
    let full = the_siege(&mut sim).full();
    run(&mut sim, "quaff vigour");
    run(&mut sim, "quaff vigour");
    assert_eq!(held(&sim, "vigour"), 17);
    run(&mut sim, "stop vigour");
    run(&mut sim, "quaff vigour");
    assert_eq!(
        sim.world()
            .resource::<Supplies>()
            .get("vigour")
            .map(|open| open.running),
        Some(1),
        "{:?}",
        said(&sim)
    );
    assert_eq!(the_siege(&mut sim).full(), full + 4);
    run(&mut sim, "quaff vigour");
    assert!(ever_said(&sim, "as fast as it can"), "{:?}", said(&sim));
    assert_eq!(held(&sim, "vigour"), 17, "the rate was paid for twice");
}

#[test]
fn stillness_hands_back_a_supply_stopped_this_round() {
    // A stopped supply's modifier stood for the round, and stillness threw it
    // away with the rest, but only the open ones were handed back.
    let mut sim = besieged(11);
    with_siege(&mut sim, |fight| fight.intent = tower::Intent::Advance);
    run(&mut sim, "debug_spawn warding 6");
    run(&mut sim, "debug_spawn stillness 3");
    run(&mut sim, "quaff warding");
    assert_eq!(held(&sim, "warding"), 4, "{:?}", said(&sim));
    run(&mut sim, "stop warding");
    run(&mut sim, "quaff stillness");
    assert!(ever_said(&sim, "a stillness falls"), "{:?}", said(&sim));
    assert_eq!(
        held(&sim, "warding"),
        6,
        "the stopped warding's ration was lost"
    );
}

#[test]
fn an_older_sieges_vigour_keeps_the_room_it_bought() {
    // Format 16 sold vigour's room for the rest of the siege; a fortify supply
    // holds it a round at a time, and the first round took it away.
    let mut sim = besieged(11);
    with_siege(&mut sim, |fight| fight.fortified = 9);
    let full = the_siege(&mut sim).full();
    let mut save = sim.snapshot();
    save.world.format = 16;
    let mut loaded = reloaded(&save);
    run(&mut loaded, "hold");
    assert!(the_siege(&mut loaded).running(), "the round ended it");
    assert_eq!(the_siege(&mut loaded).full(), full, "the room was lost");
}

#[test]
fn a_scroll_unrolling_in_a_room_must_be_stopped_before_the_wall() {
    // Paused by the siege but still open, it refused the wall without saying
    // how to free it.
    let mut sim = Sim::new(11);
    run(&mut sim, "attend laboratory");
    run(&mut sim, "debug_spawn quickening-scroll 9");
    run(&mut sim, "wield quickening-scroll");
    run(&mut sim, "attend bailey");
    run(&mut sim, "defend");
    with_siege(&mut sim, |fight| fight.intent = tower::Intent::Advance);
    run(&mut sim, "wield quickening-scroll");
    assert!(ever_said(&sim, "stop it first"), "{:?}", said(&sim));
    run(&mut sim, "stop quickening-scroll");
    run(&mut sim, "wield quickening-scroll");
    assert!(
        ever_said(&sim, "the quickening-scroll flows"),
        "{:?}",
        said(&sim)
    );
}

#[test]
fn a_heal_never_lowers_the_line() {
    // An older save's line can stand above today's most, and `mend` capped
    // the count at the room: healing killed bodies.
    let mut sim = besieged(5);
    with_siege(&mut sim, |fight| {
        fight.mustered = 18;
        fight.garrison.count = 18;
        fight.garrison.vigour = 18 * tower::siege::VIGOUR - 6;
        fight.room = 10;
        fight.heal(3);
        assert_eq!(fight.garrison.count, 18, "a heal killed bodies");
    });
}
