//! The front door, played: the orb's menu before there is a tower.
//!
//! `ORBS_DUMP` builds no `App`, so the threshold, its clock gate and the swap
//! out of it exist only here.

use std::time::{Duration, Instant};

use crate::launch::Switch;
use crate::play::{Game, available};

/// From the menu to a tower of this length, at its first prompt.
fn raise(game: &Game, length: &str) {
    game.type_raw("play");
    game.expect_drawn("open a tower");
    game.type_raw("new");
    game.expect_drawn("how long a game?");
    game.type_raw(length);
    game.expect_drawn("wizard $");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_manual_opens_before_there_is_a_tower() {
    if !available() {
        return;
    }
    let game = Game::at_the_threshold();
    game.expect_drawn("no tower yet");
    game.type_raw("manual");
    game.expect_drawn("which part?");
    game.expect_drawn("the seven rooms, and what each one is");
    // Over the menu, not instead of it.
    game.press("Escape");
    game.expect_drawn("no tower yet");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn back_and_b_go_one_page_back() {
    if !available() {
        return;
    }
    let game = Game::at_the_threshold();
    game.expect_drawn("no tower yet");
    game.type_raw("play");
    game.expect_drawn("open a tower");
    game.type_raw("back");
    game.expect_drawn("no tower yet");
    game.type_raw("settings");
    game.expect_drawn("what would you set?");
    game.type_raw("b");
    game.expect_drawn("no tower yet");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn f6_writes_nothing_until_there_is_a_session() {
    if !available() {
        return;
    }
    // At the menu `F6` would write a trace of a session that has not happened.
    let game = Game::at_the_threshold();
    game.expect_drawn("no tower yet");
    game.press("F6");
    std::thread::sleep(Duration::from_millis(1500));
    assert!(
        !game.has_written("orbs-parse.tsv"),
        "F6 at the menu wrote a trace of a session that has not happened",
    );
    raise(&game, "short");
    game.press("F6");
    let trace = game.wrote("orbs-parse.tsv");
    assert!(!trace.is_empty(), "F6 in a tower wrote an empty trace");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_boot_card_plays_once() {
    if !available() {
        return;
    }
    // Boot, menu, choose — and not boot again, which the terminal build once
    // did by building every swapped session with a live `Boot`.
    let game = Game::launch()
        .booting()
        .motion(Switch::Always)
        .at_the_threshold()
        .spawn();
    game.expect_drawn("no tower yet");
    let chosen = Instant::now();
    raise(&game, "short");
    assert!(
        chosen.elapsed() < Duration::from_secs(5),
        "the tower took {:?} to arrive — the boot card played a second time",
        chosen.elapsed(),
    );
    let screen = game.screen();
    assert!(
        !screen.contains("Bewitching"),
        "the boot card is on screen after a tower was chosen:\n{screen}",
    );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_length_sets_how_far_the_ley_line_runs() {
    if !available() {
        return;
    }
    let medium = Game::at_the_threshold();
    medium.expect_drawn("no tower yet");
    raise(&medium, "medium");
    medium.opens("weave", "0 of 60000");

    // Six figures are shortened, so the long line still fits the screen.
    let long = Game::at_the_threshold();
    long.expect_drawn("no tower yet");
    raise(&long, "long");
    long.opens("weave", "0 of 250k");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_new_game_draws_a_seed_of_its_own() {
    if !available() {
        return;
    }
    // Blank, so nothing names one — as for a player.
    let game = Game::launch()
        .at_the_threshold()
        .env("ORBS_SEED", "")
        .spawn();
    game.expect_drawn("no tower yet");
    raise(&game, "short");
    let screen = game.screen();
    let seed = screen
        .split("orb 0 ")
        .nth(1)
        .and_then(|rest| rest.split_whitespace().next())
        .unwrap_or_default()
        .to_owned();
    assert!(
        seed.parse::<u64>().is_ok(),
        "no seed on the orb's line:\n{screen}",
    );
    assert_ne!(
        seed, "11",
        "a new game drew the harness's seed — ORBS_SEED leaked in"
    );
}
