//! The installation, played: the menu, a practice tower, and the real one
//! after it.
//!
//! A dump draws the strip but builds no `App`, so the swap from the menu into
//! the practice tower exists only here. `skip` and `wake` go through
//! `type_raw`: neither reaches a world, so no `wizard $` block ever answers
//! them for `send` to wait on.

use crate::play::{Game, available};

/// From the menu to the practice tower's first step.
fn installing() -> Game {
    let game = Game::installing();
    game.expect_drawn("no tower yet");
    game.type_raw("play");
    game.expect_drawn("open a tower");
    game.type_raw("new");
    game.expect_drawn("how long a game?");
    game.type_raw("short");
    game.expect_drawn("initialising the orb");
    game.expect_drawn("step 1 of 5");
    game
}

/// The seed `status` reports, from its block.
fn seed(game: &Game) -> String {
    game.does("status", "seed");
    let block = game.last_block();
    let Some(seed) = block
        .lines()
        .find(|line| line.contains("seed"))
        .and_then(|line| line.split_whitespace().last())
    else {
        panic!("no seed in `status`:\n{}", game.screen());
    };
    seed.to_owned()
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_new_game_installs_the_orb_and_then_wakes_the_tower() {
    if !available() {
        return;
    }
    let game = installing();
    game.does("help", "recall");
    game.expect_drawn("step 2 of 5");
    game.does("survey", "laboratory");
    game.expect_drawn("step 3 of 5");
    game.does("attend laboratory", "/tower/laboratory");
    game.expect_drawn("step 4 of 5");
    game.does("go to the tower", "attend /tower");
    game.expect_drawn("step 5 of 5");
    let practised = seed(&game);
    game.expect_drawn("wake raises your own tower");

    // A bare Enter is only a line: the next one still reaches the practice
    // tower.
    game.type_raw("");
    game.does("survey", "laboratory");
    game.expect_drawn("try  wake");
    // `wake`, which reaches neither tower.
    game.type_raw("wake");
    game.expect_drawn("cold start");
    game.expect_off_screen("initialising the orb");
    assert_eq!(
        seed(&game),
        practised,
        "the tower is not the one practised in"
    );
    game.does("attend laboratory", "/tower/laboratory");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn skip_wakes_the_tower_at_once() {
    if !available() {
        return;
    }
    let game = installing();
    game.type_raw("skip");
    game.expect_drawn("cold start");
    game.expect_off_screen("initialising the orb");
    let screen = game.screen();
    assert!(
        !screen.contains("$ skip"),
        "`skip` was submitted to a world:\n{screen}",
    );
    game.does("attend laboratory", "/tower/laboratory");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_setting_turns_it_off() {
    if !available() {
        return;
    }
    // With the setting deciding, as a player's game does. `ORBS_SAVE=off`
    // keeps no file, so it starts on and the session holds what the page sets.
    let game = Game::at_the_threshold_by_setting();
    game.expect_drawn("no tower yet");
    game.type_raw("settings");
    game.expect_drawn("habits");
    game.type_raw("habits");
    game.expect_drawn("setup on a new orb [on]");
    game.type_raw("install off");
    game.expect_drawn("setup on a new orb [off]");
    game.type_raw("back");
    game.type_raw("back");
    game.expect_drawn("no tower yet");
    game.type_raw("play");
    game.expect_drawn("open a tower");
    game.type_raw("new");
    game.expect_drawn("how long a game?");
    game.type_raw("short");

    game.expect_drawn("cold start");
    let screen = game.screen();
    assert!(
        !screen.contains("initialising the orb"),
        "the installation ran with the setting off:\n{screen}",
    );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_linear_mirror_keeps_the_lesson_after_a_long_answer() {
    if !available() {
        return;
    }
    // §14's route. `help` answers in more rows than the pane, and the mirror
    // shows a stream's tail, so the strip is pinned above it.
    let game = installing();
    game.does("help", "recall");
    game.press("F5");
    game.expect_drawn("linear F5 back");
    game.expect_drawn("initialising the orb, step 2 of 5, skip ends it");
    game.expect_drawn("survey shows what is around you");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_menu_inside_the_installation_still_works() {
    if !available() {
        return;
    }
    // The practice tower is a tower, so `menu` opens the in-tower menu and
    // `resume` comes back to the strip where it was.
    let game = installing();
    game.type_raw("menu");
    game.expect_drawn("resume");
    game.type_raw("resume");
    game.expect_drawn("initialising the orb");
    game.does("help", "recall");
    game.expect_drawn("step 2 of 5");
}
