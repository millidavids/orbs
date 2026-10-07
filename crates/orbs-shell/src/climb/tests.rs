use orbs_render::{FLIP_HZ, Fill, Intensity, PLUS_SECS, ROLL_SECS};
use orbs_sim::{Ahead, Toward};

use super::{Climb, Rolled, Watch};
use crate::glance::Panel;

/// A panel whose ley gauge stands `done` into a tier of `span` ending at
/// `ahead`, with the experience that implies.
fn ley(done: u64, span: u64, ahead: u64) -> Panel {
    Panel {
        station: Toward {
            done,
            span,
            ahead: Ahead::Tier(ahead),
        },
        experience: ahead - span + done,
        ..Panel::default()
    }
}

/// The ley gauge past its last station, at `experience`.
fn topped(experience: u64) -> Panel {
    Panel {
        station: Toward {
            done: 1,
            span: 1,
            ahead: Ahead::Nothing,
        },
        experience,
        ..Panel::default()
    }
}

fn on() -> Climb {
    let mut climb = Climb::default();
    climb.set_enabled(true);
    climb
}

fn rolled(climb: &Climb, panel: &Panel) -> Rolled {
    climb.toward(Watch::Ley, panel.station)
}

fn shown(climb: &Climb, panel: &Panel) -> u64 {
    rolled(climb, panel).shown
}

/// Seconds into a roll that land on each of its three steps.
const STEPS: [f32; 3] = [0.05, 0.2, 0.4];

#[test]
fn first_sight_settles() {
    // A tower opening, loading or swapping in is not a gain.
    let mut climb = on();
    let panel = ley(8, 16, 16);
    climb.observe(&panel);
    assert_eq!(rolled(&climb, &panel), Rolled::settled(8));
}

#[test]
fn a_gain_rolls_and_arrives() {
    let mut climb = on();
    climb.observe(&ley(2, 16, 16));
    let after = ley(14, 16, 16);
    climb.observe(&after);
    assert_eq!(shown(&climb, &after), 2, "it jumped rather than rolled");

    climb.tick(ROLL_SECS / 2.0);
    let partway = shown(&climb, &after);
    assert!(
        partway > 2 && partway < 14,
        "half way through read {partway}"
    );

    climb.tick(ROLL_SECS);
    assert_eq!(shown(&climb, &after), 14, "it never arrived");
    assert_eq!(
        rolled(&climb, &after).from,
        14,
        "an arrived roll kept its start"
    );
}

#[test]
fn an_unmeasured_track_is_not_a_reading_of_nought() {
    // `Toward::default` is what a panel holds before its first refresh. Rolling
    // from it would draw the tower's whole standing as a gain.
    let mut climb = on();
    climb.observe(&Panel::default());
    let panel = ley(9, 16, 16);
    climb.observe(&panel);
    assert_eq!(rolled(&climb, &panel), Rolled::settled(9));
}

#[test]
fn out_and_back_does_not_roll() {
    // A number that left the screen is forgotten, so whatever changed while it
    // was away arrives settled.
    let mut climb = on();
    climb.observe(&ley(3, 16, 16));
    climb.observe(&Panel::default());
    let back = ley(11, 16, 16);
    climb.observe(&back);
    assert_eq!(
        rolled(&climb, &back),
        Rolled::settled(11),
        "coming back was a gain"
    );
}

#[test]
fn a_fall_rolls_down_and_says_no_plus() {
    let renown = |done| Panel {
        rankward: Toward {
            done,
            span: 25,
            ahead: Ahead::Tier(25),
        },
        renown: done,
        ..Panel::default()
    };
    let mut climb = on();
    climb.observe(&renown(20));
    let fallen = renown(4);
    climb.observe(&fallen);
    let at = |climb: &Climb| climb.toward(Watch::Renown, fallen.rankward);
    assert_eq!(at(&climb).shown, 20);
    assert_eq!(at(&climb).plus, None, "a loss was drawn as a gain");
    climb.tick(ROLL_SECS / 2.0);
    let partway = at(&climb).shown;
    assert!(partway < 20 && partway > 4, "half way down read {partway}");
}

