//! The readers, played: a line and a spell in plain English, and when the orb
//! will not guess.
//!
//! Every other scenario turns both readers off, since the trained weights
//! change on every retrain. These pin one: `stub` answers a fixed table, and
//! `grammar` reads `phrasings.toml` with no weights at all.

use crate::play::Game;

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_augury_reads_a_line_and_reading_plain_stops_it() {
    let Some(game) = Game::launch().env("ORBS_AUGURY", "stub").settled() else {
        return;
    };
    game.does("attend laboratory", "/tower/laboratory")
        .does("turn the sage into powder", "≈ grind sage")
        .expect("sage: dispensary to mortar_and_pestle");

    // `plain` is the options page's word for leaving the reader unasked.
    game.send("menu");
    game.expect_drawn("the tower waits");
    game.type_raw("settings");
    game.expect_drawn("what would you set?");
    game.type_raw("habits");
    game.expect_drawn("how lines are read");
    game.type_raw("reading plain");
    game.expect_drawn("how lines are read [plain]");
    game.type_raw("back");
    game.type_raw("back");
    game.expect_drawn("the tower waits");
    game.type_raw("resume");
    game.expect_drawn("/tower/laboratory");
    game.send("turn the sage into powder");
    game.expect_absent("≈ grind sage");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_spell_line_in_plain_english_is_read_and_marked() {
    let Some(game) = Game::launch().env("ORBS_SCRIVENER", "stub").settled() else {
        return;
    };
    game.does("attend laboratory", "/tower/laboratory")
        .opens("scribe waiting", "waiting.spell in ");
    game.type_raw("edit");
    game.type_raw("hang on ten ticks");
    game.press("Escape");
    game.type_raw("interpret");
    // The fair copy marks what it read rather than what was typed.
    game.expect_drawn("≈").expect_drawn("bide 10");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_guessed_summon_spends_no_call() {
    // A call spends a beast, so the orb will not make one on a reader's guess.
    let Some(game) = Game::launch().env("ORBS_AUGURY", "grammar").settled() else {
        return;
    };
    game.does("attend menagerie", "/tower/menagerie")
        .does("summon", "gathers at the circle")
        .does(
            "make a golem of the beast",
            "the orb will not call the beast in on a guess. type summon to call it",
        );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn winnowing_holds_a_beast() {
    let Some(game) = Game::start() else { return };
    game.does("attend menagerie", "/tower/menagerie")
        .does("debug_spell winnowing", "in a hand not yours")
        .does("invoke winnowing", "the orb takes up winnowing.spell");
    game.meditates(120);
    game.does("sift holds menagerie.log", "the circle holds");
}
