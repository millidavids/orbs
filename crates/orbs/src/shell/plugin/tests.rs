use super::*;
use crate::shell::{Reading, Standing};
use bevy::input::ButtonState;
use bevy::input::keyboard::Key;
use orbs_render::{FieldName, Outcome};

fn app() -> App {
    let mut app = App::new();
    // No renderer and no windowing backend — but `WindowPlugin` is what
    // registers `WindowResized`, which this plugin's own run conditions
    // read, so the shell genuinely depends on it.
    app.add_plugins((
        MinimalPlugins,
        bevy::input::InputPlugin,
        bevy::window::WindowPlugin::default(),
        ShellPlugin,
    ))
    .insert_resource(Tower::new(1));
    // Startup, so the systems under test see a settled world.
    app.update();
    app
}

fn press(app: &mut App, logical_key: Key, text: Option<&str>) {
    app.world_mut().write_message(KeyboardInput {
        // Not read by `type_into_line`; the whole point is that text comes
        // from `text`, not from a physical or logical key.
        key_code: KeyCode::KeyA,
        logical_key: logical_key.clone(),
        state: ButtonState::Pressed,
        text: text.map(Into::into),
        repeat: false,
        window: Entity::PLACEHOLDER,
    });
    // And released, which `tap` next door already knew: a key left down
    // means the *next* press is not a fresh one. Since `press` stamps every
    // keystroke `KeyCode::KeyA`, one unreleased press left `A` held for the
    // rest of the test — a false failure the moment `watch_focus` read it.
    app.world_mut().write_message(KeyboardInput {
        key_code: KeyCode::KeyA,
        logical_key,
        state: ButtonState::Released,
        text: None,
        repeat: false,
        window: Entity::PLACEHOLDER,
    });
}

/// Press a **physical** key, the way `input_just_pressed` sees one.
///
/// For `ButtonInput` bindings, which read `key_code` alone; [`press`] always
/// stamps `A`.
fn tap(app: &mut App, key_code: KeyCode, logical_key: Key) {
    app.world_mut().write_message(KeyboardInput {
        key_code,
        logical_key,
        state: ButtonState::Pressed,
        text: None,
        repeat: false,
        window: Entity::PLACEHOLDER,
    });
    app.update();
    // Released on the way out, or `input_just_pressed` sees it held and the
    // *next* tap of the same key is not a fresh press.
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(key_code);
}

/// Hold a modifier down and let the keyboard fall silent long enough that
/// the chord can only be a ghost.
///
/// The silence is the evidence, since winit gives `Cmd+A` the text `"a"`.
fn ghost(app: &mut App, key: KeyCode) {
    hold(app, key);
    app.world_mut()
        .resource_mut::<super::super::input::Quiet>()
        .silent_for(30.0);
}

/// Hold a modifier down, the way winit reports one.
fn hold(app: &mut App, key: KeyCode) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(key);
}

#[test]
fn unfurl_hands_the_transcript_the_keyboard_and_escape_hands_it_back() {
    // A word, since the key could not be discovered. The exit must not
    // regress: Escape, as in the editor.
    let mut app = app();
    assert!(
        !app.world().resource::<orbs_shell::Scroll>().is_reading(),
        "the transcript had the keyboard before anyone asked",
    );

    type_only(&mut app, "unfurl");
    press(&mut app, Key::Enter, Some("\r"));
    app.update();
    // The write lands on a tick, and the mode is entered from the request.
    app.world_mut().resource_mut::<Tower>().step();
    app.update();
    assert!(
        app.world().resource::<orbs_shell::Scroll>().is_reading(),
        "`unfurl` did not hand over the keyboard",
    );

    tap(&mut app, KeyCode::Escape, Key::Escape);
    assert!(
        !app.world().resource::<orbs_shell::Scroll>().is_reading(),
        "escape did not return to the prompt",
    );
}

#[test]
fn the_arrows_step_one_record_and_home_and_end_reach_the_ends() {
    // One table with the terminal (`orbs_shell::apply_to_reading`). Here an
    // arrow was a second PgUp, and Home and End did nothing at all.
    let mut app = app();
    for _ in 0..6 {
        type_line(&mut app, "status");
        app.world_mut().resource_mut::<Tower>().step();
        app.update();
    }
    app.world_mut().resource_mut::<orbs_shell::Scroll>().read();
    let back = |app: &App| app.world().resource::<orbs_shell::Scroll>().back();

    tap(&mut app, KeyCode::ArrowUp, Key::ArrowUp);
    assert_eq!(back(&app), 1, "ArrowUp is not one record");
    tap(&mut app, KeyCode::ArrowUp, Key::ArrowUp);
    tap(&mut app, KeyCode::ArrowDown, Key::ArrowDown);
    assert_eq!(back(&app), 1, "ArrowDown is not one record");

    let total = app
        .world()
        .resource::<Tower>()
        .sim()
        .scrollback()
        .records()
        .drawn_len();
    tap(&mut app, KeyCode::Home, Key::Home);
    assert_eq!(
        back(&app),
        total - 1,
        "Home did not stop at the oldest record"
    );
    tap(&mut app, KeyCode::End, Key::End);
    assert_eq!(back(&app), 0, "End did not reach the newest");
}

