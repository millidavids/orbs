//! A fresh game is a laboratory and nothing else, and the rest is earned
//! (DESIGN.md §11.5, Phase 10).
//!
//! **Driven through real runs**, as `tests/progression.rs` is: there is no
//! public way to hand the sim a deed, so a station is reached the way a player
//! reaches it, and the door that opens is the door the player finds open.

use orbs_render::{FieldName, Value};
use orbs_sim::Sim;

/// Every message the orb has said, in order.
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

/// Type each line and give the world a tick.
fn run(sim: &mut Sim, lines: &[&str]) {
    for line in lines {
        sim.submit(line);
        sim.step();
    }
}

/// Distil one clarity in a sealed tower, the cheap way.
#[cfg(debug_assertions)]
fn brew_a_clarity(sim: &mut Sim) {
    run(
        sim,
        &[
            "attend laboratory",
            "kindle charcoal",
            "debug_spawn clarified-draught 1",
            "distil clarified-draught",
        ],
    );
    sim.step_n(60);
}

#[test]
fn a_sealed_tower_is_a_laboratory_and_an_open_one_is_everything() {
    let sealed = Sim::sealed(1);
    assert!(sealed.is_open("laboratory"));
    for room in [
        "archive",
        "lens",
        "grimoire",
        "sanctum",
        "menagerie",
        "forge",
    ] {
        assert!(!sealed.is_open(room), "{room} is open in a sealed tower");
    }
    assert!(sealed.began_sealed());

    let open = Sim::new(1);
    for brief in open.briefs() {
        assert!(brief.built, "{} is dark in an open tower", brief.name);
    }
    assert!(!open.began_sealed());
}

#[test]
fn a_shut_room_refuses_from_every_door() {
    let mut sim = Sim::sealed(1);
    run(&mut sim, &["attend archive"]);
    assert!(
        said(&sim).iter().any(|line| line.contains("not yours yet")),
        "attend did not refuse in voice: {:?}",
        said(&sim),
    );
    assert_ne!(sim.location(), "/tower/archive", "the player walked in");

    // **By a path that is not the room's name**, which is why the gate asks
    // the node rather than the name.
    run(&mut sim, &["attend stacks", "attend /tower/archive/stacks"]);
    assert!(
        !sim.location().contains("archive"),
        "a path into a shut room was allowed: {}",
        sim.location(),
    );

    // Looking in from the doorway is the same door.
    let before = said(&sim).len();
    run(&mut sim, &["survey archive"]);
    assert!(
        said(&sim)[before..]
            .iter()
            .any(|line| line.contains("not yours yet")),
        "survey looked into a shut room: {:?}",
        &said(&sim)[before..],
    );
}

#[test]
fn a_shut_room_is_not_listed_named_or_reported() {
    let mut sim = Sim::sealed(1);
    // The tower's listing names what is open and nothing else.
    run(&mut sim, &["attend /tower", "survey"]);
    let listing: Vec<String> = sim
        .scrollback()
        .records()
        .iter()
        .filter_map(|record| record.field(FieldName::Name))
        .filter_map(|value| match value {
            Value::Text(text) => Some(text.to_owned()),
            _ => None,
        })
        .collect();
    assert!(
        listing.iter().any(|name| name == "laboratory"),
        "{listing:?}"
    );
    assert!(
        !listing.iter().any(|name| name == "archive"),
        "a shut room was listed: {listing:?}",
    );

    // The boot report, which ran at construction, names only the open rooms.
    assert!(
        !said(&sim).iter().any(|line| line.contains("archive")),
        "the boot report named a shut room: {:?}",
        said(&sim),
    );

    // The dark box on the rail.
    let briefs = sim.briefs();
    let archive = briefs
        .iter()
        .find(|brief| brief.name == "archive")
        .expect("seven boxes");
    assert!(!archive.built, "a shut room lit its box");
}

#[cfg(debug_assertions)]
#[test]
fn brewing_a_clarity_opens_the_archive_and_says_so_once() {
    let mut sim = Sim::sealed(1);
    brew_a_clarity(&mut sim);

    assert!(
        sim.is_open("archive"),
        "the first station did not open the archive"
    );
    let opened = said(&sim)
        .iter()
        .filter(|line| line.contains("archive is yours"))
        .count();
    assert_eq!(
        opened,
        1,
        "opening was said {opened} times: {:?}",
        said(&sim)
    );

    run(&mut sim, &["attend archive"]);
    assert_eq!(sim.location(), "/tower/archive", "the opened room refused");
    assert!(
        sim.briefs()
            .iter()
            .find(|brief| brief.name == "archive")
            .is_some_and(|brief| brief.built),
        "the rail's box stayed dark",
    );
}

