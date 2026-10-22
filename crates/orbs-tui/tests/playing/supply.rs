//! Supply, played: no single item does anything (§19, *Supply*).
//!
//! A potion, scroll or demon works only while the arsenal holds enough of it,
//! and while it works it drains. The rules are `orbs-sim`'s and tested there;
//! this is the word, the answer and the board's `drawing` row, typed.

use crate::play::{Game, available};

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_potion_flows_from_six_and_faster_when_asked_again() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend bailey", "/tower/bailey")
        .does("defend", "come up the road")
        .does("debug_spawn warding 5", "all along")
        .does("quaff warding", "you hold 5 warding. it flows from 6")
        .does("debug_spawn warding 10", "all along")
        .does("quaff warding", "the warding flows, 2 a round")
        .does("quaff warding", "the warding flows faster, 3 a round")
        // The rate it runs at, counted from one.
        .expect_drawn("drawing warding 2")
        .does(
            "stop warding",
            "you stop the warding. what this round drew stands",
        )
        .expect_drawn("drawing --");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn spent_stock_is_refused_and_kept() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("debug_spawn warding 6", "all along");
    game.meditates(3000);
    game.does("attend bailey", "/tower/bailey")
        .does("defend", "come up the road")
        .does(
            "quaff warding",
            "your warding stores are out. make more and come back",
        )
        .does("survey arsenal", "warding = 6 spent");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn one_item_flows_in_one_place() {
    if !available() {
        return;
    }
    // Open in the laboratory, then asked for on the wall: one supply an item.
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("debug_spawn quickening-scroll 8", "all along")
        .does("wield quickening-scroll", "unrolls, 1 every 300 ticks")
        .does("attend bailey", "/tower/bailey")
        .does("defend", "come up the road")
        .does(
            "wield quickening-scroll",
            "the quickening-scroll already flows elsewhere in the tower. stop it first",
        );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_spell_opens_a_supply_that_has_no_draw() {
    if !available() {
        return;
    }
    // `besieging` guards each rung with `has no draw`, so it opens the warding
    // once the garrison is hurt and never twice. A spell's records are in the
    // log, never the transcript.
    let game = Game::start();
    game.does("attend bailey", "/tower/bailey")
        .does("debug_spawn warding 15", "all along")
        .does("debug_spell besieging", "in a hand not yours")
        .does("invoke besieging", "the orb takes up besieging.spell");
    game.meditates(40);
    game.does("sift warding bailey.log", "the warding flows, 2 a round");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_batch_moves_in_one_word() {
    if !available() {
        return;
    }
    // A distillation yields three, and `move` carries the whole batch; an
    // instrument still takes one.
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("kindle charcoal", "athanor")
        .does("debug_spawn clarified-draught", "all along")
        .does("distil clarified-draught", "alembic");
    game.meditates(60);
    game.does("empty alembic", "clarity")
        .does(
            "move clarity to arsenal",
            "3 clarity: dispensary to arsenal",
        )
        .does("survey arsenal", "clarity = 3");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_lectern_copies_two_scrolls() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend archive", "/tower/archive")
        .does("debug_spawn fragment 4 lectern", "all along")
        .does("wield lectern", "lectern")
        .does("meditate 21", "the lectern yields 2 gleaning-scroll")
        .does("survey lectern", "gleaning-scroll = 2");
}