#[test]
fn the_prompt_is_deaf_while_the_transcript_is_being_read() {
    // On the line, not the run conditions, where the wiring could go wrong.
    let mut app = app();
    app.world_mut().resource_mut::<Line>().clear();
    app.world_mut().resource_mut::<orbs_shell::Scroll>().read();

    type_only(&mut app, "survey");
    assert_eq!(
        app.world().resource::<Line>().viewport(80).0,
        "",
        "typing reached the prompt while the transcript had the keyboard",
    );

    // ...and it comes back the moment reading ends.
    app.world_mut()
        .resource_mut::<orbs_shell::Scroll>()
        .stop_reading();
    type_only(&mut app, "survey");
    assert_eq!(
        app.world().resource::<Line>().viewport(80).0,
        "survey",
        "the prompt did not get the keyboard back",
    );
}

#[test]
fn a_modifier_stuck_with_no_focus_event_still_lets_you_type() {
    // Recovery with no `WindowFocused` and no reported key-up: the keystroke
    // itself is the evidence.
    let mut app = app();
    ghost(&mut app, KeyCode::SuperLeft);
    ghost(&mut app, KeyCode::ControlLeft);

    type_only(&mut app, "survey");
    assert_eq!(
        app.world().resource::<Line>().viewport(80).0,
        "survey",
        "a ghost modifier ate every keystroke, with no focus event to clear it",
    );
    assert!(
        !app.world()
            .resource::<ButtonInput<KeyCode>>()
            .any_pressed([KeyCode::SuperLeft, KeyCode::ControlLeft]),
        "the ghost survived the keystroke that disproved it",
    );
}

#[test]
fn the_editor_recovers_from_a_ghost_modifier_too() {
    // The surface it was reported on. The prompt and the editor share the
    // guard, so they shared the freeze, and a fix tested only on the prompt
    // would have been half a fix.
    let mut app = app();
    app.world_mut()
        .resource_mut::<crate::shell::Editing>()
        .open(crate::shell::Editor::open("x.spell", "laboratory", &[]));
    ghost(&mut app, KeyCode::SuperLeft);

    // `edit` drops into the buffer, then a word into the spell. Both are
    // ordinary typing, and both were being eaten.
    type_only(&mut app, "edit");
    press(&mut app, Key::Enter, Some("\r"));
    app.update();
    type_only(&mut app, "survey");

    let mut editing = app.world_mut().resource_mut::<crate::shell::Editing>();
    let editor = editing.get_mut().expect("the editor closed");
    assert_eq!(
        editor.lines(),
        ["survey"],
        "a ghost modifier froze the editor",
    );
}

#[test]
fn a_chord_in_use_is_still_a_chord() {
    // The counterweight: a fresh chord has no silence behind it, so
    // `Cmd+A` types nothing.
    let mut app = app();
    hold(&mut app, KeyCode::SuperLeft);

    type_only(&mut app, "a");
    assert_eq!(
        app.world().resource::<Line>().viewport(80).0,
        "",
        "`Cmd+A` typed its letter into the prompt",
    );

    press(&mut app, Key::Enter, Some("\r"));
    app.update();
    assert!(
        app.world()
            .resource::<ButtonInput<KeyCode>>()
            .pressed(KeyCode::SuperLeft),
        "a chord in use was mistaken for a ghost",
    );
    assert_eq!(
        app.world().resource::<Line>().viewport(80).0,
        "",
        "`Cmd+Enter` reached the prompt",
    );
}

#[test]
fn losing_the_window_forgets_held_modifiers() {
    // `Cmd+Shift+Ctrl+4` on macOS hands the window to the screenshot
    // overlay mid-chord, so Cmd and Ctrl's *release* goes there and never
    // to us. Held state then says they are down for ever and the prompt is
    // dead with nothing on screen to say why.
    let mut app = app();
    hold(&mut app, KeyCode::SuperLeft);
    hold(&mut app, KeyCode::ControlLeft);

    app.world_mut().write_message(bevy::window::WindowFocused {
        window: Entity::PLACEHOLDER,
        focused: false,
    });
    app.update();

    assert!(
        !app.world()
            .resource::<ButtonInput<KeyCode>>()
            .any_pressed([KeyCode::SuperLeft, KeyCode::ControlLeft]),
        "a stolen window left its modifiers held"
    );

    // And typing works again, which is the thing the player noticed.
    type_only(&mut app, "survey");
    assert_eq!(app.world().resource::<Line>().viewport(80).0, "survey");
}

