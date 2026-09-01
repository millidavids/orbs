//! §8.1's two forms of `verify`, typed at a real keyboard.
//!
//! `orbs-sim`'s `tests/auditing.rs` proves the rules and the arithmetic. What it
//! cannot prove is that a player sitting in front of the tower *sees* any of it:
//! the audit's cost is that the tower stops working while it runs, and "the
//! tower stops working" is a claim about the screen.

use crate::play::{Game, QUIET, available};

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_bare_verify_audits_the_tower_and_stops_the_work() {
    if !available() {
        return;
    }
    // **The price, on screen.** §8.1 makes the audit Production-class precisely
    // so that *which surface do I inspect first* is a decision — and the way a
    // player learns it is a decision is by being refused a grind while it runs.
    let game = Game::seeded(QUIET);
    game.does("attend laboratory", "/tower/laboratory")
        .does("verify", "the whole tower")
        .does("grind sage", "busy verifying");

    // ...and it finishes on its own, saying what it found. The cost is a wait,
    // never a dead end.
    game.meditates(30).expect_somewhere("what it says");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn checking_a_log_leaves_the_shelves_open() {
    if !available() {
        return;
    }
    // **The whole point of rationing.** §8.1: *"four free instant checks **are**
    // `verify --all` by another name."* One look costs you that *kind* of
    // surface and nothing else, so the player still has somewhere to look — and
    // choosing where is the mechanic.
    let game = Game::seeded(QUIET);
    game.does("attend laboratory", "/tower/laboratory")
        .does("verify laboratory.log", "sound")
        .does("verify laboratory.log", "still reading the log")
        // The shelf was never touched by that.
        .does("verify dispensary", "sound");

    // ...and the log comes back on its own.
    game.meditates(21).does("verify laboratory.log", "sound");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_audit_names_what_is_lying() {
    if !available() {
        return;
    }
    // §8.1's design rule — *"the skill is knowing which surface to inspect, not
    // deciphering an obscure clue"*. Once the duration has been paid, the answer
    // is the thing itself, by name.
    let game = Game::seeded(QUIET);
    game.does("attend laboratory", "/tower/laboratory")
        .does("debug_swap", "")
        .does("verify", "the whole tower");
    game.meditates(30).expect_somewhere("not what they say");
}
