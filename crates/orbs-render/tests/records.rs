//! The record model, exercised through the public API a command would use.
//!
//! Unit tests live beside each module. These are the ones that only mean
//! something end to end: a record emitted, drawn, spoken, and sifted, with the
//! four views checked against each other.

use orbs_render::{
    FieldName, Frame, GridSize, Outcome, Presentation, RecordKind, RecordView, Records, Rect, Role,
    Sift, Style, UtteranceKind, Value,
};

/// The alembic, mid-brew. Two reagents and a spoiled one.
fn alembic() -> Records {
    let mut records = Records::new();
    records
        .push(RecordKind::Entry)
        .text(FieldName::Name, "sage")
        .text(FieldName::State, "ready")
        .count(FieldName::Quantity, 3)
        .finish();
    records
        .push(RecordKind::Entry)
        .text(FieldName::Name, "nightshade")
        .text(FieldName::State, "spoiled")
        .count(FieldName::Quantity, 120)
        .role(Role::Danger)
        .finish();
    records
}

fn frame_of(cols: u16, rows: u16) -> Frame {
    Frame::new(GridSize { cols, rows })
}

// ---------------------------------------------------------------------------
// The model
// ---------------------------------------------------------------------------

#[test]
fn fields_come_back_named_and_typed() {
    let records = alembic();
    let sage = records.get(0).expect("first record");

    assert_eq!(sage.kind(), RecordKind::Entry);
    assert_eq!(sage.field(FieldName::Name), Some(Value::Text("sage")));
    assert_eq!(sage.field(FieldName::Quantity), Some(Value::Count(3)));
    assert_eq!(sage.field(FieldName::Path), None);

    // The quantity is still a number, not the string "3". Everything a view or
    // the balance harness wants to do with it depends on that.
    assert!(
        sage.field(FieldName::Quantity)
            .expect("quantity")
            .is_numeric()
    );
}

#[test]
fn a_single_field_speaks_as_itself() {
    let mut records = Records::new();
    records
        .push(RecordKind::Message)
        .text(FieldName::Message, "the ward has failed")
        .role(Role::Danger)
        .finish();

    // Not "message: the ward has failed". Prose must read as prose.
    assert_eq!(
        records.get(0).expect("record").to_speech(),
        "the ward has failed"
    );
}

#[test]
fn many_fields_speak_as_label_value_pairs() {
    // §14: a reader must never have to reconstruct columns from spacing.
    assert_eq!(
        alembic().get(1).expect("record").to_speech(),
        "name: nightshade, state: spoiled, qty: 120",
    );
}

#[test]
fn the_kind_decides_whether_labels_are_spoken_not_the_field_count() {
    // §14 requires labels for *tables* — "a reader must never have to
    // reconstruct columns from spacing". Reciting them over prose turns a
    // sentence into a form being read out, and deciding by field count meant a
    // second clause silently flipped one into the other.
    let mut records = Records::new();
    records
        .push(RecordKind::Message)
        .text(FieldName::Message, "the east ward has failed")
        .text(FieldName::Detail, "integrity 34 percent")
        .role(Role::Danger)
        .finish();

    assert_eq!(
        records.get(0).expect("record").to_speech(),
        "the east ward has failed, integrity 34 percent",
    );
}

#[test]
fn an_annotation_is_never_read_aloud() {
    // An annotation classifies a record for a view — is this prompt selectable,
    // does this echo need a correction affordance. Speaking it reads an internal
    // token to a player.
    let mut records = Records::new();
    records
        .push(RecordKind::Message)
        .text(FieldName::Outcome, "forced")
        .text(FieldName::Message, "survey")
        .finish();

    let record = records.get(0).expect("record");
    assert_eq!(record.to_speech(), "survey");

    // Still reachable, though: the view needs it, and `sift` searching it is how
    // a pipeline would filter on outcome.
    assert_eq!(record.content().count(), 1);
    assert_eq!(record.fields().count(), 2);
    assert_eq!(
        record.field(FieldName::Outcome),
        Some(Value::Text("forced"))
    );
    assert_eq!(records.sift(&Sift::new("forced")).count(), 1);
}

#[test]
fn a_line_view_never_puts_an_internal_token_on_screen() {
    // A line view names no columns, so it has to default to content — otherwise
    // an echo draws as "resolved survey" and a player reads a machine tag.
    let mut records = Records::new();
    records
        .push(RecordKind::Echo)
        .text(FieldName::Outcome, "resolved")
        .text(FieldName::Message, "survey")
        .finish();

    let mut frame = frame_of(30, 2);
    let area = frame.area();
    RecordView::lines().draw(&mut frame.painter(area), area, records.iter());

    let drawn = frame.to_text();
    assert!(drawn.contains("survey"));
    assert!(!drawn.contains("resolved"), "annotation drawn: {drawn:?}");
}