#[test]
fn a_chord_does_not_reach_enter_or_backspace() {
    // Both bypassed the chord guard, so `Cmd+Enter` submitted the line on
    // its way to the operating system.
    let mut app = app();
    type_only(&mut app, "survey");

    hold(&mut app, KeyCode::SuperLeft);
    press(&mut app, Key::Enter, Some("\r"));
    press(&mut app, Key::Backspace, Some("\u{8}"));
    app.update();

    assert_eq!(
        app.world().resource::<Line>().viewport(80).0,
        "survey",
        "a chord edited or submitted the line"
    );
}

/// Type `line` without submitting it, for tests that inspect the buffer.
fn type_only(app: &mut App, line: &str) {
    for glyph in line.chars() {
        // Exactly what winit delivers: the spacebar arrives as `Key::Space`
        // with `text: Some(" ")`, never as `Key::Character(" ")`.
        let key = if glyph == ' ' {
            Key::Space
        } else {
            Key::Character(glyph.to_string().into())
        };
        press(app, key, Some(&glyph.to_string()));
    }
    app.update();
}

/// Type `line` and press Enter.
fn type_line(app: &mut App, line: &str) {
    type_only(app, line);
    press(app, Key::Enter, Some("\r"));
    app.update();
}

fn messages(app: &App) -> Vec<String> {
    app.world()
        .resource::<Tower>()
        .sim()
        .scrollback()
        .records()
        .iter()
        .filter_map(|record| record.field(FieldName::Message))
        .map(|value| value.with_str(str::to_owned))
        .collect()
}

#[test]
fn a_typed_line_reaches_the_parser_and_comes_back() {
    let mut app = app();
    type_line(&mut app, "look around");

    // §6: the echo is canonical arcane whatever register was typed.
    assert_eq!(messages(&app), ["look around", "survey"]);
}

#[test]
fn the_space_bar_works() {
    // The regression this file exists for. `Key::Space` is not
    // `Key::Character(" ")`, so a buffer built from `logical_key` loses the
    // space bar and every multi-word command in §6.1 becomes untypeable —
    // with a green test suite and a dead game.
    let mut app = app();
    type_line(&mut app, "look around");
    assert_eq!(
        messages(&app).first().map(String::as_str),
        Some("look around"),
        "the space bar was swallowed",
    );
}

#[test]
fn control_characters_never_enter_the_buffer() {
    // `text` carries them — winit documents Enter as `Some("\r")` — and one
    // in the buffer takes a cell and draws nothing, so the caret drifts
    // with no visible cause. `Escape` clears the line now, and Tab
    // *completes*, so what it must not do is leave its own `\t` behind; an
    // exact line here would be asserting the completer's answer instead.
    let mut tabbed = app();
    press(&mut tabbed, Key::Tab, Some("\t"));
    tabbed.update();
    assert!(
        !tabbed.world().resource::<Line>().text().contains('\t'),
        "a tab reached the buffer"
    );

    let mut app = app();
    press(&mut app, Key::Character("a".into()), Some("a"));
    press(&mut app, Key::Enter, Some("\r"));
    app.update();
    assert_eq!(messages(&app).first().map(String::as_str), Some("a"));
}

#[test]
fn tab_extends_as_far_as_the_candidates_agree() {
    let mut app = app();
    type_line(&mut app, "attend laboratory");
    type_only(&mut app, "wield mo");
    press(&mut app, Key::Tab, Some("\t"));
    app.update();

    assert_eq!(
        app.world().resource::<Line>().viewport(80).0,
        "wield mortar_and_pestle ",
        "a lone candidate should complete and leave a space"
    );
}

#[test]
fn tab_lists_when_extending_would_add_nothing() {
    // readline's `show-all-if-ambiguous`. The candidates are **transient
    // Frame content**, never a record: a Tab press is not a submission, and
    // a frontend writing to the log breaks replay by construction.
    let mut app = app();
    type_line(&mut app, "attend laboratory");
    type_only(&mut app, "wield ");
    press(&mut app, Key::Tab, Some("\t"));
    app.update();

    let offered = &app.world().resource::<orbs_shell::Offered>().options;
    assert!(offered.len() > 1, "expected a list, got {offered:?}");
    assert!(offered.iter().any(|name| name == "alembic"), "{offered:?}");
}

