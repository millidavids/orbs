//! Whole sessions, played end to end.
//!
//! The short scenarios each prove one thing and tell you what broke. These three
//! chain systems together the way an actual sitting does, and catch what no
//! single-claim scenario can: a threshold reached by real work rather than by
//! being handed the number, a spell running against a world it changed, a room
//! that only misbehaves once something else has happened in it.
//!
//! **They assert on outcomes, never on stock.** A long run meets ambient
//! sabotage — `drift` poisons a log at 1/300 per tick — so a listing is not a
//! stable claim, while a count, a name and an experience total are.

use crate::play::{Game, available};

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_morning_in_the_laboratory() {
    if !available() {
        return;
    }
    // Brew the flagship by hand, watch the orb learn to hold a spell, and see
    // the progression screen agree about what was earned.
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("help", "the work")
        .does("recall clarity", "5 steps")
        .does("kindle charcoal", "fuel for 600 ticks")
        .does("grind sage", "to mortar_and_pestle")
        .does("meditate 9", "yields ground-sage")
        .does("empty mortar_and_pestle", "ground-sage, husks")
        .does("digest ground-sage", "to balneum_mariae")
        .does("meditate 14", "yields sage-tincture")
        .does("grind rock-salt", "to mortar_and_pestle")
        .does("meditate 9", "yields ground-salt")
        .does("empty mortar_and_pestle", "ground-salt, husks")
        .does("mix sage-tincture with ground-salt", "to flask_and_rod")
        .does("meditate 12", "yields clarified-draught")
        .does("distil clarified-draught", "to alembic")
        .does("meditate 60", "yields clarity")
        .expect("the orb can hold a spell now")
        .does("status", "experience 16")
        .expect("concentration 1")
        // The bar is against a fixed scale of 100, not against the next
        // threshold — a See-it line claiming `0 of 16` once described a screen
        // that has never existed.
        .opens("weave", "of 100");
    game.type_raw("quit");
    game.does("empty alembic", "clarity")
        .does("move clarity to arsenal", "clarity: dispensary to arsenal")
        .does("attend archive", "/tower/archive")
        .does("survey arsenal", "clarity");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn an_afternoon_in_the_stacks() {
    if !available() {
        return;
    }
    // Open a maze, walk it by hand, then hand it to the solver and take what it
    // wins. **`1 or more`, never an exact count**: how many laps land inside
    // 7200 ticks depends on the wall clock, which is not a claim about the game.
    let game = Game::seeded(11);
    game.does("attend archive", "/tower/archive")
        .does("help", "the work")
        .does("research", "a way out is in them")
        .opens("wander", "arrows walk");
    for key in ["Up", "Up", "Left", "Down", "Right"] {
        game.press(key);
    }
    game.press("Escape");

    // The solver is twenty-four rungs across eighty lines; do not hand-write
    // one. `threading` is on the grimoire's shelf from tick 0 in a debug build.
    game.does("debug_spell threading", "threading")
        .does("invoke threading", "threading.spell")
        // **Three, because this maze has been walked on.** A fresh one is solved
        // inside ~7200 ticks; this one starts from wherever the hand-walk above
        // left the reading, with marks already laid, so the ladder has further
        // to go. `MAX_MEDITATE` is 3600 and a larger number is silently clamped,
        // which is how a first pass at measuring this "found" a plateau that was
        // the cap (§19).
        .meditates(3600)
        .meditates(3600)
        .meditates(3600)
        .does("survey cabinet", "fragment");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn an_evening_writing_a_spell() {
    if !available() {
        return;
    }
    // **Two distillations first, and they are not decoration**: `bind` costs 16
    // experience, concentration is derived from work completed, and
    // `debug_spawn` deliberately earns none. There is no way to grant it, so the
    // only route to a bound spell is to actually do the work.
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("kindle charcoal", "fuel for 600 ticks")
        .does("debug_spawn clarified-draught 2", "all along")
        .does("distil clarified-draught", "to alembic")
        .does("meditate 60", "yields clarity")
        .does("empty alembic", "clarity")
        .does("distil clarified-draught", "to alembic")
        .does("meditate 60", "yields clarity")
        .does("status", "concentration 1");

    // Write one that can actually lap: a standing spell is cast again every time
    // it runs off the end, so it has to empty its own mortar or it fouls it on
    // the second pass and complains for ever.
    game.opens("scribe tending", "tending.spell in ");
    game.type_raw("edit");
    for line in ["grind sage", "empty mortar_and_pestle"] {
        game.type_raw(line);
    }
    game.press("Escape");
    game.type_raw("interpret");
    game.expect_drawn("grind sage");
    game.press("Escape");
    game.closes_editor();

    // A binding survives leaving the room; an invocation would not.
    game.does("bind tending", "will hold")
        .does("attend archive", "/tower/archive")
        .meditates(60)
        .does("sift sage laboratory.log", "sage");
    game.does("status", "experience");
    assert!(
        !game.last_block().contains("experience 16"),
        "a bound spell ran for sixty ticks in another room and earned nothing:\n{}",
        game.screen(),
    );
}
