//! The menagerie: a figure sung against the tick, on a real keyboard.
//!
//! **This file matters more than its siblings**, because `chorus` is the only
//! real-time keyboard surface in the game — so it is the only one where
//! `drive::escape_prefixed`'s class of bug is reachable, and the only one where
//! a key arriving a tick late is a *wrong answer* rather than a slow one.
//!
//! A dump cannot reach any of that: it writes no key events and advances no
//! clock, so every press lands on one tick and reads `too soon`.

use crate::play::{Game, available};

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_figure_is_drawn_and_the_board_appears_beside_the_transcript() {
    if !available() {
        return;
    }
    // **Not gated on a word**, which is every board's rule: it draws whenever a
    // chant is running, because watching a bound solver and singing it yourself
    // are different activities.
    let game = Game::start();
    game.does("attend menagerie", "/tower/menagerie")
        .does("summon", "gathers at the circle")
        .expect_drawn("leftward")
        .expect_drawn("skyward")
        .expect_drawn("earthward")
        .expect_drawn("rightward");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_arrows_answer_a_figure_and_escape_gives_them_back() {
    if !available() {
        return;
    }
    // **The whole point of the surface, and unreachable from a dump.** A press
    // here is a real `KeyboardInput` on a real clock, so what it proves is that
    // `chorus` took the keys, an arrow reached `Sim::sing`, and Escape handed
    // them back — three things `ORBS_CHANT` can only approximate.
    let game = Game::start();
    game.does("attend menagerie", "/tower/menagerie")
        .does("summon", "gathers at the circle")
        .does("chorus", "the figure is yours");

    game.press("Up");
    // **Waited for, not read immediately.** `press` does not sleep — `Sim::sing`
    // is an instant entry point like `Sim::walk`, so there is no tick to pace
    // against — but the *record* it writes still needs a frame to be drawn, and
    // reading the screen straight after the press caught it before the paint.
    //
    // The syllable's own name is what to wait on: all three answers say it
    // (`skyward rings true`, `skyward falls wide`, `skyward, but too soon`), and
    // which one arrives depends on where the figure was. Pinning that would be
    // pinning the wall clock.
    game.expect("skyward");

    // ...and the prompt has them back. §19's `HeldOver` defect is that a held
    // arrow outlives the Escape and recalls history into the line; typing a
    // whole word afterwards is what proves the prompt is really the prompt.
    game.press("Escape");
    game.does("survey circle", "circle");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn key_repeat_costs_one_syllable_and_not_four() {
    if !available() {
        return;
    }
    // **A finger held on an arrow.** `Sim::sing` reaches the world without a
    // tick, so a frontend looping a frame's events struck once per press and
    // each strike consumed a syllable — six presses ate four of them and
    // collapsed the figure. The guard is one strike a tick.
    let game = Game::start();
    game.does("attend menagerie", "/tower/menagerie")
        .does("summon", "gathers at the circle")
        .does("chorus", "the figure is yours");

    for _ in 0..6 {
        game.press("Up");
    }
    let screen = game.screen();
    assert!(
        !screen.contains("comes apart"),
        "six presses collapsed the figure: {screen}",
    );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_figure_left_alone_collapses_and_the_barrier_pays() {
    if !available() {
        return;
    }
    // **The domain's only cost, end to end through a real clock.** It also
    // proves the pylon's reading moved: `wear_by` republished through a
    // `Cwd`-scoped lookup for one commit, so the barrier fell and `survey pylon`
    // went on saying `integrity = 100` — a chant always collapses while the
    // player is standing in the menagerie, so that path was never taken.
    let game = Game::start();
    game.does("attend menagerie", "/tower/menagerie")
        .does("summon", "gathers at the circle")
        .meditates(20)
        .expect("comes apart");

    game.does("attend sanctum", "/tower/sanctum")
        .does("survey pylon", "integrity");
    let screen = game.screen();
    assert!(
        !screen.contains("integrity = 100"),
        "the barrier was worn and the reading did not move: {screen}",
    );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn both_refusals_name_the_way_forward() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend menagerie", "/tower/menagerie")
        .does("sing skyward", "summon first")
        .does("chorus", "summon first")
        .does("summon", "gathers at the circle")
        .does("summon", "already gathered");
}