#[test]
fn repeated_tab_cycles_through_the_candidates() {
    // readline's `menu-complete`. The first press says there is more than one
    // answer; every press after that has to *choose* one, or the list is a
    // dead end that leaves the player typing the name out by hand.
    let mut app = app();
    type_line(&mut app, "attend laboratory");
    type_only(&mut app, "wield ");

    // The first press lists and leaves the line alone (bash's default).
    press(&mut app, Key::Tab, Some("\t"));
    app.update();
    assert_eq!(
        app.world().resource::<Line>().text(),
        "wield ",
        "the first Tab changed the line instead of listing"
    );

    // Every press after it chooses.
    let mut seen = Vec::new();
    for _ in 0..3 {
        press(&mut app, Key::Tab, Some("\t"));
        app.update();
        seen.push(app.world().resource::<Line>().text().to_owned());
    }
    assert!(
        seen[0] != seen[1] && seen[1] != seen[2],
        "Tab did not move through the options: {seen:?}"
    );
    for line in &seen {
        assert!(line.starts_with("wield "), "{line:?}");
    }
    // ...and the list says which one the line is holding.
    assert_eq!(
        app.world().resource::<orbs_shell::Offered>().current,
        Some(2),
        "the listing does not mark where the cycle has reached"
    );
}

#[test]
fn typing_ends_the_tab_cycle() {
    // Otherwise the next Tab resumes a completion the player has typed past,
    // and splices a candidate into the middle of a word.
    let mut app = app();
    type_line(&mut app, "attend laboratory");
    type_only(&mut app, "wield ");
    press(&mut app, Key::Tab, Some("\t"));
    app.update();

    type_only(&mut app, "a");
    assert_eq!(
        app.world().resource::<orbs_shell::Offered>().current,
        None,
        "the cycle survived a keystroke"
    );
}

#[test]
fn the_ghost_prefers_history_over_completion() {
    // fish's order and zsh's default: history is the cheap half and usually
    // the right one, because a brew loop is a player repeating themselves.
    let mut app = app();
    type_line(&mut app, "attend laboratory");
    type_line(&mut app, "wield athanor");

    type_only(&mut app, "wield a");
    let scene_open = {
        let tower = app.world().resource::<Tower>();
        (
            tower.sim().scene().clone(),
            !tower.sim().choices().is_empty(),
        )
    };
    let ghost = app
        .world()
        .resource::<Line>()
        .ghost(&scene_open.0, scene_open.1);
    assert_eq!(
        ghost, "thanor",
        "history should win: completion alone would offer `alembic` too"
    );
}

#[test]
fn escape_clears_the_line() {
    let mut app = app();
    type_only(&mut app, "survey");
    press(&mut app, Key::Escape, Some("\u{1b}"));
    app.update();

    assert_eq!(app.world().resource::<Line>().viewport(80).0, "");
}

#[test]
fn the_caret_moves_and_edits_land_where_it_is() {
    let mut app = app();
    type_only(&mut app, "srvey");

    // Walk back to just after the `s` and put the missing `u` in.
    for _ in 0..4 {
        press(&mut app, Key::ArrowLeft, None);
    }
    app.update();
    type_only(&mut app, "u");

    assert_eq!(app.world().resource::<Line>().viewport(80).0, "survey");
}

#[test]
fn up_walks_history_filtered_by_what_is_typed() {
    // fish's default and zsh's `history-beginning-search-backward`: with a
    // brew loop repeating itself, typing `wield ` and pressing Up should
    // walk the wields, not the whole log.
    let mut app = app();
    for line in [
        "survey",
        "wield mortar_and_pestle",
        "status",
        "wield alembic",
    ] {
        type_line(&mut app, line);
    }

    type_only(&mut app, "wield ");
    press(&mut app, Key::ArrowUp, None);
    app.update();
    assert_eq!(
        app.world().resource::<Line>().viewport(80).0,
        "wield alembic"
    );

    press(&mut app, Key::ArrowUp, None);
    app.update();
    assert_eq!(
        app.world().resource::<Line>().viewport(80).0,
        "wield mortar_and_pestle",
        "the second Up searched the recalled line instead of the anchor"
    );

    // Walking forward past the newest restores what was being typed.
    press(&mut app, Key::ArrowDown, None);
    press(&mut app, Key::ArrowDown, None);
    app.update();
    assert_eq!(app.world().resource::<Line>().viewport(80).0, "wield ");
}

