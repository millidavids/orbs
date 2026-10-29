//! `status rates` — a room's rate, what it waits on, and the slot (DESIGN.md
//! §19, *the tower speaks up*).

mod common;

use common::{attended, run, run_all, said};
#[cfg(debug_assertions)]
use common::{cast, distil_spawned, reloaded, write};
use orbs_render::{FieldName, RecordKind, Value};
use orbs_sim::Sim;
#[cfg(debug_assertions)]
use orbs_sim::tower::Hourly;
#[cfg(debug_assertions)]
use orbs_sim::tower::spell::Hold;
use orbs_sim::tower::{Brief, State, Wait, Yield};

/// Grind one sage by hand, to completion, standing in the laboratory.
fn grind(sim: &mut Sim) {
    run(sim, "grind sage");
    sim.step_n(10);
    run(sim, "empty mortar_and_pestle");
}

/// The laboratory's yield, as `status rates` reads it.
fn laboratory(sim: &Sim) -> Yield {
    sim.yields()
        .into_iter()
        .find(|one| one.room == "laboratory")
        .expect("the laboratory is open")
}

/// The laboratory's box on the rail.
fn rail(sim: &Sim) -> Brief {
    sim.briefs()
        .into_iter()
        .find(|brief| brief.name == "laboratory")
        .expect("a laboratory box")
}

/// What `spell` is held on, as the rail and `status rates` read it.
#[cfg(debug_assertions)]
fn held(sim: &Sim, spell: &str) -> Option<Hold> {
    sim.running_spells()
        .into_iter()
        .find(|cast| cast.spell == spell)
        .and_then(|cast| cast.held)
}

/// Whether `record` opens the `rates` section.
fn opens_rates(record: &orbs_render::Record<'_>) -> bool {
    record.kind() == RecordKind::Section
        && record.field(FieldName::Kind) == Some(Value::Text("rates"))
}

/// The rows the last `status rates` drew, as `(name, detail)`.
fn rows(sim: &Sim) -> Vec<(String, String)> {
    let all: Vec<_> = sim.scrollback().records().iter().collect();
    let start = all.iter().rposition(opens_rates).expect("a rates section");
    all[start + 1..]
        .iter()
        .take_while(|record| record.kind() == RecordKind::Entry)
        .filter_map(|record| {
            let text = |field| match record.field(field) {
                Some(Value::Text(text)) => Some(text.to_owned()),
                _ => None,
            };
            Some((text(FieldName::Name)?, text(FieldName::Detail)?))
        })
        .collect()
}

#[test]
fn two_makings_make_a_pace_and_one_does_not() {
    let mut sim = attended(1, "laboratory");
    grind(&mut sim);
    assert_eq!(sim.pace_of("at:mortar_and_pestle"), None);
    grind(&mut sim);
    let pace = sim
        .pace_of("at:mortar_and_pestle")
        .expect("two grinds are a pace");
    assert_eq!(pace.made, 1);
    assert!(pace.ticks > 0, "{pace:?}");
}

#[test]
fn the_pace_does_not_move_while_nothing_is_made() {
    let mut sim = attended(1, "laboratory");
    grind(&mut sim);
    grind(&mut sim);
    let before = (sim.pace_of("at:mortar_and_pestle"), sim.earning());
    assert!(before.1.is_some(), "two grinds earned and made no pace");
    sim.step_n(600);
    assert_eq!(
        (sim.pace_of("at:mortar_and_pestle"), sim.earning()),
        before,
        "a forecast moved while nothing was made",
    );
}

#[cfg(debug_assertions)]
#[test]
fn two_brews_read_as_a_rate_and_it_survives_a_save() {
    let mut sim = Sim::new(1);
    distil_spawned(&mut sim, 2);
    let rate = laboratory(&sim).rate;
    assert!(matches!(rate, Hourly::PerHour(n) if n > 0), "{rate:?}");
    let restored = reloaded(&sim.snapshot());
    assert_eq!(
        restored.snapshot().progress.throughput,
        sim.snapshot().progress.throughput,
        "the ring or the tick it began watching moved across a reload",
    );
    assert_eq!(laboratory(&restored).rate, rate);
}

#[cfg(debug_assertions)]
#[test]
fn events_riding_on_a_completion_do_not_enter_the_ring() {
    // `bind` notes `event:bound` through `tally::note`: counted, and never a
    // making. Asked of the saved ring, since one entry makes no pace and a
    // pace-based check could not fail.
    let mut sim = Sim::new(1);
    distil_spawned(&mut sim, 2);
    assert_eq!(sim.concentration(), 1, "two brews bought no slot");
    write(&mut sim, "tending", &["grind sage"]);
    run(&mut sim, "bind tending");
    assert_eq!(sim.tally("event:bound"), 1, "the bind noted nothing");
    let ring = sim
        .snapshot()
        .progress
        .throughput
        .expect("a save always carries the ring");
    assert!(
        !ring.made.contains_key("event:bound"),
        "a note entered the ring"
    );
    assert!(ring.made.contains_key("potion"), "the brews did not");
}