#[test]
fn a_fall_clears_a_plus_still_fading() {
    let mut climb = on();
    climb.observe(&ley(2, 16, 16));
    climb.observe(&ley(10, 16, 16));
    climb.tick(ROLL_SECS);
    // Renown is the track that falls, but the rule is the track's, not the name's.
    let mut fell = ley(9, 16, 16);
    fell.experience = 8;
    climb.observe(&fell);
    assert_eq!(rolled(&climb, &fell).plus, None);
}

#[test]
fn off_draws_the_truth() {
    // Reduce-motion cuts; it never freezes a number part way.
    let mut climb = Climb::default();
    climb.observe(&ley(2, 16, 16));
    let after = ley(14, 16, 16);
    climb.observe(&after);
    assert_eq!(rolled(&climb, &after), Rolled::settled(14));

    let mut climb = on();
    climb.observe(&ley(2, 16, 16));
    climb.observe(&after);
    climb.set_enabled(false);
    assert_eq!(
        rolled(&climb, &after),
        Rolled::settled(14),
        "turning off froze it"
    );
}

#[test]
fn a_climb_never_advanced_draws_every_number_settled() {
    // `orbs-tui` and the F5 mirror hold one of these and never move it.
    let climb = Climb::default();
    let panel = ley(7, 16, 16);
    assert_eq!(rolled(&climb, &panel), Rolled::settled(7));
}

#[test]
fn a_change_mid_roll_carries_on_from_what_is_showing() {
    let mut climb = on();
    climb.observe(&ley(0, 400, 400));
    climb.observe(&ley(100, 400, 400));
    climb.tick(ROLL_SECS * 0.4);
    let before = shown(&climb, &ley(100, 400, 400));

    let retargeted = ley(300, 400, 400);
    climb.observe(&retargeted);
    assert_eq!(
        shown(&climb, &retargeted),
        before,
        "the digits jumped when the target moved",
    );
    climb.tick(ROLL_SECS);
    assert_eq!(shown(&climb, &retargeted), 300);
}

#[test]
fn digits_change_no_faster_than_the_flip_rate_however_often_the_truth_moves() {
    // The world moves at 1 Hz, so a retarget needs a hitch. Pushed far past that
    // here: the target moves every 70 ms, and still no two changes are closer
    // than one flip tick.
    let rate = 240.0_f32;
    let frame = 1.0 / rate;
    let mut climb = on();
    let mut truth = 0;
    climb.observe(&ley(truth, 10_000, 10_000));
    let mut last = shown(&climb, &ley(truth, 10_000, 10_000));
    let mut changed_at: Option<f32> = None;
    for n in 1..=480u16 {
        let now = f32::from(n) * frame;
        if n % 17 == 0 {
            truth += 300;
        }
        let panel = ley(truth, 10_000, 10_000);
        climb.observe(&panel);
        climb.tick(frame);
        // The digits only. A `+N` changes with the truth, which is a world event
        // and paced by the world's own clock, the way a cut is.
        let drawn = rolled(&climb, &panel);
        if drawn.shown != last {
            if let Some(before) = changed_at {
                assert!(
                    now - before >= 1.0 / FLIP_HZ - frame,
                    "changed at {before}s and again at {now}s",
                );
            }
            changed_at = Some(now);
            last = drawn.shown;
        }
    }
}

#[test]
fn a_pose_holds_a_roll_part_way() {
    let mut climb = Climb::default();
    climb.observe(&ley(0, 100, 100));
    climb.set_enabled(true);
    let after = ley(90, 100, 100);
    climb.observe(&after);
    climb.pose(0.4);
    let posed = shown(&climb, &after);
    assert!(posed > 0 && posed < 90, "posed at 0.4 read {posed}");
    climb.pose(1.0);
    assert_eq!(shown(&climb, &after), 90);
}