#[test]
fn backspace_deletes_and_enter_clears() {
    let mut app = app();
    for glyph in "surveyx".chars() {
        press(
            &mut app,
            Key::Character(glyph.to_string().into()),
            Some(&glyph.to_string()),
        );
    }
    press(&mut app, Key::Backspace, Some("\u{8}"));
    press(&mut app, Key::Enter, Some("\r"));
    app.update();
    assert_eq!(messages(&app).first().map(String::as_str), Some("survey"));

    // The line must not survive its own submission.
    type_line(&mut app, "look around");
    assert_eq!(messages(&app)[2], "look around");
}

#[test]
fn a_bare_enter_says_nothing() {
    // Every real shell just redraws the prompt.
    let mut app = app();
    press(&mut app, Key::Enter, Some("\r"));
    app.update();
    assert!(messages(&app).is_empty());
}

#[test]
fn an_unknown_word_comes_back_with_somewhere_to_go() {
    // §6 forbids a bare error, and a view must be able to tell a suggestion
    // from the command that will run.
    let mut app = app();
    type_line(&mut app, "xyzzy");

    let tower = app.world().resource::<Tower>();
    let outcomes: Vec<_> = tower
        .sim()
        .scrollback()
        .records()
        .iter()
        .filter_map(|record| record.outcome())
        .collect();
    assert_eq!(outcomes.first(), Some(&Outcome::Unresolved));
    assert!(
        outcomes.iter().skip(1).all(|o| *o == Outcome::Suggestion),
        "{outcomes:?}",
    );
}

#[test]
fn a_resolved_command_waits_for_the_tick_it_will_run_on() {
    // The two clocks: the echo is instant, the effect is tick-aligned.
    let mut app = app();
    type_line(&mut app, "look around");
    assert_eq!(app.world().resource::<Tower>().sim().pending().len(), 1);
}

/// Press a key the way winit does.
///
/// As a message: `keyboard_input_system` rebuilds `ButtonInput` from them in
/// `PreUpdate`.
fn press_key(app: &mut App, key_code: KeyCode, logical_key: Key) {
    app.world_mut().write_message(KeyboardInput {
        key_code,
        logical_key,
        state: ButtonState::Pressed,
        text: None,
        repeat: false,
        window: Entity::PLACEHOLDER,
    });
    app.update();
}

#[test]
fn f4_switches_focus_without_moving_the_grid() {
    // The key reaches `cycle_mode` (§9): the split changes, nothing else.
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::input::InputPlugin,
        bevy::window::WindowPlugin {
            primary_window: Some(Window {
                resolution: bevy::window::WindowResolution::new(2560, 1440),
                ..default()
            }),
            ..default()
        },
        ShellPlugin,
    ))
    .insert_resource(Tower::new(1));
    app.update();

    let before = *app.world().resource::<Screen>();
    assert!(before.is_hostable(), "the test window must host the floor");

    press_key(&mut app, KeyCode::F4, Key::F4);

    let after = *app.world().resource::<Screen>();
    assert_ne!(after.mode, before.mode, "F4 did not reach `cycle_mode`");

    assert_eq!(
        after.grid, before.grid,
        "F4 moved the grid, which is what a fixed grid exists to prevent",
    );
    assert_eq!(after.scale(), before.scale(), "F4 resized the glyphs");
    assert!(
        after.grid.fits(orbs_render::DEEP_FOCUS_FLOOR),
        "the fixed grid cannot host a second pane: {:?}",
        after.grid,
    );
    // ...and the split changed. Two panes, not `PANES`, since one pane tiles
    // alike in both modes; the loss is recorded on `PANES`.
    let request = |screen: Screen| orbs_render::ScreenRequest {
        main_panes: 2,
        mode: screen.mode,
        ..orbs_render::ScreenRequest::single(screen.grid)
    };
    assert_ne!(
        orbs_render::ScreenLayout::compute(&request(before)).main(),
        orbs_render::ScreenLayout::compute(&request(after)).main(),
        "the two focus modes tile identically, so F4 could never do anything",
    );
}

#[test]
fn typing_never_reaches_the_focus_key() {
    // F4 must not also land in the line buffer. `text` is `None` for a
    // function key, which is what keeps the two apart.
    let mut app = app();
    press(&mut app, Key::F4, None);
    press(&mut app, Key::Enter, Some("\r"));
    app.update();
    assert!(messages(&app).is_empty(), "a function key was typed");
}

/// Press or release a key carrying its **real** `key_code`.
///
/// For `watch_focus`, which tracks physical keys; `press` stamps `KeyA`.
fn physical(app: &mut App, key_code: KeyCode, logical_key: Key, state: ButtonState) {
    app.world_mut().write_message(KeyboardInput {
        key_code,
        logical_key,
        state,
        text: None,
        repeat: false,
        window: Entity::PLACEHOLDER,
    });
}

