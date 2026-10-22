use super::{Owner, Surfaces};
use crossterm::event::KeyCode;
use orbs_shell::{Editor, Menu, Scroll, Tapestry};
use orbs_sim::Sim;

/// Build the flags directly, so a state no verb can reach is still asked
/// about. The point of the sweep below is the *ties*, and a tie is by
/// definition a state the game is not supposed to be able to produce.
///
/// `menuing` is not a parameter, deliberately: the ordering is
/// `orbs_shell::focus`'s and is tested there against every combination. What
/// this file tests is the *dispatch* — that a key reaches the surface the
/// owner names.
fn surfaces(editing: bool, weaving: bool, walking: bool) -> Surfaces {
    Surfaces {
        editing: editing.then(|| Editor::open("t", "laboratory", &[])),
        weaving: weaving.then(Tapestry::default),
        walking,
        menuing: None,
        reading: None,
        opening_manual: false,
        leaving: false,
        swapping: None,
        driving: None,
        setting: None,
    }
}

fn scroll(reading: bool) -> Scroll {
    let mut scroll = Scroll::default();
    if reading {
        scroll.read();
    }
    scroll
}

#[test]
fn nothing_open_is_the_prompt() {
    assert_eq!(
        surfaces(false, false, false).owner(&scroll(false)),
        Owner::Prompt
    );
}

#[test]
fn every_surface_is_reachable_on_its_own() {
    // A precedence chain hides its own last arm if an earlier one is always
    // true, and the arm that goes missing is the one added most recently.
    for (editing, weaving, walking, reading, want) in [
        (true, false, false, false, Owner::Editor),
        (false, true, false, false, Owner::Weave),
        (false, false, true, false, Owner::Maze),
        (false, false, false, true, Owner::Reading),
    ] {
        assert_eq!(
            surfaces(editing, weaving, walking).owner(&scroll(reading)),
            want,
            "{want:?} cannot be reached even when it is the only thing open",
        );
    }
}