#[test]
fn a_table_still_draws_an_annotation_a_caller_asked_for() {
    // Skipped when speaking, never when explicitly named as a column. A caller
    // naming it has asked for it.
    let mut records = Records::new();
    records
        .push(RecordKind::Status)
        .text(FieldName::Outcome, "forced")
        .text(FieldName::Name, "survey")
        .finish();

    let mut frame = frame_of(30, 2);
    let area = frame.area();
    RecordView::table(&[FieldName::Name, FieldName::Outcome])
        .without_header()
        .draw(&mut frame.painter(area), area, records.iter());

    assert!(frame.to_text().contains("survey  forced"));
    // Spoken without its label: the annotation is skipped, leaving one content
    // field, and one field has no columns to reconstruct.
    assert_eq!(
        frame.speech().utterances().next().expect("row").text,
        "survey",
    );
}

#[test]
fn an_authored_variant_replaces_the_whole_row() {
    let mut records = Records::new();
    records
        .push(RecordKind::Message)
        .text(FieldName::Message, "t h e   d o o r   i s   o p e n")
        .presentation(Presentation::Eldritch)
        .spoken("the door is open")
        .finish();

    let record = records.get(0).expect("record");
    assert_eq!(record.to_speech(), "the door is open");
    assert_eq!(record.presentation(), Presentation::Eldritch);
}

// ---------------------------------------------------------------------------
// §3 — the corruption exemption
// ---------------------------------------------------------------------------

#[test]
fn eldritch_cannot_reach_a_log_line_even_when_asked_for() {
    // §3: log output is a diagnostic surface and must stay trustworthy *as a
    // rendering*. Asking is allowed; the answer is no, at every call site,
    // because there is no unfiltered accessor to forget to filter.
    let mut records = Records::new();
    records
        .push(RecordKind::LogLine)
        .tick(FieldName::Tick, 1247)
        .text(FieldName::Message, "east ward holding")
        .presentation(Presentation::Eldritch)
        .finish();

    let record = records.get(0).expect("record");
    assert_eq!(record.presentation(), Presentation::Plain);
    assert_eq!(record.style().presentation, Presentation::Plain);
}

#[test]
fn a_sabotage_tell_survives_on_the_surface_you_inspect() {
    // The asymmetry §3 buys by keeping the two vocabularies disjoint: the tonal
    // register is suppressed on a log line, the diagnostic one is not.
    let mut records = Records::new();
    records
        .push(RecordKind::LogLine)
        .tick(FieldName::Tick, 1248)
        .text(FieldName::Message, "east ward holding")
        .presentation(Presentation::Tampered)
        .finish();

    assert_eq!(
        records.get(0).expect("record").presentation(),
        Presentation::Tampered,
    );
}

// ---------------------------------------------------------------------------
// §7 — pipes operate on records, never on rendered text
// ---------------------------------------------------------------------------

#[test]
fn sift_finds_what_the_screen_truncated_away() {
    // The whole point of filtering the model. `nightshade` does not fit an
    // eight-cell pane, so the drawn form stops at `nightsh` — and a search for
    // `shade` must still find the record, or a narrow window would silently
    // change what a pipeline returns.
    let records = alembic();
    let mut frame = frame_of(8, 4);
    let area = frame.area();
    RecordView::table(&[FieldName::Name]).draw(&mut frame.painter(area), area, records.iter());

    let drawn = frame.to_text();
    assert!(drawn.contains("nightsh"));
    assert!(!drawn.contains("nightshade"));

    let sift = Sift::new("shade");
    assert_eq!(records.sift(&sift).count(), 1);
}

#[test]
fn sift_never_matches_the_padding_a_view_added() {
    // Columns are separated by two spaces in the drawn form. Searching for
    // "ready   " must find nothing: the gap belongs to the table, not the record.
    let records = alembic();
    assert_eq!(records.sift(&Sift::new("ready  ")).count(), 0);
    assert_eq!(records.sift(&Sift::new("ready")).count(), 1);
}

