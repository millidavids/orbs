//! Sieges since `0.17`, played: stillness, vigour, and the lie a siege leaves.
//!
//! Every die a siege rolls comes from its own stream, which advances only when
//! the player acts — so the rounds here are the dump's rounds, whatever the
//! wall clock did between keystrokes.

use crate::play::{Game, available};

/// At the wall, with a siege up.
fn besieged() -> Game {
    let game = Game::start();
    game.does("attend bailey", "/tower/bailey")
        .does("defend", "come up the road");
    game
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn stillness_ends_a_siege() {
    if !available() {
        return;
    }
    let game = besieged();
    game.does("hold", "they onslaught")
        .does("debug_spawn stillness 3", "all along")
        .does(
            "quaff stillness",
            "stillness, drunk. a stillness falls along the wall",
        )
        .does("survey rampart", "lifted");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_spell_cannot_end_a_siege() {
    if !available() {
        return;
    }
    let game = besieged();
    game.does("debug_spawn stillness 3", "all along")
        .opens("scribe calming", "calming.spell in ");
    game.type_raw("edit");
    game.type_raw("quaff stillness");
    game.press("Escape");
    game.closes_editor();
    game.does("invoke calming", "the orb takes up calming.spell");
    game.wait_ticks(3);
    // A spell's records are in the log, never the transcript.
    game.does(
        "sift stillness bailey.log",
        "stillness is drunk by hand. a spell cannot end a siege",
    );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn vigour_raises_the_wall() {
    if !available() {
        return;
    }
    let game = besieged();
    game.does("hold", "they onslaught").expect_drawn("/18");
    game.does("debug_spawn vigour 6", "all along")
        .does("quaff vigour", "the vigour flows, 2 a round")
        .expect_drawn("/21");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn recall_names_what_a_brew_costs_in_quintessence() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("recall stillness", "20 quintessence, lost if stopped");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_siege_leaves_a_lie_until_it_is_purged() {
    if !available() {
        return;
    }
    // On the quiet seed the second round gets something past the wall and it
    // lands on the menagerie's log.
    let game = besieged();
    game.does("hold", "they onslaught").does(
        "hold",
        "something got past the wall. a log is not telling the truth",
    );
    // Ended a round from won, so the lie has to outlast the siege.
    game.does("debug_siege", "").does("hold", "the wall holds");
    game.meditates(60);
    game.does("verify", "the orb turns its attention on the whole tower");
    game.meditates(30);
    game.expect("these are not what they say")
        .expect("menagerie.log");
    game.does("attend menagerie", "/tower/menagerie")
        .does("purge menagerie.log", "cleansed")
        .does("verify", "the orb turns its attention on the whole tower");
    game.meditates(30);
    game.expect("these are not what they say")
        .expect_absent("menagerie.log");
}
