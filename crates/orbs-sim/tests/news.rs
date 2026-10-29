//! The tower's news reaches the player whoever did the work (DESIGN.md §19,
//! *the tower speaks up*).

mod common;

#[cfg(debug_assertions)]
use common::{cast, distil_spawned, reloaded, run, run_all, said};
#[cfg(debug_assertions)]
use orbs_render::{FieldName, Value};
use orbs_sim::Sim;
use orbs_sim::tower::Mark;

/// The mark the rail shows on `room`.
fn mark(sim: &Sim, room: &str) -> Option<Mark> {
    sim.briefs()
        .into_iter()
        .find(|brief| brief.name == room)
        .and_then(|brief| brief.mark)
}

/// Every drawn record's message.
#[cfg(debug_assertions)]
fn drawn(sim: &Sim) -> Vec<String> {
    sim.scrollback()
        .records()
        .drawn()
        .filter_map(|record| match record.field(FieldName::Message) {
            Some(Value::Text(text)) => Some(text.to_owned()),
            _ => None,
        })
        .collect()
}

/// A tower whose first clarity a spell brewed, reaching the laboratory's first
/// station with the player standing in the room.
#[cfg(debug_assertions)]
fn brewed_by_a_spell() -> Sim {
    let mut sim = Sim::new(1);
    run_all(
        &mut sim,
        &[
            "attend laboratory",
            "kindle charcoal",
            "debug_spawn clarified-draught 1",
        ],
    );
    cast(&mut sim, "brewing", &["distil clarified-draught"]);
    sim.step_n(70);
    sim
}

#[cfg(debug_assertions)]
#[test]
fn a_station_a_spell_reached_is_drawn_and_still_credited() {
    let sim = brewed_by_a_spell();
    let records = sim.scrollback().records();
    let station = records
        .iter()
        .find(|record| {
            matches!(record.field(FieldName::Message),
                Some(Value::Text(text)) if text.contains("line advances"))
        })
        .expect("the spell's brew reached the laboratory's first station");
    assert!(
        station.field(FieldName::Spell).is_some(),
        "the station lost its culprit, so `sift` cannot name the spell",
    );
    assert!(station.is_news());
    let shown = drawn(&sim);
    assert!(
        shown.iter().any(|line| line.contains("line advances")),
        "a spell's station never reached the screen: {shown:?}",
    );
    // ...and the spell's ordinary output still goes to the log only.
    assert!(
        !shown.iter().any(|line| line.contains("alembic yields")),
        "a spell's yield was drawn: {shown:?}",
    );
}

#[cfg(debug_assertions)]
#[test]
fn news_survives_a_save() {
    let restored = reloaded(&brewed_by_a_spell().snapshot());
    assert!(
        drawn(&restored)
            .iter()
            .any(|line| line.contains("line advances")),
        "a reload hid the station a spell reached",
    );
}

#[cfg(debug_assertions)]
#[test]
fn a_station_reached_elsewhere_latches_news_and_one_in_the_room_does_not() {
    let mut here = Sim::new(1);
    distil_spawned(&mut here, 1);
    assert_eq!(mark(&here, "laboratory"), None);

    // The same brew, landing while the player is in the archive.
    let mut away = Sim::new(1);
    run_all(
        &mut away,
        &[
            "attend laboratory",
            "kindle charcoal",
            "debug_spawn clarified-draught 1",
            "distil clarified-draught",
            "attend archive",
        ],
    );
    away.step_n(60);
    assert_eq!(mark(&away, "laboratory"), Some(Mark::News));
    // Going to look is the only thing that clears it.
    run(&mut away, "attend laboratory");
    assert_eq!(mark(&away, "laboratory"), None);
}

#[cfg(debug_assertions)]
#[test]
fn a_station_a_bound_spell_reaches_inline_latches_where_the_player_is_not() {
    // A spell's line runs with `Cwd` set to the spell's room, so asking `Cwd`
    // where the player stood said "here" for everything a solver finished.
    // The player's own position rides on the line's `Casting`.
    let mut sim = Sim::new(1);
    distil_spawned(&mut sim, 2);
    assert_eq!(sim.concentration(), 1, "two brews bought no slot");
    run_all(
        &mut sim,
        &[
            "attend sanctum",
            "debug_spell holding",
            "bind holding",
            "attend laboratory",
        ],
    );
    sim.step_n(400);
    assert!(
        drawn(&sim)
            .iter()
            .any(|line| line.contains("sanctum line advances")),
        "the bound spell reached no station: {:?}",
        said(&sim),
    );
    assert_eq!(mark(&sim, "sanctum"), Some(Mark::News));
}

#[cfg(debug_assertions)]
#[test]
fn a_fork_becoming_choosable_is_said_once_and_a_restore_says_nothing() {
    let mut sim = Sim::sealed(1);
    distil_spawned(&mut sim, 3);
    let forks = |sim: &Sim| {
        said(sim)
            .iter()
            .filter(|line| line.contains("forks ahead"))
            .count()
    };
    assert_eq!(forks(&sim), 1, "{:?}", said(&sim));
    sim.step_n(10);
    assert_eq!(forks(&sim), 1, "the fork was said again");

    let mut restored = reloaded(&sim.snapshot());
    restored.step_n(10);
    assert_eq!(forks(&restored), 1, "a restore said the fork again");
}

#[test]
fn a_room_with_no_news_has_no_mark() {
    let sim = Sim::new(1);
    for room in orbs_sim::tower::DOMAINS {
        assert_eq!(mark(&sim, room), None, "{room}");
    }
}