#[test]
fn leaving_the_maze_leaves_nothing_in_the_prompt() {
    let mut app = app();
    type_line(&mut app, "attend archive");
    app.world_mut().resource_mut::<Tower>().step();
    app.update();
    type_line(&mut app, "research");
    app.world_mut().resource_mut::<Tower>().step();
    app.update();
    type_line(&mut app, "wander");
    app.world_mut().resource_mut::<Tower>().step();
    app.update();
    assert!(
        app.world().resource::<crate::shell::Walk>().is_open(),
        "the arrows never took the maze, so this asserts nothing",
    );

    // Walking: the arrow goes down and stays down, which is the state the
    // bug needs. Nothing releases it before the Escape.
    physical(
        &mut app,
        KeyCode::ArrowUp,
        Key::ArrowUp,
        ButtonState::Pressed,
    );
    app.update();

    physical(&mut app, KeyCode::Escape, Key::Escape, ButtonState::Pressed);
    app.update();
    assert!(!app.world().resource::<crate::shell::Walk>().is_open());

    // The held arrow outliving the Escape is how the word came back: key
    // repeat keeps delivering, the maze has let go by this frame, and at
    // the prompt an arrow means *recall history* — newest entry, the
    // `wander`.
    physical(
        &mut app,
        KeyCode::ArrowUp,
        Key::ArrowUp,
        ButtonState::Pressed,
    );
    app.update();

    assert_eq!(
        app.world().resource::<Line>().text(),
        "",
        "the prompt kept the word that opened the maze",
    );
    assert!(
        app.world().resource::<crate::shell::Ghost>().0.is_empty(),
        "a suggestion outlived the line it trailed",
    );
    assert!(
        app.world().resource::<crate::shell::Offered>().is_empty(),
        "a Tab listing outlived the maze",
    );

    // And the other half. Let go and press again: that is a real keystroke
    // and recall must still work. A version that swallowed the keyboard
    // from the moment a surface closed would pass everything above and
    // break `Up`.
    physical(
        &mut app,
        KeyCode::ArrowUp,
        Key::ArrowUp,
        ButtonState::Released,
    );
    app.update();
    physical(
        &mut app,
        KeyCode::ArrowUp,
        Key::ArrowUp,
        ButtonState::Pressed,
    );
    app.update();
    assert_eq!(
        app.world().resource::<Line>().text(),
        "wander",
        "a fresh press of Up no longer recalls history",
    );
}

/// Every surface swallows the keyboard, and the prompt gets nothing.
///
/// `orbs_shell::Focus`'s regression test, over a table of surfaces so a new
/// one is a missing row.
#[test]
fn no_surface_lets_a_keystroke_reach_the_prompt() {
    for (surface, opening) in [
        // A spell is written *for* a domain, so `scribe` from the tower
        // landing opens nothing — which the "asserts nothing" guard below
        // caught on the first run.
        ("editor", &["attend laboratory", "scribe drill"][..]),
        ("weave", &["weave"][..]),
        ("maze", &["attend archive", "research", "wander"][..]),
        ("reading", &["unfurl"][..]),
        // There was a fifth row, `chant`. The menagerie is typed now (§19),
        // so the row went with the surface rather than staying to assert
        // something that cannot open.
    ] {
        let mut app = app();
        for line in opening {
            type_line(&mut app, line);
            app.world_mut().resource_mut::<Tower>().step();
            app.update();
        }
        assert!(
            focus_of(&app).is_elsewhere(),
            "{surface} never took the keyboard, so this asserts nothing",
        );

        type_only(&mut app, "zzz");
        app.update();
        assert_eq!(
            app.world().resource::<Line>().text(),
            "",
            "{surface} let typing through to the prompt",
        );
    }
}

/// The word, through the real plugin stack, to the real menu.
///
/// Not the dump, which builds no `App` and so never reaches
/// `menuing::open_requested`.
#[test]
fn the_word_menu_opens_the_menu() {
    let mut app = app();
    type_line(&mut app, "menu");
    // The effect lands on the tick, not at `submit`: `submit` echoes and
    // queues, and every verb's effect runs at the next `step`, so a check
    // before this finds the flag unset — the mistake `quit` shipped once.
    app.world_mut().resource_mut::<Tower>().step();
    app.update();

    assert!(
        app.world().resource::<crate::shell::Standing>().is_open(),
        "`menu` did not open the menu",
    );
    assert_eq!(focus_of(&app), orbs_shell::Focus::Menu);
    assert!(
        app.world()
            .resource::<Messages<bevy::app::AppExit>>()
            .is_empty(),
        "opening the menu left the orb",
    );
}

