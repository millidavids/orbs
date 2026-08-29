//! §8's channel between two spells, typed at a real terminal.
//!
//! # What this layer reaches that the other two do not
//!
//! `orbs-sim`'s `tests/satchel.rs` and `tests/strands.rs` prove the rules
//! against a headless world. What is only true here is that the words are
//! *reachable by typing them* — that `queue` survives the parser at a prompt,
//! that `pull` and `alongside` survive the **editor** (which filters keystrokes
//! and re-indents as you type), and that a spell written by hand rather than by
//! `Sim::write_spell` runs.
//!
//! The editor is the interesting half. A control word arrives here one keystroke
//! at a time through `indent_around`, and `alongside filling()` is the first
//! line in the language that is a word *and* a call — two things the editor
//! indents and highlights by different rules.

use crate::play::{Game, available};

/// The gate, and what it says.
///
/// **`bind`'s shape**: the word works and is not yet available, which is a
/// different thing from a word that is broken. The sentence names the loom
/// rather than the satchel, because a player told *"there is no satchel here"*
/// looks round the room for one.
#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn queueing_before_the_loom_grants_it_says_where_to_go() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend menagerie", "/tower/menagerie")
        .does("queue skyward", "not learned to carry a satchel");
}

/// A satchel loaded by hand and read back, in order and with a repeat.
///
/// **A name twice is the point.** It is why the queue is a component rather than
/// children plus `Stock`, which would collapse the two into `skyward 2` and lose
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
        .does("queue skyward", "1 waiting")
        .does("queue earthward", "2 waiting")
        .does("queue skyward", "3 waiting")
        .does("survey satchel", "queued")
        .expect_drawn("skyward    earthward  skyward");
}

/// A spell written at the keyboard drains what a hand put in.
///
/// The editor is what this reaches: `pull note from satchel` is typed a
/// character at a time, indented by `indent_around` as it goes, and saved by the
/// same settle the Bevy build runs.
#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_spell_typed_at_the_keyboard_drains_the_satchel() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend menagerie", "/tower/menagerie")
        .does("debug_take satchel_1", "holds satchel_1")
        .does("queue skyward", "1 waiting")
        .does("queue earthward", "2 waiting")
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

/// **Both halves of a pipeline in one file**, which is the whole of what
/// `alongside` sells.
///
/// The consumer is cursor 0 and blocks on an empty satchel every tick until the
/// forked producer fills it. Before the strand split, a block ended the whole
/// spell's tick and the producer would never have been reached — so this is the
/// deadlock case, played rather than simulated.
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
    game.type_raw("queue skyward");
    game.type_raw("queue earthward");
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
    game.does("peruse menagerie.log", "skyward goes in the satchel")
        .expect_drawn("earthward goes in the satchel");
    game.does("survey satchel", "holds nothing");
}

/// The editor's guide lists the two new words and shows their shapes.
///
/// **The one surface where a shape is *read* rather than run**, and the place a
/// double bracket showed up: every control word drew `wait <<thing>>` because
/// the view wrapped a shape that already brackets its own slots.
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