#[test]
fn a_gain_says_how_much_and_fades() {
    let mut climb = on();
    climb.observe(&ley(2, 16, 16));
    let after = ley(10, 16, 16);
    climb.observe(&after);
    assert_eq!(rolled(&climb, &after).plus, Some((8, Intensity::Bright)));
    climb.tick(PLUS_SECS * 0.4);
    assert_eq!(rolled(&climb, &after).plus, Some((8, Intensity::Normal)));
    climb.tick(PLUS_SECS * 0.3);
    assert_eq!(rolled(&climb, &after).plus, Some((8, Intensity::Dim)));
    climb.tick(PLUS_SECS);
    assert_eq!(rolled(&climb, &after).plus, None, "the +N never went");
}

#[test]
fn two_gains_inside_one_roll_say_their_total() {
    let mut climb = on();
    climb.observe(&ley(2, 16, 16));
    climb.observe(&ley(6, 16, 16));
    climb.tick(ROLL_SECS * 0.2);
    let after = ley(9, 16, 16);
    climb.observe(&after);
    assert_eq!(rolled(&climb, &after).plus.map(|(gain, _)| gain), Some(7));
}

#[test]
fn a_gain_after_the_roll_is_news_of_its_own() {
    // Summing for as long as the `+N` lingers would count up for ever under a
    // spell that earns every tick.
    let mut climb = on();
    climb.observe(&ley(2, 16, 16));
    climb.observe(&ley(6, 16, 16));
    climb.tick(ROLL_SECS);
    let after = ley(9, 16, 16);
    climb.observe(&after);
    assert_eq!(rolled(&climb, &after).plus.map(|(gain, _)| gain), Some(3));
}

#[test]
fn a_crossing_holds_arrives_and_lands() {
    // `13/16 → 2/8`: the tier left holds a flip tick, shows arrived, and the
    // bar lands in the tier reached — never a count backwards.
    let mut climb = on();
    climb.observe(&ley(13, 16, 16));
    let crossed = ley(2, 8, 24);
    climb.observe(&crossed);

    let [filling, arriving, holding] = STEPS.map(|at| {
        let mut probe = Climb::default();
        probe.set_enabled(true);
        probe.observe(&ley(13, 16, 16));
        probe.observe(&crossed);
        probe.tick(at);
        rolled(&probe, &crossed)
    });
    // The first flip tick is exactly what was showing: the crossing changes no
    // cell on the frame it is seen, so the rate holds however soon it follows a
    // digit turning.
    assert_eq!((filling.shown, filling.span), (13, Some(16)));
    assert_eq!(
        filling.hue,
        Some(Fill::of(13, 16)),
        "it took a colour of its own"
    );
    for arrived in [arriving, holding] {
        assert_eq!((arrived.shown, arrived.span), (16, Some(16)));
        assert_eq!(arrived.hue, Some(Fill::Whole));
    }
    assert_eq!(filling.plus.map(|(gain, _)| gain), Some(5));

    climb.tick(ROLL_SECS);
    let landed = rolled(&climb, &crossed);
    assert_eq!((landed.shown, landed.span, landed.hue), (2, None, None));
}

#[test]
fn many_tiers_crossed_at_once_are_one_beat() {
    // `debug_renown 400`, or `meditate` over several stations: the tier on
    // screen is the one left, and the tiers between were never drawn.
    let mut climb = on();
    climb.observe(&ley(13, 16, 16));
    let far = ley(30, 200, 400);
    climb.observe(&far);
    climb.tick(STEPS[1]);
    let arrived = rolled(&climb, &far);
    assert_eq!((arrived.shown, arrived.span), (16, Some(16)));
    assert_eq!(
        arrived.plus.map(|(gain, _)| gain),
        Some(400 - 200 + 30 - 13)
    );
}

#[test]
fn crossing_the_last_station_fills_and_stays_full() {
    let mut climb = on();
    climb.observe(&ley(90, 100, 100));
    let top = topped(100);
    climb.observe(&top);
    climb.tick(STEPS[1]);
    assert_eq!(rolled(&climb, &top).hue, Some(Fill::Whole));
    climb.tick(ROLL_SECS);
    let landed = rolled(&climb, &top);
    assert_eq!(
        (landed.shown, landed.span),
        (1, None),
        "a topped bar emptied"
    );
}

