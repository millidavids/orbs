//! `debug_spawn`, and the fact that it is not part of the game.
//!
//! Not gated at the file level: everything that exercises the word is
//! `cfg(debug_assertions)`, but the last test is its complement and runs under
//! `cargo test --release` to assert the door is shut.

#[cfg(debug_assertions)]
use orbs_render::{FieldName, Value};
use orbs_sim::Sim;

/// A laboratory, stood in.
#[cfg(debug_assertions)]
fn laboratory() -> Sim {
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();
    sim
}

/// Every message the orb has said.
#[cfg(debug_assertions)]
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

/// What the place called `place` actually holds, by name.
///
/// Its children, not the scene: `Scene::nouns` includes a `Topic` for every
/// reagent, so a scene sweep says the shelf holds `ground-sage` before any has
/// ever been ground.
///
/// By name, because the archive grew a `cabinet` — finding "the `Store`"
/// returned whichever store the walk reached first.
#[cfg(debug_assertions)]
fn held_in(sim: &Sim, place: &str) -> Vec<String> {
    let world = sim.world();
    let mut stack = vec![orbs_sim::tower::root(world)];
    while let Some(node) = stack.pop() {
        if world
            .get::<orbs_sim::tower::Name>(node)
            .is_some_and(|name| name.0 == place)
        {
            return orbs_sim::tower::children_of(world, node)
                .into_iter()
                .filter_map(|held| world.get::<orbs_sim::tower::Name>(held))
                .map(|name| name.0.clone())
                .collect();
        }
        stack.extend(orbs_sim::tower::children_of(world, node));
    }
    Vec::new()
}

/// What the laboratory's shelf holds — the commonest question here.
#[cfg(debug_assertions)]
fn shelved(sim: &Sim) -> Vec<String> {
    held_in(sim, "dispensary")
}

#[cfg(debug_assertions)]
#[test]
fn a_reagent_asked_for_is_a_reagent_on_the_shelf() {
    let mut sim = laboratory();
    assert!(
        !shelved(&sim).contains(&"ground-sage".to_owned()),
        "the world already had one, so this proves nothing",
    );

    sim.submit("debug_spawn ground-sage");
    sim.step();

    assert!(
        shelved(&sim).contains(&"ground-sage".to_owned()),
        "the spawn did not land: {:?}",
        messages(&sim),
    );
}

#[cfg(debug_assertions)]
#[test]
fn what_is_spawned_is_the_same_thing_the_laboratory_makes() {
    // A spawn that produced a node the instruments did not recognise would let
    // a tester reach a state the game cannot, and chase a bug only their
    // shortcut can produce.
    let mut spawned = laboratory();
    spawned.submit("debug_spawn ground-sage");
    spawned.step();
    spawned.submit("debug_spawn ground-salt");
    spawned.step();
    // Straight into the stage that consumes them, with no grinding at all.
    spawned.submit("kindle charcoal");
    spawned.step();
    spawned.submit("mix ground-sage with ground-salt");
    spawned.step_n(30);

    assert!(
        messages(&spawned)
            .iter()
            .any(|line| line.contains("flask_and_rod")),
        "the flask would not take what was spawned: {:?}",
        messages(&spawned),
    );
}

#[cfg(debug_assertions)]
#[test]
fn a_spawned_potion_is_an_essence_like_a_distilled_one() {
    // The kind, not just the name. §10.1 makes a finished potion an `Essence`
    // and everything else crafting stock, so spawning every name as a reagent
    // would put a `clarity` in the tower that no `distil` could have made.
    let mut sim = laboratory();
    sim.submit("debug_spawn clarity");
    sim.step();
    sim.submit("debug_spawn ground-sage");
    sim.step();

    // The whole tower, because the claim is about the kind and not the room:
    // `tower::home` sends finished work to the arsenal, not to a store.
    let world = sim.world();
    let kinds: Vec<(String, orbs_sim::parser::NounKind)> = {
        let mut stack = vec![orbs_sim::tower::root(world)];
        let mut found = Vec::new();
        while let Some(node) = stack.pop() {
            if let (Some(name), Some(kind)) = (
                world.get::<orbs_sim::tower::Name>(node),
                world.get::<orbs_sim::tower::Nameable>(node),
            ) {
                found.push((name.0.clone(), kind.0));
            }
            stack.extend(orbs_sim::tower::children_of(world, node));
        }
        found
    };

    assert!(
        kinds.contains(&("clarity".to_owned(), orbs_sim::parser::NounKind::Essence)),
        "a spawned potion is not an essence: {kinds:?}",
    );
    assert!(
        kinds.contains(&(
            "ground-sage".to_owned(),
            orbs_sim::parser::NounKind::Reagent
        )),
        "a spawned reagent is not stock: {kinds:?}",
    );
}

