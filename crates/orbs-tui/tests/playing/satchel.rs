//! §8's channel between two spells, typed at a real terminal.
//!
//! `orbs-sim`'s `tests/satchel.rs` and `tests/strands.rs` prove the rules
//! against a headless world. What is only true here is that the words are
//! *reachable by typing them* — through the editor, which filters keystrokes and
//! re-indents as you type.
//!
//! The editor is the interesting half: `alongside filling()` is the first line
//! in the language that is a word *and* a call, two things the editor indents
//! and highlights by different rules.

use crate::play::{Game, available};

/// The gate, and what it says.
///
/// `bind`'s shape: the word works and is not yet available, which is different
/// from a word that is broken. The sentence names the loom rather than the
/// satchel, because a player told *"there is no satchel here"* looks for one.
#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn queueing_before_the_loom_grants_it_says_where_to_go() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend menagerie", "/tower/menagerie")
        .does("queue heed", "not learned to carry a satchel");
}

/// A satchel loaded by hand and read back, in order and with a repeat.
///
/// A name twice is the point, and why the queue is a component rather than
/// children plus `Stock`, which would collapse the two into `heed 2` and lose
/// which came first.
#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_hand_loaded_satchel_reads_back_as_a_queue() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend menagerie", "/tower/menagerie")
        .does("debug_take satchel_1", "holds satchel_1")
        .does("queue heed", "1 waiting")
        .does("queue yoke", "2 waiting")
        .does("queue heed", "3 waiting")
        .does("survey satchel", "queued")
        .expect_drawn("heed  yoke  heed");
}

/// A spell written at the keyboard drains what a hand put in.
///
/// The editor is what this reaches: `pull note from satchel` is typed a
/// character at a time, indented by `indent_around`, and saved by the same
/// settle the Bevy build runs.
#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_spell_typed_at_the_keyboard_drains_the_satchel() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend menagerie", "/tower/menagerie")
        .does("debug_take satchel_1", "holds satchel_1")
        .does("queue heed", "1 waiting")
        .does("queue yoke", "2 waiting")
        .opens("scribe drain", "drain.spell in ");
    game.type_raw("edit");
    game.type_raw("repeat 2");
    game.type_raw("pull note from satchel");
    game.type_raw("survey note");
    game.type_raw("end");
    game.press("Escape");
    game.closes_editor();
    game.does("invoke drain", "takes up drain.spell");
    game.meditates(12);
    game.does("survey satchel", "holds nothing");
}

/// Both halves of a pipeline in one file, which is what `alongside` sells.
///
/// The consumer is cursor 0 and blocks on an empty satchel every tick until the
/// forked producer fills it. Before the strand split, a block ended the whole
/// spell's tick and the producer would never have been reached — the deadlock
/// case, played rather than simulated.
#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_forked_spell_feeds_itself() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend menagerie", "/tower/menagerie")
        .does("debug_take satchel_1", "holds satchel_1")
        .does("debug_take cursors_1", "holds cursors_1")
        .opens("scribe both", "both.spell in ");
    game.type_raw("edit");
    game.type_raw("part filling()");
    game.type_raw("queue heed");
    game.type_raw("queue yoke");
    game.type_raw("end");
    game.type_raw("alongside filling()");
    game.type_raw("repeat 2");
    game.type_raw("pull note from satchel");
    game.type_raw("survey note");
    game.type_raw("end");
    game.press("Escape");
    game.closes_editor();
    game.does("invoke both", "takes up both.spell");
    game.meditates(16);
    // The producer put both in and the consumer took both out, with nothing
    // between them but the node.
    game.does("peruse menagerie.log", "heed goes in the satchel")
        .expect_drawn("yoke goes in the satchel");
    game.does("survey satchel", "holds nothing");
}

/// The editor's guide lists the two new words and shows their shapes.
///
/// The one surface where a shape is *read* rather than run, and where a double
/// bracket showed up: every control word drew `wait <<thing>>` because the view
/// wrapped a shape that already brackets its own slots.
#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_guide_offers_the_channels_words() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend menagerie", "/tower/menagerie")
        .opens("scribe guided", "guided.spell in ");
    game.type_raw("edit");
    game.expect_drawn("pull").expect_drawn("alongside");
}
