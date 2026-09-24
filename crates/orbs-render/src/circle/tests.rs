use super::board::{BALKS, DARK, LIT, TURNED};
use super::{Circle, Given, Line};

/// What a glyph is given, by name, `~` in front of a turned one.
fn given(names: &[&str]) -> Vec<Given> {
    names
        .iter()
        .map(|name| Given {
            name: name.trim_start_matches(TURNED).to_owned(),
            turned: name.starts_with(TURNED),
        })
        .collect()
}

fn circle() -> Circle {
    let lit = |sense: usize| (0..8).map(|row| (row >> (2 - sense)) & 1 == 1).collect();
    Circle {
        lines: vec![
            Line {
                glyph: "sunwise".into(),
                humour: "yoke".into(),
                given: given(&["blood", "~bone"]),
            },
            Line {
                glyph: "widdershins".into(),
                humour: "heed".into(),
                given: given(&["bone", "breath"]),
            },
            Line {
                glyph: "keystone".into(),
                humour: "oppose".into(),
                given: given(&["sunwise", "widdershins"]),
            },
        ],
        senses: vec!["blood".into(), "bone".into(), "breath".into()],
        lit: vec![lit(0), lit(1), lit(2)],
        temper: vec![false, true, true, true, false, true, true, false],
        answer: Some(vec![false, true, true, true, false, true, false, false]),
        labels: ["temper".into(), "answer".into()],
        tally: "2 calls, 1 row balks".into(),
    }
}

#[test]
fn every_row_is_the_board_width_and_there_are_exactly_its_rows() {
    let board = circle();
    let (cols, rows) = Circle::size();
    for index in 0..usize::from(rows) {
        let Some(line) = board.line(index) else {
            panic!("row {index} is missing");
        };
        assert_eq!(
            line.chars().count(),
            usize::from(cols),
            "row {index}: {line:?}"
        );
    }
    assert!(board.row(usize::from(rows)).is_none());
}

/// A number stands over its own cells, or *"balks at row 7"* points at the
/// wrong one.
#[test]
fn the_numbers_stand_over_their_cells_and_the_marks_under_them() {
    let board = circle();
    let numbers = board.line(4).unwrap_or_default();
    let temper = board.line(8).unwrap_or_default();
    let marks = board.line(10).unwrap_or_default();
    let at = |line: &str, wanted: char| line.chars().position(|glyph| glyph == wanted);
    let seven = at(&numbers, '7');
    assert_eq!(at(&marks, BALKS), seven, "the mark is not under row 7");
    let cell = temper.chars().nth(seven.unwrap_or(0));
    assert_eq!(
        cell,
        Some(LIT),
        "row 7 of the temper is not under its number"
    );
}

/// Senses and labels are reloadable prose, so a long one must not push a
/// cell out from under its number.
#[test]
fn a_long_name_is_clipped_so_its_cells_stay_under_their_numbers() {
    let mut board = circle();
    board.labels[1] = "what the circle answers".into();
    board.senses[0] = "a very long sense name".into();
    let numbers = board.line(4).unwrap_or_default();
    let seven = numbers.chars().position(|glyph| glyph == '7');
    for index in [5, 9] {
        let row = board.line(index).unwrap_or_default();
        let cell = row.chars().nth(seven.unwrap_or(0));
        assert!(
            cell == Some(LIT) || cell == Some(DARK),
            "row {index} moved off its numbers: {row:?}",
        );
        assert_eq!(row.chars().nth(Circle::LABEL - 1), Some(' '), "{row:?}");
    }
}

#[test]
fn nothing_balks_before_the_first_call_and_the_answer_row_is_blank() {
    let mut board = circle();
    board.answer = None;
    assert!(board.balking().is_empty());
    let answer = board.line(9).unwrap_or_default();
    assert!(
        !answer.contains(LIT) && !answer.contains(DARK),
        "{answer:?}"
    );
    assert!(!board.line(10).unwrap_or_default().contains(BALKS));
}

/// A turned wire is marked on its own name only; nothing else moves.
#[test]
fn a_turned_wire_is_marked_on_its_name_and_nowhere_else() {
    let board = circle();
    assert_eq!(
        board.line(0).unwrap_or_default().trim_end(),
        "sunwise     yoke   ← blood, ~bone"
    );
    assert!(!board.line(1).unwrap_or_default().contains(TURNED));
    assert!(!board.line(2).unwrap_or_default().contains(TURNED));
    for index in 3..usize::from(Circle::size().1) {
        assert!(
            !board.line(index).unwrap_or_default().contains(TURNED),
            "row {index} drew a turn",
        );
    }
    assert!(crate::is_renderable(TURNED), "`~` is not in CP437");
}

/// Clipping eats the name, never the `~` in front of it.
#[test]
fn a_long_turned_name_is_clipped_at_the_edge_with_its_mark_kept() {
    let mut board = circle();
    board.lines[0].given = given(&["~a-sense-with-a-very-long-name", "blood"]);
    let row = board.line(0).unwrap_or_default();
    assert_eq!(row.chars().count(), usize::from(Circle::COLS));
    assert!(row.contains("← ~a-sense-with"), "{row:?}");
}

/// A lesser circle is the same footprint with less in it, so the transcript
/// does not move when the whole circle opens.
#[test]
fn a_lesser_board_draws_one_line_and_four_columns_in_the_same_footprint() {
    let lit = |sense: usize| (0..4).map(|row| (row >> (1 - sense)) & 1 == 1).collect();
    let board = Circle {
        lines: vec![Line {
            glyph: "keystone".into(),
            humour: "heed".into(),
            given: given(&["blood", "bone"]),
        }],
        senses: vec!["blood".into(), "bone".into()],
        lit: vec![lit(0), lit(1)],
        temper: vec![false, true, true, false],
        answer: Some(vec![false, true, true, true]),
        labels: ["temper".into(), "answer".into()],
        tally: "1 call, 1 row balks".into(),
    };
    let (cols, rows) = Circle::size();
    for index in 0..usize::from(rows) {
        let line = board.line(index).unwrap_or_default();
        assert_eq!(line.chars().count(), usize::from(cols), "row {index}");
    }
    assert!(board.row(usize::from(rows)).is_none());
    let numbers = board.line(2).unwrap_or_default();
    assert!(
        numbers.contains("1 2 3 4") && !numbers.contains('5'),
        "{numbers:?}"
    );
    assert_eq!(board.balking(), vec![4]);
    let marks = board.line(7).unwrap_or_default();
    let at = |line: &str, wanted: char| line.chars().position(|glyph| glyph == wanted);
    assert_eq!(at(&marks, BALKS), at(&numbers, '4'));
    assert!(board.line(8).unwrap_or_default().starts_with("1 call"));
    assert!(board.line(11).unwrap_or_default().trim().is_empty());
}

#[test]
fn the_rows_that_balk_are_the_rows_that_differ() {
    assert_eq!(circle().balking(), vec![7]);
}

/// Checked by test, not by eye: `is_renderable` lints authored prose, not a
/// painter's Rust literals (§19).
#[test]
fn every_glyph_the_board_draws_is_in_the_code_page() {
    let board = circle();
    for index in 0..usize::from(Circle::size().1) {
        for glyph in board.line(index).unwrap_or_default().chars() {
            assert!(crate::is_renderable(glyph), "{glyph:?} is not in CP437");
        }
    }
}
