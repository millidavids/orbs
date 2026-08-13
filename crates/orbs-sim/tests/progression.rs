//! What work is worth, and what it buys (DESIGN.md §11.5).
//!
//! Driven through real runs rather than by calling the grant: the number is
//! only worth anything if it is the number a player actually earns, and the two
//! hooks (`work::produce::transmute` and the archive's branch of `land::finish`)
//! are the thing most likely to be wired to the wrong place.

use orbs_render::{FieldName, Value};
use orbs_sim::Sim;

/// A laboratory, stood in.
fn laboratory() -> Sim {
    let mut sim = Sim::new(1);
    run(&mut sim, &["attend laboratory"]);
    sim
}

/// Submit each line and give the world a tick to do it.
fn run(sim: &mut Sim, lines: &[&str]) {
    for line in lines {
        sim.submit(line);
        sim.step();
    }
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

/// Grind one sage to completion, which is the cheapest complete run there is.
fn grind(sim: &mut Sim) {
    run(sim, &["grind sage"]);
    sim.step_n(12);
    run(sim, &["empty mortar_and_pestle"]);
}

#[test]
fn each_instrument_earns_what_the_curve_prices_it_at() {
    // **Binary, so each tier of tool is worth every use of the one below.** The
    // numbers are content; what this pins is that the hook is wired to the
    // instrument that did the work rather than to whatever was nearest.
    let mut sim = laboratory();
    run(&mut sim, &["kindle charcoal"]);

    grind(&mut sim);
    assert_eq!(sim.experience(), 1, "a grind");

    run(&mut sim, &["digest ground-sage"]);
    sim.step_n(16);
    run(&mut sim, &["empty balneum_mariae"]);
    assert_eq!(sim.experience(), 3, "a digest is worth two");

    grind(&mut sim);
    run(&mut sim, &["grind rock-salt"]);
    sim.step_n(12);
    run(&mut sim, &["empty mortar_and_pestle"]);
    assert_eq!(sim.experience(), 5, "two more grinds");

    run(&mut sim, &["mix sage-tincture with ground-salt"]);
    sim.step_n(14);
    run(&mut sim, &["empty flask_and_rod"]);
    assert_eq!(sim.experience(), 9, "a mix is worth four");

    run(&mut sim, &["distil clarified-draught"]);
    sim.step_n(60);
    assert_eq!(sim.experience(), 17, "a distillation is worth eight");
}

#[test]
fn one_clarity_is_exactly_the_first_threshold() {
    // **The number the whole curve is anchored to.** A recipe moving, a weight
    // moving, or the hook firing twice all show up here — and the point of
    // walking the brew rather than adding five numbers is that this is the path
    // a player takes, once, before automation exists.
    let mut sim = laboratory();
    assert_eq!(sim.concentration(), 0, "the tower started with a slot");

    run(&mut sim, &["kindle charcoal", "grind sage"]);
    sim.step_n(12);
    run(&mut sim, &["empty mortar_and_pestle", "digest ground-sage"]);
    sim.step_n(16);
    run(&mut sim, &["empty balneum_mariae", "grind rock-salt"]);
    sim.step_n(12);
    run(
        &mut sim,
        &[
            "empty mortar_and_pestle",
            "mix sage-tincture with ground-salt",
        ],
    );
    sim.step_n(14);
    run(&mut sim, &["empty flask_and_rod"]);
    assert_eq!(sim.concentration(), 0, "it arrived before the potion did");

    run(&mut sim, &["distil clarified-draught"]);
    sim.step_n(60);

    assert_eq!(
        sim.experience(),
        16,
        "one clarity is not 16: {:?}",
        messages(&sim)
    );
    assert_eq!(sim.concentration(), 1);
}

#[test]
fn the_archive_earns_too() {
    // One of the two rooms the game opens with, and §10 calls it the domain
    // played most. At nothing it would be dead progression for half the opening.
    //
    // **Solved rather than merely opened.** `divine` used to hold the slot for
    // twelve ticks and pay for it; it now opens a labyrinth, and what earns is
    // reaching the way out — so this walks one, by the same Trémaux rule a
    // player writes as a spell: prefer a passage nobody has walked, and fall
    // back to the least-walked way out.
    let mut sim = Sim::new(1);
    run(&mut sim, &["attend archive", "research"]);

    for _ in 0..4000 {
        if sim.experience() > 0 {
            break;
        }
        let Some(way) = choose(&sim) else { break };
        run(&mut sim, &[&format!("follow {way}")]);
    }

    assert!(
        sim.experience() > 0,
        "the archive earned nothing: {:?}",
        messages(&sim).last(),
    );
}

/// The next way a Trémaux solver would take, read off the four readings.
fn choose(sim: &Sim) -> Option<&'static str> {
    let ways = ["north", "east", "south", "west"];
    // **Five rungs, and the last two are what make it terminate.** The four-rung
    // version is not Tremaux and only ever solved small mazes: at a junction
    // where two ways read alike, a fixed compass order sends it back where it
    // came from and it cycles. `back` is the word that breaks the tie, and it is
    // a *second* fact about a way rather than a fifth reading — see
    // `orbs_sim::tower::maze::BACK`.
    for word in ["exit", "passage"] {
        if let Some(way) = ways.into_iter().find(|way| reads(sim, way, word)) {
            return Some(way);
        }
    }
    for word in ["walked", "twice"] {
        if let Some(way) = ways
            .into_iter()
            .find(|way| reads(sim, way, word) && !reads(sim, way, "back"))
        {
            return Some(way);
        }
    }
    ways.into_iter().find(|way| reads(sim, way, "back"))
}

