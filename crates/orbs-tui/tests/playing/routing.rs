//! Which surface a keystroke reaches — the thing only a real keyboard can ask.
//!
//! `surfaces.rs`'s own unit tests prove the *rule*: for every combination of
//! open surfaces, exactly one owner, decided rather than accidental. What they
//! cannot prove is the **dispatch** — that a real arrow key, arriving through
//! crossterm from a real terminal, is delivered to that owner and to nobody
//! else. Two surfaces consuming one keystroke is invisible until a player types
//! `:wq` and finds it in their command history.

use crate::play::{Game, available};

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn arrows_walk_the_maze_and_never_reach_the_prompt() {
    if !available() {
        return;
    }
    // At the prompt an arrow means *recall history*. Inside `wander` it means
    // *walk*, and the prompt must see none of it.
    let game = Game::seeded(11);
    game.does("attend archive", "/tower/archive")
        .does("research", "a way out is in them")
        .opens("wander", "arrows walk")
        .expect_drawn("arrows walk");
    for key in ["Up", "Up", "Left"] {
        game.press(key);
    }
    game.press("Escape");
    game.expect_drawn("wizard $");
    // Whatever the arrows did, they did not put a recalled line in the prompt.
    let live = game.screen();
    let prompt = live
        .lines()
        .rfind(|row| !row.starts_with('│'))
        .unwrap_or_default()
        .trim()
        .to_owned();
    assert_eq!(
        prompt, "wizard $",
        "an arrow that belonged to the maze reached the prompt:\n{live}",
    );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn walking_the_maze_spends_no_world_time() {
    if !available() {
        return;
    }
    // **`Sim::walk` is the third entry point**: it reaches the world without a
    // tick boundary, so a player walks as fast as they can press and no brew
    // advances while they do. Eight presses must not be eight seconds.
    let game = Game::seeded(11);
    game.does("attend archive", "/tower/archive")
        .does("research", "a way out is in them")
        .opens("wander", "arrows walk");
    let before = game.tick();
    for key in ["Up", "Down", "Up", "Down", "Up", "Down", "Up", "Down"] {
        game.press(key);
    }
    let after = game.tick();
    assert!(
        after - before <= 1,
        "eight arrow presses advanced the world {} ticks; a walk should cost none",
        after - before,
    );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_map_keeps_up_with_the_keys_rather_than_the_clock() {
    if !available() {
        return;
    }
    // **A regression lock on a defect this suite was written to find.** `Panel`
    // is rebuilt once a tick, on the stated ground that the world moves at 1 Hz
    // — and `walk` is the one thing that moves it between ticks. The map is
    // drawn from `panel.stacks`, so every arrow was invisible for up to a
    // second: three rapid presses moved the reading three cells and showed none
    // of it until the tick. The Bevy build never had this, because `walk` takes
    // `Tower` by `&mut` and `refresh_panel` hangs on `resource_changed`.
    let game = Game::seeded(11);
    game.does("attend archive", "/tower/archive")
        .does("research", "a way out is in them")
        .opens("wander", "arrows walk");
    let before = reading(&game);
    game.press("Up");
    // A frame is 33ms. Half a tick is an eternity beside that, and still nowhere
    // near the second the bug cost.
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(400);
    while std::time::Instant::now() < deadline {
        if reading(&game) != before {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    panic!(
        "the map did not follow the arrow within 400ms:\n{}",
        game.screen()
    );
}

/// Where `☼`, the reading, is on screen.
fn reading(game: &Game) -> Option<(usize, usize)> {
    game.screen()
        .lines()
        .enumerate()
        .find_map(|(row, line)| line.chars().position(|g| g == '☼').map(|col| (row, col)))
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn leaving_the_maze_gives_the_keyboard_back() {
    if !available() {
        return;
    }
    let game = Game::seeded(11);
    game.does("attend archive", "/tower/archive")
        .does("research", "a way out is in them")
        .opens("wander", "arrows walk")
        .expect_drawn("esc leaves");
    game.press("Escape");
    game.does("status", "experience 0");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_weave_screen_takes_the_arrows_only_after_a_word() {
    if !available() {
        return;
    }
    // **The arrows do nothing until a word has gone into a track.** `ley` or
    // `mastery` is what hands them over, the way `edit` drops into the editor's
    // buffer — so pressing an arrow first is testing the refusal.
    let game = Game::start();
    game.opens("weave", "say ley or mastery")
        .expect_drawn("say ley or mastery");
    game.press("Down");
    game.expect_drawn("say ley or mastery");
    game.type_raw("mastery");
    game.expect_drawn("arrows move");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn escape_inside_the_weave_goes_back_a_step_and_not_out() {
    if !available() {
        return;
    }
    // **A surface that swallows the prompt**, found by playing rather than by
    // reading: one Escape returns from browsing to aiming, and a player who
    // assumes it left types their next command into the weave, which answers
    // *"the orb weaves nothing called scribe morning"*. You must type `quit`.
    let game = Game::start();
    game.opens("weave", "say ley or mastery");
    game.type_raw("mastery");
    game.expect_drawn("arrows move");
    game.press("Escape");
    // Back to aiming — the footer offers the two tracks again. **`say ley or
    // mastery` is the *details* panel and only greets you once**, so it is the
    // footer that says which state this is.
    game.expect_drawn("ley  mastery  take  quit");
    game.type_raw("quit");
    game.does("status", "experience 0");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn typing_while_the_weave_is_open_goes_to_the_weave() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.opens("weave", "say ley or mastery");
    game.type_raw("scribe morning");
    game.expect_drawn("weaves nothing called");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_function_key_works_from_inside_a_surface() {
    if !available() {
        return;
    }
    // The function keys are ungated by surface, exactly as they are in the Bevy
    // build, where they are `input_just_pressed` in `Update` with no run
    // condition.
    //
    // **The proof has to come after the editor closes**, and that is a fact
    // about the screen rather than about the key: the linear stream is drawn
    // *instead of* the transcript, and while the editor owns the whole pane
    // there is no transcript for it to replace. So `F5` pressed inside the
    // editor is invisible until you leave — and it being on when you do is
    // exactly the claim that the editor did not swallow it.
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("survey dispensary", "sage")
        .opens("scribe morning", "morning.spell in ");
    game.press("F5");
    game.type_raw("quit");
    game.expect_drawn("name:");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn unfurl_pages_back_the_moment_it_is_typed() {
    if !available() {
        return;
    }
    // **A word that only took the keyboard did nothing visible.** `unfurl` used
    // to call `scroll.read()` and stop, so the pane showed the screen the player
    // was already looking at and the only thing that changed was the border
    // hint. The Bevy build's `start_reading` pages by one screen on open for
    // exactly that reason.
    let game = Game::start();
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
    if !available() {
        return;
    }
    // The Bevy build binds `scroll_back`/`scroll_forward` to Up/Down gated on
    // `editing::reading`; this build had only the page keys, so a reader could
    // jump a screenful but not nudge two entries into line beside each other.
    let game = Game::start();
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
fn the_transcript_pages_from_behind_an_open_surface() {
    if !available() {
        return;
    }
    // **The editor swallowed both page keys.** `Session::typed` handed every
    // keystroke to the surface and returned, so the global arms below were
    // unreachable whenever anything was open — while a comment twenty lines
    // under them claimed parity with the Bevy build, where `scroll_back` sits in
    // `Update` gated only on `booted` and pages happily behind the editor.
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("help", "the work")
        .does("recall clarity", "steps")
        .opens("scribe morning", "morning.spell in ");
    let before = game.screen();
    game.press("PageUp");
    moved(&game, &before, "PageUp was swallowed by the editor");
}

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
fn the_transcript_scrolls_without_asking_for_permission() {
    if !available() {
        return;
    }
    // **`unfurl` exists because the *key* could not be discovered**, not because
    // the key needs permission — the border only advertises `PgDn newest` once
    // you are already scrolled back. So PageUp works at the bare prompt.
    let game = Game::start();
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