/// `quit` asks once, and the second one goes.
///
/// Superseded (§19): for one iteration `quit` opened the menu, so stopping
/// was two steps through a screen nobody asked for. Now `quit` leaves and
/// the two steps are a *question*, like every other refusal in the game.
#[test]
fn quit_asks_once_and_the_second_one_leaves() {
    let mut app = app();
    type_line(&mut app, "quit");
    app.world_mut().resource_mut::<Tower>().step();
    app.update();
    assert!(
        app.world()
            .resource::<Messages<bevy::app::AppExit>>()
            .is_empty(),
        "the first `quit` left the orb without asking",
    );
    assert!(
        app.world().resource::<Tower>().sim().is_asking_to_quit(),
        "the first `quit` did not ask",
    );
    assert!(
        !app.world().resource::<crate::shell::Standing>().is_open(),
        "`quit` opened the menu, which is the behaviour that was superseded",
    );

    type_line(&mut app, "quit");
    app.world_mut().resource_mut::<Tower>().step();
    app.update();
    assert!(
        !app.world()
            .resource::<Messages<bevy::app::AppExit>>()
            .is_empty(),
        "the second `quit` did not leave",
    );
}

/// ...and anything else answers *no*.
#[test]
fn any_other_word_calls_off_a_pending_quit() {
    // The surprise the question exists to prevent: a `quit` typed and
    // thought better of, ending a session three commands later.
    let mut app = app();
    type_line(&mut app, "quit");
    app.world_mut().resource_mut::<Tower>().step();
    app.update();

    type_line(&mut app, "look around");
    app.world_mut().resource_mut::<Tower>().step();
    app.update();
    assert!(
        !app.world().resource::<Tower>().sim().is_asking_to_quit(),
        "the question outlived the next line",
    );

    type_line(&mut app, "quit");
    app.world_mut().resource_mut::<Tower>().step();
    app.update();
    assert!(
        app.world()
            .resource::<Messages<bevy::app::AppExit>>()
            .is_empty(),
        "a `quit` after a cancelled one left without asking again",
    );
}

#[test]
fn escape_out_of_the_manual_leaves_the_menu_standing() {
    // Two readers, one Escape: the manual shuts, and the menu under it must
    // not read the same key. Only an `App` sees it.
    let mut app = app();
    app.world_mut().resource_mut::<Standing>().open_menu(
        orbs_shell::Stance::InTower,
        orbs_shell::Driver::default(),
        crate::shell::setting::defaults(),
    );
    let book = orbs_shell::manual_book(app.world().resource::<Tower>().sim());
    app.world_mut().resource_mut::<Reading>().open(book);
    app.update();
    assert!(app.world().resource::<Reading>().is_open());
    assert!(app.world().resource::<Standing>().is_open());

    // One Escape: out of the manual, and no further.
    app.world_mut().write_message(KeyboardInput {
        key_code: KeyCode::Escape,
        logical_key: Key::Escape,
        state: ButtonState::Pressed,
        text: None,
        repeat: false,
        window: Entity::PLACEHOLDER,
    });
    app.update();

    assert!(
        !app.world().resource::<Reading>().is_open(),
        "Escape did not leave the manual",
    );
    assert!(
        app.world().resource::<Standing>().is_open(),
        "the same Escape closed the menu behind it",
    );
}

#[test]
fn a_word_typed_at_the_manual_does_not_also_drive_the_menu() {
    // The other half, and the one that could leave the game: `q` is not a
    // chapter, so the menu underneath prefix-matched its own `quit` and
    // wrote an `AppExit` from inside the manual.
    let mut app = app();
    app.world_mut().resource_mut::<Standing>().open_menu(
        orbs_shell::Stance::InTower,
        orbs_shell::Driver::default(),
        crate::shell::setting::defaults(),
    );
    let book = orbs_shell::manual_book(app.world().resource::<Tower>().sim());
    app.world_mut().resource_mut::<Reading>().open(book);
    app.update();

    type_only(&mut app, "q");
    app.world_mut().write_message(KeyboardInput {
        key_code: KeyCode::Enter,
        logical_key: Key::Enter,
        state: ButtonState::Pressed,
        text: Some("\r".into()),
        repeat: false,
        window: Entity::PLACEHOLDER,
    });
    app.update();

    assert!(
        app.world()
            .resource::<Messages<bevy::app::AppExit>>()
            .is_empty(),
        "typing at the manual left the orb",
    );
    assert!(
        app.world().resource::<Standing>().is_open(),
        "typing at the manual closed the menu behind it",
    );
}

