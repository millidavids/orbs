//! A word a command cannot use is named, and a word the orb cannot place is
//! asked about — never dropped, and never run around.

use crate::play::Game;

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_word_a_verb_cannot_use_is_named() {
    let Some(game) = Game::start() else { return };
    game.does(
        "status gibberish",
        "status takes only rates - 'gibberish' is not one of them",
    )
    .does("verify gibberish", "¿ verify");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_word_the_orb_cannot_place_is_asked_about() {
    let Some(game) = Game::start() else { return };
    game.does("attend menagerie", "/tower/menagerie")
        .does("summon", "gathers at the circle")
        // The verb and the gate it could place, and nothing run.
        .does("limn keystone xyzzy", "¿ limn keystone")
        .expect_absent("is limned")
        .does("dial", "there is no first here to dial with");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_spell_walks_every_store() {
    // `for each store` with `has spent`: a nameable word no `has` can reach
    // would compile, run and do nothing for ever. A quaff of each spent store is
    // refused by name in the log, which is the proof the loop reached them all —
    // a survey from a spell lands in no log at all.
    let Some(game) = Game::start() else { return };
    game.does("attend bailey", "/tower/bailey")
        .does("debug_spawn warding 1", "all along")
        .does("debug_spawn clarity 1", "all along");
    game.meditates(3000);
    game.does("defend", "come up the road")
        .opens("scribe keeping", "keeping.spell in ");
    game.type_raw("edit");
    for line in [
        "for each store",
        "if the store has spent",
        "quaff store",
        "end",
        "end",
    ] {
        game.type_raw(line);
    }
    game.press("Escape");
    game.closes_editor();
    game.does("invoke keeping", "the orb takes up keeping.spell");
    game.wait_ticks(4);
    game.does("peruse bailey.log", "your warding stores are out")
        .expect("your clarity stores are out");
}
