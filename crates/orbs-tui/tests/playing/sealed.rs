//! A fresh game is a laboratory and nothing else, and the rest is earned.
//!
//! Every other scenario opens the whole tower (`ORBS_SEALED=0`); these are the
//! only ones that start where a player starts, and they are the See-it line for
//! Phase 10's sealed rooms as a *played* thing rather than a still one.

use crate::play::{Game, available};

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_fresh_game_refuses_the_archive_and_a_clarity_opens_it() {
    if !available() {
        return;
    }
    // **Earned the way a player earns it, at a tester's pace.** One clarity is
    // the laboratory's first station and the archive is what it opens;
    // `debug_spawn` skips the four runs before the still and `meditate` skips
    // the minute the still takes, which is the difference between a scenario
    // and an afternoon.
    let game = Game::sealed();
    game.does("attend archive", "the archive is not yours yet")
        .does("attend laboratory", "/tower/laboratory")
        .does("kindle charcoal", "takes light")
        .does("debug_spawn clarified-draught 1", "clarified-draught")
        .does("distil clarified-draught", "alembic")
        .does("meditate 60", "the archive is yours now")
        .does("attend archive", "/tower/archive");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_towers_own_line_opens_the_grimoire() {
    if !available() {
        return;
    }
    // **Two clarities is sixteen, and sixteen is the tower's line rather than
    // any room's.** The grimoire and the forge are the only two rooms that open
    // that way, and nothing applied a Ley step's `opens` at all until a review
    // found it — every test there was drove the mastery track, so a played gate
    // is what this route has been missing.
    let game = Game::sealed();
    game.does("attend grimoire", "the grimoire is not yours yet")
        .does("attend laboratory", "/tower/laboratory")
        .does("kindle charcoal", "takes light")
        .does("debug_spawn clarified-draught 2", "clarified-draught")
        .does("distil clarified-draught", "alembic")
        .does("meditate 60", "the alembic yields clarity")
        .does("empty alembic", "clarity")
        .does("distil clarified-draught", "alembic")
        .does("meditate 60", "the grimoire is yours now")
        .does("attend grimoire", "/grimoire");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_towers_listing_names_only_what_is_open() {
    if !available() {
        return;
    }
    let game = Game::sealed();
    game.does("attend /tower", "/tower").send("survey");
    game.expect("laboratory");
    game.refute_after(2, "archive");
}