#[cfg(debug_assertions)]
#[test]
fn a_count_puts_that_many_on_the_shelf() {
    let mut sim = laboratory();
    sim.submit("debug_spawn ground-sage 3");
    sim.step();

    // Three of one thing is one node with a count, not three nodes — `give`
    // merges, which is what keeps `survey` to one row per reagent.
    sim.submit("survey dispensary");
    sim.step();
    let rows: Vec<String> = messages(&sim);
    assert!(
        shelved(&sim).contains(&"ground-sage".to_owned()),
        "{rows:?}",
    );
    // Spend all three, which only works if all three are there.
    for _ in 0..3 {
        sim.submit("grind ground-sage");
        sim.step_n(20);
    }
    assert!(
        !messages(&sim)
            .iter()
            .any(|line| line.contains("nothing here answers")),
        "fewer than three arrived: {:?}",
        messages(&sim),
    );
}

#[cfg(debug_assertions)]
#[test]
fn a_count_of_none_is_refused_rather_than_spawning_a_nought() {
    // `give` would spawn a node holding `Counted(0)`, breaking `take`'s
    // invariant that a pile reaching zero is despawned — a nought-node answers
    // `has ash` yes for ever with nothing in it.
    let mut sim = laboratory();
    sim.submit("debug_spawn ash 0");
    sim.step_n(2);

    assert!(
        !shelved(&sim).contains(&"ash".to_owned()),
        "a nought-node reached the shelf: {:?}",
        shelved(&sim),
    );
    // Refused the way `lots` is refused: an ordinary unresolved line, because
    // `debug_spawn ash 0` is not a spawn order at all.
    assert!(
        !messages(&sim).iter().any(|line| line.contains("all along")),
        "it reported a spawn: {:?}",
        messages(&sim),
    );
}

#[cfg(debug_assertions)]
#[test]
fn a_name_the_tower_does_not_have_is_refused_and_says_what_it_could_have_been() {
    let mut sim = laboratory();
    sim.submit("debug_spawn moonstone");
    sim.step();
    assert!(
        messages(&sim)
            .iter()
            .any(|line| line.contains("nothing in the tower is called")),
        "{:?}",
        messages(&sim),
    );

    // ...and bare, it lists what there is, which is how a tester finds the name.
    sim.submit("debug_spawn");
    sim.step();
    let listed = messages(&sim)
        .into_iter()
        .find(|line| line.contains("could find"))
        .expect("nothing was listed");
    for expected in ["sage", "ground-sage", "charcoal"] {
        assert!(
            listed.contains(expected),
            "{expected:?} missing from {listed:?}"
        );
    }
}

#[cfg(debug_assertions)]
#[test]
fn it_reaches_the_dispensary_from_wherever_the_player_is_standing() {
    // The point of the tool is skipping the walk, and there is only one shelf
    // in the tower, so there is nothing for "which one" to mean.
    let mut sim = Sim::new(1);
    sim.submit("attend archive");
    sim.step();
    sim.submit("debug_spawn ground-sage");
    sim.step();
    assert!(
        messages(&sim).iter().any(|line| line.contains("all along")),
        "it refused from another room: {:?}",
        messages(&sim),
    );

    // ...and it went to the shelf, which is where the laboratory finds it.
    sim.submit("attend laboratory");
    sim.step();
    assert!(
        shelved(&sim).contains(&"ground-sage".to_owned()),
        "it landed somewhere the laboratory cannot reach: {:?}",
        messages(&sim),
    );
}

