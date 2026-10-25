//! The sanctum: a course of wards across three stations, and the rule about them.

use crate::play::{Game, QUIET};

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_course_is_drawn_and_the_board_appears_beside_the_transcript() {
    // The board is not gated on a word — the map's rule and the sheet's: it
    // draws whenever a course is standing, because watching a bound solver and
    // solving it yourself are different activities.
    let Some(game) = Game::start() else { return };
    game.does("attend sanctum", "/tower/sanctum")
        .does("muster", "well up in the wellspring")
        .expect_drawn("wellspring")
        .expect_drawn("conduit")
        .expect_drawn("barrier");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_haul_is_answered_on_the_tick_it_is_typed() {
    // Neither verb takes the production slot and neither schedules anything, so
    // the answer is in the same block as the word — no `meditate` between.
    let Some(game) = Game::start() else { return };
    game.does("attend sanctum", "/tower/sanctum")
        .does("muster", "well up in the wellspring")
        .does(
            "haul wellspring barrier",
            "drawn from the wellspring into the barrier",
        );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_one_rule_is_refused_in_voice() {
    // The whole puzzle, in one refusal. A player who hits this has learned the
    // rule, which is why the line says what happened and not what to do instead.
    let Some(game) = Game::start() else { return };
    game.does("attend sanctum", "/tower/sanctum")
        .does("muster", "well up in the wellspring")
        .does("haul wellspring barrier", "drawn from the wellspring")
        .does("haul wellspring barrier", "will not rest upon the lesser");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_rail_says_how_the_barrier_stands_from_another_room() {
    // The one meter in the tower that counts up. Every other rail detail is work
    // remaining; this is a thing you want more of, so `detail_of` prints the
    // value — and it must be there when the barrier is whole, which is exactly
    // where a `left > 0` gate would have silenced it.
    let Some(game) = Game::seeded(QUIET) else {
        return;
    };
    game.does("attend laboratory", "/tower/laboratory")
        .expect_drawn("sanctum")
        .expect_drawn("py 100%");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_barrier_fades_while_the_player_works() {
    // A point every 30 ticks. A bound, not an exact number, which tmux latency
    // would miss and hang on.
    let Some(game) = Game::seeded(QUIET) else {
        return;
    };
    game.does("attend sanctum", "/tower/sanctum")
        .does("survey pylon", "integrity")
        .meditates(600);

    let standing: u32 = game
        .screen()
        .lines()
        .rev()
        .find(|line| line.contains("integrity"))
        .and_then(|line| line.split_whitespace().find_map(|word| word.parse().ok()))
        .expect("the pylon published no integrity");
    assert_eq!(standing, 100, "the first reading was already worn");

    game.does("survey pylon", "integrity");
    let worn: u32 = game
        .screen()
        .lines()
        .rev()
        .find(|line| line.contains("integrity"))
        .and_then(|line| line.split_whitespace().find_map(|word| word.parse().ok()))
        .expect("the pylon published no integrity");
    assert!(
        (75..=85).contains(&worn),
        "ten minutes wore the barrier to {worn}, which is nowhere near the ~20 points expected",
    );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_neglected_tower_musters_a_taller_course() {
    // The whole of what erosion does today, and the reason the parity has to be
    // readable: an hour unattended takes the barrier to nothing, and a barrier
    // at nothing musters six or seven wards where a kept tower gets three.
    let Some(game) = Game::seeded(QUIET) else {
        return;
    };
    game.does("attend sanctum", "/tower/sanctum")
        .meditates(3600)
        .does("survey pylon", "integrity")
        .does("muster", "well up in the wellspring");
    // The line arrives boxed and marked — `│√ 7 wards well up in…` — so the
    // count is the first token that parses, not the first token.
    let screen = game.screen();
    let drawn = screen
        .lines()
        .filter(|line| line.contains("well up in the wellspring"))
        .find_map(|line| {
            line.split_whitespace()
                .find_map(|word| word.parse::<u32>().ok())
        })
        .expect("no course was mustered");
    assert!(
        drawn > 3,
        "an hour of neglect drew {drawn} wards, which is what a kept tower gets",
    );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_words_reach_nothing_in_another_room() {
    let Some(game) = Game::start() else { return };
    game.does("attend laboratory", "/tower/laboratory")
        .does("muster", "no pylon");
}
