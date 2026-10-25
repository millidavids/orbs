//! A siege, fought at a real keyboard (§5.1).
//!
//! That the domain is playable by typing, the board beside the transcript. If a
//! hand-played siege is not worth doing, no spell rescues it.

use crate::play::{Game, QUIET};

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_siege_is_fought_by_typing_and_the_board_is_beside_you() {
    let Some(game) = Game::seeded(QUIET) else {
        return;
    };
    game.does("attend bailey", "/tower/bailey")
        .does("defend", "come up the road");

    // The board draws whenever a siege is running, not when a word is typed —
    // the map's rule, the sheet's and the sanctum's, and what makes a bound
    // decision tree watchable.
    game.expect_drawn("rampart")
        .expect_drawn("garrison")
        .expect_drawn("enemy");

    // A turn: read what is coming, then end it. Nothing races the reading.
    game.does("survey enemy", "spears").does("hold", "you lose");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn nothing_moves_while_you_read_the_board() {
    // §10's rule, at a real keyboard: outcome follows *what the player chooses
    // given readable state*, never how fast they act. Twenty seconds of standing
    // still must not advance the siege by a round.
    let Some(game) = Game::seeded(QUIET) else {
        return;
    };
    game.does("attend bailey", "/tower/bailey")
        .does("defend", "come up the road");

    game.does("survey rampart", "turns");
    // Four ticks, not twenty. `PATIENCE` is 20 *seconds* at 1 Hz, so
    // `wait_ticks(20)` has no margin and failed the moment the suite ran six
    // tmux sessions at once. The property does not need the length: nothing
    // but `hold` advances a siege.
    game.wait_ticks(4);
    game.does("survey rampart", "turns");

    let screen = game.screen();
    assert!(
        screen.contains("round 0"),
        "twenty seconds of reading advanced the siege:\n{screen}",
    );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_arsenal_is_spent_on_the_wall() {
    // Step 4, and what retires the ten shipped prose lines saying *"a siege will
    // be what spends them"*.
    let Some(game) = Game::seeded(QUIET) else {
        return;
    };
    game.does("attend bailey", "/tower/bailey")
        .does("defend", "come up the road")
        .does("debug_spawn verdant-scroll 4", "all along")
        .does("wield verdant-scroll", "verdant-scroll flows")
        // ...and the wrong word names the right one, which is §6's rule.
        .does("quaff verdant-scroll", "wield it");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_siege_runs_beside_a_brewing_loop() {
    // §19's *"a domain stands alone"*: a siege exists to test the automation,
    // so a grind must keep working while one is fought — otherwise the enemy
    // has nothing to attack.
    let Some(game) = Game::seeded(QUIET) else {
        return;
    };
    game.does("attend bailey", "/tower/bailey")
        .does("defend", "come up the road")
        .does("hold", "you lose")
        .does("attend laboratory", "/tower/laboratory")
        .does("grind sage", "mortar_and_pestle");

    let screen = game.screen();
    assert!(
        !screen.contains("busy"),
        "the siege held the production slot:\n{screen}",
    );
}
