//! The far orb: four sigils of six, repeats and all, and nothing but the two
//! numbers and which way they went.

use crate::play::{Game, available};

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_probe_opens_a_reading_and_reports_the_figure() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend lens", "/tower/lens")
        .does("probe", "four sigils hold it shut")
        .expect("astray");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_press_is_instant_and_costs_the_tower_nothing() {
    if !available() {
        return;
    }
    // **`PRESS_TICKS = 0`** (§19). A press was twelve ticks of the one
    // production slot; that scarcity is withdrawn, so a bound solver runs
    // beside a full brewing loop with no contention at all. The claim here is
    // that the answer is in the same block as the word — no `meditate` between.
    let game = Game::start();
    game.does("attend lens", "/tower/lens")
        .does("probe", "aligned");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn there_is_no_lens_to_look_through_elsewhere() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        // **The refusal names the fixture, not the room** — `tower::fixture_of`,
        // which is what made a verb scope itself to the domain that declares it.
        .does("probe", "there is no prism here to probe with");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_socket_turns_to_a_sigil_by_name() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend lens", "/tower/lens")
        .does("probe", "four sigils hold it shut")
        .does("dial second borax", "the second socket turns to borax");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_socket_that_does_not_exist_is_answered_with_ones_that_do() {
    if !available() {
        return;
    }
    // **`fifth` never reaches the socket check**, and that is the interesting
    // part: the parser resolves nouns before the verb sees them, so an unknown
    // word comes back as numbered suggestions (`»`) drawn from what the tower
    // actually has. A scenario expecting *"the four are first to fourth"* is
    // expecting a refusal the player never gets.
    let game = Game::start();
    game.does("attend lens", "/tower/lens")
        .does("probe", "four sigils hold it shut")
        .does("dial fifth nitre", "»");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_dial_before_a_reading_says_to_probe_first() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend lens", "/tower/lens")
        .does("dial second borax", "probe first");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn survey_answers_two_different_questions_about_one_ward() {
    if !available() {
        return;
    }
    // **Two channels onto one ward**: a player reads the numbers and deduces, a
    // spell reads which way each one moved. It was three questions — a socket
    // also reported `settled`, `loose` and a count of what it had left to try,
    // which is the orb answering *is this position right?* (§19).
    let game = Game::start();
    game.does("attend lens", "/tower/lens")
        .does("probe", "four sigils hold it shut")
        // The prism carries the counts and both deltas; a socket carries what is
        // in it, and nothing else at all.
        .does("survey prism", "aligned")
        .expect("astray")
        .expect("level")
        .does("dial second borax", "turns to borax")
        .does("survey second", "borax");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_socket_never_says_whether_it_is_right() {
    if !available() {
        return;
    }
    // **The rule the whole domain rests on, from the player's side.** No
    // codemaker answers a question about one position; whether a socket is
    // settled is the player's own bookkeeping. Three words used to leak it —
    // `settled`, `loose` and `untried` — and the last did it silently, by
    // falling to nought on exactly that fact.
    let game = Game::start();
    game.does("attend lens", "/tower/lens")
        .does("probe", "four sigils hold it shut")
        .does("survey second", "alum")
        .expect_absent("settled")
        .expect_absent("loose")
        .expect_absent("untried");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_dial_moves_one_socket_and_a_sigil_may_repeat() {
    if !available() {
        return;
    }
    // **What allowing repeats bought.** A dial used to *exchange* with whichever
    // socket held the sigil you named, so two moved at once and `aligned` rising
    // could not be attributed to either. The opening aperture is
    // `nitre alum borax quartz`, so this puts a second `alum` on the board.
    let game = Game::start();
    game.does("attend lens", "/tower/lens")
        .does("probe", "four sigils hold it shut")
        .does("dial first alum", "the first socket turns to alum")
        .does("survey second", "alum");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_board_is_drawn_whenever_a_reading_is_open() {
    if !available() {
        return;
    }
    // **The sheet names its own columns and its own sigils**, and it was ten
    // cells before it was thirty-nine (§19): nothing said which column was
    // which socket, and nothing anywhere said `♦` was `pewter`. Both name
    // tables come from the sim through `Ward::view` — they are content, and
    // `orbs-render` may not depend on `orbs-sim`.
    let game = Game::start();
    game.does("attend lens", "/tower/lens")
        .does("probe", "four sigils hold it shut")
        .expect_drawn("first")
        .expect_drawn("fourth")
        .expect_drawn("pewter");
}