/// Whether one way answers to `word` — a reading, or `back`.
fn reads(sim: &Sim, way: &str, word: &str) -> bool {
    let world = sim.world();
    let cwd = world.resource::<orbs_sim::Cwd>().0;
    let Some(node) = orbs_sim::children_of(world, cwd).into_iter().find(|node| {
        world
            .get::<orbs_sim::Name>(*node)
            .is_some_and(|name| name.0 == way)
    }) else {
        return false;
    };
    orbs_sim::children_of(world, node)
        .into_iter()
        .filter_map(|held| world.get::<orbs_sim::Name>(held))
        .any(|name| name.0 == word)
}

#[test]
fn only_work_that_succeeded_earns() {
    let mut sim = laboratory();

    // A scour: `finish` runs for it, and it makes nothing.
    run(&mut sim, &["purge mortar_and_pestle"]);
    sim.step_n(8);
    assert_eq!(sim.experience(), 0, "a scour earned something");

    // A refusal: no run starts at all.
    run(&mut sim, &["grind moonstone"]);
    sim.step_n(8);
    assert_eq!(sim.experience(), 0, "a refusal earned something");

    // A run whose contents match no recipe. `wield` charges nothing, so this is
    // the `wield_no_recipe` path — a run that ended without making anything.
    run(&mut sim, &["move sage to alembic", "wield alembic"]);
    sim.step_n(60);
    assert_eq!(
        sim.experience(),
        0,
        "a run that made nothing earned something: {:?}",
        messages(&sim),
    );
}

#[cfg(debug_assertions)]
#[test]
fn a_tester_cannot_farm_the_curve() {
    // `debug_spawn` makes reagents without work, which is what it is for — and
    // is exactly why it must not pay. A tester who can buy concentration with a
    // keystroke is a tester whose session says nothing about the curve.
    let mut sim = laboratory();
    for _ in 0..8 {
        sim.submit("debug_spawn clarity");
        sim.step();
    }
    assert_eq!(sim.experience(), 0);
    assert_eq!(sim.concentration(), 0);
}

#[test]
fn crossing_the_threshold_is_said_once_and_names_the_verb() {
    // §6 forbids a bare fact where a sentence would teach, and this is the first
    // thing in the game that *opens* rather than refuses. It names `bind`,
    // because a capability nobody can find is a capability nobody has.
    let mut sim = laboratory();
    run(&mut sim, &["kindle charcoal"]);
    for _ in 0..20 {
        grind(&mut sim);
    }

    let said: Vec<String> = messages(&sim)
        .into_iter()
        .filter(|line| line.contains("can hold a spell"))
        .collect();
    assert_eq!(said.len(), 1, "said {} times: {said:?}", said.len());
    assert!(
        said[0].contains("bind"),
        "it did not name the verb: {said:?}"
    );
    assert!(sim.experience() > 16, "the sweep never crossed it");
}

#[test]
fn what_was_earned_is_on_the_line_that_earned_it() {
    // §3 forbids unlogged output, and a second record per run would double the
    // laboratory's traffic. The run already says what it made; what it earned
    // belongs in that sentence — and **after** it, so the reward does not
    // announce itself before the work.
    let mut sim = laboratory();
    run(&mut sim, &["kindle charcoal"]);
    grind(&mut sim);

    let yielded = messages(&sim)
        .into_iter()
        .find(|line| line.contains("yields ground-sage"))
        .expect("the grind never reported");
    assert!(
        yielded.contains("+1"),
        "no experience on the line: {yielded:?}"
    );
}

#[test]
fn the_same_work_earns_the_same_on_a_replay() {
    let mut live = laboratory();
    run(&mut live, &["kindle charcoal"]);
    grind(&mut live);
    grind(&mut live);

    let mut replayed = Sim::new(1);
    for (tick, submission) in live.submissions().all().to_vec() {
        while replayed.tick() < tick {
            replayed.step();
        }
        replayed.replay(submission);
    }
    while replayed.tick() < live.tick() {
        replayed.step();
    }

    assert_eq!(replayed.experience(), live.experience());
    assert!(live.experience() > 0, "the session earned nothing");
}