#[test]
fn a_topped_track_still_says_what_it_earned() {
    let mut climb = on();
    climb.observe(&topped(10_000));
    let more = topped(10_016);
    climb.observe(&more);
    assert_eq!(rolled(&climb, &more).plus.map(|(gain, _)| gain), Some(16));
    assert_eq!(rolled(&climb, &more).shown, 1, "a topped bar moved");
}

#[test]
fn falling_across_a_tier_cuts() {
    let renown = |done, span, ahead, whole| Panel {
        rankward: Toward {
            done,
            span,
            ahead: Ahead::Tier(ahead),
        },
        renown: whole,
        ..Panel::default()
    };
    let mut climb = on();
    climb.observe(&renown(5, 35, 60, 30));
    let fallen = renown(20, 25, 25, 20);
    climb.observe(&fallen);
    assert_eq!(
        climb.toward(Watch::Renown, fallen.rankward),
        Rolled::settled(20)
    );
}

#[test]
fn a_change_mid_beat_cuts_to_the_truth() {
    let mut climb = on();
    climb.observe(&ley(13, 16, 16));
    climb.observe(&ley(2, 8, 24));
    climb.tick(STEPS[0]);
    let again = ley(5, 8, 24);
    climb.observe(&again);
    let cut = rolled(&climb, &again);
    assert_eq!((cut.shown, cut.span, cut.hue), (5, None, None));
    assert_eq!(
        cut.plus.map(|(gain, _)| gain),
        Some(8),
        "the gains were not summed"
    );
}

/// A panel standing in `room`, whose first ledger row has `count` made.
fn room(id: &str, count: u32, spelled: u32) -> Panel {
    Panel {
        ledger: Some(orbs_sim::tower::LedgerRow {
            id: id.to_owned(),
            domain: Some("laboratory".to_owned()),
            count,
            by_spell: spelled,
        }),
        ..Panel::default()
    }
}

#[test]
fn a_rooms_count_rolls_and_says_how_much() {
    let mut climb = on();
    climb.observe(&room("potions", 40, 10));
    climb.observe(&room("potions", 43, 13));
    let made = climb.count(Watch::Room, "potions", 43);
    assert_eq!(made.shown, 40, "the count jumped");
    assert_eq!(made.plus.map(|(gain, _)| gain), Some(3));
    let spelled = climb.count(Watch::Spells, "potions", 13);
    assert_eq!(spelled.plus.map(|(gain, _)| gain), Some(3));
    climb.tick(ROLL_SECS);
    assert_eq!(climb.count(Watch::Room, "potions", 43).shown, 43);
}

#[test]
fn walking_into_another_room_is_not_a_gain() {
    // Two rooms' counts in one slot: a change of name is a different number.
    let mut climb = on();
    climb.observe(&room("potions", 40, 0));
    climb.observe(&room("scrolls", 3, 0));
    assert_eq!(
        climb.count(Watch::Room, "scrolls", 3),
        Rolled::settled(3),
        "the archive's count rolled up from the laboratory's",
    );
}

#[test]
fn reaching_a_station_does_not_roll_the_road_back() {
    // `5 of 5` becomes `0 of 10` at the next station: a new count, settled.
    let stop = |id: &str, done| orbs_sim::Stop {
        id: id.to_owned(),
        walk: orbs_sim::Walk::Next,
        done,
        needed: 10,
        opens: Vec::new(),
    };
    let road = |stop| Panel {
        line: Some(orbs_sim::Line {
            domain: "laboratory",
            open: true,
            stops: vec![stop],
        }),
        ..Panel::default()
    };
    let mut climb = on();
    climb.observe(&road(stop("laboratory_2", 4)));
    climb.observe(&road(stop("laboratory_2", 5)));
    assert_eq!(
        climb.count(Watch::Road, "laboratory_2", 5).shown,
        4,
        "it did not roll"
    );
    climb.observe(&road(stop("laboratory_4", 0)));
    assert_eq!(
        climb.count(Watch::Road, "laboratory_4", 0),
        Rolled::settled(0)
    );
}
