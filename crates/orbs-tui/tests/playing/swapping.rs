//! A swap replaces the tower and nothing else: not another tower's slot, not
//! the tower being left, and not the process's own threads.

use crate::launch::{Kept, Switch};
use crate::play::{Game, available};

/// The orb's menu, in a game that keeps its towers in `kept`.
fn at_the_menu(kept: &Kept) -> Game {
    let game = Game::launch().at_the_threshold().kept_in(kept).spawn();
    game.expect_drawn("no tower yet");
    game
}

/// From the play page, a short tower, at its first prompt.
fn raise(game: &Game) {
    game.type_raw("play");
    game.expect_drawn("open a tower");
    game.type_raw("new");
    game.expect_drawn("how long a game?");
    game.type_raw("short");
    game.expect_drawn("wizard $");
}

/// The menu, from inside a tower.
fn menu(game: &Game) {
    game.send("menu");
    game.expect_drawn("the tower waits");
}

/// One saved tower in `kept`, written by putting the orb down.
fn one_tower_kept(kept: &Kept) {
    let game = at_the_menu(kept);
    raise(&game);
    game.does("attend laboratory", "/tower/laboratory");
    game.does("quit", "leave the orb? quit again to go");
    game.type_raw("quit");
    game.expect_gone();
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_second_tower_never_takes_the_firsts_slot() {
    if !available() {
        return;
    }
    // Before its first autosave the first tower is not on the disk, so the menu
    // offered its slot to the second; the swap wrote the first there and the
    // second wrote over it.
    let kept = Kept::default();
    let game = at_the_menu(&kept);
    raise(&game);
    game.does("attend laboratory", "/tower/laboratory");
    menu(&game);
    raise(&game);
    game.expect_off_screen("/tower/laboratory");

    menu(&game);
    game.type_raw("play");
    game.expect_drawn("1 wizard, a short game");
    game.type_raw("1");
    game.expect_drawn("/tower/laboratory");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_second_tower_installed_blames_no_other_orb() {
    if !available() {
        return;
    }
    // With the installation on, the default, the slot was carried to the wake
    // unasked: the wake found the first tower in it and said another orb had
    // taken its place.
    let kept = Kept::default();
    let game = Game::launch()
        .at_the_threshold()
        .kept_in(&kept)
        .installs(Switch::Always)
        .spawn();
    game.expect_drawn("no tower yet");
    raise(&game);
    game.type_raw("skip");
    game.expect_drawn("cold start");
    game.does("attend laboratory", "/tower/laboratory");
    menu(&game);
    raise(&game);
    game.type_raw("skip");
    game.expect_off_screen("/tower/laboratory");
    game.expect_drawn("cold start");
    game.refute_after(2, "another orb");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn loading_the_tower_you_are_in_keeps_where_you_are() {
    if !available() {
        return;
    }
    // The play page lists the slot in hand too. Written first and then read,
    // nothing since the last save is lost.
    let kept = Kept::default();
    one_tower_kept(&kept);
    let game = at_the_menu(&kept);
    game.type_raw("play");
    game.expect_drawn("1 wizard, a short game");
    game.type_raw("1");
    game.expect_drawn("/tower/laboratory");
    // A sealed tower has one room, so the move is out of it. The border title is
    // where the player stands; the transcript keeps the history.
    game.does("attend tower", "attend /tower");
    game.expect_drawn("┌ /tower F4");
    menu(&game);
    game.type_raw("play");
    game.expect_drawn("1 wizard, a short game");
    game.type_raw("1");
    game.expect_off_screen("open a tower");
    // Back where the player stood, not where the last save had them.
    game.expect_drawn("┌ /tower F4");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn an_unreadable_tower_leaves_you_where_you_were() {
    if !available() {
        return;
    }
    let kept = Kept::default();
    one_tower_kept(&kept);
    let game = at_the_menu(&kept);
    game.type_raw("play");
    game.expect_drawn("1 wizard, a short game");
    // Spoiled after the listing was read, so the menu still offers it.
    std::fs::write(kept.save(), "not a tower").expect("the save can be spoiled");
    game.type_raw("1");
    // Set aside and listed again, and said; still the menu, no tower raised.
    game.expect_drawn("no towers yet");
    game.expect_drawn("the tower in 1 is not one this orb can read");
    game.expect_drawn("open a tower");
    assert!(
        !game.screen().contains("wizard $"),
        "an unreadable save raised a tower:\n{}",
        game.screen(),
    );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_tower_gone_since_it_was_listed_raises_nothing() {
    if !available() {
        return;
    }
    // Choosing a stale listing once raised a fresh tower at that path.
    let kept = Kept::default();
    one_tower_kept(&kept);
    let game = at_the_menu(&kept);
    game.type_raw("play");
    game.expect_drawn("1 wizard, a short game");
    std::fs::remove_file(kept.save()).expect("the save can be removed");
    game.type_raw("1");
    game.expect_drawn("no towers yet");
    game.expect_drawn("there is no tower in 1");
    assert!(
        !game.screen().contains("wizard $"),
        "a vanished save raised a tower:\n{}",
        game.screen(),
    );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_orb_keeps_six_towers() {
    if !available() {
        return;
    }
    // Six slots. Each new tower takes the next as the one being left is written;
    // the seventh has nowhere to go and says so, and after it the menu refuses.
    let kept = Kept::default();
    let game = at_the_menu(&kept);
    raise(&game);
    for _ in 0..6 {
        menu(&game);
        raise(&game);
    }
    game.expect_drawn("the orb cannot write what it remembers down");
    menu(&game);
    game.type_raw("play");
    game.expect_drawn("6 wizard, a short game");
    game.type_raw("new");
    game.expect_drawn("the orb keeps no more towers. open one and quit from it");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_swap_starts_no_thread_of_its_own() {
    if !available() {
        return;
    }
    // Every swap ran the loop again, and the loop registered its signal
    // handlers and started a watchdog thread each time.
    let game = Game::at_the_threshold();
    game.expect_drawn("no tower yet");
    raise(&game);
    let pid = game.pid().expect("the game's pid");
    let threads = || {
        std::fs::read_dir(format!("/proc/{pid}/task"))
            .map(Iterator::count)
            .unwrap_or_default()
    };
    let after_one = threads();
    for _ in 0..3 {
        menu(&game);
        raise(&game);
    }
    assert_eq!(
        threads(),
        after_one,
        "three swaps changed how many threads the game runs",
    );
}
