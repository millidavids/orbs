//! The tower's spell: written at `/tower`, watching every room, and starting
//! and stopping the rooms' spells and nothing else (§19, *The tower's spell*).
//!
//! Debug builds only: `debug_ley` is how a test reaches the concentration a
//! tower spell is written from, and `seneschal` is a dev spell.

#![cfg(debug_assertions)]

mod common;

use common::{ever_said, run, run_all, said};
use orbs_sim::{Sim, tower};

/// A tower at `/tower`, at concentration 2.
fn at_the_tower() -> Sim {
    let mut sim = Sim::new(11);
    run_all(&mut sim, &["debug_ley 96", "attend tower"]);
    sim
}

/// `name`, written for the tower: `write_spell` homes a spell where the player
/// stands, as the editor's save does.
fn tower_spell(sim: &mut Sim, name: &str, lines: &[&str]) {
    run(sim, "attend tower");
    let lines: Vec<String> = lines.iter().map(|line| (*line).to_owned()).collect();
    sim.write_spell(name, &lines);
    sim.step();
}

/// Lines said since `before`, so a test reads only what its own step did.
fn since(sim: &Sim, before: usize) -> Vec<String> {
    said(sim).into_iter().skip(before).collect()
}

#[test]
fn a_tower_spell_is_written_once_the_orb_can_hold_two() {
    let mut sim = Sim::new(11);
    run_all(&mut sim, &["debug_ley 16", "attend tower", "scribe keeper"]);
    assert!(
        sim.opening().is_none(),
        "a tower spell opened at concentration 1"
    );
    assert!(
        ever_said(&sim, "go where the work is first"),
        "{:?}",
        said(&sim)
    );

    run_all(&mut sim, &["debug_ley 96", "scribe keeper"]);
    let opened = sim
        .opening()
        .expect("the editor did not open at concentration 2");
    assert_eq!(opened.domain, "tower");

    // The grimoire is still nowhere work happens.
    run_all(&mut sim, &["attend grimoire", "scribe keeper2"]);
    assert!(sim.opening().is_none(), "a spell opened in the grimoire");
}

#[test]
fn the_orb_says_once_when_it_can_watch_the_whole_tower() {
    let mut sim = Sim::new(11);
    run_all(&mut sim, &["debug_ley 16"]);
    let before = said(&sim).len();
    run_all(&mut sim, &["debug_ley 96"]);
    // In place of the first spell's line, which would tell of a slot the orb
    // already had.
    let heard = since(&sim, before);
    assert!(
        !heard.iter().any(|line| line.contains("hold a spell now")),
        "{heard:?}"
    );
    run_all(&mut sim, &["debug_ley 300"]);
    let said: Vec<String> = said(&sim)
        .into_iter()
        .filter(|line| line.contains("watch the whole tower"))
        .collect();
    assert_eq!(said.len(), 1, "{said:?}");
}

#[test]
fn a_tower_spell_lives_anywhere_in_the_tower_and_ends_outside_it() {
    let mut sim = at_the_tower();
    tower_spell(&mut sim, "keeper", &["repeat", "bide 5", "end"]);
    run_all(&mut sim, &["invoke keeper", "attend laboratory"]);
    sim.step_n(10);
    assert!(
        sim.running_line("keeper").is_some(),
        "a tower spell ended when the player walked into a room"
    );

    run_all(&mut sim, &["attend grimoire"]);
    sim.step_n(2);
    assert!(
        sim.running_line("keeper").is_none(),
        "it outlived the player leaving the tower"
    );
    assert!(
        ever_said(&sim, "keeper.spell needed you there"),
        "{:?}",
        said(&sim)
    );
}

#[test]
fn an_attended_spell_does_not_cast_what_would_end_at_once() {
    // By hand, from the tower: `holding` is the sanctum's, and the player is
    // in the tower, not the sanctum.
    let mut sim = at_the_tower();
    tower_spell(&mut sim, "keeper", &["invoke holding", "bide 3"]);
    let before = said(&sim).len();
    run_all(&mut sim, &["invoke keeper"]);
    sim.step_n(3);
    let heard = since(&sim, before);
    assert!(
        heard
            .iter()
            .any(|line| line.contains("holding.spell needs you in its room")),
        "{heard:?}"
    );
    assert!(sim.running_line("holding").is_none());
    assert!(
        !heard.iter().any(|line| line.contains("needed you there")),
        "it cast a run only to end it: {heard:?}"
    );
    // Cost, not a fault: walking about with a hand-invoked tower spell is fine.
    assert!(sim.tower_brief().is_none_or(|brief| brief.mark.is_none()));

    // Standing in the sanctum, the same line casts.
    run_all(
        &mut sim,
        &["stop keeper", "attend sanctum", "invoke keeper"],
    );
    sim.step_n(3);
    assert!(sim.running_line("holding").is_some(), "{:?}", said(&sim));
}

