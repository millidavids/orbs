//! Writing a spell, reading it back, and casting it.

use crate::play::{Game, available};

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn scribe_opens_a_fresh_page_for_the_room_you_are_in() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .opens("scribe morning", "morning.spell in laboratory")
        .expect_drawn("laboratory");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_spell_is_written_where_its_work_is() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("scribe morning", "go where the work is first");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_editor_opens_in_command_state_and_lists_its_whole_vocabulary() {
    if !available() {
        return;
    }
    // Three words, and that is all of them.
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .opens("scribe morning", "morning.spell in laboratory")
        .expect_drawn("edit")
        .expect_drawn("interpret")
        .expect_drawn("quit");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_bare_end_reaches_the_buffer_as_a_word() {
    if !available() {
        return;
    }
    // **The trap that made a whole class of scenario impossible**: `tmux
    // send-keys 'end'` sends the *End key*, and `end` closes every `repeat` and
    // every `if` in the spell language. Typed literally it is a line like any
    // other, and this is the assertion that it arrived as one.
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .opens("scribe looping", "looping.spell in ");
    game.type_raw("edit");
    for line in ["repeat 4", "grind sage", "end"] {
        game.type_raw(line);
    }
    game.press("Escape");
    game.expect_drawn("repeat 4").expect_drawn("end");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn interpret_shows_what_the_orb_hears_rather_than_what_was_typed() {
    if !available() {
        return;
    }
    // The fair copy is words: `make a potion of clarity` resolves to `recall
    // clarity`, and a name it cannot place survives as written.
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .opens("scribe check", "check.spell in ");
    game.type_raw("edit");
    game.type_raw("make a potion of clarity");
    game.press("Escape");
    game.type_raw("interpret");
    game.expect_drawn("recall clarity");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_count_survives_into_the_fair_copy() {
    if !available() {
        return;
    }
    // **This used to print `if cabinet has fragment`** — the number silently
    // swallowed with no fault raised, which is §19's *"the orb writes down a
    // shorter command than it heard"* arriving through the one surface built to
    // catch it.
    let game = Game::start();
    game.does("attend archive", "/tower/archive")
        .opens("scribe check", "check.spell in ");
    game.type_raw("edit");
    game.type_raw("if the cabinet has 4 fragment");
    game.type_raw("wield lectern");
    game.type_raw("end");
    game.press("Escape");
    game.type_raw("interpret");
    game.expect_drawn("4 fragment");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_buffer_writes_itself_out_a_beat_after_the_typing_stops() {
    if !available() {
        return;
    }
    // **§19: there is no `save`.** The settle clock is measured off `Time`,
    // which no dump ever advances — so this beat exists only in a live loop and
    // is invisible to every other layer in the project. `quit` is the flush a
    // dump has to use; here the point is that simply stopping is enough.
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .opens("scribe settling", "settling.spell in ");
    game.type_raw("edit");
    game.type_raw("grind sage");
    game.press("Escape");
    // No `quit`, no `w`. Just stop, let the world run, and leave.
    game.wait_ticks(4);
    game.closes_editor();
    game.does("peruse settling.spell", "grind sage");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_spell_is_kept_exactly_as_it_was_typed() {
    if !available() {
        return;
    }
    // The orb never rewrites a spell; `peruse` gives you your own words back,
    // even the ones it could not read.
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .opens("scribe verbatim", "verbatim.spell in ");
    game.type_raw("edit");
    game.type_raw("make a potion of clarity");
    game.press("Escape");
    game.closes_editor();
    game.does("peruse verbatim.spell", "make a potion of clarity");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_spells_own_records_go_to_the_log_and_not_the_pane() {
    if !available() {
        return;
    }
    // `prompt.rs` draws *"what the player did, not what their spells did"* — a
    // `repeat` loop would otherwise push the player's last line off screen in
    // seconds. So a dump that casts a spell and looks at the transcript finds
    // nothing and looks broken; the log is the same stream read another way.
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("invoke first_light", "first_light.spell")
        .meditates(12)
        .does("sift charcoal laboratory.log", "charcoal");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn an_invocation_needs_you_there_and_stops_when_you_leave() {
    if !available() {
        return;
    }
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("invoke first_light", "first_light.spell")
        .does("attend archive", "/tower/archive")
        .expect_somewhere("it stops");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_orb_cannot_hold_a_spell_until_it_has_learned_to() {
    if !available() {
        return;
    }
    // `bind` costs 16 experience and there is no way to grant it: concentration
    // is derived from work completed and `debug_spawn` deliberately earns none.
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("bind first_light", "the orb cannot hold a spell yet");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn every_dev_spell_is_on_the_shelf_from_the_first_tick() {
    if !available() {
        return;
    }
    // In a debug build only: `raise_grimoire` shelves them under
    // `cfg(debug_assertions)`, so a release shelf holds only the shipped ones.
    let game = Game::start();
    game.does("survey grimoire", "threading")
        .expect("breaking")
        .expect("assembling");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_fault_latches_on_the_room_until_you_go_and_look() {
    if !available() {
        return;
    }
    // A refused command is *not* a fault; the four that are come from
    // `say_failure` at `Role::Danger`. `‼` is the mark, and going to look is
    // the only thing that clears it.
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .opens("scribe broken", "broken.spell in ");
    game.type_raw("edit");
    game.type_raw("repeat 5");
    game.type_raw("wield zzz");
    game.type_raw("end");
    game.press("Escape");
    game.closes_editor();
    game.does("invoke broken", "broken.spell")
        .meditates(20)
        .does("attend archive", "/tower/archive")
        .expect_drawn("‼");
}
