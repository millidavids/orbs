use orbs_render::{Fill, Frame, GridSize, Intensity, Rect};
use orbs_sim::tower::LedgerRow;
use orbs_sim::{Ahead, Prose, Toward};

use super::row::{LABEL, row};
use super::strip::{BESIDE, KEEP, LEAST, ROWS, paint, split};
use crate::climb::{Climb, Rolled};

/// A number that is not rolling.
const fn still(toward: Toward) -> Rolled {
    Rolled::settled(toward.done)
}

/// A number on its way from `from`, drawn at `shown`.
const fn moving(shown: u64, from: u64) -> Rolled {
    Rolled {
        shown,
        from,
        ..Rolled::settled(0)
    }
}

/// One ley row, `cols` wide, drawn from `rolled`.
fn ley_row(toward: Toward, rolled: Rolled, cols: u16) -> Frame {
    let prose = Prose::builtin();
    let mut frame = Frame::new(GridSize::new(120, 8));
    let mut painter = frame.painter(Rect::new(0, 0, 120, 8));
    row(
        &mut painter,
        Rect::new(0, 0, cols, 1),
        &prose.line("gauge_ley", &[]),
        toward,
        rolled,
        None,
        &prose,
    );
    frame
}

const fn tier(done: u64, span: u64) -> Toward {
    Toward {
        done,
        span,
        ahead: Ahead::Tier(span),
    }
}

#[test]
fn the_gauges_yield_before_the_transcript_does() {
    let roomy = split(Rect::new(1, 1, 60, 20));
    assert_eq!(roomy.area.rows, ROWS);
    assert_eq!(roomy.rest.rows, 18);
    assert_eq!(roomy.rest.row, 3);

    // A short pane keeps its whole body: §9's main window does not yield.
    let short = split(Rect::new(1, 1, 60, KEEP));
    assert!(short.area.is_empty(), "the gauges squeezed a short pane");
    assert_eq!(short.rest.rows, KEEP);

    // ...and so does a narrow one, where the reading could not be read.
    let narrow = split(Rect::new(1, 1, LEAST - 1, 20));
    assert!(narrow.area.is_empty(), "the gauges drew in a narrow pane");
}

/// `KEEP` means what it says, on the row either side of the edge. The guard
/// was `body.rows <= KEEP`, which is exact for `road::split` because the road
/// takes one row; these take two, so a body of `KEEP + 1` kept `KEEP - 1`. What
/// needs testing is not that a short pane bails but that the *first* pane the
/// gauges draw in leaves the floor standing.
#[test]
fn the_body_keeps_its_floor_on_the_first_row_the_gauges_draw_in() {
    for rows in 0..=KEEP + ROWS - 1 {
        let split = split(Rect::new(1, 1, 60, rows));
        assert!(
            split.area.is_empty(),
            "the gauges drew in a {rows}-row body and left {}, under {KEEP}",
            split.rest.rows,
        );
        assert_eq!(split.rest.rows, rows, "a bailed split kept the body");
    }

    let least = split(Rect::new(1, 1, 60, KEEP + ROWS));
    assert_eq!(least.area.rows, ROWS, "the gauges never draw at all");
    assert_eq!(least.rest.rows, KEEP, "the body kept less than its floor");
}

/// A row too narrow to draw a bar still says what the bar would have (§14).
/// `Painter::gauge` keeps that by pushing its utterance ahead of its own clip
/// test, and this row defeated it by returning before calling it — so a reader
/// lost both standings and a sighted player got an orphan label. Reachable at
/// the endgame reading in a `LEAST`-wide pane.
#[test]
fn a_row_with_no_room_for_a_bar_still_speaks_its_reading() {
    let prose = Prose::builtin();
    let toward = tier(3, 16);

    for cols in [LEAST, LABEL + 4, LABEL, 0] {
        let mut frame = Frame::new(GridSize::new(80, 8));
        let mut painter = frame.painter(Rect::new(0, 0, 80, 8));
        row(
            &mut painter,
            Rect::new(0, 0, cols, 1),
            &prose.line("gauge_ley", &[]),
            toward,
            still(toward),
            // The longest tail there is: the endgame reading and a rank.
            Some("archmage"),
            &prose,
        );
        assert!(
            frame
                .speech()
                .utterances()
                .any(|utterance| utterance.text.contains("3 of 16")),
            "a {cols}-cell row said nothing about where the tower stands",
        );
    }

    // ...and the label does not draw alone when the bar could not.
    let frame = ley_row(toward, still(toward), LABEL);
    assert!(
        !frame.to_text().contains("ley line"),
        "a bar-less row drew an orphan label:\n{}",
        frame.to_text(),
    );
}