/// The keystrokes and the opening in one frame, where a reader that never ran
/// would spell `menu` into the menu it opened.
#[test]
fn opening_the_menu_does_not_eat_the_word_that_opened_it() {
    let mut app = app();

    // The keystrokes, with no update between them and the tick.
    type_only(&mut app, "menu");
    app.world_mut().write_message(KeyboardInput {
        key_code: KeyCode::Enter,
        logical_key: Key::Enter,
        state: ButtonState::Pressed,
        text: Some("\r".into()),
        repeat: false,
        window: Entity::PLACEHOLDER,
    });
    {
        let mut tower = app.world_mut().resource_mut::<Tower>();
        tower.submit("menu");
        tower.step();
    }
    app.update();

    assert!(
        app.world().resource::<crate::shell::Standing>().is_open(),
        "the menu ate the word that opened it and closed again",
    );
    assert!(
        app.world()
            .resource::<Messages<bevy::app::AppExit>>()
            .is_empty(),
        "the menu read back the word that opened it and left the orb",
    );
    assert_eq!(
        app.world()
            .resource::<crate::shell::Standing>()
            .get()
            .map(orbs_shell::Menu::command),
        Some(""),
        "the keystrokes that opened the menu were typed into it",
    );
}

/// The shell standing at the threshold: no tower chosen, the menu up.
///
/// `Threshold::Waiting` is inserted directly rather than by `SimPlugin`,
/// which this harness does not install — what is under test here is the
/// *keyboard*, and the plugin's own half has tests beside it in `sim`.
fn at_the_threshold() -> App {
    let mut app = app();
    app.insert_resource(orbs_shell::Threshold::Waiting);
    // One frame for `open_at_the_threshold` to put the menu up.
    app.update();
    app
}

/// The keyboard belongs to the menu, whatever is pressed at it.
///
/// A real keyboard, not a dump (§19). It holds the routing, not "never
/// closed", which `the_threshold_has_no_way_to_close_the_menu` holds.
#[test]
fn the_threshold_gives_no_keystroke_to_the_prompt_behind_it() {
    let mut app = at_the_threshold();
    assert!(
        app.world().resource::<crate::shell::Standing>().is_open(),
        "the threshold did not put its menu up",
    );

    // Escape, the way out of every other surface in the game; `resume`,
    // whole and by the prefix that reaches it over a tower; and an ordinary
    // sentence. After all of it the menu still has the keyboard.
    for _ in 0..3 {
        tap(&mut app, KeyCode::Escape, Key::Escape);
    }
    type_line(&mut app, "resume");
    type_line(&mut app, "r");
    type_line(&mut app, "look around");
    assert!(
        app.world().resource::<crate::shell::Standing>().is_open(),
        "the menu did not have the keyboard after all that",
    );

    // And nothing above leaked into a prompt nobody can see.
    assert!(
        app.world()
            .resource::<Messages<bevy::app::AppExit>>()
            .is_empty(),
        "a keystroke at the threshold left the orb",
    );
    assert_eq!(
        app.world().resource::<Line>().text(),
        "",
        "the menu's keystrokes reached the prompt behind it",
    );
}

/// ...and `quit` still leaves, which is what makes the above a rule about
/// *closing* rather than about the keyboard being dead.
#[test]
fn quit_leaves_the_orb_from_the_threshold() {
    let mut app = at_the_threshold();
    type_line(&mut app, "quit");
    assert!(
        !app.world()
            .resource::<Messages<bevy::app::AppExit>>()
            .is_empty(),
        "the threshold had no way out of the orb at all",
    );
}

/// The menu is up before the first keystroke can land anywhere else.
///
/// With `ORBS_BOOT=0` the frame the sequence finishes is frame one, which is
/// exactly when a stray keystroke is most likely to still be in the queue —
/// and `menuing.rs` records what a surface opening on such a frame costs.
#[test]
fn the_threshold_takes_the_keyboard_before_the_prompt_does() {
    let mut app = app();
    app.insert_resource(orbs_shell::Threshold::Waiting);
    // Typed on the very frame the menu goes up, with no update in between.
    type_only(&mut app, "look around");
    assert!(
        app.world().resource::<crate::shell::Standing>().is_open(),
        "the threshold never opened its menu",
    );
    assert_eq!(
        app.world().resource::<Line>().text(),
        "",
        "a keystroke reached the prompt behind the threshold's menu",
    );
}

/// because a `SystemParam` needs a system to live in and this is a helper
/// inside an assertion. The ordering it asks about is `orbs_shell::Focus`'s
/// either way, which is the whole point of that type.
fn focus_of(app: &App) -> orbs_shell::Focus {
    let world = app.world();
    orbs_shell::Focus::of(crate::shell::input::opened(
        world.resource(),
        world.resource(),
        world.resource(),
        world.resource(),
        world.resource(),
        world.resource(),
    ))
}
