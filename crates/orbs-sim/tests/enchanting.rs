//! The forge's charms, driven through the real schedule.
//!
//! `tests/persistence.rs` proves a charm survives a save and `tower::charm`'s own
//! tests prove the arithmetic. Neither asks the question this file does: **does a
//! charm actually reach the number it is supposed to change**, through the same
//! `Sim` a player drives?
//!
//! That distinction is not pedantry here. The whole of Phase 9 is turning one
//! `if` into a composition, and `tower::dice` says why it had to be: *"it works
//! because there is exactly one source and one effect, and it does not
//! generalise — a second source would need the call site to know about it."* A
//! test that checks the component is present would pass against a composition
//! that reaches no call site at all.
//!
//! # Charms are placed through `world_mut` until the forge exists
//!
//! `imbue` arrives with the room, two steps after the component. Waiting until
//! then to test any of this is how a thing gets carried on faith — which is the
//! blind spot `persistence.rs` records having hit once already. The escape hatch
//! is documented for *"mutating the world between steps"*, which is exactly this.

use orbs_sim::Sim;
use orbs_sim::tower::{Charm, Charmed, charm::Kind};

/// Lay a charm on a node by hand, as the forge will once it is built.
fn lay(sim: &mut Sim, on: bevy_ecs::entity::Entity, kind: Kind, ticks: u64) {
    let now = *sim.world().resource::<orbs_sim::Tick>();
    let held = sim
        .world()
        .get::<Charmed>(on)
        .cloned()
        .unwrap_or_else(|| Charmed(Vec::new()));
    let mut held = held;
    held.lay(Charm {
        kind,
        from: now,
        ticks,
    });
    sim.world_mut().entity_mut(on).insert(held);
}

/// The laboratory, with the mortar loaded and nothing else going on.
fn laboratory() -> Sim {
    let mut sim = Sim::new(3);
    sim.submit("attend laboratory");
    sim.step();
    sim
}

/// What the one run in flight has left to do, as `(done, total)`.
fn meter(sim: &Sim) -> Option<(u64, u64)> {
    sim.instruments()
        .into_iter()
        .find_map(|instrument| instrument.meter)
        .map(|meter| (meter.done, meter.total))
}

/// The mortar, wherever it is.
fn mortar(sim: &Sim) -> bevy_ecs::entity::Entity {
    orbs_sim::tower::reach::look(sim.world())
        .kind(orbs_sim::parser::NounKind::Place)
        .find("mortar_and_pestle")
        .expect("the laboratory has a mortar")
}

/// A grind is eight ticks; under `hurried` it is four.
///
/// **The number, not the component.** This is the claim the whole composition
/// exists to make good on, and it is measured at the meter — the same interval
/// `land::finish` compares against and the panel is drawn from — rather than by
/// asking whether a `Charmed` is present.
#[test]
fn a_charmed_tool_works_at_the_charmed_rate() {
    let plain = {
        let mut sim = laboratory();
        sim.submit("grind sage");
        sim.step();
        meter(&sim).expect("the mortar is running").1
    };

    let charmed = {
        let mut sim = laboratory();
        let at = mortar(&sim);
        lay(&mut sim, at, Kind::Hurried, 500);
        sim.submit("grind sage");
        sim.step();
        meter(&sim).expect("the mortar is running").1
    };

    assert!(
        charmed < plain,
        "a hurried mortar took as long as a plain one: {plain} then {charmed}",
    );
}

/// ...and it stops when the charm does.
///
/// **Read when a run starts, like heat.** §10.1 checks the athanor at `begin`
/// and lets the run finish even if the fire dies under it, and speed follows
/// heat — so what has to lapse is the *next* run, not the one in flight. A test
/// that only checked the charm was gone would pass against a `hastened` that had
/// stopped reading charms entirely.
#[test]
fn a_lapsed_charm_stops_changing_the_number() {
    let mut sim = laboratory();
    let at = mortar(&sim);
    lay(&mut sim, at, Kind::Hurried, 20);

    sim.submit("grind sage");
    sim.step();
    let short = meter(&sim).expect("the mortar is running").1;

    // Past the charm, and past the run it shortened.
    sim.submit("meditate 60");
    sim.step();
    sim.submit("empty mortar_and_pestle");
    sim.step();
    sim.submit("meditate 6");
    sim.step();

    sim.submit("grind sage");
    sim.step();
    let long = meter(&sim).expect("the mortar is running again").1;

    assert!(
        long > short,
        "the charm went on working after it lapsed: {short} then {long}",
    );
}

