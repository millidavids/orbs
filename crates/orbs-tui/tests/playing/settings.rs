//! The orb made yours, played: a settings row taking effect in a tower.
//!
//! Every scenario here runs with `ORBS_SAVE=off`, so a row's effect is the
//! session's alone — the case where this build once rebuilt a row from a file
//! it could not write.

use crate::launch::Switch;
use crate::play::Game;

/// A settings page, from the menu a tower opens.
fn page(game: &Game, which: &str) {
    game.send("menu");
    game.expect_drawn("the tower waits");
    game.type_raw("settings");
    game.expect_drawn("what would you set?");
    game.type_raw(which);
}

/// Back out of a settings page to the tower.
fn resume(game: &Game) {
    game.type_raw("back");
    game.type_raw("back");
    game.expect_drawn("the tower waits");
    game.type_raw("resume");
    game.expect_drawn("/tower");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn skip_on_stays_on_when_another_row_moves() {
    let Some(game) = Game::start() else { return };
    page(&game, "habits");
    game.expect_drawn("the boot sequence");
    game.type_raw("skip on");
    game.expect_drawn("skip the boot sequence [on]");
    game.type_raw("install on");
    game.expect_drawn("install setup on a new orb [on]");
    game.expect_drawn("skip the boot sequence [on]");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn echo_changed_leaves_a_plain_echo_out() {
    let Some(game) = Game::start() else { return };
    page(&game, "access");
    game.expect_drawn("the line read back");
    game.type_raw("echo changed");
    game.expect_drawn("the line read back [changed]");
    resume(&game);
    // A line read as typed says nothing back...
    game.does("survey", "laboratory").expect_absent("→ survey");
    // ...and one the orb read differently still shows what it heard.
    game.does("look around", "→ survey");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn motion_off_stills_the_fire() {
    // The athanor's cells, two captures apart: a burning fire changes between
    // them and a still one does not. The row decides, not `ORBS_MOTION`, or the
    // fire is still before the row is touched and this proves nothing.
    let Some(game) = Game::launch().motion(Switch::BySetting).settled() else {
        return;
    };
    game.does("attend laboratory", "/tower/laboratory")
        .does("kindle charcoal", "fuel for 600 ticks");
    let moving = game.ink(95, 103);
    std::thread::sleep(std::time::Duration::from_millis(400));
    assert_ne!(
        moving,
        game.ink(95, 103),
        "the fire did not move with motion on, so this proves nothing",
    );

    page(&game, "access");
    game.expect_drawn("anything moving");
    game.type_raw("motion off");
    game.expect_drawn("motion anything moving [off]");
    resume(&game);
    let still = game.ink(95, 103);
    std::thread::sleep(std::time::Duration::from_millis(400));
    assert_eq!(
        still,
        game.ink(95, 103),
        "the fire still moved after motion off"
    );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_menu_keeps_secrets() {
    // Answered, never offered. `ORBS_DATE` poses a day, since a menu reads the
    // clock.
    let Some(game) = Game::launch()
        .at_the_threshold()
        .env("ORBS_DATE", "2026-10-31")
        .spawn()
    else {
        return;
    };
    game.expect_drawn("no tower yet");
    game.type_raw("xyzzy");
    game.expect_drawn("nothing happens");
    game.type_raw("plugh");
    game.expect_drawn("a hollow voice says 'fool'");
    game.type_raw("moon");
    game.expect_drawn("it is all hallows' eve");
}
