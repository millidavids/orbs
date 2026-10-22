//! What a terminal with no colour, or a player who wants none, still reads.

use crate::launch::{Kept, Switch};
use crate::play::{Game, available};

/// The panel band, with sage in the mortar — green when the hue is there.
///
/// `presentation::` checks this band is not green before the sage goes in, so
/// a green here is the material's tint and not the furniture.
fn sage_in_the_mortar(game: &Game) -> String {
    game.does("attend laboratory", "/tower/laboratory")
        .does("move sage to mortar_and_pestle", "to mortar_and_pestle");
    game.ink(89, 103)
}

/// Whether a decoded band carries any hue at all.
fn hued(ink: &str) -> bool {
    ["red", "green", "yellow", "blue", "magenta", "cyan"]
        .iter()
        .any(|hue| ink.contains(hue))
}

/// A short tower raised from the menu, at its first prompt.
fn raise(game: &Game) {
    game.type_raw("play");
    game.expect_drawn("open a tower");
    game.type_raw("new");
    game.expect_drawn("how long a game?");
    game.type_raw("short");
    game.expect_drawn("wizard $");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn f8_takes_every_hue_out_and_puts_it_back() {
    if !available() {
        return;
    }
    let game = Game::start();
    let coloured = sage_in_the_mortar(&game);
    assert!(
        coloured.contains("green"),
        "no green to take out:\n{coloured}"
    );

    game.press("F8");
    game.wait_ticks(1);
    let grey = game.ink(89, 103);
    assert!(!hued(&grey), "F8 left a hue on screen:\n{grey}");

    game.press("F8");
    game.wait_ticks(1);
    let back = game.ink(89, 103);
    assert!(
        back.contains("green"),
        "the second F8 did not bring it back:\n{back}"
    );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn orbs_sight_greyscale_opens_grey() {
    if !available() {
        return;
    }
    let game = Game::launch().env("ORBS_SIGHT", "greyscale").settled();
    let grey = sage_in_the_mortar(&game);
    assert!(!hued(&grey), "ORBS_SIGHT=greyscale left a hue:\n{grey}");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_sight_row_follows_f8() {
    if !available() {
        return;
    }
    // The key and the row are one setting seen twice.
    let game = Game::start();
    game.press("F8");
    game.send("menu");
    game.expect_drawn("the tower waits");
    game.type_raw("settings");
    game.expect_drawn("what would you set?");
    game.type_raw("access");
    game.expect_drawn("sight hue, or none [greyscale]");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn greyscale_carries_into_a_chosen_tower() {
    if !available() {
        return;
    }
    // A new tower is a new session here, and a new rasteriser with it.
    let game = Game::at_the_threshold();
    game.expect_drawn("no tower yet");
    game.press("F8");
    raise(&game);
    let grey = sage_in_the_mortar(&game);
    assert!(
        !hued(&grey),
        "greyscale was lost on the way into a tower:\n{grey}"
    );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn greyscale_is_remembered() {
    if !available() {
        return;
    }
    let kept = Kept::default();
    {
        let game = Game::launch()
            .at_the_threshold()
            .kept_in(&kept)
            .motion(Switch::BySetting)
            .spawn();
        game.expect_drawn("no tower yet");
        game.press("F8");
        // Written on the keystroke; the threshold runs no clock to wait on.
        std::thread::sleep(std::time::Duration::from_millis(300));
    }
    let game = Game::launch().at_the_threshold().kept_in(&kept).spawn();
    game.expect_drawn("no tower yet");
    raise(&game);
    let grey = sage_in_the_mortar(&game);
    assert!(
        !hued(&grey),
        "greyscale did not survive a relaunch:\n{grey}"
    );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn no_color_keeps_the_weights() {
    if !available() {
        return;
    }
    // `NO_COLOR` makes crossterm write every colour as `ESC[m`, SGR 0. The fire
    // is one ramp of two hues and four weights; with the hues gone the weights
    // are what is left, and they were what that reset used to erase.
    let game = Game::launch().env("NO_COLOR", "1").settled();
    game.does("attend laboratory", "/tower/laboratory")
        .does("kindle charcoal", "fuel for 600 ticks")
        .meditates(3);
    let ink = game.ink(95, 103);
    assert!(
        !ink.contains("yellow"),
        "NO_COLOR was set and the fire still took a colour:\n{ink}",
    );
    assert!(
        ink.contains("/dim") && ink.contains("/bold"),
        "the fire lost its weights with its colour:\n{ink}",
    );
}