#[test]
fn the_ley_line_opens_the_two_rooms_no_deed_reaches() {
    // **The grimoire and the forge hang off the tower's line, not a room's.**
    // Passing a step *is* the grant, so nothing had to be written down — but a
    // room is a set the world holds, and with nothing applying a station's
    // `opens` both rooms stayed shut for ever in the only tower a player ever
    // gets, taking the forge line's charm with them.
    let mut sim = Sim::sealed(1);
    assert!(!sim.is_open("grimoire"));
    assert!(!sim.is_open("forge"));

    orbs_sim::tower::credit(sim.world_mut(), 16);
    assert!(
        sim.is_open("grimoire"),
        "the step at 16 did not open the grimoire"
    );
    assert!(!sim.is_open("forge"), "the forge came early");

    orbs_sim::tower::credit(sim.world_mut(), 40);
    assert!(
        sim.is_open("forge"),
        "the step at 56 did not open the forge"
    );

    // Said once each, and never again as the total climbs past them.
    orbs_sim::tower::credit(sim.world_mut(), 10_000);
    for room in ["grimoire", "forge"] {
        let opened = said(&sim)
            .iter()
            .filter(|line| line.contains(&format!("the {room} is yours")))
            .count();
        assert_eq!(opened, 1, "{room} opening was said {opened} times");
    }
}

#[test]
fn a_sealed_tower_and_the_same_tower_reloaded_carry_the_same_marks() {
    // **`Sealed` is derived, so no document compares it** — which is what let a
    // built tower and its own reload disagree about four nodes. `seal` marks the
    // tree that exists when it runs, and a fresh tower publishes the pylon's
    // integrity and each die's price *after* raising while a restore seals
    // last. Two worlds that must be identical, differing in the component two
    // sabotage queries read with a `Without` filter.
    let sim = Sim::sealed(11);
    let restored = Sim::restored(&sim.snapshot());
    assert_eq!(
        marked(&sim),
        marked(&restored),
        "a reloaded tower is sealed differently from the one it was saved from",
    );
    assert!(
        marked(&sim).iter().any(|path| path.contains("archive")),
        "nothing under a shut room is marked at all",
    );
    assert!(
        !marked(&sim).iter().any(|path| path.contains("laboratory")),
        "the open room is marked",
    );
}

/// Every node under a shut room, by path, in order.
fn marked(sim: &Sim) -> Vec<String> {
    let world = sim.world();
    let mut paths: Vec<String> = world
        .iter_entities()
        .filter(|&node| node.contains::<orbs_sim::tower::Sealed>())
        .map(|node| orbs_sim::tower::path_of(world, node.id()))
        .collect();
    paths.sort();
    paths
}

#[test]
fn a_document_whose_line_outran_its_rooms_catches_up_on_load_and_says_nothing() {
    // The total and what a step opened are two records of one fact, and only
    // the second is written down — so a document with the experience and not
    // the room would keep that room shut for ever, since the crossing happens
    // at an edge already in the past. Caught up silently on load, for
    // `Experience::restore`'s reason.
    //
    // Built from an *open* tower, so nothing about the grimoire was ever said
    // in the stream the save carries: what the load does or does not say is
    // then the only thing the count can be measuring.
    let mut sim = Sim::new(1);
    orbs_sim::tower::credit(sim.world_mut(), 16);
    let mut save = sim.snapshot();
    save.world.sealed = true;
    save.progress.opened = Some(vec!["domain:laboratory".to_owned()]);

    let restored = Sim::restored(&save);
    assert!(
        restored.is_open("grimoire"),
        "the room the line had already paid for stayed shut",
    );
    assert!(
        !restored.is_open("forge"),
        "a station the line has not reached opened anyway",
    );
    assert!(
        !restored.is_open("archive"),
        "a room the tower's line does not open came open with it",
    );
    assert!(
        !said(&restored).iter().any(|line| line.contains("is yours")),
        "a load announced work done yesterday: {:?}",
        said(&restored),
    );
}

#[cfg(debug_assertions)]
#[test]
fn a_document_that_reached_a_station_without_its_opening_catches_up_too() {
    // **A mastery station's `opens` lands on exactly one tick, ever**: `advance`
    // skips anything already in `Reached`, so a document that holds the station
    // and not the thing it opened would never open it — the tally is long past
    // what the deed asks, and the walk goes straight by. The ley line's
    // catching-up needed a twin, and the reasoning that said otherwise was
    // wrong about the code.
    let mut sim = Sim::sealed(1);
    brew_a_clarity(&mut sim);
    let mut save = sim.snapshot();
    assert!(save.progress.reached.contains(&"laboratory_1".to_owned()));
    if let Some(opened) = save.progress.opened.as_mut() {
        opened.retain(|key| key != "domain:archive");
    }

    let restored = Sim::restored(&save);
    assert!(
        restored.is_open("archive"),
        "a station the document says was reached never opened its room",
    );
}

