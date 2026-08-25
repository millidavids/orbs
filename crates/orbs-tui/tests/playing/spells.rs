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

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_guide_lists_the_language_and_the_rooms_own_verbs() {
    if !available() {
        return;
    }
    // The pane is open by default — a guide nobody knows to ask for helps
    // nobody — and the verbs are the **spell's** domain, not the player's.
    let game = Game::start();
    game.does("attend archive", "/tower/archive")
        .opens("scribe threading", "threading.spell in archive")
        .expect_drawn("the language")
        .expect_drawn("here you can")
        .expect_drawn("follow");

    // `guide` closes it, and opens it again. **`expect_off_screen`, not
    // `expect_absent`** — the latter asks about the newest command block and the
    // guide is a pane, so it would pass without looking.
    game.type_raw("guide");
    game.expect_off_screen("the language");
    game.type_raw("guide");
    game.expect_drawn("the language");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_word_a_spell_may_not_issue_is_not_in_the_guide() {
    if !available() {
        return;
    }
    // `attend` passes `Scene::offers` and fails `may_issue`: a spell is written
    // *for* a domain and does not walk. Listing it would teach a line the
    // runner refuses, which is worse than a short list.
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .opens("scribe morning", "morning.spell in laboratory")
        .expect_drawn("here you can")
        // The listing has landed whole, so what is missing from it is now a
        // fair question. `grind` proves we are reading the verbs and not an
        // empty pane.
        .expect_drawn("grind")
        .expect_off_screen("attend");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_guide_answers_what_may_follow_where_the_caret_is() {
    if !available() {
        return;
    }
    // The reactive half, and the one the feature was asked for. `is ` has
    // exactly eight answers and nothing on screen said so.
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .opens("scribe asking", "asking.spell in laboratory");
    game.type_raw("edit");
    // No Enter: the guide follows the caret, not the line.
    game.send_text("if the mortar_and_pestle is ");
    game.expect_drawn("what can follow")
        .expect_drawn("idle")
        .expect_drawn("working");

    // **`wait` is not `is`.** It stores a thing and resolves it against the
    // record stream, so offering a state there would teach a line that waits
    // for ever and latches a fault.
    game.press("Escape");
    game.type_raw("quit");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn tab_finishes_a_word_in_the_editor() {
    if !available() {
        return;
    }
    // readline's rules, shared with the prompt: a lone candidate is written out
    // whole with a space after it.
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .opens("scribe tabbing", "tabbing.spell in laboratory");
    game.type_raw("edit");
    game.send_text("gri");
    game.press("Tab");
    game.expect_drawn("grind");
    game.send_text("sa");
    game.press("Tab");
    game.expect_drawn("grind sage");
    game.press("Escape");
    game.type_raw("quit");
}
