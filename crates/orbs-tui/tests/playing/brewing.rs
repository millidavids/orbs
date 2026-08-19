//! The laboratory, played: five instruments, a fire, and the flagship brew.

use crate::play::{Game, available};

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_reagent_goes_into_the_mortar_and_comes_out_ground() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("grind sage", "sage: dispensary to mortar_and_pestle")
        .does("meditate 9", "yields ground-sage");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_athanor_takes_light_and_says_how_long_for() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("kindle charcoal", "charcoal: dispensary to athanor")
        .expect("fuel for 600 ticks");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_fire_already_lit_refuses_a_second_light() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("kindle charcoal", "fuel for 600 ticks")
        .does("kindle charcoal", "the athanor is already burning");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn damping_the_fire_banks_what_is_left_of_it() {
    if !available() {
        return;
    }
    // **The banking, which is the whole reason `stop` is worth typing.** A fresh
    // light is always `600`, so a re-light reporting a *four* digit number is
    // exactly the claim: what was left was kept and added to. Asserting the
    // digits rather than `1188` on the nose keeps it honest about the two or
    // three ticks that pass while the commands are typed.
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("kindle charcoal", "fuel for 600 ticks")
        .meditates(12)
        .does("stop athanor", "ticks of fuel keep")
        .does("kindle charcoal", "the athanor takes light. fuel for 1");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_heated_stage_refuses_a_cold_athanor() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("debug_spawn ground-sage", "all along")
        .does("digest ground-sage", "wants heat, and the athanor is cold");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn an_instrument_at_work_will_not_take_a_second_load() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("grind sage", "to mortar_and_pestle")
        .does("grind rock-salt", "the mortar_and_pestle is at work");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn emptying_an_instrument_returns_the_work_and_the_waste() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("grind sage", "to mortar_and_pestle")
        .does("meditate 9", "yields ground-sage")
        .does("empty mortar_and_pestle", "ground-sage, husks");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn an_empty_instrument_says_so_rather_than_pretending() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory").does(
        "empty mortar_and_pestle",
        "the mortar_and_pestle is already empty",
    );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_shelf_is_not_an_instrument_to_be_emptied() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("empty dispensary", "move things out of it by name");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_orb_will_not_unmake_a_shelf() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("purge dispensary", "will not unmake the dispensary");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn scouring_an_instrument_takes_time_and_then_reports_clean() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("grind sage", "to mortar_and_pestle")
        .does("meditate 9", "yields ground-sage")
        .does("purge mortar_and_pestle", "scouring the mortar_and_pestle")
        .does("meditate 12", "the mortar_and_pestle is clean");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_rail_says_what_the_room_is_doing() {
    if !available() {
        return;
    }
    // The rail is a *different pane* from the transcript, so this is the one
    // assertion that has to look at the whole screen rather than a block.
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("grind sage", "to mortar_and_pestle")
        .expect_drawn("mp");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_bath_takes_from_the_mortar_without_being_told_to() {
    if !available() {
        return;
    }
    // **The fetch, which is what makes the loop `move`-free.** `reachable`
    // walks every unbusy instrument before the shelf, so the origin named here
    // is the *mortar* — not the dispensary the reagent would have been emptied
    // into. That single word is the whole claim.
    let game = Game::start();
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
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("debug_spawn sage-tincture", "all along")
        .does("debug_spawn ground-salt", "all along")
        .does("mix sage-tincture with ground-salt", "to flask_and_rod")
        .does("meditate 12", "yields clarified-draught");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_whole_clarity_brews_by_hand_and_earns_sixteen() {
    if !available() {
        return;
    }
    // **The flagship, end to end, through the keyboard.** `orbs-balance`'s
    // `BY_HAND` proves the same arithmetic in milliseconds against `Sim`; what
    // this adds is that a person typing these fifteen lines into a terminal
    // reaches the same place — every echo, every tick boundary, every refusal
    // that would have interrupted them.
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("kindle charcoal", "fuel for 600 ticks")
        .does("grind sage", "to mortar_and_pestle")
        .does("meditate 9", "yields ground-sage")
        .does("empty mortar_and_pestle", "ground-sage, husks")
        .does("digest ground-sage", "to balneum_mariae")
        .does("meditate 14", "yields sage-tincture")
        .does("grind rock-salt", "to mortar_and_pestle")
        .does("meditate 9", "yields ground-salt")
        .does("empty mortar_and_pestle", "ground-salt, husks")
        .does("mix sage-tincture with ground-salt", "to flask_and_rod")
        .does("meditate 12", "yields clarified-draught")
        .does("distil clarified-draught", "to alembic")
        .does("meditate 60", "yields clarity")
        // The threshold announces itself on the tick the alembic lands, which is
        // the same block — a player learns they can bind at the moment they earn
        // it, not the next time they ask.
        .expect("the orb can hold a spell now")
        .does("status", "experience 16")
        .expect("concentration 1");
}