#[test]
fn a_capability_no_station_gates_outlives_a_document_that_never_named_it() {
    // **The authoritative-save failure, arriving through capabilities.** A
    // charm, a gated product or an eighth room that this build ships *ungated*
    // cannot be named by a document written before it existed — and replacing
    // the set outright shut it for the life of every save, with no station able
    // to open it. Unioned with `Opened::start` instead, which is that set's own
    // rule.
    let sim = Sim::sealed(1);
    let mut save = sim.snapshot();
    save.progress.opened = Some(Vec::new());

    let restored = Sim::restored(&save);
    assert!(
        restored.has_opened("charm:hurried"),
        "the forge's flagship charm, which no station gates, was lost on load",
    );
    assert!(
        restored.is_open("laboratory"),
        "the room the game starts in was lost on load",
    );
    assert!(
        !restored.is_open("archive"),
        "a room a station does gate came open with them",
    );
}

#[cfg(debug_assertions)]
#[test]
fn the_wall_is_armed_by_the_sanctum_and_a_charm_by_its_station() {
    let mut sim = Sim::sealed(1);
    run(&mut sim, &["debug_reach laboratory_3"]);
    assert!(
        sim.is_open("sanctum"),
        "the third station did not open the sanctum"
    );
    assert!(
        !sim.has_opened("siege"),
        "the wall is armed before a course"
    );

    // The bailey follows the wall.
    run(&mut sim, &["attend bailey", "defend"]);
    assert!(
        !sim.location().contains("bailey"),
        "the bailey opened before the wall: {}",
        sim.location(),
    );

    run(&mut sim, &["debug_reach sanctum_1"]);
    assert!(sim.has_opened("siege"));
    run(&mut sim, &["attend bailey"]);
    assert_eq!(sim.location(), "/tower/bailey");

    // A charm the forge has not earned.
    run(&mut sim, &["debug_reach lens_1"]);
    assert!(
        sim.has_opened("charm:hurried"),
        "the flagship charm is shut"
    );
    assert!(!sim.has_opened("charm:shielded"));
    run(&mut sim, &["debug_reach lens_2"]);
    assert!(
        sim.has_opened("charm:shielded"),
        "the lens's second station did not open it"
    );
}

#[cfg(debug_assertions)]
#[test]
fn a_sealed_tower_replays_to_the_same_world() {
    let mut live = Sim::sealed(1);
    brew_a_clarity(&mut live);
    run(&mut live, &["attend archive"]);

    let mut replayed = Sim::sealed(1);
    for (tick, submission) in live.submissions().all().to_vec() {
        while replayed.tick() < tick {
            replayed.step();
        }
        replayed.replay(submission);
    }
    while replayed.tick() < live.tick() {
        replayed.step();
    }
    assert_eq!(replayed.location(), live.location());
    assert_eq!(replayed.reached(), live.reached());
    assert!(replayed.is_open("archive"));

    // ...and an *open* tower with the same submissions is a different world,
    // which is why sealed-ness travels with the seed.
    let mut open = Sim::new(1);
    for (tick, submission) in live.submissions().all().to_vec() {
        while open.tick() < tick {
            open.step();
        }
        open.replay(submission);
    }
    assert!(!open.began_sealed());
}

#[test]
fn a_save_from_before_sealing_restores_open_and_a_sealed_one_stays_sealed() {
    let sealed = Sim::sealed(1);
    let save = sealed.snapshot();
    assert!(save.world.sealed);
    let restored = Sim::restored(&save);
    assert!(restored.began_sealed());
    assert!(!restored.is_open("archive"), "a restore opened a shut room");
    assert!(restored.is_open("laboratory"));

    // A document that never had `opened` is a tower that stood in all seven.
    let mut older = save;
    older.progress.opened = None;
    older.world.sealed = false;
    let migrated = Sim::restored(&older);
    for brief in migrated.briefs() {
        assert!(brief.built, "{} is dark after a migration", brief.name);
    }
}

#[test]
fn sabotage_never_strikes_a_shut_room() {
    // The calm layer poisons a log now and then; a strike in a room the player
    // cannot enter would latch a mark on a dark box and be a fault nobody can
    // find. Two hours of ticks, and every log under a shut room stays clean.
    let mut sim = Sim::sealed(3);
    sim.step_n(7200);
    for domain in [
        "archive",
        "lens",
        "grimoire",
        "sanctum",
        "menagerie",
        "forge",
    ] {
        assert!(!sim.is_open(domain), "{domain} opened on its own");
    }
    let poisoned = sim
        .scrollback()
        .records()
        .iter()
        .filter_map(|record| record.field(FieldName::Source))
        .filter_map(|value| match value {
            Value::Text(text) => Some(text.to_owned()),
            _ => None,
        })
        .filter(|source| {
            [
                "archive",
                "lens",
                "grimoire",
                "sanctum",
                "menagerie",
                "forge",
            ]
            .iter()
            .any(|room| source.contains(room))
        })
        .count();
    assert_eq!(poisoned, 0, "something happened in a shut room");
}
