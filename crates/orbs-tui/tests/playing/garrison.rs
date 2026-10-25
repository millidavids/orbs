//! The standing garrison, played: the tower's own, refilled by the menagerie.
//!
//! Six always, ten at most, and desertion between. `defend` levies all of it.

use crate::play::Game;

/// A garrison filled to its most: six standing and four from a held beast.
fn at_its_most() -> Option<Game> {
    let game = Game::start()?;
    game.does("attend menagerie", "/tower/menagerie")
        .does("summon", "gathers at the circle")
        // Silent when it has a beast to limn, so there is nothing to wait on.
        .does("debug_circle", "")
        .does("summon", "4 join the garrison")
        .does("survey roster", "spears = 10");
    Some(game)
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn defend_levies_the_whole_garrison() {
    let Some(game) = at_its_most() else { return };
    game.does("attend bailey", "/tower/bailey")
        .does("defend", "come up the road")
        .does("survey garrison", "spears = 10");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn above_six_one_deserts_every_240_ticks() {
    let Some(game) = at_its_most() else { return };
    game.meditates(240);
    game.does("survey roster", "spears = 9");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_full_garrison_turns_bodies_away() {
    let Some(game) = at_its_most() else { return };
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
    // `deploy troop` was retired with the arsenal's troops: the word now reaches
    // only what the arsenal holds, so it is offered the names that answer.
    let Some(game) = Game::start() else { return };
    game.does("attend bailey", "/tower/bailey")
        .does("defend", "come up the road")
        .does("deploy troop", "» deploy")
        .expect_absent("join");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_sealed_tower_stands_on_six() {
    // A sealed tower arms its wall before it opens a menagerie.
    let Some(game) = Game::sealed() else { return };
    game.does("debug_reach sanctum_1", "the orb reaches sanctum_1")
        .does("status", "garrison 6")
        .expect("garrison most 10");
}
