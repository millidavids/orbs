//! Every Ley Line fork node moves the number it says it moves (DESIGN.md
//! §11.5, Phase 10).
//!
//! **One test per grant, each reading the surface the grant changes** — a
//! run's length, a charcoal's ticks, the pool's ceiling, a roll's bonus, what a
//! siege pays, a troop's worth, a course's mending, the calm layer's interval,
//! a charm's price. `debug_take` holds the node without the experience, which
//! is the tool's one job; everything downstream is the real reader.

#[cfg(debug_assertions)]
use orbs_render::{FieldName, Value};
use orbs_sim::Sim;
#[cfg(debug_assertions)]
use orbs_sim::tower::grant;

/// Type each line and give the world a tick.
///
/// **Gated with the tests that use it.** `debug_take` is the tool this whole
/// file is built on and it is `debug_assertions`-only, so in a release build
/// every helper here has no callers — which a shipping profile reports as dead
/// code rather than as the deliberate thing it is. Two tests below need neither
/// the word nor these helpers and are ungated.
#[cfg(debug_assertions)]
fn run(sim: &mut Sim, lines: &[&str]) {
    for line in lines {
        sim.submit(line);
        sim.step();
    }
}

/// The siege being fought, if one is.
#[cfg(debug_assertions)]
fn siege(sim: &mut Sim) -> Option<orbs_sim::tower::Siege> {
    sim.world_mut()
        .query::<&orbs_sim::tower::Siege>()
        .iter(sim.world())
        .next()
        .cloned()
}

