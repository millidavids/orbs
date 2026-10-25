//! Which surface a keystroke reaches — the thing only a real keyboard can ask.
//!
//! `surfaces`'s unit tests prove the rule: for every combination of open
//! surfaces, exactly one owner, decided rather than accidental. Only a real
//! arrow key arriving through crossterm proves the dispatch. Two surfaces
//! consuming one keystroke is invisible until a player types `:wq` and finds it
//! in their command history.

use crate::play::Game;

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn arrows_walk_the_maze_and_never_reach_the_prompt() {
    // At the prompt an arrow means *recall history*. Inside `wander` it means
    // *walk*, and the prompt must see none of it.
    let Some(game) = Game::seeded(11) else { return };
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
    // `Sim::walk` reaches the world without a tick boundary, so a player walks
    // as fast as they can press. Eight presses must not be eight seconds.
    let Some(game) = Game::seeded(11) else { return };
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
    // `Panel` is rebuilt once a tick and the map is drawn from `panel.stacks`,
    // so every arrow was invisible for up to a second — and `walk` is the one
    // thing that moves the world between ticks. The Bevy build never had this:
    // `walk` takes `Tower` by `&mut` and `refresh_panel` hangs on
    // `resource_changed`.
    let Some(game) = Game::seeded(11) else { return };
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
    let Some(game) = Game::seeded(11) else { return };
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
    // `ley` or `mastery` hands the arrows over, the way `edit` drops into the
    // editor's buffer — so pressing an arrow first is testing the refusal.
    let Some(game) = Game::start() else { return };
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
    // One Escape returns from browsing to aiming, not out — a player who
    // assumes it left types their next command into the weave, which answers
    // *"the orb weaves nothing called scribe morning"*. You must type `quit`.
    let Some(game) = Game::start() else { return };
    game.opens("weave", "say ley or mastery");
    game.type_raw("mastery");
    game.expect_drawn("arrows move");
    game.press("Escape");
    // Back to aiming. `say ley or mastery` is the details panel and greets you
    // once, so the footer is what says which state this is.
    game.expect_drawn("ley  mastery  take  quit");
    game.type_raw("quit");
    game.does("status", "experience 0");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn typing_while_the_weave_is_open_goes_to_the_weave() {
    let Some(game) = Game::start() else { return };
    game.opens("weave", "say ley or mastery");
    game.type_raw("scribe morning");
    game.expect_drawn("weaves nothing called");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_function_key_works_from_inside_a_surface() {
    // The function keys are ungated by surface, as in the Bevy build.
    //
    // The proof comes after the editor closes, which is a fact about the screen
    // rather than the key: the linear stream is drawn instead of the transcript,
    // and the editor owns the whole pane. `F5` being on when you leave is the
    // claim that the editor did not swallow it.
    let Some(game) = Game::start() else { return };
    game.does("attend laboratory", "/tower/laboratory")
        .does("survey dispensary", "sage")
        .opens("scribe morning", "morning.spell in ");
    game.press("F5");
    game.type_raw("quit");
    game.expect_drawn("name:");
}

/// The manual pages, and the transcript behind it does not.
///
/// `PgUp`/`PgDn` scroll the transcript from behind every other surface, so the
/// manual — the first surface that wants those keys for itself — needs a guard
/// or one press moves both. Neither build had it.
///
/// A dump cannot ask this: it presses no keys and paints once.
#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_manual_pages_and_the_transcript_behind_it_does_not() {
    let Some(game) = Game::start() else { return };
    // A transcript with more in it than fits, so it has somewhere to scroll to.
    game.does("attend laboratory", "/tower/laboratory")
        .does("help", "the work")
        .does("recall clarity", "clarity");

    game.send("menu");
    game.expect_drawn("resume");
    game.type_raw("manual");
    game.expect_drawn("which part?");
    // A chapter taller than the pane, which `keys` is not: it fits whole, so
    // `Reader::scroll` clamps to nothing and paging it is correctly a no-op.
    game.type_raw("spells");
    // A line only the chapter has. Waiting on one the contents page also draws
    // returns before the chapter is up, and everything after it races.
    let opening = "A spell is a file of the same lines";
    game.expect_drawn(opening);
    game.expect_drawn("pgdn for more");

    // Asserted on the chapter's own first line: waiting for any screen
    // difference passes on the 1 Hz clock alone, which it did for a whole
    // version while neither frontend's key mapper produced `Key::PageDown`.
    game.press("PageDown");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    while std::time::Instant::now() < deadline && game.screen().contains(opening) {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(
        !game.screen().contains(opening),
        "PageDown did not page the manual — its first line is still on screen:\n{}",
        game.screen(),
    );

    // ...and PageUp brings it back, which also says the scroll is clamped
    // rather than running off.
    game.press("PageUp");
    game.press("PageUp");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    while std::time::Instant::now() < deadline && !game.screen().contains(opening) {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(
        game.screen().contains(opening),
        "PageUp did not come back to the top of the chapter:\n{}",
        game.screen(),
    );

    // Out again, and the transcript is where it was — at the newest end.
    game.press("Escape");
    game.expect_drawn("which part?");
    game.press("Escape");
    game.expect_drawn("resume");
    game.type_raw("resume");
    game.expect_drawn("wizard $");
    let live = game.screen();
    assert!(
        !live.contains("PgDn newest"),
        "paging the manual scrolled the transcript behind it:\n{live}",
    );
}

/// The front door, played: a menu, a choice, and a tower behind it.
///
/// The only scenario without `ORBS_THRESHOLD=0`, and a dump builds no `App`:
/// this alone opens the game as a player does.
#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_threshold_reaches_a_tower() {
    let Some(game) = Game::at_the_threshold() else {
        return;
    };
    // The menu, not a prompt: the orb is awake and has raised nothing.
    game.expect_drawn("no tower yet");
    // `resume` is refused rather than silently ignored (§6), and the menu is
    // still there afterwards — there is nothing behind it to go back to.
    //
    // `type_raw` sends Enter itself; a second one is an empty line, which
    // `Menu::enter` answers by clearing the complaint the assertion reads.
    game.type_raw("resume");
    game.expect_drawn("does not know");
    // Escape is the way out of every other surface in the game and is not one
    // here, for the same reason.
    game.press("Escape");
    game.expect_drawn("no tower yet");

    // ...and the way through is a tower. `ORBS_SAVE=off` is set for every
    // scenario, so nothing is kept and `play` can only offer to raise one.
    game.type_raw("play");
    game.expect_drawn("open a tower");
    game.type_raw("new");
    game.expect_drawn("how long a game?");
    game.type_raw("short");

    // A prompt, a room, and a world that is running — which is the whole claim.
    game.expect_drawn("wizard $");
    game.does("attend laboratory", "/tower/laboratory");
    let before = game.tick();
    game.wait_ticks(2);
    assert!(
        game.tick() > before,
        "the clock never started after the threshold was crossed:\n{}",
        game.screen(),
    );
}
