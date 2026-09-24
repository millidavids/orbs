//! The keys that are neither the prompt nor a surface, and the ways out.

use crate::play::{Game, available};

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn f4_flips_the_focus_the_border_advertises() {
    if !available() {
        return;
    }
    // `prompt.rs` draws `F4 deep` into the border every frame and this build
    // shipped without answering it. Until multiplexing returns the second pane
    // in Phase 9a, moving the hint is the whole claim.
    let game = Game::start();
    game.expect_drawn("F4 deep");
    game.press("F4");
    game.expect_drawn("F4 wide");
    game.press("F4");
    game.expect_drawn("F4 deep");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn f5_shows_the_linear_stream_the_screen_reader_would_hear() {
    if !available() {
        return;
    }
    // Rule 5's linear stream: structured — `name:`, `qty:`, `state:` — where
    // the pane is prose.
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("survey dispensary", "sage");
    game.press("F5");
    game.expect_drawn("name:");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn f6_writes_the_parse_trace_where_the_game_is_running() {
    if !available() {
        return;
    }
    // `F6` says nothing on the transcript and the register `F7` cycles is
    // invisible on screen but is a column in the file, so this is the only way
    // to see either key is bound.
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("grind sage", "to mortar_and_pestle");
    game.press("F7");
    game.press("F6");
    let trace = game.wrote("orbs-parse.tsv");
    assert!(
        trace.contains("grind"),
        "the trace was written but holds nothing that was typed:\n{trace}",
    );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn quit_asks_before_it_ends_the_session() {
    if !available() {
        return;
    }
    // It asks because a `quit` meant for the spell editor and typed one surface
    // too high would end the game instead of closing a buffer. Each half lands
    // at the tick rather than at `submit` — the same beat every other word has.
    let game = Game::start();
    game.send("quit");
    game.expect_drawn("leave the orb?");

    // Anything else answers no, and the question is gone.
    game.does("status", "experience");
    game.send("quit");
    game.expect_drawn("leave the orb?");

    // ...and the second one goes.
    game.send("quit");
    game.expect_gone();
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn menu_opens_the_orbs_own_screen_and_resume_comes_back() {
    if !available() {
        return;
    }
    // `menu` is its own word, not `quit` (§19). Also the only thing that would
    // have caught the menu's shipping defect: a gated `type_into_menu` kept its
    // cursor and typed the opening word back into the menu, while `ORBS_DUMP`
    // presses no key and drew it perfectly throughout.
    let game = Game::start();
    game.send("menu");
    game.expect_drawn("the tower waits");
    game.expect_drawn("resume");

    // Nothing of `menu` was typed into it: the line under `>` is empty, which
    // is what `shell/plugin.rs`'s ordering buys.
    game.expect_drawn("what now?");

    // `send` waits for the prompt to echo; the menu has no prompt, so the way
    // back goes through `type_raw` and is waited for by what it draws.
    game.type_raw("resume");
    game.expect_drawn("/tower");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn f10_leaves_from_anywhere() {
    if !available() {
        return;
    }
    // Not Escape — all four surfaces use it to leave something *smaller*. F10
    // matches the Bevy build, so a player who learns one build's exit is not
    // trapped in the other.
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .opens("scribe morning", "morning.spell in ");
    game.press("F10");
    game.expect_gone();
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn ctrl_c_is_answered_rather_than_ignored() {
    if !available() {
        return;
    }
    // Raw mode means no SIGINT, so it is ours to answer — and a full-screen
    // program nobody can leave by the chord everybody tries is a trap.
    let game = Game::start();
    game.press("C-c");
    game.expect_gone();
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn ctrl_d_leaves_too() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.press("C-d");
    game.expect_gone();
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn every_other_chord_is_swallowed_rather_than_typed() {
    if !available() {
        return;
    }
    // A chord that reached the line would put a stray character in a command.
    let game = Game::start();
    game.press("C-a");
    game.press("C-k");
    game.wait_ticks(2);
    assert!(game.alive(), "a harmless chord ended the session");
    game.does("status", "experience 0");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_manual_opens_on_the_room_and_then_the_words() {
    if !available() {
        return;
    }
    // `help` is bare `recall`: the room's primer before twenty-five verbs,
    // because a player who types it in the lens is asking what a ward is. A
    // room lists only its own operations — `research`, `follow` and `wander`
    // used to be offered in the laboratory.
    let game = Game::start();
    game.does("attend lens", "/tower/lens")
        .does("help", "four sigils")
        .expect("probe");
    assert!(
        !game.last_block().contains("wander"),
        "the lens offered the archive's words:\n{}",
        game.screen(),
    );
}
