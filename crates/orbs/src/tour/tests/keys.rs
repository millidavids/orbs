//! The keyboard: the tour's keys reach the game, and a person's take it back.

use bevy::input::ButtonState;
use bevy::prelude::*;
use orbs_sim::parser::Outcome;

use super::{SECOND, TWO, app, person, run, scenes, tour};
use crate::sim::Tower;

#[test]
fn typed_keys_reach_the_prompt_as_a_players_do() {
    // The first scene alone, so it is still the one showing at the end.
    let mut app = app(scenes(TWO).into_iter().take(1).collect());
    run(&mut app, 8 * SECOND);
    let sim = app.world().resource::<Tower>().sim();
    let read: Vec<(&str, Outcome)> = sim
        .parse_log()
        .records()
        .iter()
        .map(|record| (record.input.as_str(), record.outcome))
        .collect();
    assert!(
        read.contains(&("status", Outcome::Resolved)),
        "the line never reached the parser: {read:?}"
    );
    // Its own tick each: the second line landed rather than refusing.
    assert_eq!(sim.domain().as_deref(), Some("forge"), "{read:?}");
}

#[test]
fn a_persons_key_takes_the_scene_and_f11_moves_on() {
    let mut app = app(scenes(TWO));
    person(&mut app, KeyCode::KeyA, ButtonState::Pressed);
    run(&mut app, 8 * SECOND);
    assert!(tour(&app).is_yours());
    assert_eq!(tour(&app).at(), 0, "a taken scene moved on by itself");
    let sim = app.world().resource::<Tower>().sim();
    assert!(
        sim.parse_log()
            .records()
            .iter()
            .all(|record| record.input != "status"),
        "the tour typed over a person"
    );

    person(&mut app, KeyCode::F11, ButtonState::Pressed);
    run(&mut app, 6);
    assert_eq!(tour(&app).at(), 1);
    assert!(!tour(&app).is_yours(), "the next scene began taken");
}

#[test]
fn f11_moves_once_and_its_release_and_a_modifier_do_nothing() {
    let mut app = app(scenes(TWO));
    person(&mut app, KeyCode::F11, ButtonState::Pressed);
    run(&mut app, 12);
    assert_eq!(tour(&app).at(), 1);
    // The release lands after the next scene has begun.
    person(&mut app, KeyCode::F11, ButtonState::Released);
    // ...and a focus regained with Control held.
    person(&mut app, KeyCode::ControlLeft, ButtonState::Pressed);
    run(&mut app, 12);
    assert_eq!(tour(&app).at(), 1, "a release moved the tour again");
    assert!(
        !tour(&app).is_yours(),
        "a release or a modifier took the scene"
    );

    person(&mut app, KeyCode::ShiftLeft, ButtonState::Pressed);
    person(&mut app, KeyCode::F11, ButtonState::Pressed);
    run(&mut app, 12);
    assert_eq!(tour(&app).at(), 0, "Shift+F11 did not go back");
}