#[test]
fn only_one_spell_runs_the_tower() {
    let mut sim = at_the_tower();
    tower_spell(&mut sim, "keeper", &["repeat", "bide 5", "end"]);
    tower_spell(&mut sim, "warden", &["repeat", "bide 5", "end"]);
    run_all(&mut sim, &["bind keeper", "bind warden", "invoke warden"]);
    let refusals = said(&sim)
        .into_iter()
        .filter(|line| line.contains("keeper.spell already runs the tower"))
        .count();
    assert_eq!(refusals, 2, "{:?}", said(&sim));
    assert!(sim.running_line("warden").is_none());

    // ...and letting the first go makes room for the second.
    run_all(&mut sim, &["stop keeper", "bind warden"]);
    assert!(ever_said(&sim, "takes up warden.spell"), "{:?}", said(&sim));
}

#[test]
fn a_tower_spell_only_starts_and_stops_spells() {
    let mut sim = at_the_tower();
    tower_spell(
        &mut sim,
        "keeper",
        &[
            "survey",
            "stop forge",
            "wield mortar_and_pestle",
            "grind sage",
            "invoke forging",
            "stop forging",
        ],
    );
    let before = said(&sim).len();
    run_all(&mut sim, &["bind keeper"]);
    sim.step_n(8);
    let said = since(&sim, before);

    for line in [
        "survey",
        "stop forge",
        "wield mortar_and_pestle",
        "grind sage",
    ] {
        assert!(
            said.iter()
                .any(|said| said.contains(&format!("the tower only starts spells, not '{line}'"))),
            "{line:?} was not refused: {said:?}"
        );
    }
    // Started, then called off: both reach the room's spell.
    assert!(
        said.iter()
            .any(|line| line.contains("takes up forging.spell")),
        "{said:?}"
    );
    assert!(
        said.iter()
            .any(|line| line.contains("you let forging.spell go")),
        "{said:?}"
    );
    // Said at cast, too, so a line in an untaken branch is not silent.
    assert!(
        said.iter()
            .any(|line| line.contains("a tower spell only starts and stops spells")),
        "{said:?}"
    );
    // A fault, on the tower's own brief rather than any room's.
    let brief = sim.tower_brief().expect("a held tower spell has a brief");
    assert_eq!(brief.mark, Some(tower::Mark::Fault));
    assert!(
        sim.briefs()
            .iter()
            .all(|room| room.mark != Some(tower::Mark::Fault))
    );
}

#[test]
fn a_bound_name_does_not_hide_a_room_verb() {
    // What the name stands for waits for the run; the verb beside it does not,
    // so a line in an untaken branch is said at cast and in the editor.
    let mut sim = at_the_tower();
    let lines = [
        "let m be mortar_and_pestle",
        "let s be forging",
        "if the lens is working",
        "grind m",
        "end",
        "invoke s",
    ];
    let owned: Vec<String> = lines.iter().map(|line| (*line).to_owned()).collect();
    let reading = sim.read_spell("tower", &owned);
    assert_eq!(
        reading[3].fault.as_ref().map(|fault| fault.key),
        Some("spell_tower_only"),
        "{reading:?}"
    );
    assert!(reading[5].fault.is_none(), "{reading:?}");

    tower_spell(&mut sim, "keeper", &lines);
    run_all(&mut sim, &["invoke keeper"]);
    sim.step_n(2);
    assert!(
        ever_said(
            &sim,
            "keeper.spell line 4: a tower spell only starts and stops spells"
        ),
        "{:?}",
        said(&sim)
    );
}

#[test]
fn a_spell_cannot_start_the_tower_spell() {
    let mut sim = at_the_tower();
    tower_spell(&mut sim, "keeper", &["repeat", "bide 5", "end"]);
    run(&mut sim, "attend laboratory");
    sim.write_spell("climbing", &["invoke keeper".to_owned()]);
    sim.step();
    run_all(&mut sim, &["invoke climbing"]);
    sim.step_n(3);
    assert!(
        ever_said(
            &sim,
            "climbing.spell line 1: only you start a spell for the tower"
        ),
        "{:?}",
        said(&sim)
    );
    assert!(sim.running_line("keeper").is_none());
}

