//! The two shipped searches over every beast, and the solver the circle refuses
//! to publish for.
//!
//! Two totals each, both pinned. *Uniform* is every distinct puzzle once; *by
//! circuit* is every entry of the draw's list — how often a player actually
//! meets each puzzle, which is what `orbs-balance` pins (§19).

use std::collections::BTreeSet;

use super::super::circuit::{self, drawable};
use super::super::temper::{OPENING, ROWS};
use super::super::{Glyph, Humour};
use super::oracle::{keystones, puzzles, solutions, taming, wide, winnowing};

/// The shipped search holds every beast, in at most 216 calls. The worst and
/// both totals are pinned because §19 and SEEING-IT quote them: a number in a
/// document that no test holds is a number that drifts.
#[test]
fn the_taming_search_holds_every_puzzle_and_its_worst_is_174_calls() {
    let mut worst = 0;
    let mut uniform = 0;
    for puzzle in puzzles() {
        let Some(calls) = taming(puzzle) else {
            panic!("[taming] never holds {puzzle:?}");
        };
        assert!(calls <= 216);
        worst = worst.max(calls);
        uniform += wide(calls);
    }
    assert_eq!(worst, 174);
    assert_eq!(uniform, 190_254, "over the 2,994 puzzles");
    let by_circuit: u64 = drawable()
        .iter()
        .map(|puzzle| wide(taming(*puzzle).unwrap_or(216)))
        .sum();
    assert_eq!(
        by_circuit, 663_948,
        "over the 9,756 circuits a draw picks from"
    );
}

/// The ladder is exactly what the logic allows — every keystone on a rung holds
/// some beast lit on that many rows, and no beast is held by a keystone its rung
/// leaves out. Either half wrong is a spell that wastes calls or one that never
/// holds. Turned wires change none of the rungs.
#[test]
fn each_rung_of_the_winnowing_ladder_names_every_keystone_that_fervour_allows() {
    let table = puzzles();
    for fervour in 1..8 {
        let allowed: BTreeSet<Humour> = table
            .iter()
            .filter(|puzzle| puzzle.temper.fervour() == fervour)
            .flat_map(|puzzle| solutions(*puzzle))
            .map(|[keystone, _, _]| keystone)
            .collect();
        let rung: BTreeSet<Humour> = keystones(fervour).iter().copied().collect();
        assert_eq!(allowed, rung, "the rung for {fervour} rows");
    }
}

/// What a player's knowledge buys the orb. The fervour ladder prunes the
/// keystone, and De Morgan halves sunwise: every hold has a twin whose sunwise
/// is the other of its pair, so `yoke`, `heed` and `oppose` cover all six.
///
/// At most six keystones of eighteen calls each, so never past 108.
#[test]
fn the_winnowing_search_holds_every_puzzle_in_fewer_calls_than_taming() {
    let mut worst = 0;
    let mut uniform = 0;
    let mut searched = 0;
    for puzzle in puzzles() {
        let Some(calls) = winnowing(puzzle) else {
            panic!("[winnowing] never holds {puzzle:?}");
        };
        assert!(calls <= 108);
        worst = worst.max(calls);
        uniform += wide(calls);
        searched += wide(taming(puzzle).unwrap_or(216));
    }
    assert_eq!(worst, 90);
    assert_eq!(uniform, 112_494, "over the 2,994 puzzles");
    assert!(
        uniform < searched,
        "{uniform} calls against [taming]'s {searched}"
    );
    let by_circuit: u64 = drawable()
        .iter()
        .map(|puzzle| wide(winnowing(*puzzle).unwrap_or(108)))
        .sum();
    assert_eq!(
        by_circuit, 367_860,
        "over the 9,756 circuits a draw picks from"
    );
}

/// Why no *closer* or *further* is published. A climb that keeps whichever
/// humour agrees on more rows — the lens's sweep, transplanted — stalls on 1,104
/// of the 2,994 puzzles, 37%, because one glyph can mask another: the lens's
/// sockets are independent and this circle's glyphs are not.
///
/// The climb: from the opening, try every humour at every glyph in turn, keep a
/// change whenever it agrees on strictly more rows than the best so far, and
/// stop when a whole pass keeps nothing.
#[test]
fn a_climb_on_rows_agreeing_stalls_on_a_large_share_of_circles() {
    let stalls = puzzles()
        .into_iter()
        .filter(|puzzle| {
            let rows = |glyphs| {
                circuit::answer(puzzle.wiring, puzzle.turned, glyphs).agreeing(puzzle.temper)
            };
            let mut glyphs = OPENING;
            let mut best = rows(glyphs);
            loop {
                let mut rose = false;
                for glyph in Glyph::ALL {
                    for humour in Humour::ALL {
                        let mut tried = glyphs;
                        tried[glyph.index()] = humour;
                        let now = rows(tried);
                        if now > best {
                            best = now;
                            glyphs = tried;
                            rose = true;
                        }
                    }
                }
                if best == u32::try_from(ROWS).unwrap_or(u32::MAX) {
                    return false;
                }
                if !rose {
                    return true;
                }
            }
        })
        .count();
    assert_eq!(stalls, 1_104, "the share quoted in §19 moved");
}
