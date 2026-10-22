use orbs_render::{Frame, Passage};

use super::clock::{DURATION, FLOOR};
use super::{Passing, Showing};
use crate::focus::Open;
use crate::glance::Panel;

fn panel(room: &str) -> Panel {
    Panel {
        room: room.to_owned(),
        ..Panel::default()
    }
}

fn showing(room: &str) -> Showing {
    Showing::of(Open::default(), &panel(room), false)
}

fn frame() -> Frame {
    Frame::new(orbs_render::GridSize::new(4, 2))
}

/// A `Passing` with a screen already kept and the floor already elapsed —
/// what every frame of an ordinary session looks like.
fn ready() -> Passing {
    let mut passing = Passing::new(true);
    passing.set_enabled(true);
    passing.pose_kept(&frame());
    passing.advance(FLOOR, None);
    passing
}

/// The shape a crossing is running by, if one is.
fn shape(passing: &Passing) -> Option<Passage> {
    passing.crossing().map(|crossing| crossing.passage)
}

#[test]
fn driving_wakes_once_and_crosses_on_a_room_change() {
    // The window's two systems as one call: the switch first, then wake,
    // advance and observe. Both builds drive it this way. Permitted here
    // rather than read, or an exported `ORBS_PASSAGE=0` fails it.
    let mut passing = Passing::new(true);
    passing.advance(0.0, Some(true));
    passing.drive(0.0, &showing("laboratory"));
    assert!(passing.is_waking(), "the first live frame did not wake");
    passing.drive(DURATION, &showing("laboratory"));
    assert!(passing.is_settled(), "the wake did not end");

    passing.keep(&frame());
    passing.drive(FLOOR, &showing("forge"));
    assert!(!passing.is_settled(), "a room change did not cross");
    assert!(!passing.is_waking(), "a room change moved the rail");
}

#[test]
fn a_still_drive_never_crosses_and_keeps_nothing() {
    let mut passing = Passing::new(true);
    passing.advance(0.0, Some(false));
    passing.drive(0.0, &showing("laboratory"));
    assert!(passing.is_settled(), "a still session woke with a crossing");
    passing.keep(&frame());
    passing.drive(FLOOR, &showing("forge"));
    assert!(passing.is_settled(), "a still session crossed");
    assert!(passing.kept().is_empty(), "a still session kept a screen");
}

#[test]
fn the_first_showing_settles() {
    // Otherwise the frame after boot crosses out of nothing into the
    // laboratory, and every `dumps.sh` baseline in the repository moves.
    let mut passing = ready();
    passing.observe(&showing("laboratory"));
    assert!(passing.is_settled());
    assert!(passing.crossing().is_none());
}

#[test]
fn a_new_room_crosses() {
    let mut passing = ready();
    passing.observe(&showing("laboratory"));
    passing.observe(&showing("forge"));
    assert!(!passing.is_settled());
    // The shape is the whole of what this resource decides; which regions
    // move is the painter's, in `prompt::Crossed`.
    assert_eq!(shape(&passing), Some(Passage::Wipe));
}

#[test]
fn each_kind_of_change_has_its_own_shape() {
    // §19's *"tempo is the organising principle"*: what is running should be
    // legible from across the room. Three kinds, three motions, asserted
    // rather than left to the one place that writes it.
    let tool = Showing::of(
        Open {
            walking: true,
            ..Open::default()
        },
        &panel("archive"),
        false,
    );
    let mirror = Showing::of(Open::default(), &panel("archive"), true);
    for (next, expected) in [
        (showing("forge"), Passage::Wipe),
        (tool, Passage::Gather),
        (mirror, Passage::Furl),
    ] {
        let mut passing = ready();
        passing.observe(&showing("archive"));
        passing.observe(&next);
        assert_eq!(shape(&passing), Some(expected), "for {next:?}");
    }
}

#[test]
fn a_tool_outranks_a_room_when_both_move() {
    // A spell can walk and then open a maze inside one tick, so both fields
    // differ at the next observation. Sparing the transcript there would draw
    // a crossing around a hole with nothing left in it.
    let mut passing = ready();
    passing.observe(&showing("laboratory"));
    passing.observe(&Showing::of(
        Open {
            walking: true,
            ..Open::default()
        },
        &panel("archive"),
        false,
    ));
    assert_eq!(shape(&passing), Some(Passage::Gather));
}

#[test]
fn a_fixture_inside_the_room_is_not_a_crossing() {
    // `attend alembic` from the laboratory changes the *leaf* and not the
    // room, and `Panel::room` is what makes the two different questions.
    let mut passing = ready();
    passing.observe(&showing("laboratory"));
    passing.observe(&showing("laboratory"));
    assert!(passing.is_settled());
}