#[cfg(debug_assertions)]
#[test]
fn it_lands_on_a_tick_boundary_like_every_other_effect() {
    // A debug command that mutated the world from inside `submit` would land
    // off a tick boundary, so the state it produced could not be reproduced
    // from `(seed, submissions)` and the session would stop replaying.
    let mut sim = laboratory();
    sim.submit("debug_spawn ground-sage");
    assert!(
        !shelved(&sim).contains(&"ground-sage".to_owned()),
        "it landed inside the input call",
    );
    sim.step();
    assert!(shelved(&sim).contains(&"ground-sage".to_owned()));
}

#[cfg(debug_assertions)]
#[test]
fn a_session_that_used_it_still_replays() {
    let mut live = laboratory();
    live.submit("debug_spawn ground-sage 2");
    live.step();
    live.submit("grind ground-sage");
    live.step_n(20);

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

    assert_eq!(messages(&replayed), messages(&live), "the replay diverged");
}

#[cfg(debug_assertions)]
#[test]
fn the_word_is_not_part_of_the_game() {
    // §6.1's tutorial lists the verbs that work, so a word that only exists in
    // some builds would be offered and then absent from the build a player has.
    // Asserted against the vocabulary itself, so a new surface built out of
    // `Verb::ALL` is covered the day it is written.
    for verb in orbs_sim::parser::Verb::ALL {
        assert!(
            !verb.canonical().contains("debug"),
            "{} is in the game's vocabulary",
            verb.canonical(),
        );
    }
    for synonym in orbs_sim::parser::SYNONYMS {
        assert!(
            !synonym.words.iter().any(|word| word.contains("debug")),
            "{:?} is a phrase the parser knows",
            synonym.words,
        );
    }

    // ...and it is not fuzzy-matched: a near miss is an ordinary unresolved line,
    // not a tower that quietly changed under a tester who mistyped.
    let mut sim = laboratory();
    sim.submit("debug_spaw sage");
    sim.step();
    assert!(
        !shelved(&sim).contains(&"sage".to_owned())
            || messages(&sim)
                .iter()
                .all(|line| !line.contains("all along")),
        "a typo spawned something: {:?}",
        messages(&sim),
    );
}

#[cfg(not(debug_assertions))]
#[test]
fn the_word_does_nothing_in_a_release_build() {
    // Every test above is `cfg(debug_assertions)`, so a release build runs none
    // of them and the door is only ever tried from the side it opens on. Run by
    // `cargo test --release`. Four lines of prose still ship — `prose.toml` is
    // compiled in whole — but nothing that can act on them does.
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();

    let shelf: Vec<String> = {
        let world = sim.world();
        let mut stack = vec![orbs_sim::tower::root(world)];
        let mut found = Vec::new();
        while let Some(node) = stack.pop() {
            if world.get::<orbs_sim::tower::Store>(node).is_some() {
                found = orbs_sim::tower::children_of(world, node)
                    .into_iter()
                    .filter_map(|held| world.get::<orbs_sim::tower::Name>(held))
                    .map(|name| name.0.clone())
                    .collect();
                break;
            }
            stack.extend(orbs_sim::tower::children_of(world, node));
        }
        found
    };

    sim.submit("debug_spawn ground-sage 99");
    sim.step_n(4);

    let after: Vec<String> = {
        let world = sim.world();
        let mut stack = vec![orbs_sim::tower::root(world)];
        let mut found = Vec::new();
        while let Some(node) = stack.pop() {
            if world.get::<orbs_sim::tower::Store>(node).is_some() {
                found = orbs_sim::tower::children_of(world, node)
                    .into_iter()
                    .filter_map(|held| world.get::<orbs_sim::tower::Name>(held))
                    .map(|name| name.0.clone())
                    .collect();
                break;
            }
            stack.extend(orbs_sim::tower::children_of(world, node));
        }
        found
    };

    assert_eq!(after, shelf, "a release build spawned something");
}
