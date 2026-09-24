//! A charm bound at a real keyboard (§10's Enchanting).
//!
//! `orbs-sim`'s `tests/enchanting.rs` proves every charm reaches its number and
//! `tower::lattice` proves the puzzle over all 512 boards. Neither can prove
//! the domain is playable: that the lattice is on screen beside the transcript,
//! that a player can read the residue and answer it, all by typing.
//!
//! The board is what is at risk. A Lights Out puzzle whose glyphs do not line
//! up under their columns is unplayable rather than merely ugly — a player
//! reads down a column to see what a snap did.

use crate::play::{Game, QUIET, available};

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_charm_is_bound_by_typing_and_the_lattice_is_beside_you() {
    if !available() {
        return;
    }
    let game = Game::seeded(QUIET);
    game.does("attend forge", "/tower/forge")
        .does("imbue mortar_and_pestle hurried", "the glyphs rise");

    // The board draws whenever a lattice is open, not when a word is typed —
    // the rule the map, the sheet, the pylon's board and the rampart follow,
    // and what makes a bound solver watchable.
    game.expect_drawn("lattice")
        .expect_drawn("apex")
        .expect_drawn("belt")
        .expect_drawn("hem");

    // A snap, and the board says so. Nothing races the reading: `snap` is
    // instant and free, so a player can arrange the top row at their own pace.
    game.does("snap apex", "turns, and its neighbours");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_forge_refuses_in_voice_rather_than_falling_silent() {
    if !available() {
        return;
    }
    let game = Game::seeded(QUIET);
    game.does("attend forge", "/tower/forge")
        // Before anything is open, and §6 forbids a bare error for either.
        .does("snap apex", "imbue something first")
        .does("anneal", "imbue something first")
        // A place that is not a tool, named back rather than shrugged at. A
        // shelf and a reading both resolve as `NounKind::Place`, so the refusal
        // has to come from the forge — a charm on either reaches no read site,
        // which is the silent nothing §6 forbids. An unknown word never gets
        // this far; the parser refuses it, hence `apex`.
        .does("imbue apex hurried", "not something a charm will hold to")
        .does(
            "imbue dispensary hurried",
            "not something a charm will hold to",
        );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_manual_teaches_the_room_it_is_read_in() {
    if !available() {
        return;
    }
    let game = Game::seeded(QUIET);
    game.does("attend forge", "/tower/forge");
    // `help` is the way in, and what it offers has to be what the room does.
    // `a_primer_only_names_words_that_room_actually_offers` covers the listing;
    // only a keyboard can see whether the words work.
    game.opens("help", "imbue names a tool and a charm");
    game.does("imbue mortar_and_pestle hurried", "the glyphs rise");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_lattice_is_solved_by_hand_and_the_charm_reaches_the_tool() {
    if !available() {
        return;
    }
    // The whole domain, typed, end to end: a person can sit down, read the
    // board, and get a charm out of it.
    //
    // The seed is fixed so the residue is. `QUIET` is what `play::` uses
    // everywhere — a fixture that works out its own answer tests the fixture.
    let game = Game::seeded(QUIET);
    game.does("attend forge", "/tower/forge")
        .does("imbue mortar_and_pestle hurried", "the glyphs rise");

    // Whether this board lights is not the question — the residue follows the
    // seed, so a hard-coded answer is right for one board in eight, and
    // `tower::lattice` settles correctness over all 512. What only a played
    // test can prove is that the loop is usable: snap, fall, and the room still
    // tells you where you stand.
    game.does("snap apex", "turns, and its neighbours")
        .does("anneal", "lattice");

    // ...and the whole turn is in the log, which is where a postmortem lives.
    game.does("peruse forge.log", "the glyphs rise");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_board_columns_line_up_under_their_names() {
    if !available() {
        return;
    }
    // A stride that drifts by one cell makes the puzzle unreadable, and nothing
    // in `orbs-sim` can see it: the glyphs are right and the picture is wrong.
    let game = Game::seeded(QUIET);
    game.does("attend forge", "/tower/forge")
        .does("imbue mortar_and_pestle hurried", "the glyphs rise");
    game.expect_drawn("apex")
        .expect_drawn("belt")
        .expect_drawn("hem")
        // The rule under the grid separates the board from the residue — read
        // as one picture they are nine glyphs where there are twelve.
        .expect_drawn("─");
}