#[test]
fn pgup_is_not_a_crossing() {
    // It does not take the pane: the transcript scrolling back replaces
    // nothing at all.
    let open = Open {
        reading: true,
        ..Open::default()
    };
    let mut passing = ready();
    passing.observe(&showing("archive"));
    passing.observe(&Showing::of(open, &panel("archive"), false));
    assert!(passing.is_settled(), "{open:?} started a crossing");
}

#[test]
fn a_tool_takes_the_whole_pane() {
    let mut passing = ready();
    passing.observe(&showing("archive"));
    passing.observe(&Showing::of(
        Open {
            walking: true,
            ..Open::default()
        },
        &panel("archive"),
        false,
    ));
    assert!(!passing.is_settled());
    assert_eq!(
        shape(&passing),
        Some(Passage::Gather),
        "the maze flies into the middle rather than wiping",
    );
}

#[test]
fn the_mirror_crosses_and_furls() {
    // `F5` is not a `Focus` variant, so without `Showing::mirrored` the one
    // surface that replaces the pane without taking the keyboard would never
    // cross at all.
    let mut passing = ready();
    passing.observe(&showing("lens"));
    passing.observe(&Showing::of(Open::default(), &panel("lens"), true));
    assert!(!passing.is_settled());
    assert_eq!(shape(&passing), Some(Passage::Furl));
}

#[test]
fn a_crossing_cannot_begin_within_a_tick_of_the_last() {
    // The photosensitivity floor, asserted rather than argued. A held key or
    // a mashed `F5` would otherwise strobe the pane at four crossings a
    // second, and §19 counts each crossing as one flash.
    let mut passing = ready();
    passing.observe(&showing("laboratory"));
    passing.observe(&showing("forge"));
    assert!(!passing.is_settled());

    // The crossing finishes, but the floor has not.
    passing.advance(DURATION, None);
    assert!(passing.is_settled());
    passing.observe(&showing("lens"));
    assert!(
        passing.is_settled(),
        "a second crossing began inside the floor"
    );

    // ...and once the tick is up it may.
    passing.advance(FLOOR, None);
    passing.observe(&showing("sanctum"));
    assert!(!passing.is_settled());
}

#[test]
fn a_change_mid_crossing_does_not_restart_it() {
    // The arriving half draws whatever is live, so a second change needs no
    // second crossing — and restarting would double the flash count that the
    // floor exists to bound.
    let mut passing = ready();
    passing.observe(&showing("laboratory"));
    passing.observe(&showing("forge"));
    passing.advance(DURATION / 2.0, None);
    let midway = passing.crossing().map(|crossing| crossing.progress);

    passing.observe(&showing("lens"));
    let after = passing.crossing().map(|crossing| crossing.progress);
    assert_eq!(midway, after, "the crossing restarted");
}

#[test]
fn motion_off_keeps_nothing_and_starts_nothing() {
    // §14's requirement, and the `DESIGN.md:9182` trap with it: reduce-motion
    // must *cut*, never freeze a half-drawn screen. With nothing kept there
    // is no crossing to freeze.
    let mut passing = ready();
    passing.observe(&showing("laboratory"));
    passing.advance(0.0, Some(false));
    assert!(
        passing.kept().is_empty(),
        "a kept screen survived the switch"
    );

    passing.keep(&frame());
    assert!(
        passing.kept().is_empty(),
        "it kept a screen with motion off"
    );

    passing.observe(&showing("forge"));
    assert!(passing.is_settled(), "it crossed with motion off");
}

#[test]
fn a_missing_switch_is_not_a_switch_set_to_off() {
    // A caller with no switch to hand over passes `None`. Defaulting to off
    // there would make the crossing invisible for a reason nobody asked for —
    // the defect `shell::motion` records for the fire.
    let mut passing = ready();
    passing.advance(0.0, None);
    passing.observe(&showing("laboratory"));
    passing.observe(&showing("forge"));
    assert!(!passing.is_settled());
}

#[test]
fn keeping_is_refused_while_a_crossing_runs() {
    // Or it overwrites its own source with its own output and eats itself.
    let mut passing = ready();
    passing.observe(&showing("laboratory"));
    passing.observe(&showing("forge"));

    let mut frame = frame();
    let whole = frame.area();
    frame
        .painter(whole)
        .fill(whole, 'x', orbs_render::Style::NORMAL);
    passing.keep(&frame);

    assert!(
        passing.kept().cell(orbs_render::Pos::ORIGIN).is_blank(),
        "the crossing overwrote what it was departing from",
    );
}

#[test]
fn a_crossing_ends_at_its_duration() {
    let mut passing = ready();
    passing.observe(&showing("laboratory"));
    passing.observe(&showing("forge"));
    passing.advance(DURATION - 0.01, None);
    assert!(!passing.is_settled());
    passing.advance(0.02, None);
    assert!(passing.is_settled());
    assert!(passing.crossing().is_none());
}