/// A charm the tool does not hold changes nothing.
///
/// **Absent is nought**, which is this tower's rule everywhere and is sharper
/// than it looks: `watch::many_at` answers an absent reading with nought, so the
/// failure mode of getting this wrong is a plausible number rather than an
/// error. Every new read site gets this test.
#[test]
fn a_charm_of_another_kind_changes_nothing() {
    let plain = {
        let mut sim = laboratory();
        sim.submit("grind sage");
        sim.step();
        meter(&sim).expect("the mortar is running").1
    };

    let mut sim = laboratory();
    let at = mortar(&sim);
    lay(&mut sim, at, Kind::Whetted, 500);
    sim.submit("grind sage");
    sim.step();

    assert_eq!(
        meter(&sim).expect("the mortar is running").1,
        plain,
        "a whetted mortar ground at a different rate",
    );
}

/// Both sources reach one call site, which is the architecture in one test.
///
/// A `quickening-scroll` sets `Quickened` on the **room**; a charm sits on one
/// **tool**. `tower::dice` predicted that a second source would need the call
/// site to know about it — so the call site does not know, and this is what says
/// so: the two produce the same number, and neither is stacked on the other.
#[cfg(debug_assertions)]
#[test]
fn a_scroll_and_a_charm_reach_the_same_rate() {
    let by_scroll = {
        let mut sim = laboratory();
        for line in ["debug_spawn quickening-scroll", "wield quickening-scroll"] {
            sim.submit(line);
            sim.step();
        }
        sim.submit("grind sage");
        sim.step();
        meter(&sim).expect("the mortar is running").1
    };

    let by_charm = {
        let mut sim = laboratory();
        let at = mortar(&sim);
        lay(&mut sim, at, Kind::Hurried, 500);
        sim.submit("grind sage");
        sim.step();
        meter(&sim).expect("the mortar is running").1
    };

    assert_eq!(
        by_scroll, by_charm,
        "the two sources of one effect disagree at the call site",
    );

    // ...and holding both is not a quartering. Two halvings would put a
    // §11.5 Production duration into the Instant band, which is a different
    // game rather than a stronger charm.
    let by_both = {
        let mut sim = laboratory();
        let at = mortar(&sim);
        lay(&mut sim, at, Kind::Hurried, 500);
        for line in ["debug_spawn quickening-scroll", "wield quickening-scroll"] {
            sim.submit(line);
            sim.step();
        }
        sim.submit("grind sage");
        sim.step();
        meter(&sim).expect("the mortar is running").1
    };
    assert_eq!(by_both, by_charm, "two sources stacked into a quartering");
}

/// **The forge replays, through a fall that fails and a charm it cannot pay
/// for.**
///
/// Determinism is the one risk in this phase with no partial failure mode: a
/// divergent replay still looks like a working game, and it would make the
/// balance harness, offline catch-up and every regression test unsound at once.
///
/// The forge is where it would land, because `imbue` is the only thing in the
/// domain that draws — one number from `RngStream::Forge` per lattice. So this
/// drives a session that opens several lattices, fails falls, and runs the pool
/// dry, then replays the whole thing from its own submissions.
///
/// **A run that never hits the refusal tests the path that did not change.** The
/// assertion below is what makes sure it does.
#[cfg(debug_assertions)]
#[test]
fn a_forge_session_replays_through_a_failed_fall_and_an_empty_pool() {
    let script = {
        let mut script = vec!["attend forge".to_owned()];
        // Six or more apiece against a ceiling of 24, with a failed fall costing
        // the same as a good one — so the pool runs out partway through and the
        // rest are refused.
        //
        // **A `meditate` between the falls, because a fall takes the slot.**
        // Without it every anneal after the first bounces off the busy lock and
        // the pool never drains — which is what this test read as *the refusal
        // path is unreachable* when the slot became real.
        for _ in 0..8 {
            script.push("imbue mortar_and_pestle hurried".to_owned());
            script.push("snap belt".to_owned());
            script.push("anneal".to_owned());
            script.push("meditate 25".to_owned());
        }
        script
    };

    let mut played = Sim::new(5);
    for line in &script {
        played.submit(line);
        played.step();
    }
    let spoken = |sim: &Sim| -> Vec<String> {
        sim.scrollback()
            .records()
            .iter()
            .filter_map(
                |record| match record.field(orbs_render::FieldName::Message) {
                    Some(orbs_render::Value::Text(text)) => Some(text.to_owned()),
                    _ => None,
                },
            )
            .collect()
    };
    assert!(
        spoken(&played).iter().any(|line| line.contains("you hold")),
        "the script never ran the pool dry, so it tests the unchanged path: {:?}",
        spoken(&played),
    );

    let mut replayed = Sim::new(5);
    for (tick, submission) in played.submissions().all().to_vec() {
        while replayed.tick() < tick {
            replayed.step();
        }
        replayed.replay(submission);
    }
    while replayed.tick() < played.tick() {
        replayed.step();
    }

    assert_eq!(
        spoken(&played),
        spoken(&replayed),
        "a forge session replayed differently",
    );
}