#[test]
fn sift_folds_case_and_can_be_restricted_to_a_field() {
    let records = alembic();
    assert_eq!(records.sift(&Sift::new("SPOILED")).count(), 1);

    // "sage" is a name, never a state. Restricting proves the match ran against
    // the named field rather than the concatenation of the row.
    let by_state = Sift::new("sage").in_field(FieldName::State);
    assert_eq!(records.sift(&by_state).count(), 0);
    assert_eq!(
        records
            .sift(&Sift::new("sage").in_field(FieldName::Name))
            .count(),
        1,
    );
}

// ---------------------------------------------------------------------------
// The views
// ---------------------------------------------------------------------------

#[test]
fn a_table_speaks_every_row_in_full_however_narrow_the_pane() {
    // Truncation is a visual constraint and must never become an informational
    // one — the same contract `Painter::span` keeps.
    let records = alembic();
    let mut frame = frame_of(8, 4);
    let area = frame.area();
    RecordView::table(&[FieldName::Name, FieldName::State, FieldName::Quantity]).draw(
        &mut frame.painter(area),
        area,
        records.iter(),
    );

    let spoken: Vec<_> = frame.speech().utterances().collect();
    assert_eq!(spoken.len(), 2, "one utterance per row, header silent");
    assert_eq!(spoken[0].text, "name: sage, state: ready, qty: 3");
    assert_eq!(spoken[1].text, "name: nightshade, state: spoiled, qty: 120");
    assert_eq!(spoken[1].role, Role::Danger);
    assert_eq!(spoken[0].kind, UtteranceKind::TableRow);
}

#[test]
fn numbers_right_align_because_the_record_kept_them_numbers() {
    let records = alembic();
    let mut frame = frame_of(40, 4);
    let area = frame.area();
    RecordView::table(&[FieldName::Name, FieldName::Quantity]).draw(
        &mut frame.painter(area),
        area,
        records.iter(),
    );

    // `name` is ten cells wide (`nightshade`), then a two-cell gap, so the
    // quantity column starts at 12 and is three wide (`120`).
    let rows: Vec<String> = frame
        .rows()
        .map(|row| row.iter().map(|cell| cell.glyph).collect())
        .collect();
    assert_eq!(rows[0].trim_end(), "name        qty");
    assert_eq!(rows[1].trim_end(), "sage          3");
    assert_eq!(rows[2].trim_end(), "nightshade  120");
}

#[test]
fn a_missing_field_leaves_the_gap_that_is_the_tell() {
    // §8.1 names malformed record boundaries as a structural sabotage
    // signature. A record short a field draws as a hole with no special case.
    let mut records = Records::new();
    records
        .push(RecordKind::LogLine)
        .tick(FieldName::Tick, 1247)
        .text(FieldName::Source, "alembic")
        .finish();
    records
        .push(RecordKind::LogLine)
        .tick(FieldName::Tick, 1248)
        .finish();

    let mut frame = frame_of(20, 4);
    let area = frame.area();
    RecordView::table(&[FieldName::Tick, FieldName::Source])
        .without_header()
        .draw(&mut frame.painter(area), area, records.iter());

    let rows: Vec<String> = frame
        .rows()
        .map(|row| row.iter().map(|cell| cell.glyph).collect())
        .collect();
    assert_eq!(rows[0].trim_end(), "1247  alembic");
    assert_eq!(rows[1].trim_end(), "1248");
}

#[test]
fn the_line_view_carries_style_and_speech_together() {
    let mut records = Records::new();
    records
        .push(RecordKind::Message)
        .text(FieldName::Message, "s i l e n c e")
        .presentation(Presentation::Eldritch)
        .spoken("silence")
        .finish();

    let mut frame = frame_of(40, 2);
    let area = frame.area();
    RecordView::lines().draw(&mut frame.painter(area), area, records.iter());

    assert!(frame.to_text().contains("s i l e n c e"));
    let spoken: Vec<_> = frame.speech().utterances().collect();
    assert_eq!(spoken[0].text, "silence");

    let cell = frame.cell(area.origin()).expect("first cell");
    assert_eq!(cell.style.presentation, Presentation::Eldritch);
}

#[test]
fn a_view_never_paints_outside_the_area_it_was_given() {
    // The bug the `screens` example caught once already: content eating a
    // border because no sub-painter was established.
    let records = alembic();
    let mut frame = frame_of(40, 6);
    let area = frame.area();
    let mut painter = frame.painter(area);
    painter.border(area, Some("alembic"), Style::DIM);

    let inner = Rect::new(area.col + 1, area.row + 1, area.cols - 2, area.rows - 2);
    RecordView::table(&[FieldName::Name, FieldName::State]).draw(
        &mut painter,
        inner,
        records.iter(),
    );

    let rows: Vec<&[orbs_render::Cell]> = frame.rows().collect();
    assert_eq!((rows[0][0].glyph, rows[0][39].glyph), ('┌', '┐'));
    assert_eq!((rows[5][0].glyph, rows[5][39].glyph), ('└', '┘'));
    for row in &rows[1..5] {
        assert_eq!(row[0].glyph, '│', "left border eaten");
        assert_eq!(row[39].glyph, '│', "right border eaten");
    }
}

