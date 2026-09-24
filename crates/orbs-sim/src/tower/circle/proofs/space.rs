//! What the circuits make: how many beasts, of what kinds, and that every one is
//! a real question with a real answer.

use std::collections::BTreeSet;

use super::super::circuit::{self, Circuit, Turned, Wiring, drawable, is_puzzle};
use super::super::temper::{OPENING, SENSES, Temper};
use super::super::{Glyph, Humour};
use super::oracle::{puzzles, solutions};

/// The space, measured outside this code: 6 wirings × 9 masks × 216 limnings,
/// then the valid ones, the distinct puzzles, the boards and the tempers.
#[test]
fn the_circuits_make_the_beasts_that_were_measured() {
    assert_eq!(Circuit::all().count(), 11_664);
    assert_eq!(drawable().len(), 9_756, "valid circuits");
    let table = puzzles();
    assert_eq!(table.len(), 2_994, "distinct puzzles");
    let boards: BTreeSet<(Wiring, Temper)> =
        table.iter().map(|one| (one.wiring, one.temper)).collect();
    assert_eq!(boards.len(), 624, "distinct boards");
    let tempers: BTreeSet<Temper> = table.iter().map(|one| one.temper).collect();
    assert_eq!(tempers.len(), 192, "distinct tempers");
}

#[test]
fn no_puzzle_is_one_with_a_question_missing() {
    for puzzle in puzzles() {
        let lit = puzzle.temper.fervour();
        assert!(
            lit > 0 && lit < 8,
            "{puzzle:?} answers the same on every row"
        );
        assert!(
            puzzle.temper.turns_on_every_sense(),
            "{puzzle:?} has a sense that changes nothing",
        );
        assert_ne!(
            puzzle.temper,
            circuit::answer(puzzle.wiring, puzzle.turned, OPENING),
            "{puzzle:?} is held by a circle nobody limned",
        );
        assert!(
            puzzle.turned.is_allowed(),
            "{puzzle:?} turns both of a glyph's wires"
        );
    }
}

/// Never one solution, by De Morgan: negate both outer glyphs, swap the
/// keystone for its dual, and every row answers as it did.
#[test]
fn every_puzzle_can_be_held_at_least_two_ways_and_every_hold_has_its_twin() {
    for puzzle in puzzles() {
        let held = solutions(puzzle);
        assert!(
            (2..=6).contains(&held.len()),
            "{puzzle:?} has {} solutions",
            held.len()
        );
        for [keystone, sunwise, widdershins] in &held {
            let twin = [keystone.dual(), sunwise.negated(), widdershins.negated()];
            assert!(held.contains(&twin), "{puzzle:?}: no twin for {held:?}");
        }
    }
}

/// Every valid circuit in order, one entry each — the distribution by circuit.
#[test]
fn the_drawable_list_is_every_valid_circuit_in_order() {
    let expected: Vec<_> = Circuit::all().filter_map(Circuit::puzzle).collect();
    assert_eq!(drawable(), expected.as_slice());
    let by_puzzle = |puzzle| drawable().iter().filter(|one| **one == puzzle).count();
    for puzzle in puzzles() {
        assert_eq!(
            by_puzzle(puzzle),
            solutions(puzzle).len(),
            "{puzzle:?} is not in the list once a limning that holds it",
        );
    }
}

/// `is_puzzle` against the oracle over every temper, wiring and mask: exactly
/// the drawn puzzles are accepted, whatever a hand-edited save claims.
#[test]
fn a_restore_accepts_exactly_the_puzzles_a_circuit_makes() {
    let table = puzzles();
    let mut accepted = 0;
    for wiring in Wiring::all() {
        for turned in Turned::ALL {
            for bits in 0..=u8::MAX {
                let puzzle = circuit::Puzzle {
                    wiring,
                    turned,
                    temper: Temper::from_bits(bits, SENSES),
                };
                assert_eq!(is_puzzle(puzzle), table.contains(&puzzle), "{puzzle:?}");
                accepted += usize::from(is_puzzle(puzzle));
            }
        }
    }
    assert_eq!(accepted, 2_994);
}

/// Why no mask turns both wires of a glyph: `h(~a, ~b)` is `dual(h)(a, b)`.
#[test]
fn both_wires_turned_is_the_dual_humour_so_no_mask_draws_it() {
    for humour in Humour::ALL {
        for (a, b) in [(false, false), (false, true), (true, false), (true, true)] {
            assert_eq!(
                humour.answer(!a, !b),
                humour.dual().answer(a, b),
                "{humour:?}"
            );
        }
    }
    let allowed: Vec<Turned> = (0u8..16)
        .filter_map(|bits| Turned::from_wires(&wires(bits)))
        .collect();
    assert_eq!(allowed, Turned::ALL.to_vec());
    for turned in Turned::ALL {
        for glyph in [Glyph::Sunwise, Glyph::Widdershins] {
            assert!(
                !(turned.wire(glyph, 0) && turned.wire(glyph, 1)),
                "{turned:?} turns both of {glyph:?}'s wires",
            );
        }
        assert!(!turned.wire(Glyph::Keystone, 0) && !turned.wire(Glyph::Keystone, 1));
    }
}

/// A mask's four wires as a save writes them, for a bit pattern.
fn wires(bits: u8) -> String {
    (0..4)
        .map(|bit| if (bits >> bit) & 1 == 1 { '1' } else { '0' })
        .collect()
}

#[test]
fn a_mask_reads_back_as_itself_and_nothing_else_reads() {
    for turned in Turned::ALL {
        assert_eq!(Turned::from_wires(&turned.to_wires()), Some(turned));
    }
    assert_eq!(Turned::from_wires("0000"), Some(Turned::NONE));
    for bad in ["1100", "0011", "1111", "010", "01000", "01a0", "", "0x10"] {
        assert!(Turned::from_wires(bad).is_none(), "{bad:?} read as a mask");
    }
}

/// A turned wire negates the sense on its own glyph only: every single-wire
/// mask, against the answer negated by hand.
#[test]
fn a_turned_wire_hands_its_glyph_the_sense_turned_over() {
    for wiring in Wiring::all() {
        for glyphs in circuit::limnings() {
            let [keystone, sunwise, widdershins] = glyphs;
            for (mask, glyph, input) in [
                ("1000", Glyph::Sunwise, 0),
                ("0100", Glyph::Sunwise, 1),
                ("0010", Glyph::Widdershins, 0),
                ("0001", Glyph::Widdershins, 1),
            ] {
                let turned = Turned::from_wires(mask).unwrap_or(Turned::NONE);
                let got = circuit::answer(wiring, turned, glyphs);
                for row in 0..8 {
                    let lit = |sense: usize| (row >> (SENSES - 1 - sense)) & 1 == 1;
                    let mut inputs = [
                        lit(wiring.sunwise[0]),
                        lit(wiring.sunwise[1]),
                        lit(wiring.widdershins[0]),
                        lit(wiring.widdershins[1]),
                    ];
                    let at = usize::from(glyph == Glyph::Widdershins) * 2 + input;
                    inputs[at] = !inputs[at];
                    let want = keystone.answer(
                        sunwise.answer(inputs[0], inputs[1]),
                        widdershins.answer(inputs[2], inputs[3]),
                    );
                    assert_eq!(got.lit(row), want, "{wiring:?} {mask} {glyphs:?} row {row}");
                }
            }
        }
    }
}