#[test]
fn a_room_waiting_on_a_ready_mortar_says_so() {
    let mut sim = attended(1, "laboratory");
    run(&mut sim, "grind sage");
    sim.step_n(10);
    match laboratory(&sim).wait {
        Wait::At {
            instrument, state, ..
        } => {
            assert_eq!(instrument, "mortar_and_pestle");
            assert_eq!(state, State::Ready);
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_working_mortar_says_how_long_and_holds_the_slot() {
    let mut sim = attended(1, "laboratory");
    run(&mut sim, "grind sage");
    match laboratory(&sim).wait {
        Wait::At { state, left, .. } => {
            assert_eq!(state, State::Working);
            assert!(left.is_some_and(|left| left > 0), "{left:?}");
        }
        other => panic!("{other:?}"),
    }
    let slot = sim.slot().expect("a grind holds the slot");
    assert_eq!(slot.name, "mortar_and_pestle");
}

#[cfg(debug_assertions)]
#[test]
fn a_flask_holding_half_a_recipe_names_what_it_lacks() {
    let mut sim = attended(1, "laboratory");
    run_all(
        &mut sim,
        &[
            "debug_spawn ground-salt 1",
            "move ground-salt to flask_and_rod",
        ],
    );
    let instruments = sim.instruments();
    let flask = instruments
        .iter()
        .find(|instrument| instrument.name == "flask_and_rod")
        .expect("a flask");
    assert_eq!(flask.state, State::Gathering);
    let lack = flask
        .wants
        .as_ref()
        .expect("a gathering flask lacks something");
    assert!(lack.material().ends_with("tincture"), "{lack:?}");
}

#[cfg(debug_assertions)]
#[test]
fn every_blocking_path_names_its_blocker() {
    // `bide`, `wait for` and an empty `pull` yield without `would_block`; each
    // must still say what holds it, as a busy tool always did.
    let mut sim = attended(11, "laboratory");
    run(&mut sim, "debug_take satchel_1");
    let paths: [(&str, &str, fn(&Hold) -> bool); 3] = [
        (
            "patient",
            "bide 30",
            |hold| matches!(hold, Hold::Biding(left) if *left > 0),
        ),
        ("watchful", "wait for the mortar_and_pestle", |hold| {
            matches!(hold, Hold::Watching(_))
        }),
        ("hungry", "pull note from satchel", |hold| {
            matches!(hold, Hold::Pulling(_))
        }),
    ];
    for (spell, line, holds) in paths {
        cast(&mut sim, spell, &[line]);
        sim.step_n(2);
        let hold = held(&sim, spell);
        assert!(hold.as_ref().is_some_and(holds), "{spell}: {hold:?}");
        run(&mut sim, &format!("stop {spell}"));
    }
    run(&mut sim, "grind sage");
    cast(&mut sim, "eager", &["grind sage"]);
    sim.step_n(2);
    let hold = held(&sim, "eager");
    assert!(matches!(hold, Some(Hold::Busy(_))), "{hold:?}");
}

#[cfg(debug_assertions)]
#[test]
fn a_retimed_spells_hold_does_not_flicker() {
    // Sabotage skips most of a retimed spell's ticks. Clearing the hold on
    // those flipped the rail between the hold and the tool every second.
    let mut sim = attended(11, "laboratory");
    cast(&mut sim, "patient", &["bide 120"]);
    sim.step_n(2);
    let node = sim
        .world_mut()
        .query::<(bevy_ecs::entity::Entity, &orbs_sim::tower::spell::Running)>()
        .iter(sim.world())
        .map(|(entity, _)| entity)
        .next()
        .expect("the spell is running");
    sim.world_mut()
        .entity_mut(node)
        .insert(orbs_sim::tower::Retimed { drag: 3 });
    for tick in 0..12 {
        sim.step();
        assert!(
            held(&sim, "patient").is_some(),
            "the hold went blank on tick {tick} of a retimed bide",
        );
    }
}

#[test]
fn status_rates_lists_every_open_room_and_the_slot() {
    let mut sim = attended(1, "laboratory");
    run_all(&mut sim, &["grind sage", "status rates"]);
    let rows = rows(&sim);
    let names: Vec<&str> = rows.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(
        names,
        [
            "laboratory",
            "archive",
            "lens",
            "forge",
            "menagerie",
            "sanctum",
            "slot"
        ]
    );
    let (_, slot) = rows.last().expect("a slot row");
    assert!(slot.contains("mortar_and_pestle"), "{slot}");
}

#[test]
fn bare_status_is_unchanged_and_draws_no_rates() {
    let mut sim = Sim::new(1);
    run(&mut sim, "status");
    let records = sim.scrollback().records();
    assert!(
        !records.iter().any(|record| opens_rates(&record)),
        "bare status drew the rates",
    );
}

#[test]
fn how_are_things_going_still_reads_the_report() {
    // The plain register runs past its command; a closed slot must not swallow
    // `going` and answer with a refusal.
    let mut sim = Sim::new(1);
    run(&mut sim, "how are things going");
    assert!(
        sim.scrollback()
            .records()
            .iter()
            .any(|record| record.field(FieldName::Name) == Some(Value::Text("tick"))),
        "the report did not run: {:?}",
        said(&sim)
    );
}

#[test]
fn status_with_a_word_it_does_not_know_names_the_word_it_does() {
    let mut sim = Sim::new(1);
    run(&mut sim, "status zorb");
    let last = said(&sim).last().cloned().unwrap_or_default();
    assert!(last.contains("rates"), "{last}");
    assert!(last.contains("zorb"), "{last}");
}

#[test]
fn the_rail_reads_a_stalled_room_as_waiting_not_idle() {
    // ROADMAP's defect: the rail was most silent exactly when a room stalled.
    let mut sim = attended(1, "laboratory");
    run(&mut sim, "grind sage");
    sim.step_n(10);
    run(&mut sim, "attend archive");
    let brief = rail(&sim);
    assert_eq!(brief.state, State::Ready);
    assert_eq!(brief.detail.as_deref(), Some("at mp"));
    let spoken = brief.detail_spoken.expect("a shorthand is heard in full");
    assert!(spoken.contains("mortar_and_pestle is ready"), "{spoken}");
}

#[test]
fn a_resting_laboratory_is_idle_and_says_nothing_more() {
    let sim = attended(1, "archive");
    assert_eq!(rail(&sim).state, State::Empty);
    assert_eq!(rail(&sim).detail, None);
}

#[test]
fn the_rail_and_status_rates_read_one_function() {
    let mut sim = attended(1, "laboratory");
    run_all(&mut sim, &["grind sage", "attend archive"]);
    let yields = sim.yields();
    for brief in sim.briefs().into_iter().filter(|brief| brief.built) {
        let one = yields
            .iter()
            .find(|one| one.room == brief.name)
            .expect("every open box is a row");
        assert_eq!(brief.wait, one.wait, "{}", brief.name);
    }
}

#[cfg(debug_assertions)]
#[test]
fn a_held_spell_is_on_the_rail() {
    let mut sim = attended(11, "laboratory");
    cast(&mut sim, "patient", &["bide 30"]);
    sim.step_n(2);
    let detail = rail(&sim).detail;
    assert!(
        detail
            .as_deref()
            .is_some_and(|detail| detail.starts_with("bides")),
        "{detail:?}"
    );
}

#[test]
fn the_forges_detail_says_what_the_nodes_publish() {
    use orbs_sim::tower::{Charm, Charmed, charm::Kind};
    // Laid by hand, as `tests/enchanting.rs` does, so no seed's puzzle decides
    // whether there is a charm to read.
    let mut sim = attended(3, "laboratory");
    let mortar = orbs_sim::tower::reach::look(sim.world())
        .kind(orbs_sim::parser::NounKind::Place)
        .find("mortar_and_pestle")
        .expect("the laboratory has a mortar");
    let from = *sim.world().resource::<orbs_sim::Tick>();
    sim.world_mut()
        .entity_mut(mortar)
        .insert(Charmed(vec![Charm {
            kind: Kind::Hurried,
            from,
            ticks: 300,
        }]));
    sim.step();
    let forge = |sim: &Sim| {
        sim.briefs()
            .into_iter()
            .find(|brief| brief.name == "forge")
            .and_then(|brief| brief.detail)
            .unwrap_or_default()
    };
    let graced = forge(&sim);
    assert!(graced.starts_with("graced"), "{graced}");
    // ...and it flips where the published reading does.
    let left: u64 = graced
        .trim_start_matches("graced ")
        .trim_end_matches('t')
        .parse()
        .expect("ticks left");
    sim.step_n(left - orbs_sim::tower::charm::EBBING_AT);
    let ebbing = forge(&sim);
    assert!(ebbing.starts_with("ebbing"), "{ebbing}");
}
