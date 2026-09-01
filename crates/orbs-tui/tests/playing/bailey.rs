//! A siege, fought at a real keyboard (§5.1).
//!
//! `orbs-sim`'s `tests/besieging.rs` proves the rules and the arithmetic. What
//! it cannot prove is that the domain is *playable*: that the board is on screen
//! beside the transcript, that a player can read what is coming and answer it,
//! and that the whole thing can be fought start to finish by typing.
//!
//! **Step 3's See-it line is the one this domain is judged on.** If a
//! hand-played siege is not worth doing, no spell will rescue it — so the
//! scenario below fights one the way a person would rather than driving the
//! model.

use crate::play::{Game, QUIET, available};

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_siege_is_fought_by_typing_and_the_board_is_beside_you() {
    if !available() {
        return;
    }
    let game = Game::seeded(QUIET);
    game.does("attend bailey", "/tower/bailey")
        .does("defend", "come up the road");

    // **The board draws whenever a siege is running**, not when a word is typed
    // — the rule the map, the sheet and the sanctum's board all follow, and what
    // makes a bound decision tree watchable.
    game.expect_drawn("rampart")
        .expect_drawn("garrison")
        .expect_drawn("enemy");

    // A turn: read what is coming, then end it. Nothing races the reading.
    game.does("survey enemy", "spears").does("hold", "you lose");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn nothing_moves_while_you_read_the_board() {
    if !available() {
        return;
    }
    // §10's rule, at a real keyboard: outcome follows *what the player chooses
    // given readable state*, never how fast they act. Twenty seconds of standing
    // still must not advance the siege by a round.
    let game = Game::seeded(QUIET);
    game.does("attend bailey", "/tower/bailey")
        .does("defend", "come up the road");

    game.does("survey rampart", "turns");
    // **Four ticks, not twenty.** `PATIENCE` is 20 *seconds* and the world runs
    // at 1 Hz, so `wait_ticks(20)` has zero margin by construction — it can only
    // pass on a machine idle enough to run the game at real time with no
    // overhead, and it failed the moment the suite ran six tmux sessions at
    // once. Every other scenario in the suite waits 2–4.
    //
    // The property does not need the length: nothing but `hold` advances a
    // siege, so any interval with no `hold` in it proves the same thing.
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
    if !available() {
        return;
    }
    // Step 4, and what retires the ten shipped prose lines saying *"a siege will
    // be what spends them"*.
    let game = Game::seeded(QUIET);
    game.does("attend bailey", "/tower/bailey")
        .does("defend", "come up the road")
        .does("debug_spawn troop 2", "all along")
        .does("deploy troop", "goes down to the line")
        // ...and the wrong word names the right one, which is §6's rule.
        .does("quaff troop", "deploy it");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_siege_runs_beside_a_brewing_loop() {
    if !available() {
        return;
    }
    // **§19's *"a domain stands alone"*, and the domain that needs it most.** A
    // siege exists to test the automation, so a grind must keep working while
    // one is fought — otherwise the enemy has nothing to attack.
    let game = Game::seeded(QUIET);
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
