//! The menagerie: a beast's temper, held by limning the circle, on a real keyboard.
//!
//! **Typed, like every room but the archive's walk.** The menagerie was the one
//! real-time surface in the game while it was a chant; it is a logic puzzle now
//! (§19), so what is worth playing here is what a dump cannot reach — the words
//! resolving from a real prompt, the board appearing beside the transcript and
//! moving when a call is answered, and a spell written at the keyboard doing the
//! search.

use crate::play::{Game, available};

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_beast_is_drawn_and_the_board_appears_beside_the_transcript() {
    if !available() {
        return;
    }
    // **Not gated on a word**, which is every board's rule: it draws whenever a
    // beast waits, because watching a bound search and limning it yourself are
    // different activities.
    let game = Game::start();
    game.does("attend menagerie", "/tower/menagerie")
        .does("summon", "gathers at the circle")
        .expect_drawn("keystone")
        .expect_drawn("sunwise")
        .expect_drawn("widdershins")
        .expect_drawn("temper")
        .expect_drawn("not yet called in");
}

/// **A turned wire reaches a real terminal as `~` on its sense's name.** Seed 181
/// draws a beast whose sunwise glyph is given blood and breath turned over; the
/// mark has to survive the redraw diff and the terminal's own rendering of `~`.
#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_turned_wire_is_marked_on_the_board() {
    if !available() {
        return;
    }
    let game = Game::seeded(181);
    game.does("attend menagerie", "/tower/menagerie")
        .does("summon", "gathers at the circle")
        .expect_drawn("blood, ~breath");
}

/// **A fresh game's first beast is lesser** — one glyph, four columns — and a
/// dark glyph is refused in voice.
#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_sealed_games_first_beast_is_lesser() {
    if !available() {
        return;
    }
    let game = Game::sealed();
    game.does("debug_reach lens_1", "menagerie is yours")
        .does("attend menagerie", "/tower/menagerie")
        .does("summon", "a lesser beast gathers")
        .expect_drawn("1 2 3 4")
        .does("limn sunwise heed", "dark in a lesser circle");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn limning_and_calling_moves_the_board() {
    if !available() {
        return;
    }
    // **The answer row is what a call draws**, and it is the half of the board a
    // painter can get wrong — so the proof is that it appears, with a tally, the
    // moment a call is answered.
    let game = Game::start();
    game.does("attend menagerie", "/tower/menagerie")
        .does("summon", "gathers at the circle")
        .does("limn keystone heed", "keystone is limned heed")
        .does("limn sunwise", "sunwise is limned spurn")
        .does("summon", "on call 1")
        .expect_drawn("answer")
        .expect_drawn("1 call");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_hold_brings_troops_to_the_arsenal() {
    if !available() {
        return;
    }
    // `debug_circle` answers in silence when it has a beast to limn, as
    // `debug_course` does, so there is nothing of its own to wait for.
    let game = Game::start();
    game.does("attend menagerie", "/tower/menagerie")
        .does("summon", "gathers at the circle")
        .does("debug_circle", "")
        .does("summon", "the circle holds")
        .does("survey arsenal", "troop = 4");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn every_refusal_names_the_way_forward() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend menagerie", "/tower/menagerie")
        .does("limn keystone heed", "summon first")
        .does("summon", "gathers at the circle")
        .does("limn keystone sunwise", "is not a humour")
        .does("stop circle", "slips away unheld");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_shipped_search_holds_a_beast_from_the_prompt() {
    if !available() {
        return;
    }
    // **Invoked, not bound** — `bind` costs sixteen this fixture has not earned —
    // and the evidence is in the log, never the pane: a spell's records go to
    // `menagerie.log` (§19).
    let game = Game::start();
    game.does("attend menagerie", "/tower/menagerie")
        .does("invoke taming", "takes up taming.spell");
    game.meditates(900);
    game.does("peruse menagerie.log", "the circle holds");
}
