//! The tally learns whose hand: `status` keeps a ledger, and a room says what
//! its spells did.

use crate::play::{Game, available};

/// The archive, with a spell copying a scroll for the player.
fn a_spell_at_work() -> Game {
    let game = Game::start();
    game.does("attend archive", "/tower/archive")
        .does("debug_spawn fragment 8", "all along")
        .does("debug_spell assembling", "in a hand not yours")
        .does("invoke assembling", "the orb takes up assembling.spell");
    game.meditates(120);
    game
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn status_keeps_a_ledger_of_whose_hand() {
    if !available() {
        return;
    }
    let game = a_spell_at_work();
    game.does("status", "ledger")
        .expect("scrolls copied 1")
        .expect("scrolls copied by your spells 1")
        .expect("done by your spells 1");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_room_counts_its_spells_share() {
    if !available() {
        return;
    }
    let game = a_spell_at_work();
    game.expect_drawn("your spells' share 1");
}