#[test]
fn exactly_one_surface_holds_the_keyboard_in_every_state() {
    // The property the module exists for, over all sixteen states. `owner`
    // returns one `Owner` by construction, so what is at stake is the answer
    // being *decided* rather than accidental: the winner is always the first
    // open surface in the documented order. A silent tie is the harder bug —
    // two surfaces acting on one keystroke is invisible until a player types
    // `:wq` and finds it in their command history.
    for bits in 0..16u8 {
        let (editing, weaving, walking, reading) =
            (bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let want = if editing {
            Owner::Editor
        } else if weaving {
            Owner::Weave
        } else if walking {
            Owner::Maze
        } else if reading {
            Owner::Reading
        } else {
            Owner::Prompt
        };
        assert_eq!(
            surfaces(editing, weaving, walking).owner(&scroll(reading)),
            want,
            "editing={editing} weaving={weaving} walking={walking} reading={reading}",
        );
    }
}

#[test]
fn the_menu_takes_the_keys_and_leaving_is_a_choice_on_it() {
    // The one surface that can end the session, and the one whose keys never
    // touch the `Sim` — so this is the whole dispatch, with no world.
    let mut surfaces = surfaces(false, false, false);
    surfaces.menuing = Some(Menu::default());
    assert_eq!(
        surfaces.owner(&scroll(false)),
        Owner::Menu,
        "the menu did not take the keyboard from the prompt",
    );

    // It wins over everything, which is the tie `Focus::of` documents: the
    // way out is what a player meant.
    let mut over = self::surfaces(true, true, true);
    over.menuing = Some(Menu::default());
    assert_eq!(over.owner(&scroll(true)), Owner::Menu);

    // `resume` gives the keyboard back and leaves nothing behind.
    let mut scroll = scroll(false);
    let mut sim = Sim::new(1);
    for code in "resume".chars().map(KeyCode::Char) {
        surfaces.typed(
            Owner::Menu,
            code,
            &mut sim,
            &mut scroll,
            &orbs_sim::Verbatim,
        );
    }
    surfaces.typed(
        Owner::Menu,
        KeyCode::Enter,
        &mut sim,
        &mut scroll,
        &orbs_sim::Verbatim,
    );
    assert!(surfaces.menuing.is_none(), "resume did not close the menu");
    assert!(!surfaces.leaving, "resume asked to leave the orb");

    // ...and `quit` on the menu is what asks to leave, which the session reads.
    surfaces.menuing = Some(Menu::default());
    for code in "quit".chars().map(KeyCode::Char) {
        surfaces.typed(
            Owner::Menu,
            code,
            &mut sim,
            &mut scroll,
            &orbs_sim::Verbatim,
        );
    }
    surfaces.typed(
        Owner::Menu,
        KeyCode::Enter,
        &mut sim,
        &mut scroll,
        &orbs_sim::Verbatim,
    );
    assert!(surfaces.leaving, "quit on the menu did not ask to leave");
}

#[test]
fn a_secret_is_silent_once_f7_has_turned_the_tower_tense() {
    // `F7` is the session's and lands before the menu sees a key; the menu must
    // read the register on the next keystroke, not on the next tick (§3).
    let answer = |tense: bool| {
        let mut surfaces = surfaces(false, false, false);
        surfaces.menuing = Some(Menu::default());
        let mut sim = Sim::new(1);
        let mut scroll = scroll(false);
        if tense {
            orbs_shell::cycle_register(&mut sim);
        }
        for code in "xyzzy".chars().map(KeyCode::Char).chain([KeyCode::Enter]) {
            surfaces.typed(
                Owner::Menu,
                code,
                &mut sim,
                &mut scroll,
                &orbs_sim::Verbatim,
            );
        }
        surfaces.menuing.as_ref().and_then(Menu::complaint).cloned()
    };
    let calm = answer(false);
    assert!(
        calm.is_some(),
        "the control: the secret was not answered at all"
    );
    assert_ne!(
        answer(true),
        calm,
        "a secret was answered in a tense register",
    );
}

#[test]
fn a_verb_hands_its_surface_over_once_and_not_twice() {
    // The take-once handshake from the frontend's side: the sim raises a
    // flag and reading it clears it, so a second `open` on the same tick
    // must find nothing or a closed surface reopens under the player.
    let mut sim = Sim::new(3);
    sim.submit("attend archive");
    sim.step();
    sim.submit("research");
    sim.step();
    sim.submit("wander");
    sim.step();

    let mut surfaces = Surfaces::default();
    let mut scroll = Scroll::default();
    surfaces.open(
        &mut sim,
        &mut scroll,
        1,
        crate::carried::Carried::default(),
        &orbs_sim::Verbatim,
    );
    assert_eq!(
        surfaces.owner(&scroll),
        Owner::Maze,
        "wander did not hand the keys over"
    );

    surfaces.walking = false;
    surfaces.open(
        &mut sim,
        &mut scroll,
        1,
        crate::carried::Carried::default(),
        &orbs_sim::Verbatim,
    );
    assert_eq!(
        surfaces.owner(&scroll),
        Owner::Prompt,
        "the wander request was still pending after being taken, so the maze reopened",
    );
}

#[test]
fn a_maze_that_closes_takes_the_keyboard_with_it() {
    // §19's worst ending: holding the arrows over a pane with no map on it.
    // A spell can `stop lectern`, and the player is left walking nothing.
    let mut sim = Sim::new(3);
    for line in ["attend archive", "research", "wander"] {
        sim.submit(line);
        sim.step();
    }
    let mut surfaces = Surfaces::default();
    let mut scroll = Scroll::default();
    surfaces.open(
        &mut sim,
        &mut scroll,
        1,
        crate::carried::Carried::default(),
        &orbs_sim::Verbatim,
    );
    assert_eq!(surfaces.owner(&scroll), Owner::Maze);

    sim.submit("stop stacks");
    sim.step();
    surfaces.tick(&sim);
    assert_eq!(
        surfaces.owner(&scroll),
        Owner::Prompt,
        "the stacks closed and the arrows still belonged to the maze",
    );
}