/// A `fruitful` tool yields twice, and never doubles what it throws away.
///
/// **The second half is the one worth asserting.** A charm that doubled the
/// husks would double the *scouring* — the opposite of a boon, and a defect a
/// test that only counted the output would never see.
#[cfg(debug_assertions)]
#[test]
fn a_fruitful_tool_yields_twice_and_leaves_no_more_behind() {
    let ground = |charmed: bool| {
        let mut sim = laboratory();
        if charmed {
            let at = mortar(&sim);
            lay(&mut sim, at, Kind::Fruitful, 500);
        }
        for line in ["grind sage", "meditate 9"] {
            sim.submit(line);
            sim.step();
        }
        let shelf = orbs_sim::tower::home(sim.world(), "sage").expect("the dispensary");
        let at = mortar(&sim);
        let count = |node, what: &str| {
            orbs_sim::tower::holdings(sim.world(), node)
                .into_iter()
                .find(|(name, _)| name == what)
                .map_or(0, |(_, held)| held)
        };
        (count(at, "ground-sage"), count(at, "husks"), shelf)
    };

    let (plain_made, plain_left, _) = ground(false);
    let (charmed_made, charmed_left, _) = ground(true);
    assert!(
        charmed_made > plain_made,
        "a fruitful mortar yielded no more: {plain_made} then {charmed_made}",
    );
    assert_eq!(
        charmed_left, plain_left,
        "a fruitful mortar left more husks behind, which is a charm that \
         doubles the scouring",
    );
}

/// A `bountiful` stacks pays two fragments for one walk.
#[cfg(debug_assertions)]
#[test]
fn a_bountiful_stacks_pays_twice() {
    let paid = |charmed: bool| {
        let mut sim = Sim::new(3);
        for line in ["attend archive", "research"] {
            sim.submit(line);
            sim.step();
        }
        if charmed {
            let stacks = orbs_sim::tower::reach::look(sim.world())
                .kind(orbs_sim::parser::NounKind::Place)
                .find("stacks")
                .expect("the archive has stacks");
            // **Longer than the walk**, which the first version was not: a maze
            // takes thousands of ticks and a five-thousand-tick charm lapsed
            // before the fragment landed, so the test read as *the charm does
            // nothing* when it was really *the charm ended first*.
            lay(&mut sim, stacks, Kind::Bountiful, 100_000);
        }
        // **A solver, not a blind walk.** Four bearings tried in a fixed order
        // is not a maze algorithm — it cycles at any junction where two ways
        // read alike, which is §19's *"the solver that was never a solver"*.
        // `roaming` is shipped and solves this seed.
        sim.submit("invoke roaming");
        sim.step();
        for _ in 0..3 {
            sim.submit("meditate 3600");
            sim.step();
        }
        let cabinet =
            orbs_sim::tower::home(sim.world(), "fragment").expect("fragments have a home");
        orbs_sim::tower::holdings(sim.world(), cabinet)
            .into_iter()
            .find(|(name, _)| name == "fragment")
            .map_or(0, |(_, held)| held)
    };

    let plain = paid(false);
    assert!(
        plain > 0,
        "the walk never paid at all — the fixture is broken"
    );
    assert!(
        paid(true) > plain,
        "a bountiful stacks paid the same as a plain one: {plain}",
    );
}

/// A charm on one tool does not reach another.
///
/// **The containment claim**, and it is not obvious: `charmed` walks *up* to the
/// domain as well as asking the node, because a `quickening-scroll` hangs its
/// state on the room. So a charm laid on one instrument must not be found on its
/// neighbour — which is what this asks and what a domain-wide lookup would fail.
#[cfg(debug_assertions)]
#[test]
fn a_charm_on_one_tool_does_not_reach_its_neighbour() {
    let mut sim = laboratory();
    let at = mortar(&sim);
    lay(&mut sim, at, Kind::Hurried, 500);

    let alembic = orbs_sim::tower::reach::look(sim.world())
        .kind(orbs_sim::parser::NounKind::Place)
        .find("alembic")
        .expect("the laboratory has an alembic");
    assert!(
        orbs_sim::tower::charmed(sim.world(), at, Kind::Hurried),
        "the mortar lost its own charm",
    );
    assert!(
        !orbs_sim::tower::charmed(sim.world(), alembic, Kind::Hurried),
        "a charm laid on the mortar was found on the alembic too",
    );
}
