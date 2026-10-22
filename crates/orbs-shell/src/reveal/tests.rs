use orbs_render::{FieldName, Outcome, RecordKind, Records};

use super::burst::{LONGEST, tail_cells};
use super::{Pace, Reveal};

/// A reveal that has had its first look, at an empty stream.
fn watching() -> Reveal {
    let mut reveal = Reveal::default();
    reveal.observe(0, 0);
    reveal
}

/// A burst of `cells` characters, just arrived.
fn burst(cells: u16) -> Reveal {
    let mut reveal = watching();
    reveal.observe(1, cells);
    reveal
}

#[test]
fn what_was_on_screen_before_the_first_look_is_not_news() {
    // A tower opened from the menu typed its whole history out again.
    let mut reveal = Reveal::default();
    reveal.observe(12, 400);
    assert!(reveal.is_settled(), "the history typed itself");
    reveal.observe(13, 20);
    assert!(!reveal.is_settled(), "new output after it did not type");
    assert_eq!(reveal.from, 12);
}

/// A stream with one command and its answer in it.
fn answered() -> Records {
    let mut records = Records::new();
    records
        .push(RecordKind::Input)
        .text(FieldName::Message, "recall clarity")
        .finish();
    records
        .push(RecordKind::Entry)
        .text(FieldName::Name, "a potion of clear sight")
        .finish();
    records
}

#[test]
fn a_still_drive_lands_whole_and_a_moving_one_types() {
    let records = answered();
    let mut still = watching();
    still.drive(&records, 1.0 / 60.0, Some(false), Pace::Baud1200);
    assert!(still.is_settled(), "text typed itself with nothing to move");

    let mut moving = watching();
    moving.drive(&records, 1.0 / 60.0, Some(true), Pace::Baud1200);
    assert!(!moving.is_settled(), "nothing typed with motion on");
}

#[test]
fn a_burst_counts_only_what_is_drawn_after_it_began() {
    let mut records = Records::new();
    records.set_terse_echo(true);
    for line in ["brew", "kindle", "look"] {
        records
            .push(RecordKind::Input)
            .text(FieldName::Message, line)
            .finish();
        records
            .push(RecordKind::Echo)
            .outcome(Outcome::Resolved)
            .text(FieldName::Message, line)
            .finish();
    }
    let from = records.drawn_len();
    records
        .push(RecordKind::Entry)
        .text(FieldName::Name, "athanor")
        .finish();
    let new = records.iter().last().expect("pushed").to_line();
    // Three hidden echoes before the burst must not be counted into it.
    assert_eq!(usize::from(tail_cells(&records, from)), new.chars().count());
}

#[test]
fn nothing_to_reveal_is_already_settled() {
    // Most frames have no new output: they must cost nothing.
    assert!(Reveal::default().is_settled());
    assert_eq!(Reveal::default().budget(0), None);
}

#[test]
fn the_first_character_is_available_on_the_frame_the_record_arrives() {
    // §19: a terminal that takes a second to answer reads as broken. The
    // reveal may take time; the response may not.
    let mut reveal = burst(40);
    reveal.advance(1.0 / 60.0);
    let (_, cells) = reveal.budget(0).expect("mid-reveal");
    assert!(cells >= 1, "nothing had arrived after a frame");
}

#[test]
fn a_long_burst_still_finishes_inside_the_ceiling() {
    // The rate rises to fit rather than the reveal being truncated.
    let mut reveal = burst(u16::MAX);
    let mut elapsed = 0.0;
    while !reveal.is_settled() && elapsed < 10.0 {
        reveal.advance(1.0 / 60.0);
        elapsed += 1.0 / 60.0;
    }
    assert!(reveal.is_settled(), "it never finished");
    assert!(
        elapsed <= LONGEST + 0.1,
        "{elapsed}s is a wall, not flavour"
    );
}

#[test]
fn a_slower_pace_has_less_on_screen_and_instant_has_all_of_it() {
    let shown = |pace| {
        let mut reveal = burst(200);
        reveal.advance_at(0.1, pace);
        reveal.budget(0).map_or(200, |(_, cells)| cells)
    };
    assert!(shown(Pace::Baud1200) < shown(Pace::Baud4800));
    assert!(shown(Pace::Baud4800) < shown(Pace::Baud9600));
    assert_eq!(shown(Pace::Instant), 200, "instant still typed");
    // The ceiling holds at every pace: a slow pace never makes a log a wall.
    let mut reveal = burst(u16::MAX);
    let mut elapsed = 0.0;
    while !reveal.is_settled() && elapsed < 10.0 {
        reveal.advance_at(1.0 / 60.0, Pace::Baud1200);
        elapsed += 1.0 / 60.0;
    }
    assert!(elapsed <= LONGEST + 0.1, "{elapsed}s at 1200 baud");
}

#[test]
fn a_pace_is_its_own_word() {
    for pace in Pace::ALL {
        assert_eq!(Pace::named(pace.word()), Some(pace));
    }
    assert_eq!(Pace::named("300"), None);
}

#[test]
fn a_short_burst_still_types_itself() {
    // The other end: the ceiling must not make a one-line echo instant.
    let mut reveal = burst(20);
    reveal.advance(1.0 / 60.0);
    assert!(!reveal.is_settled(), "a short line arrived all at once");
}

#[test]
fn a_keystroke_finishes_it() {
    let mut reveal = burst(500);
    reveal.advance(1.0 / 60.0);
    assert!(!reveal.is_settled());
    reveal.finish();
    assert!(reveal.is_settled());
    assert_eq!(reveal.budget(0), None);
}

#[test]
fn a_second_burst_abandons_the_first_rather_than_queueing_behind_it() {
    // Queueing would let a fast player build a backlog, and the screen
    // would fall further behind the world with every command.
    let mut reveal = watching();
    reveal.observe(3, 300);
    reveal.advance(0.05);
    reveal.observe(5, 40);

    assert_eq!(reveal.total, 40, "the new burst inherited the old length");
    assert_eq!(reveal.shown, 0, "the new burst started part-way through");
    assert_eq!(reveal.from, 3, "the first burst's records are not settled");
}

#[test]
fn records_put_back_on_screen_do_not_type_themselves() {
    let mut reveal = watching();
    reveal.observe(10, 100);
    reveal.finish();
    reveal.settle_at(16);
    reveal.observe(16, 60);
    assert!(reveal.is_settled(), "the echoes put back typed again");
}

#[test]
fn a_shrinking_stream_resyncs_instead_of_revealing_from_a_stale_index() {
    let mut reveal = watching();
    reveal.observe(10, 100);
    reveal.finish();
    reveal.observe(2, 0);
    assert!(reveal.is_settled());

    reveal.observe(4, 30);
    assert_eq!(reveal.from, 2, "the burst began from a record that is gone");
}

#[test]
fn the_budget_is_relative_to_what_the_pane_actually_draws() {
    // The pane skips the head of the stream, and the view indexes from the
    // first record it is handed, not the first that exists.
    let mut reveal = watching();
    // A settled session of 14 records, then a burst adding six more.
    reveal.observe(14, 200);
    reveal.finish();
    reveal.observe(20, 50);
    reveal.advance(0.01);

    // The pane shows the last 12, so the burst begins at drawn record 2.
    let (after, _) = reveal.budget(12).expect("mid-reveal");
    assert_eq!(after, 2, "the reveal started at the wrong drawn record");
    // The pane skipped past the burst's start: everything drawn is new.
    let (after, _) = reveal.budget(99).expect("mid-reveal");
    assert_eq!(
        after, 0,
        "skipping past the burst should reveal from the top"
    );
}
