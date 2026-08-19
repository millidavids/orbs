//! The stacks: a maze, four fragments, a lectern, and three scrolls.

use crate::play::{Game, available};

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn research_opens_a_reading_and_refuses_a_second() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend archive", "/tower/archive")
        .does("research", "a way out is in them")
        .does("research", "a reading is already open");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn there_are_no_stacks_outside_the_archive() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("research", "there is no stacks here to research with");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_map_is_drawn_the_moment_a_reading_opens() {
    if !available() {
        return;
    }
    // **Not gated on `wander`**, which is what makes a spell solving one
    // watchable. `☼` is the reading; `Ω` is the way out.
    let game = Game::start();
    game.does("attend archive", "/tower/archive")
        .does("research", "a way out is in them")
        .expect_drawn("☼");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_wall_is_loud_and_a_step_is_quiet() {
    if !available() {
        return;
    }
    // **`follow`'s success is `quiet`** (§19): the map already shows the move,
    // so a step is filtered out of the pane by `Records::drawn` and lives only
    // in the log. A wall is still spoken, because nothing moved and the map
    // reports nothing. Seed 11 opens with north and west passable, east and
    // south walls.
    let game = Game::seeded(11);
    game.does("attend archive", "/tower/archive")
        .does("research", "a way out is in them")
        .does("follow east", "there is no way east");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_step_reaches_the_log_even_though_it_skips_the_transcript() {
    if !available() {
        return;
    }
    let game = Game::seeded(11);
    game.does("attend archive", "/tower/archive")
        .does("research", "a way out is in them")
        .send("follow north");
    // **The one place a negative assertion is the claim itself**, so it is the
    // one place this driver waits a fixed number of ticks instead of waiting for
    // evidence: there is no event to pace against when the whole point is that
    // nothing was said. `Records::drawn` filters a `quiet` record out of the
    // pane, and the map already showed the move.
    game.refute_after(2, "the reading goes north");
    // ...and the same record, in the log, which is the stream read another way.
    game.does("peruse archive.log", "the reading goes north");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_way_reports_what_walking_it_would_do() {
    if !available() {
        return;
    }
    let game = Game::seeded(11);
    game.does("attend archive", "/tower/archive")
        .does("research", "a way out is in them")
        .does("survey north", "passage")
        .does("survey east", "wall");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_way_counts_its_marks_rather_than_bucketing_them() {
    if !available() {
        return;
    }
    // `walked` and `twice` are gone; a way reports a count, so a square walked
    // nine times no longer reads exactly like one walked twice.
    let game = Game::seeded(11);
    game.does("attend archive", "/tower/archive")
        .does("research", "a way out is in them")
        .send("follow north");
    game.send("follow south");
    game.does("survey north", "marks");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_stacks_can_be_let_go_and_opened_again() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend archive", "/tower/archive")
        .does("research", "a way out is in them")
        .does("stop stacks", "you let the stacks close")
        .does("research", "a way out is in them");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn four_fragments_in_the_lectern_become_a_scroll() {
    if !available() {
        return;
    }
    // The archive's loop is the laboratory's: load the instrument, wield it,
    // wait. `wield_done_clean` is the lectern's form — a yield with no
    // byproduct beside it.
    let game = Game::start();
    game.does("attend archive", "/tower/archive")
        .does("debug_spawn fragment 4 lectern", "all along")
        .does("wield lectern", "lectern")
        .does("meditate 21", "the lectern yields");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_lectern_part_way_to_a_scroll_is_gathering_not_fouled() {
    if !available() {
        return;
    }
    // **The state that exists because of this instrument.** One to three
    // fragments must not read `fouled`: they are a job in progress, not a
    // mistake to scour out.
    let game = Game::start();
    game.does("attend archive", "/tower/archive")
        .does("debug_spawn fragment 3 lectern", "all along")
        .does("survey lectern", "fragment = 3");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_gleaning_scroll_swaps_the_way_out_for_things_to_gather() {
    if !available() {
        return;
    }
    // The errand is **a word on the stacks**, which is what a spell asks for —
    // and the map loses its `Ω` entirely, because an inert exit would have a
    // solver walk onto it and take the same rung for ever.
    let game = Game::start();
    game.does("attend archive", "/tower/archive")
        .does("research", "a way out is in them")
        .does("debug_spawn gleaning-scroll", "all along")
        .does("wield gleaning-scroll", "and no way out")
        .does("survey stacks", "gleaning");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_quickening_scroll_halves_the_work_in_hand() {
    if !available() {
        return;
    }
    // **It never refuses for want of something to hurry** — it was a one-shot
    // on the run in hand, which made it useless at the moment a player reaches
    // for one. An 8-tick grind lands in 4.
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("debug_spawn quickening-scroll", "all along")
        .does("wield quickening-scroll", "works quick for")
        .does("grind sage", "to mortar_and_pestle")
        .does("meditate 4", "yields ground-sage");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_verdant_scroll_puts_a_new_herb_on_the_shelf_and_then_runs_out() {
    if !available() {
        return;
    }
    // **What may be unlocked is derived, never listed**: a base reagent is one
    // the vocabulary knows that nothing in the tower makes. The first version
    // asked `Recipes::outputs` and shelved every byproduct as an inexhaustible
    // herb, with the whole suite green — so the fourth scroll refusing is as
    // much the assertion as the first three working.
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("debug_spawn verdant-scroll 4", "all along")
        .does("wield verdant-scroll", "the shelf remembers")
        .does("wield verdant-scroll", "the shelf remembers")
        .does("wield verdant-scroll", "the shelf remembers")
        .does(
            "wield verdant-scroll",
            "every herb the verdant-scroll knows",
        );
    game.does("survey dispensary", "∞");
}
