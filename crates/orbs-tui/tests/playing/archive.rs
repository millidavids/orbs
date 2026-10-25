//! The stacks: a maze, four fragments, a lectern, and three scrolls.

use crate::play::Game;

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn research_opens_a_reading_and_refuses_a_second() {
    let Some(game) = Game::start() else { return };
    game.does("attend archive", "/tower/archive")
        .does("research", "a way out is in them")
        .does("research", "a reading is already open");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn there_are_no_stacks_outside_the_archive() {
    let Some(game) = Game::start() else { return };
    game.does("attend laboratory", "/tower/laboratory")
        .does("research", "there is no stacks here to research with");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_map_is_drawn_the_moment_a_reading_opens() {
    // Not gated on `wander`, so a spell solving one is watchable. `☼` is the
    // reading; `Ω` is the way out.
    let Some(game) = Game::start() else { return };
    game.does("attend archive", "/tower/archive")
        .does("research", "a way out is in them")
        .expect_drawn("☼");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_wall_is_loud_and_a_step_is_quiet() {
    // `follow`'s success is `quiet` (§19), so a step lives only in the log; a
    // wall is still spoken. Seed 11 opens with north and west passable.
    let Some(game) = Game::seeded(11) else { return };
    game.does("attend archive", "/tower/archive")
        .does("research", "a way out is in them")
        .does("follow east", "there is no way east");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_step_reaches_the_log_even_though_it_skips_the_transcript() {
    let Some(game) = Game::seeded(11) else { return };
    game.does("attend archive", "/tower/archive")
        .does("research", "a way out is in them")
        .send("follow north");
    // A negative assertion has no event to pace against, so this is the one
    // place the driver waits a fixed number of ticks rather than for evidence.
    game.refute_after(2, "the reading goes north");
    // ...and the same record, in the log, which is the stream read another way.
    game.does("peruse archive.log", "the reading goes north");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_way_reports_what_walking_it_would_do() {
    let Some(game) = Game::seeded(11) else { return };
    game.does("attend archive", "/tower/archive")
        .does("research", "a way out is in them")
        .does("survey north", "passage")
        .does("survey east", "wall");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_way_counts_its_marks_rather_than_bucketing_them() {
    // `walked` and `twice` are gone; a way reports a count, so a square walked
    // nine times no longer reads exactly like one walked twice.
    let Some(game) = Game::seeded(11) else { return };
    game.does("attend archive", "/tower/archive")
        .does("research", "a way out is in them")
        .send("follow north");
    game.send("follow south");
    game.does("survey north", "marks");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_stacks_can_be_let_go_and_opened_again() {
    let Some(game) = Game::start() else { return };
    game.does("attend archive", "/tower/archive")
        .does("research", "a way out is in them")
        .does("stop stacks", "you let the stacks close")
        .does("research", "a way out is in them");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn four_fragments_in_the_lectern_become_a_scroll() {
    // The archive's loop is the laboratory's: load the instrument, wield it,
    // wait. `wield_done_clean` is the lectern's form — a yield with no
    // byproduct beside it.
    let Some(game) = Game::start() else { return };
    game.does("attend archive", "/tower/archive")
        .does("debug_spawn fragment 4 lectern", "all along")
        .does("wield lectern", "lectern")
        .does("meditate 21", "the lectern yields");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_lectern_part_way_to_a_scroll_is_gathering_not_fouled() {
    // One to three fragments must not read `fouled`: a job in progress, not a
    // mistake to scour out.
    let Some(game) = Game::start() else { return };
    game.does("attend archive", "/tower/archive")
        .does("debug_spawn fragment 3 lectern", "all along")
        .does("survey lectern", "fragment = 3");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_gleaning_scroll_swaps_the_way_out_for_things_to_gather() {
    // The errand is a word on the stacks, and the map loses its `Ω` — an inert
    // exit would have a solver walk onto it and take the same rung for ever.
    let Some(game) = Game::start() else { return };
    game.does("attend archive", "/tower/archive")
        .does("research", "a way out is in them")
        .does("debug_spawn gleaning-scroll 4", "all along")
        .does("wield gleaning-scroll", "and no way out")
        .does("survey stacks", "gleaning");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_quickening_supply_halves_the_work_in_hand() {
    // Four scrolls open it, and an 8-tick grind lands in 4 (§19, *Supply*).
    let Some(game) = Game::start() else { return };
    game.does("attend laboratory", "/tower/laboratory")
        .does("debug_spawn quickening-scroll 4", "all along")
        .does("wield quickening-scroll", "2 times quick")
        .does("grind sage", "to mortar_and_pestle")
        .does("meditate 4", "yields ground-sage");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_verdant_supply_flowers_a_new_herb_on_the_shelf() {
    // Cultivation: one at the word, one more at the raise (it grows only the
    // difference), and the next draw flowers the first herb (§19, *Supply*).
    let Some(game) = Game::start() else { return };
    game.does("attend laboratory", "/tower/laboratory")
        .does("debug_spawn verdant-scroll 8", "all along")
        .does("wield verdant-scroll", "unrolls")
        .does("wield verdant-scroll", "unrolls faster")
        .does("meditate 300", "amber flowers on the shelf");
    game.does("survey dispensary", "∞");
}
