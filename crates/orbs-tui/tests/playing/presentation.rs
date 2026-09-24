//! Resize, redraw, and colour — the three things no dump can reach.

use crate::play::{Game, available};

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn shrinking_a_full_screen_leaves_nothing_of_the_old_one() {
    if !available() {
        return;
    }
    // `blit`'s shadow buffer is addressed by `(col, row)`, so keeping it across
    // a resize wrote this frame's cells at last frame's coordinates. A full
    // screen shrunk is the case that shows it: every dropped cell has to go.
    let mut game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("help", "the work")
        .does("recall clarity", "steps");
    game.resize(90, 24);
    let screen = game.screen();
    for row in screen.lines() {
        assert!(
            row.chars().count() <= 90,
            "a row outlived the resize and is wider than the window:\n{screen}",
        );
    }
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn growing_a_window_fills_the_new_room_rather_than_leaving_it_blank() {
    if !available() {
        return;
    }
    let mut game = Game::sized(90, 24);
    game.does("attend laboratory", "/tower/laboratory")
        .does("survey dispensary", "sage");
    game.resize(120, 45);
    game.expect_drawn("tower");
    let screen = game.screen();
    let last = screen.lines().filter(|row| row.starts_with('┌')).count();
    assert!(
        last > 0,
        "the grown window drew no border at all:\n{screen}"
    );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_rail_yields_whole_and_falls_back_into_the_border() {
    if !available() {
        return;
    }
    // `fits_rail` wants 36 rows; below that the rail is gone and its readings
    // fall back into the session border's title, which is why the driver reads
    // the tick from two places.
    let game = Game::sized(90, 24);
    game.does("attend laboratory", "/tower/laboratory");
    let title = game.screen().lines().next().unwrap_or_default().to_owned();
    assert!(
        title.contains("tick"),
        "with no room for a rail the border should carry the readings: {title:?}",
    );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn below_the_floor_the_orb_asks_for_a_larger_window() {
    if !available() {
        return;
    }
    // §19 names *"a terminal the user shrank"* as a normal runtime state, so
    // this is a real screen rather than a refusal.
    let mut game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("survey dispensary", "sage");
    game.resize(60, 18);
    game.expect_drawn("larger window");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_window_that_comes_back_brings_the_game_with_it() {
    if !available() {
        return;
    }
    let mut game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("survey dispensary", "sage");
    game.resize(60, 18);
    game.expect_drawn("larger window");
    game.resize(120, 45);
    game.does("status", "experience 0");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_fire_is_orange_all_the_way_up() {
    if !available() {
        return;
    }
    // Orange on every tube (§19): the four heats are one hue differing by
    // weight, and `ORBS_DUMP` throws colour away, so this decoder is the only
    // instrument that can see it.
    //
    // The columns are the whole test. The athanor burns at 100-101, inside the
    // instrument panel; decoding the tower rail instead reads a band with no
    // fire in it and passes whatever the ramp does.
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("kindle charcoal", "fuel for 600 ticks")
        .meditates(3);
    let ink = game.ink(95, 103);
    assert!(
        ink.contains("yellow"),
        "no fire in the panel band at all, so this asserts nothing:\n{ink}",
    );
    assert!(
        !ink.contains("white") && !ink.contains("red"),
        "the fire left the orange family; it should brighten, not change \
         substance — the four heats are two hues and the weight axis:\n{ink}",
    );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_rails_fault_mark_is_drawn_in_the_danger_hue() {
    if !available() {
        return;
    }
    // The mark and its colour are two claims, and only the second needs this
    // layer: `‼` in the success hue would say a broken room was fine.
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .opens("scribe broken", "broken.spell in ");
    game.type_raw("edit");
    game.type_raw("wield zzz");
    game.press("Escape");
    game.closes_editor();
    game.does("invoke broken", "broken.spell")
        .meditates(8)
        .does("attend archive", "/tower/archive")
        .expect_drawn("‼");
    // The mark's own cell, not "some red in the sidebar": `ink` prints
    // `'glyph':colour/weight` per cell, so the mark can be asked directly.
    let ink = game.ink(103, 119);
    let hue = ink
        .split("'‼':")
        .nth(1)
        .and_then(|rest| rest.split_whitespace().next())
        .unwrap_or("<the mark was not in the decoded band at all>");
    assert!(
        hue.contains("red"),
        "the fault mark is drawn {hue}, and a broken room in the success hue \
         says the opposite of what happened:\n{ink}",
    );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn a_material_tint_reaches_the_cell_it_was_authored_for() {
    if !available() {
        return;
    }
    // A tint changes no glyph, so a colour that never reaches a cell draws in
    // the base hue and looks exactly like a material nobody has tinted yet.
    // sage grinds green.
    //
    // Read the panel band, and take a control first: the transcript's
    // `Role::Success` is already green, so decoding that band passes with the
    // tint path dead. Green that was not there and then is came from the
    // material.
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory");
    let bare = game.ink(89, 103);
    assert!(
        !bare.contains("green"),
        "the panel band was already green with an empty mortar, so this test \
         cannot tell a tint from the furniture:\n{bare}",
    );

    game.does("move sage to mortar_and_pestle", "to mortar_and_pestle");
    let loaded = game.ink(89, 103);
    assert!(
        loaded.contains("green"),
        "sage in the mortar should tint its bar green:\n{loaded}",
    );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_transcript_marks_a_refusal_differently_from_a_success() {
    if !available() {
        return;
    }
    // `√` and `¬` are the glyphs; the colours are what a player reads at a
    // glance, and §14 says the glyph must carry it too.
    //
    // An instrument that is already empty is not a refusal — `empty` on a bare
    // mortar answers `√ the mortar_and_pestle is already empty`. A second load
    // onto a working one is the real thing.
    let game = Game::start();
    game.does("attend laboratory", "/tower/laboratory")
        .does("grind sage", "to mortar_and_pestle")
        .does("grind rock-salt", "is at work");
    let pane = game.pane();
    assert!(
        pane.contains('¬'),
        "a refusal should be marked as one:\n{pane}",
    );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_boot_report_is_on_the_transcript_before_anything_is_typed() {
    if !available() {
        return;
    }
    // §4's report is records rather than a picture, and `tower/boot.rs` builds
    // it by walking the world so it cannot go stale — so this asks the rooms by
    // name rather than for a banner. It is the screen `ORBS_BOOT=0` lands on.
    let game = Game::start();
    game.expect_somewhere("orb")
        .expect_somewhere("laboratory")
        .expect_somewhere("archive")
        .expect_somewhere("lens");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_orb_is_found_before_it_is_switched_on() {
    if !available() {
        return;
    }
    // §4's sequence. `Stage::Dark` paints nothing, so the orb reads as *found*
    // rather than powering on like a monitor, and the logo then prints itself a
    // letter at a time.
    let game = Game::booting();
    game.expect_drawn("Operational")
        .expect_drawn("Operational Relic Bewitching System");
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_card_names_this_machine_and_not_the_other_one() {
    if !available() {
        return;
    }
    // The card is a diegetic inventory of the machine (§4), so a shared painter
    // hard-coding `bevy 0.19.0` would be a lie here, in a binary that does not
    // link Bevy at all.
    let game = Game::booting();
    game.expect_drawn("crossterm");
    assert!(
        !game.screen().contains("bevy"),
        "the terminal build's card claimed a component it is not built out of:\n{}",
        game.screen(),
    );
}

#[test]
#[ignore = "plays a real game through tmux; run with scripts/play.sh"]
fn the_sequence_hands_over_to_a_tower_whose_clock_never_started() {
    if !available() {
        return;
    }
    // Not cosmetic: `tower::drift` rolls once per tick, so a sim left running
    // through the animation advances its RNG stream by a wall-clock-dependent
    // number of draws — one seed reaching different worlds on different
    // machines.
    let game = Game::booting();
    game.expect_drawn("wizard $");
    let tick = game.tick();
    assert!(
        tick <= 2,
        "the world ran through the boot sequence and reached tick {tick}",
    );
}
