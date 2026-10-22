//! Demons, played: the circle's second function binds a hold into the arsenal.

use crate::play::{Game, available};

/// A golem bound from a held beast — four within par.
fn bind_a_golem(game: &Game) {
    game.does("summon", "gathers at the circle")
        .does("debug_circle", "")
        .does("summon golem", "4 golem bound to the arsenal");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_golem_is_bound_and_deployed() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend menagerie", "/tower/menagerie");
    bind_a_golem(&game);
    bind_a_golem(&game);
    game.does("survey arsenal", "golem = 8")
        .does("attend bailey", "/tower/bailey")
        .does("defend", "come up the road")
        .does("deploy golem", "the golem flows, 2 a round")
        .does("survey arsenal", "golem = 6");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn too_few_golems_will_not_flow() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend menagerie", "/tower/menagerie");
    bind_a_golem(&game);
    game.does("attend bailey", "/tower/bailey")
        .does("defend", "come up the road")
        .does("deploy golem", "you hold 4 golem. it flows from 6");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_sealed_tower_knows_no_demon_yet() {
    if !available() {
        return;
    }
    // A station opens each demon; at a lesser circle none is open, so the name
    // does not resolve at all.
    let game = Game::sealed();
    game.does("debug_reach lens_1", "menagerie is yours")
        .does("attend menagerie", "/tower/menagerie")
        .does("summon", "a lesser beast gathers")
        .does("summon golem", "¿ summon");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn plain_english_stops_a_supply() {
    if !available() {
        return;
    }
    // `grammar` is deterministic and reads `phrasings.toml`, where `stub` holds
    // six laboratory lines and nothing about a wall.
    let game = Game::launch().env("ORBS_AUGURY", "grammar").settled();
    game.does("attend bailey", "/tower/bailey")
        .does("defend", "come up the road")
        .does("debug_spawn warding 6", "all along")
        .does("quaff warding", "the warding flows, 2 a round")
        .does("cut off the warding", "≈ stop warding")
        .expect("you stop the warding");
}
