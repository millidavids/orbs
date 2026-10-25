//! The siege board counting, and renown moving — what a fight costs and earns.

use crate::play::Game;

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_pledge_counts_the_coffer_down() {
    let Some(game) = Game::start() else { return };
    game.does("attend bailey", "/tower/bailey")
        .does("defend", "come up the road")
        .expect_drawn("coffer d6 1 d8 2 d20 5 24")
        .does("pledge d20 to buckler", "d20 goes behind the buckler for 5")
        .expect_drawn("buckler d20 1 to 20")
        .expect_drawn("coffer d6 1 d8 2 19");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn petition_buys_a_smaller_crowd_with_renown() {
    let Some(game) = Game::start() else { return };
    game.does("attend bailey", "/tower/bailey")
        .does("petition", "that would take 7 renown, and you hold 0")
        .does("debug_renown 9000", "")
        .does(
            "petition",
            "word goes out. at most 10 will come, for 7 renown",
        );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_won_fight_is_sung_about_once() {
    let Some(game) = Game::start() else { return };
    game.does("attend bailey", "/tower/bailey")
        .does("defend", "come up the road")
        .does("debug_siege", "")
        .does("hold", "they are singing about it")
        // Said once, by the round that won — the next word is refused.
        .does("hold", "it is over. defend again when the next one comes")
        .expect_absent("singing");
}
