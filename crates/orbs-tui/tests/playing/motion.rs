//! Motion in the terminal, on the `motion` row: numbers roll and a gain says how
//! much, or everything lands settled.
//!
//! End states and the `+N` only — a capture taken mid-roll is timing, and the
//! session's unit tests drive the clocks with a delta of their own.

use crate::launch::Switch;
use crate::play::{Game, available};

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_gain_rolls_and_says_how_much() {
    if !available() {
        return;
    }
    let game = Game::launch().motion(Switch::Always).settled();
    game.send("debug_renown 20");
    // Bright, then normal, then dim over a second — long enough to be seen.
    game.expect_drawn("+20");
    game.expect_drawn("] 20/25");
    game.expect_off_screen("+20");
}

/// The first and last lines of `recall clarity`'s answer, which at 1200 baud
/// takes the whole 1.4-second ceiling to type.
const FIRST: &str = "a potion of clear sight";
const LAST: &str = "(flask_and_rod, 14t)";

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn text_types_itself_and_a_key_finishes_it() {
    if !available() {
        return;
    }
    let game = Game::launch().motion(Switch::Always).settled();
    game.send("menu");
    game.expect_drawn("the tower waits");
    game.type_raw("settings");
    game.expect_drawn("what would you set?");
    game.type_raw("habits");
    game.type_raw("pace 1200");
    game.expect_drawn("pace typing speed, baud [1200]");
    game.type_raw("back");
    game.type_raw("back");
    game.expect_drawn("the tower waits");
    game.type_raw("resume");
    game.expect_off_screen("the tower waits");

    game.send("recall clarity");
    game.expect(FIRST);
    let early = game.screen();
    assert!(
        !early.contains(LAST),
        "the whole answer was on screen as its first line landed:\n{early}",
    );
    game.press("End");
    game.expect(LAST);
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn still_text_lands_whole() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.send("recall clarity");
    game.expect(FIRST);
    let screen = game.screen();
    assert!(
        screen.contains(LAST),
        "a still game typed its answer:\n{screen}",
    );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_still_game_lands_every_number_settled() {
    if !available() {
        return;
    }
    // The harness's own default, held to its word: still means no roll and no
    // `+N` on the frame the number lands.
    let game = Game::start();
    game.send("debug_renown 20");
    game.expect_drawn("] 20/25");
    let screen = game.screen();
    assert!(
        !screen.contains("+20"),
        "a still game showed a gain rolling:\n{screen}",
    );
}
