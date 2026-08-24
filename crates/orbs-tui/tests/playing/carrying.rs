//! Where things are, and how they get somewhere else.

use crate::play::{Game, available};

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn move_names_the_origin_it_took_from() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory").does(
        "move sage to mortar_and_pestle",
        "sage: dispensary to mortar_and_pestle",
    );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_name_nothing_answers_to_is_answered_with_names_that_do() {
    if !available() {
        return;
    }
    // **This asserted `"zzz"` and passed on its own argument.** The needle was a
    // substring of the line being typed, and the block it was searched in began
    // with that line — so the wait was satisfied by the echo and the refusal was
    // never read. It is the reason `block` drops its header.
    //
    // What actually happens is better than a refusal: the parser answers an
    // unplaceable noun with numbered suggestions drawn from what is really on
    // the shelf, so the reply names three reagents the room has.
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("move zzz to mortar_and_pestle", "»")
        .expect("sage");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_shelf_holds_its_base_stock_endlessly() {
    if !available() {
        return;
    }
    // `∞` is literal, and it is the whole difference between a herb and a
    // thing you can run out of.
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("survey dispensary", "∞");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn spawned_stock_lands_in_the_room_that_makes_it() {
    if !available() {
        return;
    }
    // **`tower::home` is a rule over the content, not a list**, so this asks
    // for three things from one room and checks each went somewhere different.
    let game = Game::start();
    game.does("attend archive", "/tower/archive")
        .does("debug_spawn fragment 4", "all along")
        .does("survey cabinet", "fragment = 4");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn finished_work_keeps_itself_wherever_it_was_asked_for() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend archive", "/tower/archive")
        .does("debug_spawn clarity", "all along")
        .does("survey arsenal", "clarity");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_reagent_goes_home_to_the_room_that_consumes_it() {
    if !available() {
        return;
    }
    // Asked for from the archive; it belongs to the laboratory, and the
    // archive's own shelf must not have taken it.
    let game = Game::start();
    game.does("attend archive", "/tower/archive")
        .does("debug_spawn sage", "all along")
        .does("attend laboratory", "/tower/laboratory")
        .does("survey dispensary", "sage");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_tool_refuses_a_name_the_tower_does_not_have() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("debug_spawn zzz", "nothing in the tower is called");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_way_is_a_fixture_and_not_a_shelf() {
    if !available() {
        return;
    }
    // **The one that surprises.** A `way` carries `Fixture` so the maze can
    // publish readings into it, and `research::refresh` despawns everything in
    // one on the step after — so a reagent put there would be a pile that
    // vanishes with no line saying so.
    let game = Game::start();
    game.does("attend archive", "/tower/archive")
        .does("debug_spawn sage 1 north", "there is no shelf here");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_arsenal_takes_finished_work_and_says_so_when_it_will_not() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("move sage to arsenal", "the arsenal keeps finished work");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_potion_carries_from_the_room_that_made_it_to_every_other() {
    if !available() {
        return;
    }
    // **The whole reason the arsenal exists** (§19): before it, a potion made
    // in the laboratory could not leave the laboratory, because `move` wants a
    // fixture where you are standing. Brewed here rather than spawned, because
    // spawning puts it in the arsenal already and would test nothing.
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("kindle charcoal", "fuel for 600 ticks")
        .does("debug_spawn clarified-draught", "all along")
        .does("distil clarified-draught", "to alembic")
        .does("meditate 60", "yields clarity")
        .does("empty alembic", "clarity")
        .does("move clarity to arsenal", "clarity: dispensary to arsenal")
        .does("attend archive", "/tower/archive")
        .does("survey arsenal", "clarity");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_kept_potion_can_be_named_from_any_room() {
    if !available() {
        return;
    }
    // **Nameable is not enough.** `purge` and `verify` take `NounKind::Any`, so
    // they can name a potion from anywhere — and a verb that names what it
    // cannot reach says *"there is no clarity within reach"*, which is the one
    // answer that is false.
    let game = Game::start();
    game.does("attend archive", "/tower/archive")
        .does("debug_spawn clarity", "all along")
        .send("verify clarity");
    assert!(
        !game.last_block().contains("within reach"),
        "the arsenal held it and `verify` said it could not be reached:\n{}",
        game.screen(),
    );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn purging_a_kept_potion_actually_empties_the_arsenal() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend archive", "/tower/archive")
        .does("debug_spawn clarity", "all along")
        .does("survey arsenal", "clarity")
        .does("purge clarity", "clarity")
        // **A tick between the purge and the look, or this is a race.** A
        // `purge` queues its despawn for the *next* tick like every other
        // effect, and `does` waits only for the echo — which the prompt prints
        // at once. Two lines can therefore land inside one tick, and the survey
        // then reads a world the purge has not been applied to yet. It passed
        // for as long as the screen happened to be slow enough.
        .wait_ticks(2)
        .does("survey arsenal", "arsenal");
    assert!(
        !game.last_block().contains("clarity"),
        "purge resolved and left the potion where it was:\n{}",
        game.screen(),
    );
}