/// A track with nothing left to reach must not speak of a next tier.
/// `Toward::FULL` is `1/1` because that fills the bar, and the sentence read
/// those numbers anyway — so the screen said `nothing more authored` while a
/// listener was told the tower stood one step from a tier that does not exist,
/// the two halves of §14's one stream disagreeing.
#[test]
fn a_topped_gauge_speaks_what_it_draws() {
    let topped = Toward {
        done: 1,
        span: 1,
        ahead: Ahead::Nothing,
    };
    let frame = ley_row(topped, still(topped), 80);

    let said = frame.speech().to_transcript();
    assert!(
        said.contains("nothing more authored"),
        "a topped gauge did not say it had topped: {said}",
    );
    assert!(
        !said.contains("toward the next"),
        "a topped gauge spoke of a tier that is not authored: {said}",
    );
    assert!(
        frame.to_text().contains("nothing more authored"),
        "the row and its speech disagree:\n{}",
        frame.to_text(),
    );
}

/// An unmeasured track says nothing rather than saying it has finished.
/// `Toward::default` is what a `Panel` holds before its first refresh and used
/// to be indistinguishable from a topped track — both `at: None` — so the
/// emptiest tower drew as the most finished one. Never on screen, because every
/// frontend refreshes before it paints; one forgotten refresh was all it would
/// have taken.
#[test]
fn an_unmeasured_track_draws_and_says_nothing() {
    let frame = ley_row(Toward::default(), still(Toward::default()), 80);
    assert_eq!(
        frame.to_text().trim(),
        "",
        "an unasked gauge drew a reading"
    );
    assert!(
        frame.speech().to_transcript().is_empty(),
        "an unasked gauge spoke: {}",
        frame.speech().to_transcript(),
    );
}

/// The title yields to the bar, which is what the doc always promised. The rank
/// was appended to the reading and the whole tail charged to the bar, so
/// `nothing more authored  remembered` — 33 cells against the 25 a `LEAST`-wide
/// pane leaves — drew no row at all.
#[test]
fn a_long_title_yields_before_the_bar_does() {
    let prose = Prose::builtin();
    let topped = Toward {
        done: 1,
        span: 1,
        ahead: Ahead::Nothing,
    };

    let mut frame = Frame::new(GridSize::new(80, 8));
    let mut painter = frame.painter(Rect::new(0, 0, 80, 8));
    row(
        &mut painter,
        Rect::new(0, 0, LEAST, 1),
        &prose.line("gauge_renown", &[]),
        topped,
        still(topped),
        Some("remembered"),
        &prose,
    );

    let drawn = frame.to_text();
    assert!(
        drawn.contains('['),
        "the bar was starved by the title:\n{drawn}"
    );
    assert!(
        !drawn.contains("remembered"),
        "the title was kept at the bar's expense:\n{drawn}",
    );

    // ...and where both fit, the title is still drawn.
    let mut roomy = Frame::new(GridSize::new(80, 8));
    let mut painter = roomy.painter(Rect::new(0, 0, 80, 8));
    row(
        &mut painter,
        Rect::new(0, 0, 80, 1),
        &prose.line("gauge_renown", &[]),
        topped,
        still(topped),
        Some("remembered"),
        &prose,
    );
    assert!(
        roomy.to_text().contains("remembered"),
        "a wide pane dropped a title that fitted:\n{}",
        roomy.to_text(),
    );
}

/// §14: a listener hears the settled screen, never a number on its way.
#[test]
fn a_rolling_row_says_exactly_what_the_settled_row_says() {
    let truth = tier(12, 16);
    let settled = ley_row(truth, still(truth), 80);
    for shown in [3, 7, 11] {
        let rolling = ley_row(truth, moving(shown, 3), 80);
        assert_eq!(
            rolling.speech().to_transcript(),
            settled.speech().to_transcript(),
            "a row at {shown} on its way to 12 spoke differently",
        );
        assert!(
            rolling.to_text().contains(&format!("{shown}/16")),
            "the row did not draw {shown}:\n{}",
            rolling.to_text(),
        );
    }
}

/// The bar's colour is the truth's: a rolling length coloured by itself would
/// turn every filled cell over together on each step.
#[test]
fn a_rolling_bar_wears_the_truths_colour() {
    let truth = tier(15, 16);
    let settled = ley_row(truth, still(truth), 80);
    let rolling = ley_row(truth, moving(2, 2), 80);
    let hue = |frame: &Frame| {
        frame
            .row(0)
            .into_iter()
            .flatten()
            .find(|cell| cell.glyph == '|')
            .map(|cell| cell.style.depicted())
    };
    assert!(hue(&rolling).is_some(), "the rolling bar drew no fill");
    assert_eq!(hue(&rolling), hue(&settled), "the fill took its own colour");
}