#[test]
fn clearing_a_stream_keeps_its_allocations() {
    let mut records = alembic();
    assert_eq!(records.len(), 2);
    records.clear();
    assert!(records.is_empty());
    assert_eq!(records.iter().count(), 0);
    assert!(records.get(0).is_none());
}

#[test]
fn a_listener_can_tell_an_error_from_an_offer() {
    // The prompt draws the difference as a marker glyph and a brightness. A
    // screen-reader player has neither channel, so without the outcome on the
    // utterance `xyzzy` linearises as three identical Echo lines and the error
    // is indistinguishable from the suggestions under it (§14, rule 2).
    let mut records = Records::new();
    for (outcome, message) in [
        (Outcome::Unresolved, "xyzzy"),
        (Outcome::Suggestion, "survey"),
        (Outcome::Suggestion, "scribe"),
    ] {
        records
            .push(RecordKind::Echo)
            .outcome(outcome)
            .text(FieldName::Message, message)
            .finish();
    }

    let mut frame = frame_of(40, 4);
    let area = frame.area();
    RecordView::prompt().draw(&mut frame.painter(area), area, records.iter());

    let spoken: Vec<_> = frame.speech().utterances().collect();
    assert_eq!(
        spoken.iter().map(|u| u.outcome).collect::<Vec<_>>(),
        [
            Some(Outcome::Unresolved),
            Some(Outcome::Suggestion),
            Some(Outcome::Suggestion),
        ],
    );
    // The text alone is not enough to tell them apart, which is the point.
    assert_eq!(spoken[1].kind, spoken[0].kind);
    assert_eq!(spoken[1].role, spoken[0].role);
}

#[test]
fn a_selectable_candidate_is_distinguishable_by_ear() {
    // §6 numbers tied readings and has the player pick one. A listener must be
    // able to tell that prompt from a command about to run.
    let mut records = Records::new();
    for outcome in [Outcome::Resolved, Outcome::Candidate] {
        records
            .push(RecordKind::Echo)
            .outcome(outcome)
            .text(FieldName::Message, "survey")
            .finish();
    }

    let mut frame = frame_of(40, 4);
    let area = frame.area();
    RecordView::prompt().draw(&mut frame.painter(area), area, records.iter());

    let spoken: Vec<_> = frame.speech().utterances().collect();
    assert_eq!(spoken[0].text, spoken[1].text, "identical as text");
    assert_ne!(spoken[0].outcome, spoken[1].outcome, "and only as text");
    assert!(
        spoken[1].outcome.is_some_and(Outcome::is_selectable),
        "the selectable one must say so",
    );
}

#[test]
fn one_column_is_not_a_table_worth_labelling() {
    // §14 wants labels so a listener never reconstructs columns from spacing.
    // A row with a single field has no columns, so `message: tick 33` is noise —
    // which is exactly what a re-emitted log line sounded like before this.
    let mut records = Records::new();
    records
        .push(RecordKind::LogLine)
        .text(FieldName::Message, "tick 33")
        .finish();
    records
        .push(RecordKind::LogLine)
        .tick(FieldName::Tick, 1247)
        .text(FieldName::Message, "east ward holding")
        .finish();

    assert_eq!(records.get(0).expect("one field").to_speech(), "tick 33");
    assert_eq!(
        records.get(1).expect("two fields").to_speech(),
        "tick: 1247, message: east ward holding",
    );
}

#[test]
fn a_records_drawn_form_never_nests_another_records_speech() {
    // A command re-emitting a record into the log stores its *drawn* form. With
    // the spoken form the label came too, and reading the log back gave
    // `message: name: seed, qty: 12648430`.
    let mut records = Records::new();
    records
        .push(RecordKind::Status)
        .text(FieldName::Name, "seed")
        .count(FieldName::Quantity, 12_648_430)
        .finish();

    let source = records.get(0).expect("record");
    assert_eq!(source.to_line(), "seed 12648430");
    assert_eq!(source.to_speech(), "name: seed, qty: 12648430");
}
