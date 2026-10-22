//! What the orb remembers from one launch to the next.
//!
//! Every other scenario runs with `ORBS_SAVE=off`, so nothing here could be
//! played until a game could be given a folder of its own: settings, towers, and
//! the one-per-machine warning all live beside the save.

use crate::launch::{Kept, Switch};
use crate::play::{Game, available};

/// The orb's menu, in a game that keeps its towers in `kept`. The `motion` row
/// decides, since what a relaunch remembers is the subject here.
fn at_the_menu(kept: &Kept) -> Game {
    let game = Game::launch()
        .at_the_threshold()
        .kept_in(kept)
        .motion(Switch::BySetting)
        .spawn();
    game.expect_drawn("no tower yet");
    game
}

/// A short tower raised from the menu, at its first prompt.
fn raised(game: &Game) {
    game.type_raw("play");
    game.expect_drawn("open a tower");
    game.type_raw("new");
    game.expect_drawn("how long a game?");
    game.type_raw("short");
    game.expect_drawn("wizard $");
}

/// Put the orb down from inside a tower — `quit` asks once — and wait for the
/// exit-save to land.
fn put_down(game: &Game) {
    game.does("quit", "leave the orb? quit again to go");
    game.type_raw("quit");
    game.expect_gone();
}

/// A row on the access page, as the menu draws it.
fn access_page(game: &Game) {
    game.type_raw("settings");
    game.expect_drawn("what would you set?");
    game.type_raw("access");
    game.expect_drawn("anything moving");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn settings_are_remembered_across_launches() {
    if !available() {
        return;
    }
    let kept = Kept::default();
    {
        let game = at_the_menu(&kept);
        access_page(&game);
        game.expect_drawn("motion anything moving [on]");
        game.type_raw("motion off");
        game.expect_drawn("motion anything moving [off]");
    }
    // A new process, reading only the file the first one wrote.
    let game = at_the_menu(&kept);
    access_page(&game);
    game.expect_drawn("motion anything moving [off]");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn abandon_asks_once() {
    if !available() {
        return;
    }
    let kept = Kept::default();
    {
        let game = at_the_menu(&kept);
        raised(&game);
        put_down(&game);
    }
    let game = at_the_menu(&kept);
    game.type_raw("play");
    game.expect_drawn("1 wizard, a short game, 0 earned");
    game.type_raw("abandon 1");
    game.expect_drawn("set the tower in 1 aside? abandon 1 again to do it");
    game.type_raw("abandon 1");
    game.expect_drawn("no towers yet");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn loading_another_leaves_the_first_as_it_was() {
    if !available() {
        return;
    }
    let kept = Kept::default();
    {
        let game = at_the_menu(&kept);
        raised(&game);
        game.does("attend laboratory", "/tower/laboratory");
        put_down(&game);
    }
    let game = at_the_menu(&kept);
    game.type_raw("play");
    game.expect_drawn("1 wizard, a short game");
    game.type_raw("1");
    game.expect_drawn("/tower/laboratory");

    // A second tower, beside the first: raising it writes the first.
    game.send("menu");
    game.expect_drawn("the tower waits");
    raised(&game);
    game.expect_off_screen("/tower/laboratory");

    // ...and the first comes back where it was left, the second written too.
    game.send("menu");
    game.expect_drawn("the tower waits");
    game.type_raw("play");
    game.expect_drawn("1 wizard, a short game");
    game.type_raw("1");
    game.expect_drawn("/tower/laboratory");
    game.send("menu");
    game.expect_drawn("the tower waits");
    game.type_raw("play");
    game.expect_drawn("2 wizard, a short game");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_new_tower_takes_the_chosen_name() {
    if !available() {
        return;
    }
    // Blank, so the setting decides: an exported `ORBS_WIZARD` outranks it.
    let game = Game::launch()
        .at_the_threshold()
        .env("ORBS_WIZARD", "")
        .spawn();
    game.expect_drawn("no tower yet");
    game.type_raw("settings");
    game.expect_drawn("what would you set?");
    game.type_raw("habits");
    game.expect_drawn("the wizard's name");
    game.type_raw("name morgana");
    game.expect_drawn("the wizard's name [morgana]");
    game.type_raw("back");
    game.type_raw("back");
    game.expect_drawn("no tower yet");
    game.type_raw("play");
    game.expect_drawn("open a tower");
    game.type_raw("new");
    game.expect_drawn("how long a game?");
    game.type_raw("short");
    game.expect_drawn("morgana $");
}

/// §14's warning is said once per machine, and a machine is where the settings
/// live — so a dump, or `ORBS_SAVE=off`, never says it at all.
#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_health_warning_is_said_once_per_machine() {
    if !available() {
        return;
    }
    let kept = Kept::default();
    {
        let game = at_the_menu(&kept);
        game.expect_drawn("if moving light troubles you: settings, access, motion off");
    }
    // It goes up with the menu's first frame, so the menu being drawn without it
    // is the whole claim — there is nothing later to wait for.
    let game = at_the_menu(&kept);
    let screen = game.screen();
    assert!(
        !screen.contains("moving light troubles you"),
        "the warning was said a second time on the same machine:\n{screen}",
    );
}
