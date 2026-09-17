//! What the board is handed: every fact the beast has, in the order it is drawn.

use super::super::circuit::{self, Wiring};
use super::super::temper::sense_lit;
use super::super::{Beast, Glyph, Shape, view};
use crate::content::Prose;
use crate::rng::Rngs;

/// **Every turned wire reaches the board on the right name, and nothing else is
/// turned** — over a thousand drawn beasts, each glyph line checked against the
/// beast's own wiring and mask, and the keystone's never turned.
#[test]
fn every_turned_wire_reaches_the_board_on_its_own_sense() {
    let prose = Prose::builtin();
    let senses: Vec<String> = (0..3)
        .map(|sense| prose.line(&format!("circle_sense_{sense}"), &[]))
        .collect();
    let mut rngs = Rngs::from_seed(9);
    let mut turned_seen = 0;
    for _ in 0..1000 {
        let beast = Beast::draw(&mut rngs, true);
        let Shape::Whole(puzzle) = beast.shape() else {
            panic!("a whole draw was lesser");
        };
        let board = view(&beast, &prose);
        let [sunwise, widdershins, keystone] = board.lines.as_slice() else {
            panic!("a whole board has {} lines", board.lines.len());
        };
        for (line, glyph, pair) in [
            (sunwise, Glyph::Sunwise, puzzle.wiring.sunwise),
            (widdershins, Glyph::Widdershins, puzzle.wiring.widdershins),
        ] {
            assert_eq!(line.glyph, glyph.word());
            assert_eq!(line.humour, beast.humour(glyph).word());
            for (input, given) in line.given.iter().enumerate() {
                assert_eq!(given.name, senses[pair[input]]);
                assert_eq!(given.turned, puzzle.turned.wire(glyph, input));
                turned_seen += usize::from(given.turned);
            }
        }
        assert_eq!(keystone.glyph, Glyph::Keystone.word());
        assert!(keystone.given.iter().all(|given| !given.turned));
        assert_eq!(
            board.temper,
            (0..8).map(|row| puzzle.temper.lit(row)).collect::<Vec<_>>(),
        );
        assert!(board.answer.is_none());
    }
    assert!(turned_seen > 1000, "only {turned_seen} turned wires drawn");
}

/// **The painted sense columns are the table's own rows**, so a player reading
/// the board and the model agree on what row 3 is — for both circles.
#[test]
fn the_sense_columns_are_the_rows_the_temper_is_written_in() {
    let prose = Prose::builtin();
    for whole in [true, false] {
        let beast = Beast::draw(&mut Rngs::from_seed(4), whole);
        let board = view(&beast, &prose);
        let senses = beast.shape().senses();
        assert_eq!(board.senses.len(), senses);
        for (sense, column) in board.lit.iter().enumerate() {
            let expected: Vec<bool> = (0..beast.rows())
                .map(|row| sense_lit(row, sense, senses))
                .collect();
            assert_eq!(column, &expected, "sense {sense}, whole {whole}");
        }
    }
}

/// **A lesser board is one keystone line over the two senses, untouched.**
#[test]
fn a_lesser_board_is_the_keystone_over_two_plain_senses() {
    let prose = Prose::builtin();
    let beast = Beast::draw(&mut Rngs::from_seed(4), false);
    let board = view(&beast, &prose);
    let [keystone] = board.lines.as_slice() else {
        panic!("a lesser board has {} lines", board.lines.len());
    };
    assert_eq!(keystone.glyph, Glyph::Keystone.word());
    assert_eq!(keystone.given.len(), 2);
    assert!(keystone.given.iter().all(|given| !given.turned));
    assert_eq!(board.temper.len(), 4);
}

/// **A call's answer and tally reach the board**, and the answer row is the model's.
#[test]
fn a_called_board_carries_the_answer_the_model_gave() {
    let prose = Prose::builtin();
    let mut beast = Beast::draw(&mut Rngs::from_seed(12), true);
    beast.call();
    let board = view(&beast, &prose);
    let Shape::Whole(puzzle) = beast.shape() else {
        panic!("a whole draw was lesser");
    };
    let expected = circuit::answer(puzzle.wiring, puzzle.turned, beast.glyphs());
    assert_eq!(
        board.answer,
        Some((0..8).map(|row| expected.lit(row)).collect::<Vec<_>>())
    );
    assert!(board.tally.starts_with("1 call"), "{}", board.tally);
    assert!(Wiring::all().contains(&puzzle.wiring));
}
