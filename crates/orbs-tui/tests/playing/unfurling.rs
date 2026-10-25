//! Reading back through the transcript: `unfurl`, the page keys, and the keys
//! that step it — one table with the window since `0.19.13`.

use crate::play::Game;

/// Wait for the screen to change, or fail with it.
fn moved(game: &Game, before: &str, complaint: &str) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while std::time::Instant::now() < deadline {
        if game.screen() != before && game.pane() != before {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!("{complaint}:\n{}", game.screen());
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn unfurl_pages_back_the_moment_it_is_typed() {
    // `unfurl` used to call `scroll.read()` and stop, so the pane showed the
    // screen the player was already looking at and only the border hint
    // changed. The Bevy build's `start_reading` pages by one screen on open.
    let Some(game) = Game::start() else { return };
    game.does("attend laboratory", "/tower/laboratory")
        .does("help", "the work")
        .does("recall clarity", "steps")
        .does("survey dispensary", "sage");
    let before = game.pane();
    game.send("unfurl");
    moved(
        &game,
        &before,
        "unfurl took the keyboard and showed the same screen",
    );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_arrows_step_the_transcript_one_record_at_a_time() {
    // A reader can jump a screenful with the page keys or nudge two entries
    // into line beside each other with the arrows — both builds, one table.
    let Some(game) = Game::start() else { return };
    game.does("attend laboratory", "/tower/laboratory")
        .does("help", "the work")
        .does("recall clarity", "steps");
    game.send("unfurl");
    let before = game.pane();
    game.press("Up");
    moved(&game, &before, "an arrow inside `unfurl` stepped nothing");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn home_and_end_reach_the_ends() {
    // Enough answer that the start of the session is off the screen.
    let Some(game) = Game::start() else { return };
    game.does("attend laboratory", "/tower/laboratory");
    for _ in 0..4 {
        game.does("recall clarity", "steps");
    }
    game.expect_off_screen("cold start");
    game.send("unfurl");
    game.press("Home");
    game.expect_drawn("cold start");
    game.press("End");
    game.expect_off_screen("cold start");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_transcript_pages_from_behind_an_open_surface() {
    // The editor swallowed both page keys: `Session::typed` handed every
    // keystroke to the surface and returned, so the global arms below were
    // unreachable whenever anything was open. The Bevy build's `scroll_back` is
    // gated only on `booted` and pages happily behind the editor.
    let Some(game) = Game::start() else { return };
    game.does("attend laboratory", "/tower/laboratory")
        .does("help", "the work")
        .does("recall clarity", "steps")
        .opens("scribe morning", "morning.spell in ");
    let before = game.screen();
    game.press("PageUp");
    moved(&game, &before, "PageUp was swallowed by the editor");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_transcript_scrolls_without_asking_for_permission() {
    // `unfurl` exists because the key could not be discovered, not because it
    // needs permission — so PageUp works at the bare prompt.
    let Some(game) = Game::start() else { return };
    game.does("attend laboratory", "/tower/laboratory")
        .does("help", "the work")
        .does("recall clarity", "clarity");
    let before = game.pane();
    game.press("PageUp");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    while std::time::Instant::now() < deadline {
        if game.pane() != before {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!("PageUp at the prompt scrolled nothing:\n{}", game.screen());
}