#[test]
fn a_tower_spell_is_offered_only_what_it_may_say() {
    // The editor's guide lists `spell_vocabulary`, and Tab offers
    // `spell_expect`: neither offers a verb the runner would refuse.
    use orbs_sim::parser::Verb;
    let sim = at_the_tower();
    let verbs = orbs_sim::spell_vocabulary(sim.world(), "tower");
    assert!(
        verbs
            .iter()
            .all(|verb| matches!(verb, Verb::Invoke | Verb::Stop)),
        "{verbs:?}"
    );
    assert!(verbs.contains(&Verb::Invoke), "{verbs:?}");

    let offered = orbs_sim::spell_expect(sim.world(), "tower", "", 0, &[]);
    let verbs: Vec<String> = offered
        .expected
        .iter()
        .filter(|item| item.why == orbs_sim::parser::Reason::Verb)
        .map(|item| item.text.clone())
        .collect();
    assert!(verbs.iter().any(|verb| verb == "invoke"), "{verbs:?}");
    assert!(
        !verbs
            .iter()
            .any(|verb| verb == "survey" || verb == "peruse"),
        "Tab offers a tower spell a verb it may not say: {verbs:?}"
    );

    // ...and what follows `stop` is a spell, never a tool it would be refused.
    let stoppable = orbs_sim::spell_expect(sim.world(), "tower", "stop ", 5, &[]);
    assert!(!stoppable.expected.is_empty());
    assert!(
        stoppable
            .expected
            .iter()
            .all(|item| orbs_sim::content::without_extension(&item.text) != item.text),
        "{:?}",
        stoppable.texts()
    );
}

#[test]
fn nothing_writes_a_tower_spell_below_the_gate() {
    // `scribe` refuses it, and so does the save behind it, or any other door
    // makes a spell that may only start spells and that no spell may start.
    let mut sim = Sim::new(11);
    run_all(&mut sim, &["debug_ley 16", "attend tower"]);
    sim.write_spell("keeper", &["bide 1".to_owned()]);
    sim.step();
    assert!(
        ever_said(&sim, "go where the work is first"),
        "{:?}",
        said(&sim)
    );
    run_all(&mut sim, &["invoke keeper"]);
    assert!(sim.running_line("keeper").is_none());

    // At the gate it is written, and from the grimoire it never is.
    run_all(&mut sim, &["debug_ley 96", "attend grimoire"]);
    sim.write_spell("keeper2", &["bide 1".to_owned()]);
    sim.step();
    run_all(&mut sim, &["invoke keeper2"]);
    assert!(sim.running_line("keeper2").is_none());
    run(&mut sim, "attend tower");
    sim.write_spell("keeper3", &["bide 3".to_owned()]);
    sim.step();
    run_all(&mut sim, &["invoke keeper3"]);
    assert!(sim.running_line("keeper3").is_some(), "{:?}", said(&sim));
}

#[test]
fn past_the_tower_the_orb_says_how_many_it_can_hold() {
    let mut sim = Sim::new(11);
    run_all(&mut sim, &["debug_ley 96"]);
    let before = said(&sim).len();
    run_all(&mut sim, &["debug_ley 256"]);
    let heard = since(&sim, before);
    assert!(
        heard
            .iter()
            .any(|line| line.contains("the orb can hold 3 spells now")),
        "{heard:?}"
    );
    assert!(
        !heard.iter().any(|line| line.contains("hold a spell now")),
        "{heard:?}"
    );
}

#[test]
fn a_tower_spell_counts_with_what_it_starts() {
    let mut sim = at_the_tower();
    run_all(&mut sim, &["bind seneschal"]);
    sim.step_n(4);
    assert_eq!(
        sim.held(),
        ["forging.spell", "seneschal.spell"],
        "{:?}",
        said(&sim)
    );
}

#[test]
fn one_spell_works_four_rooms_while_the_player_is_away() {
    let mut sim = at_the_tower();
    run_all(&mut sim, &["bind seneschal", "attend grimoire"]);
    sim.step_n(3600);
    for room in ["forge", "sanctum", "menagerie", "lens"] {
        let row = sim
            .ledger_of(room)
            .expect("every one of these rooms keeps a row");
        assert!(
            row.by_spell > 0,
            "nothing in the {room} by a spell in an hour: {row:?}"
        );
    }
    assert!(sim.tower_brief().is_some_and(|brief| brief.mark.is_none()));
    assert!(
        sim.briefs()
            .iter()
            .all(|room| room.mark != Some(tower::Mark::Fault))
    );
}

