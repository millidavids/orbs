//! The laboratory, played: five instruments, a fire, and the flagship brew.

use crate::play::Game;

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_reagent_goes_into_the_mortar_and_comes_out_ground() {
    let Some(game) = Game::start() else { return };
    game.does("attend laboratory", "/tower/laboratory")
        .does("grind sage", "sage: dispensary to mortar_and_pestle")
        .does("meditate 9", "yields ground-sage");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_athanor_takes_light_and_says_how_long_for() {
    let Some(game) = Game::start() else { return };
    game.does("attend laboratory", "/tower/laboratory")
        .does("kindle charcoal", "charcoal: dispensary to athanor")
        .expect("fuel for 600 ticks");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_fire_already_lit_refuses_a_second_light() {
    let Some(game) = Game::start() else { return };
    game.does("attend laboratory", "/tower/laboratory")
        .does("kindle charcoal", "fuel for 600 ticks")
        .does("kindle charcoal", "the athanor is already burning");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn damping_the_fire_banks_what_is_left_of_it() {
    // The banking, which is the whole reason `stop` is worth typing. A fresh
    // light is always `600`, so a re-light reporting four digits is the claim:
    // what was left was kept and added to. Asserting the digits rather than
    // `1188` keeps it honest about the ticks spent typing the commands.
    let Some(game) = Game::start() else { return };
    game.does("attend laboratory", "/tower/laboratory")
        .does("kindle charcoal", "fuel for 600 ticks")
        .meditates(12)
        .does("stop athanor", "ticks of fuel keep")
        .does("kindle charcoal", "the athanor takes light. fuel for 1");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_heated_stage_refuses_a_cold_athanor() {
    let Some(game) = Game::start() else { return };
    game.does("attend laboratory", "/tower/laboratory")
        .does("debug_spawn ground-sage", "all along")
        .does("digest ground-sage", "wants heat, and the athanor is cold");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn an_instrument_at_work_will_not_take_a_second_load() {
    let Some(game) = Game::start() else { return };
    game.does("attend laboratory", "/tower/laboratory")
        .does("grind sage", "to mortar_and_pestle")
        .does("grind rock-salt", "the mortar_and_pestle is at work");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn emptying_an_instrument_returns_the_work_and_the_waste() {
    let Some(game) = Game::start() else { return };
    game.does("attend laboratory", "/tower/laboratory")
        .does("grind sage", "to mortar_and_pestle")
        .does("meditate 9", "yields ground-sage")
        .does("empty mortar_and_pestle", "ground-sage, husks");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn what_an_instrument_or_a_shelf_will_not_do_it_says() {
    // None of these changes the world, so one game asks them all.
    let Some(game) = Game::start() else { return };
    game.does("attend laboratory", "/tower/laboratory");
    for (line, refusal) in [
        (
            "empty mortar_and_pestle",
            "the mortar_and_pestle is already empty",
        ),
        ("empty dispensary", "move things out of it by name"),
        ("purge dispensary", "will not unmake the dispensary"),
    ] {
        game.does(line, refusal);
    }
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn scouring_an_instrument_takes_time_and_then_reports_clean() {
    let Some(game) = Game::start() else { return };
    game.does("attend laboratory", "/tower/laboratory")
        .does("grind sage", "to mortar_and_pestle")
        .does("meditate 9", "yields ground-sage")
        .does("purge mortar_and_pestle", "scouring the mortar_and_pestle")
        .does("meditate 12", "the mortar_and_pestle is clean");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_rail_says_what_the_room_is_doing() {
    // The rail is a *different pane* from the transcript, so this is the one
    // assertion that has to look at the whole screen rather than a block.
    let Some(game) = Game::start() else { return };
    game.does("attend laboratory", "/tower/laboratory")
        .does("grind sage", "to mortar_and_pestle")
        .expect_drawn("mp");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_bath_takes_from_the_mortar_without_being_told_to() {
    // The fetch, which is what makes the loop `move`-free: `reachable` walks
    // every unbusy instrument before the shelf, so the origin named here is the
    // mortar, not the dispensary the reagent would have been emptied into. That
    // single word is the whole claim.
    let Some(game) = Game::start() else { return };
    game.does("attend laboratory", "/tower/laboratory")
        .does("kindle charcoal", "fuel for 600 ticks")
        .does("grind sage", "to mortar_and_pestle")
        .does("meditate 9", "yields ground-sage")
        .does(
            "digest ground-sage",
            "ground-sage: mortar_and_pestle to balneum_mariae",
        );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn mixing_names_both_ingredients_arriving() {
    let Some(game) = Game::start() else { return };
    game.does("attend laboratory", "/tower/laboratory")
        .does("debug_spawn sage-tincture", "all along")
        .does("debug_spawn ground-salt", "all along")
        .does("mix sage-tincture with ground-salt", "to flask_and_rod")
        .does("meditate 12", "yields clarified-draught");
}