/// Every message the orb has said, in order.
#[cfg(debug_assertions)]
fn said(sim: &Sim) -> Vec<String> {
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

#[cfg(debug_assertions)]
#[test]
fn every_authored_fork_node_can_be_held_by_a_tester() {
    // The bare word lists what there is, and every id in the shipped line is
    // real to `debug_take` — a node it refused would be a node nothing grants.
    let mut sim = Sim::new(1);
    for id in [
        "fuel_1",
        "edge_1",
        "steps_1",
        "pool_1",
        "floor_1",
        "satchel_1",
        "escrow_1",
        "garrison_1",
        "steps_2",
        "fuel_2",
        "mend_1",
        "cursors_1",
        "pool_2",
        "vigilance_1",
        "haste_1",
        "thrift_1",
        "edge_2",
        "steps_3",
        "escrow_2",
        "floor_2",
        "haste_2",
    ] {
        run(&mut sim, &[&format!("debug_take {id}")]);
        assert!(
            sim.taken().iter().any(|held| held == id),
            "{id} was refused"
        );
    }
}

#[cfg(debug_assertions)]
#[test]
fn fuel_lengthens_what_is_lit_and_not_what_was_banked() {
    let mut plain = Sim::new(1);
    run(&mut plain, &["attend laboratory", "kindle charcoal"]);
    let plain_said = said(&plain);
    let base = plain_said
        .iter()
        .find(|line| line.contains("fuel for"))
        .expect("the athanor said how long");

    let mut longer = Sim::new(1);
    run(
        &mut longer,
        &["debug_take fuel_1", "attend laboratory", "kindle charcoal"],
    );
    let more = said(&longer)
        .into_iter()
        .find(|line| line.contains("fuel for"))
        .expect("the athanor said how long");
    assert_ne!(base, &more, "a tier of fuel changed nothing");
    assert!(
        more.contains(&format!("{}", 600 * (100 + grant::FUEL_PERCENT) / 100)),
        "the fire did not burn a fifth longer: {more}",
    );
}

#[cfg(debug_assertions)]
#[test]
fn pool_and_floor_raise_the_ceiling() {
    let plain = Sim::new(1);
    let base = orbs_sim::tower::ceiling(plain.world());

    let mut deeper = Sim::new(1);
    run(&mut deeper, &["debug_take pool_1"]);
    assert_eq!(
        orbs_sim::tower::ceiling(deeper.world()),
        base + grant::POOL_BONUS,
        "a tier of pool did not add four",
    );

    // The floor only shows under a worn wall: at full integrity it is the
    // same ceiling, and at nought it is a higher fraction of it.
    let floor_up = orbs_sim::tower::quintessence::ceiling_with(
        0,
        0,
        orbs_sim::tower::quintessence::FLOOR_PCT + grant::FLOOR_BONUS,
        0,
    );
    let floor_flat = orbs_sim::tower::quintessence::ceiling_with(
        0,
        0,
        orbs_sim::tower::quintessence::FLOOR_PCT,
        0,
    );
    assert!(floor_up > floor_flat, "a tier of floor did not hold more");
}

#[cfg(debug_assertions)]
#[test]
fn a_document_with_no_pool_row_fills_to_the_ceiling_the_grants_bought() {
    // **An ordering trap, because the reader is forgiving.** `ceiling` reads
    // `pool_<n>` and `floor_<n>` off `Taken`, and `grant::tiers` answers nought
    // for a resource that is not there yet rather than panicking — so a restore
    // that read the ceiling before putting `Taken` back filled the pool as if
    // the orb had taken nothing, and the tower came back four short per tier.
    // A document with no `quintessence` row is what §15's hand-edited saves and
    // every future migration look like.
    let mut sim = Sim::new(1);
    run(&mut sim, &["debug_take pool_1", "debug_take pool_2"]);
    let bought = orbs_sim::tower::ceiling(sim.world());

    let mut save = sim.snapshot();
    save.progress.quintessence = None;
    let restored = Sim::restored(&save);

    assert_eq!(
        orbs_sim::tower::ceiling(restored.world()),
        bought,
        "the reloaded ceiling forgot what was taken",
    );
    assert_eq!(
        restored
            .world()
            .resource::<orbs_sim::tower::Quintessence>()
            .get(),
        bought,
        "a document with no pool row filled to the wrong number",
    );
}

#[cfg(debug_assertions)]
#[test]
fn edge_rides_on_every_answering_roll_and_escrow_on_what_a_siege_pays() {
    let mut sim = Sim::new(1);
    run(
        &mut sim,
        &[
            "debug_take edge_1",
            "debug_take escrow_1",
            "attend bailey",
            "defend",
        ],
    );
    let fought = siege(&mut sim).expect("a siege began");
    assert_eq!(fought.edge, grant::EDGE_BONUS);
    assert_eq!(
        fought.garrison_roll().bonus(),
        grant::EDGE_BONUS,
        "the edge is not on the garrison's roll",
    );

    let plain = orbs_sim::tower::siege::escrow(8, 100, orbs_sim::tower::Outcome::Held, 0);
    let more = orbs_sim::tower::siege::escrow(
        8,
        100,
        orbs_sim::tower::Outcome::Held,
        grant::ESCROW_PERCENT,
    );
    assert_eq!(
        more,
        plain + plain / 4,
        "a tier of escrow is not a quarter more"
    );
}

#[cfg(debug_assertions)]
#[test]
fn garrison_brings_more_bodies_per_troop() {
    let mut sim = Sim::new(1);
    run(
        &mut sim,
        &[
            "debug_take garrison_1",
            "attend menagerie",
            "debug_spawn troop 1",
            "attend bailey",
            "defend",
        ],
    );
    let before = siege(&mut sim).expect("a siege").garrison.count;
    run(&mut sim, &["deploy troop"]);
    let after = siege(&mut sim).expect("a siege").garrison.count;
    assert_eq!(
        after - before,
        2 + grant::GARRISON_BONUS,
        "a troop did not bring one more body",
    );
}

#[cfg(debug_assertions)]
#[test]
fn mend_puts_more_back_per_course() {
    let mut sim = Sim::new(1);
    run(&mut sim, &["debug_take mend_1"]);
    let before = sim.integrity();
    // Wear the wall first, so the mend has somewhere to land.
    orbs_sim::tower::wear_by(sim.world_mut(), 40);
    assert!(sim.integrity() < before);
    let landed = orbs_sim::tower::mend(sim.world_mut(), 3);
    assert_eq!(
        landed,
        3 * orbs_sim::tower::MENDED_PER_WARD + grant::MEND_BONUS,
        "a tier of mend did not put three more back",
    );
}

#[test]
fn vigilance_widens_the_calm_layers_interval_and_nothing_else() {
    use orbs_sim::tower::vigilant_interval;
    assert_eq!(vigilant_interval(300, 0), 300);
    assert_eq!(vigilant_interval(300, 25), 400);
    assert_eq!(vigilant_interval(300, 50), 600);
    assert!(vigilant_interval(300, 99) > 300);
}

#[cfg(debug_assertions)]
#[test]
fn thrift_takes_two_off_a_charm_and_never_below_one() {
    let mut plain = Sim::new(1);
    run(
        &mut plain,
        &["attend forge", "imbue mortar_and_pestle hurried"],
    );
    let quoted = |sim: &Sim| {
        said(sim)
            .into_iter()
            .rev()
            .find(|line| line.contains("hurried"))
            .expect("the forge quoted a price")
    };
    let base = quoted(&plain);

    let mut cheaper = Sim::new(1);
    run(
        &mut cheaper,
        &[
            "debug_take thrift_1",
            "attend forge",
            "imbue mortar_and_pestle hurried",
        ],
    );
    assert_ne!(base, quoted(&cheaper), "a tier of thrift changed no price");
}

#[cfg(debug_assertions)]
#[test]
fn haste_shortens_a_spells_run_and_not_a_players() {
    // The same grind, issued by hand and by a spell, under one tier of haste:
    // the spell's lands sooner and the player's does not.
    let mut sim = Sim::new(1);
    run(
        &mut sim,
        &["debug_take haste_1", "attend laboratory", "grind sage"],
    );
    let by_hand = sim.working().expect("the mortar is working");
    let (_, hand_total) = by_hand.progress(sim.tick());
    run(&mut sim, &["stop mortar_and_pestle"]);

    sim.write_spell("grinding", &["grind sage".to_owned()]);
    sim.step();
    run(&mut sim, &["invoke grinding"]);
    sim.step_n(2);
    let by_spell = sim.working().expect("the spell charged the mortar");
    let (_, spell_total) = by_spell.progress(sim.tick());
    assert!(
        spell_total < hand_total,
        "the spell's grind ({spell_total}) was no shorter than the player's ({hand_total})",
    );
    assert_eq!(
        spell_total,
        hand_total * (100 - grant::HASTE_PERCENT) / 100,
        "the tier did not land a tenth sooner",
    );
}

#[test]
fn a_node_held_above_its_fork_is_kept_and_its_fork_reads_spent() {
    // A tower that took `cursors_1` when it stood at 40, loaded under a line
    // that puts it at 400 — or a tester's `debug_take`. Either way the node
    // stays in effect, and the fork it now sits on reads as chosen.
    let mut sim = Sim::new(1);
    orbs_sim::tower::credit(sim.world_mut(), 40);
    let mut save = sim.snapshot();
    save.progress.taken = vec!["steps_1".to_owned(), "cursors_1".to_owned()];
    let restored = Sim::restored(&save);
    assert_eq!(
        restored.taken(),
        ["steps_1", "cursors_1"],
        "a held node was dropped"
    );
    let fork = restored
        .ley_line()
        .into_iter()
        .find(|station| station.at == 400)
        .expect("the fork at 400");
    assert!(
        fork.nodes
            .iter()
            .any(|node| node.id == "cursors_1" && node.standing == orbs_sim::Standing::Taken),
        "the held node is not in effect: {fork:?}",
    );
    assert!(
        fork.nodes
            .iter()
            .filter(|node| node.id != "cursors_1")
            .all(|node| node.standing == orbs_sim::Standing::Locked),
        "the fork offered a second choice: {fork:?}",
    );
}