/// Whether a siege has dragged `name`'s clock.
fn retimed(sim: &mut Sim, name: &str) -> bool {
    let world = sim.world_mut();
    world
        .query_filtered::<&tower::Name, bevy_ecs::query::With<tower::Retimed>>()
        .iter(world)
        .any(|named| named.0 == name)
}

#[test]
fn a_siege_can_drag_the_tower_spell_and_purge_mends_it() {
    // The enemy attacks the automation, and the tower's spell is the most of it
    // in one place. The clock's pool is every bound spell, and only the tower's
    // is bound here: what it starts runs unbound. A seed whose rounds reach it.
    let found = (1..=20).find_map(|seed| {
        let mut sim = Sim::new(seed);
        run_all(
            &mut sim,
            &["debug_ley 96", "bind seneschal", "attend bailey", "defend"],
        );
        for _ in 0..12 {
            run(&mut sim, "hold");
            if retimed(&mut sim, "seneschal.spell") {
                return Some(sim);
            }
        }
        None
    });
    let mut sim = found.expect("no seed in twenty reached the tower's spell");

    run_all(&mut sim, &["attend grimoire", "purge seneschal"]);
    sim.step_n(10);
    assert!(
        !retimed(&mut sim, "seneschal.spell"),
        "purge did not mend the tower's spell: {:?}",
        said(&sim)
    );
}

#[test]
fn a_rewritten_invoke_line_misdirects() {
    // The script surface marks a line's last word with `-`. On an `invoke`, a
    // fuzzy match back to the spell would make the tower's spell immune to it.
    let mut sim = at_the_tower();
    tower_spell(&mut sim, "keeper", &["invoke holding-"]);
    run_all(&mut sim, &["bind keeper"]);
    sim.step_n(4);
    assert!(
        sim.running_line("holding").is_none(),
        "`holding-` still reached holding.spell: {:?}",
        said(&sim)
    );
}

#[test]
fn seneschal_reads_clean_and_stops_when_let_go() {
    let mut sim = at_the_tower();
    run_all(&mut sim, &["bind seneschal"]);
    sim.step_n(20);
    let unread: Vec<String> = said(&sim)
        .into_iter()
        .filter(|line| line.contains("means nothing") || line.contains("cannot read"))
        .collect();
    assert!(unread.is_empty(), "{unread:?}");

    // Let go: it stops, and what it started finishes on its own.
    run_all(&mut sim, &["stop seneschal"]);
    sim.step_n(7200);
    assert!(sim.running_line("seneschal").is_none());
    assert!(
        sim.running_spells().is_empty(),
        "{:?}",
        sim.running_spells()
    );
}

#[test]
fn no_line_a_tower_spell_can_say_has_a_hole_in_it() {
    // The twin of `spells.rs`'s sweep, over the lines only a tower spell, or a
    // full orb, can say.
    let mut sim = at_the_tower();
    tower_spell(
        &mut sim,
        "broken",
        &[
            "survey",
            "if the moonwell is idle and forging is running",
            "end",
            "invoke holding",
            "wait for a moonrise",
        ],
    );
    run_all(&mut sim, &["invoke broken"]);
    sim.step_n(160);
    run(&mut sim, "stop broken");

    // Bound, it fills the orb, and a second tower spell and a room spell's
    // reach for it are both refused.
    tower_spell(
        &mut sim,
        "keeper",
        &[
            "invoke forging",
            "invoke holding",
            "repeat",
            "bide 5",
            "end",
        ],
    );
    tower_spell(&mut sim, "warden", &["repeat", "bide 5", "end"]);
    run_all(&mut sim, &["bind keeper", "bind warden"]);
    sim.step_n(4);
    run(&mut sim, "attend laboratory");
    sim.write_spell("climbing", &["invoke warden".to_owned()]);
    sim.step();
    run_all(&mut sim, &["invoke climbing"]);
    sim.step_n(3);

    let holes: Vec<String> = said(&sim)
        .into_iter()
        .filter(|line| line.contains('{'))
        .collect();
    assert!(
        holes.is_empty(),
        "an unfilled placeholder was said: {holes:?}"
    );
    for expected in [
        "the tower only starts spells",
        "a tower spell only starts and stops spells",
        "to ask about",
        "needs you in its room",
        "waited too long",
        "the orb is full, holding",
        "already runs the tower",
        "only you start a spell for the tower",
    ] {
        assert!(
            ever_said(&sim, expected),
            "{expected:?} never said: {:?}",
            said(&sim)
        );
    }
}
