use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use orbs_render::GridSize;
use orbs_shell::{Boot, Threshold};
use orbs_sim::Sim;

use super::Session;
use crate::carried::Carried;

/// A session on a world with something in its transcript to page through.
fn session(threshold: Threshold) -> Session {
    let mut sim = Sim::new(1);
    for _ in 0..4 {
        sim.submit("status");
        sim.step();
    }
    Session::new(
        sim,
        None,
        Boot::finished(),
        threshold,
        Carried::default(),
        GridSize::new(120, 45),
        true,
        String::new(),
    )
}

/// A session in a tower, moving or still, that has drawn its first frames and
/// woken out of the boot card.
fn in_a_tower(motion: bool) -> Session {
    let mut session = Session::new(
        Sim::new(1),
        None,
        Boot::finished(),
        Threshold::Playing,
        Carried {
            motion,
            ..Carried::default()
        },
        GridSize::new(120, 45),
        true,
        String::new(),
    );
    // Permitted here rather than read, or an exported `ORBS_PASSAGE=0` fails
    // every crossing below.
    session.passing = orbs_shell::Passing::new(true);
    // A second of frames: the wake runs out and the floor between crossings
    // has passed, as in any session a player has been in for a moment.
    for _ in 0..40 {
        frame(&mut session);
    }
    session
}

/// One frame, drawn into nothing.
fn frame(session: &mut Session) {
    session
        .frame(std::time::Duration::from_millis(33), 0.0, &mut Vec::new())
        .expect("a Vec cannot fail");
}

/// Run a line through the world and rebuild what the painters read, as a tick
/// does.
fn run(session: &mut Session, line: &str) {
    session.sim.submit(line);
    session.sim.step();
    session.panel.refresh(&session.sim);
}

#[test]
fn a_moving_session_crosses_to_another_room() {
    let mut session = in_a_tower(true);
    assert!(session.passing.is_settled(), "the wake never ended");
    run(&mut session, "attend forge");
    frame(&mut session);
    assert!(
        !session.passing.is_settled(),
        "the room changed and nothing crossed",
    );
}

#[test]
fn a_still_session_cuts_to_another_room() {
    let mut session = in_a_tower(false);
    run(&mut session, "attend forge");
    frame(&mut session);
    assert!(session.passing.is_settled(), "a still session crossed");
    assert!(
        session.passing.kept().is_empty(),
        "a still session kept a screen it can never cross from",
    );
}

#[test]
fn a_moving_session_rolls_a_gain_and_a_still_one_lands_it() {
    use orbs_shell::Watch;

    for motion in [true, false] {
        let mut session = in_a_tower(motion);
        run(&mut session, "debug_renown 20");
        session
            .frame(std::time::Duration::from_millis(1), 0.0, &mut Vec::new())
            .expect("a Vec cannot fail");
        let truth = session.panel.rankward;
        let rolled = session.climb.toward(Watch::Renown, truth);
        if motion {
            assert!(rolled.plus.is_some(), "a gain rolled with no +N");
        } else {
            assert!(rolled.plus.is_none(), "a still session showed a +N");
            assert_eq!(rolled.shown, truth.done, "a still session rolled");
        }
    }
}

#[test]
fn a_keystroke_puts_the_whole_answer_on_screen() {
    let mut session = in_a_tower(true);
    session.held.pace = orbs_shell::Pace::Baud1200;
    run(&mut session, "recall clarity");
    session
        .frame(std::time::Duration::from_millis(1), 0.0, &mut Vec::new())
        .expect("a Vec cannot fail");
    assert!(!session.reveal.is_settled(), "nothing was typing");
    session.typed(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE));
    assert!(session.reveal.is_settled(), "a keystroke left it typing");
}

#[test]
fn a_still_session_lands_text_whole() {
    let mut session = in_a_tower(false);
    run(&mut session, "recall clarity");
    frame(&mut session);
    assert!(
        session.reveal.is_settled(),
        "text typed itself in a still session"
    );
}

#[test]
fn echoes_put_back_by_the_row_do_not_type_themselves() {
    // The window's `keep_echo`: changing the row moves the drawn count under
    // the reveal, and the echoes it brings back are not a new burst.
    let mut session = in_a_tower(true);
    session.held.echo = true;
    for line in ["status", "survey", "status"] {
        run(&mut session, line);
        for _ in 0..60 {
            frame(&mut session);
        }
    }
    assert!(session.reveal.is_settled());
    session.surfaces.setting = Some((
        orbs_shell::settings::ECHO.to_owned(),
        orbs_shell::settings::ECHOES[0].to_owned(),
    ));
    session.answered();
    frame(&mut session);
    assert!(
        session.reveal.is_settled(),
        "the echoes the row put back typed themselves",
    );
}

#[test]
fn paging_at_the_threshold_moves_nothing_behind_the_menu() {
    // The transcript behind the threshold's menu is a scratch world's. The Bevy
    // build gates paging on `playing`; here it scrolled where nobody could see.
    let mut waiting = session(Threshold::Waiting);
    waiting.typed(KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE));
    assert_eq!(
        waiting.scroll.back(),
        0,
        "PgUp scrolled the scratch transcript"
    );

    // The control: the same key in a tower does page, so the assertion above is
    // about the gate and not about a transcript too short to move.
    let mut playing = session(Threshold::Playing);
    playing.typed(KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE));
    assert!(playing.scroll.back() > 0, "PgUp did not page in a tower");
}