/// Every width is the truth's, so no roll moves the bar — in a pane wide
/// enough that the bar is capped, and in one narrow enough that it is not.
/// A rise keeps the slash still too; a fall's extra digit spills right.
#[test]
fn a_roll_never_moves_the_bar() {
    let marks = |frame: &Frame| {
        let text = frame.to_text();
        let line = text.lines().next().unwrap_or_default().to_owned();
        (line.find('['), line.find(']'), line.find('/'))
    };
    for cols in [80, LEAST] {
        // A rise from nine to ten, and a fall from ten to nine.
        for (from, to) in [(9, 10), (10, 9)] {
            let truth = tier(to, 16);
            let (open, close, slash) = marks(&ley_row(truth, moving(from, from), cols));
            let settled = marks(&ley_row(truth, still(truth), cols));
            assert_eq!(
                (open, close),
                (settled.0, settled.1),
                "{from} -> {to} moved the bar at {cols}"
            );
            if from < to {
                assert_eq!(slash, settled.2, "a rise moved the slash at {cols}");
            }
        }
    }
}

/// A spell's first hand in a room's work widens nothing: the column is measured
/// with its share's label before there is a share to draw.
#[test]
fn a_spells_first_share_moves_no_bar() {
    let climb = Climb::default();
    for cols in [80, 100] {
        let alone = strip(cols, Some(&potions(42, 0)), &climb);
        let shared = strip(cols, Some(&potions(42, 1)), &climb);
        assert_eq!(bar_ends(&alone), bar_ends(&shared), "at {cols}");
        let column = |frame: &Frame| {
            frame
                .to_text()
                .lines()
                .next()
                .map(|line| line.find("potions"))
        };
        assert_eq!(
            column(&alone),
            column(&shared),
            "the column moved at {cols}"
        );
    }
}

/// A gain's `+N` sits after the reading and is never spoken: renown's earning is
/// silent, and a run's own sentence already says what it paid.
#[test]
fn a_plus_is_drawn_after_the_reading_and_says_nothing() {
    let truth = tier(10, 16);
    let settled = ley_row(truth, still(truth), 80);
    let plus = Rolled {
        plus: Some((8, Intensity::Bright)),
        ..moving(4, 2)
    };
    let gained = ley_row(truth, plus, 80);

    let line = gained
        .to_text()
        .lines()
        .next()
        .unwrap_or_default()
        .to_owned();
    let reading = line.find("/16").unwrap_or(usize::MAX);
    let mark = line.find("+8").unwrap_or(0);
    assert!(mark > reading, "the +N was not after the reading: {line}");
    assert_eq!(
        gained.speech().to_transcript(),
        settled.speech().to_transcript(),
        "the +N was spoken",
    );
}

/// A `+N` with no room is dropped, never squeezed into the bar.
#[test]
fn a_plus_with_no_room_is_dropped() {
    let truth = tier(10, 16);
    let plus = Rolled {
        plus: Some((80_000, Intensity::Bright)),
        ..still(truth)
    };
    let narrow = ley_row(truth, plus, LEAST);
    assert!(!narrow.to_text().contains("+80000"), "{}", narrow.to_text());
    assert_eq!(
        narrow.to_text(),
        ley_row(truth, still(truth), LEAST).to_text(),
        "a +N that did not fit moved something",
    );
}

/// A crossing draws the tier it is leaving — its digits, its span and, once
/// arrived, the ramp's own green — inside the width the truth will need.
#[test]
fn a_crossing_draws_the_tier_it_leaves() {
    let truth = tier(2, 8);
    let arrived = Rolled {
        span: Some(16),
        hue: Some(Fill::Whole),
        ..moving(16, 13)
    };
    let frame = ley_row(truth, arrived, 80);
    let text = frame.to_text();
    assert!(text.contains("16/16"), "the arrival was not drawn: {text}");
    let fill = frame
        .row(0)
        .into_iter()
        .flatten()
        .find(|cell| cell.glyph == '|')
        .map(|cell| cell.style.depicted());
    assert_eq!(fill, Some(orbs_render::Depiction::gauge(Fill::Whole)));
    assert!(
        frame.speech().to_transcript().contains("2 of 8"),
        "a crossing spoke the tier it left",
    );
}

