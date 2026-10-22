//! The standing garrison, played: the tower's own, refilled by the menagerie.
//!
//! Six always, ten at most, and desertion between. `defend` levies all of it.

use crate::play::{Game, available};

/// A garrison filled to its most: six standing and four from a held beast.
fn at_its_most() -> Game {
    let game = Game::start();
    game.does("attend menagerie", "/tower/menagerie")
        .does("summon", "gathers at the circle")
        // Silent when it has a beast to limn, so there is nothing to wait on.
        .does("debug_circle", "")
        .does("summon", "4 join the garrison")
        .does("survey roster", "spears = 10");
    game
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn defend_levies_the_whole_garrison() {
    if !available() {
        return;
    }
    let game = at_its_most();
    game.does("attend bailey", "/tower/bailey")
        .does("defend", "come up the road")
        .does("survey garrison", "spears = 10");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn above_six_one_deserts_every_240_ticks() {
    if !available() {
        return;
    }
    let game = at_its_most();
    game.meditates(240);
    game.does("survey roster", "spears = 9");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_full_garrison_turns_bodies_away() {
    if !available() {
        return;
    }
    let game = at_its_most();
    game.does("summon", "gathers at the circle")
        .does("debug_circle", "")
        .does(
            "summon",
            "the garrison is at its most, and they are turned away",
        );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn troops_are_no_longer_deployed() {
    if !available() {
        return;
    }
    // `deploy troop` was retired with the arsenal's troops: the word now reaches
    // only what the arsenal holds, so it is offered the names that answer.
    let game = Game::start();
    game.does("attend bailey", "/tower/bailey")
        .does("defend", "come up the road")
        .does("deploy troop", "» deploy")
        .expect_absent("join");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_sealed_tower_stands_on_six() {
    if !available() {
        return;
    }
    // A sealed tower arms its wall before it opens a menagerie.
    let game = Game::sealed();
    game.does("debug_reach sanctum_1", "the orb reaches sanctum_1")
        .does("status", "garrison 6")
        .expect("garrison most 10");
}