/// Into the top of the track, the tier left is still drawn in digits — and the
/// reading's width is the truth's, so the bar does not move as it lands.
#[test]
fn crossing_the_last_station_draws_digits_until_it_lands() {
    let truth = Toward {
        done: 1,
        span: 1,
        ahead: Ahead::Nothing,
    };
    let arrived = Rolled {
        span: Some(16),
        hue: Some(Fill::Whole),
        ..moving(16, 13)
    };
    let during = ley_row(truth, arrived, 80).to_text();
    let after = ley_row(truth, still(truth), 80).to_text();
    assert!(during.contains("16/16"), "{during}");
    let bracket = |text: &str| text.lines().next().and_then(|line| line.find(']'));
    assert_eq!(
        bracket(&during),
        bracket(&after),
        "the bar moved as it landed"
    );
}

/// The laboratory's row, `count` potions with `spelled` of them a spell's.
fn potions(count: u32, spelled: u32) -> LedgerRow {
    LedgerRow {
        id: "potions".to_owned(),
        domain: Some("laboratory".to_owned()),
        count,
        by_spell: spelled,
    }
}

/// The whole strip, `cols` wide, with a rank and maybe a room's row.
fn strip(cols: u16, room: Option<&LedgerRow>, climb: &Climb) -> Frame {
    let prose = Prose::builtin();
    let mut frame = Frame::new(GridSize::new(cols, 2));
    let mut painter = frame.painter(Rect::new(0, 0, cols, 2));
    paint(
        &mut painter,
        Rect::new(0, 0, cols, 2),
        tier(8, 16),
        Toward {
            done: 40,
            span: 75,
            ahead: Ahead::Tier(135),
        },
        Some("cunning_man"),
        room,
        climb,
        &prose,
    );
    frame
}

/// Where the bars end, row by row.
fn bar_ends(frame: &Frame) -> Vec<Option<usize>> {
    frame.to_text().lines().map(|line| line.find(']')).collect()
}

#[test]
fn a_room_counts_what_it_made_at_the_strips_edge() {
    let climb = Climb::default();
    let row = potions(42, 30);
    let frame = strip(100, Some(&row), &climb);
    let text = frame.to_text();
    let lines: Vec<&str> = text.lines().collect();
    // Two readings, counts in one column, a slot kept after them for a `+N`.
    let end = |line: &str, count: &str| line.find(count).map(|at| at + count.len());
    assert!(lines[0].contains("potions brewed"), "{text}");
    assert!(lines[1].contains("your spells' share"), "{text}");
    assert_eq!(end(lines[0], "42"), end(lines[1], "30"), "{text}");
    assert_eq!(
        lines[0].chars().count() - end(lines[0], "42").unwrap_or(0),
        5,
        "no slot was kept for a +N: {text}",
    );
}

#[test]
fn the_column_is_silent_and_costs_the_gauges_nothing_they_need() {
    // Silent, the rail's precedent: `status` is the spoken whole.
    let climb = Climb::default();
    let row = potions(42, 30);
    let with = strip(100, Some(&row), &climb);
    let without = strip(100, None, &climb);
    assert_eq!(
        with.speech().to_transcript(),
        without.speech().to_transcript(),
        "the column spoke",
    );
    // The bars keep at least `BESIDE` beside it, titles and all.
    for (line, end) in with.to_text().lines().zip(bar_ends(&with)) {
        let start = line.find('[').unwrap_or(0);
        assert!(
            end.unwrap_or(0) - start + 1 >= usize::from(BESIDE),
            "a bar shrank past {BESIDE}: {line}",
        );
    }
    assert!(
        with.to_text().contains("cunning man"),
        "the title was dropped"
    );
}

#[test]
fn a_strip_too_narrow_for_both_drops_the_column_whole() {
    let climb = Climb::default();
    let row = potions(42, 30);
    for cols in [LEAST, 60, 70] {
        let with = strip(cols, Some(&row), &climb);
        let without = strip(cols, None, &climb);
        assert_eq!(
            with.to_text(),
            without.to_text(),
            "a {cols}-wide strip squeezed the column in",
        );
    }
}

#[test]
fn a_gauges_plus_never_reaches_the_column() {
    let mut climb = Climb::default();
    climb.set_enabled(true);
    let before = crate::glance::Panel {
        station: tier(0, 16),
        ..crate::glance::Panel::default()
    };
    climb.observe(&before);
    let after = crate::glance::Panel {
        station: tier(8, 16),
        experience: 80_000,
        ..crate::glance::Panel::default()
    };
    climb.observe(&after);
    let row = potions(42, 0);
    let frame = strip(100, Some(&row), &climb);
    let line = frame
        .to_text()
        .lines()
        .next()
        .unwrap_or_default()
        .to_owned();
    let column = line.find("potions").unwrap_or(usize::MAX);
    if let Some(plus) = line.find("+80000") {
        assert!(plus + 6 < column, "the +N ran into the column: {line}");
    }
    assert!(
        line.contains("potions brewed") && line.trim_end().ends_with("42"),
        "{line}"
    );
}
