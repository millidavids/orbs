//! What every proof walks: the puzzles, what holds each, and the two shipped
//! searches as Rust copies of their spells.

use std::collections::BTreeSet;

use super::super::circuit::{self, Circuit, Puzzle, limnings};
use super::super::{Beast, Call, Glyph, Humour, Shape};

/// Every distinct puzzle a circuit can make — the oracle the draw is checked
/// against.
pub(super) fn puzzles() -> BTreeSet<Puzzle> {
    Circuit::all().filter_map(Circuit::puzzle).collect()
}

/// Every limning that holds `puzzle`.
pub(super) fn solutions(puzzle: Puzzle) -> Vec<[Humour; 3]> {
    limnings()
        .filter(|glyphs| circuit::answer(puzzle.wiring, puzzle.turned, *glyphs) == puzzle.temper)
        .collect()
}

/// `[taming]`, line for line: step widdershins and call, six times; step
/// sunwise; six times; step the keystone; six times — stopping when held.
///
/// Returns how many calls it took, or `None` if it never held.
pub(super) fn taming(puzzle: Puzzle) -> Option<u32> {
    let mut beast = Beast::arriving(Shape::Whole(puzzle));
    for _ in 0..Humour::ALL.len() {
        for _ in 0..Humour::ALL.len() {
            for _ in 0..Humour::ALL.len() {
                beast.step(Glyph::Widdershins);
                if beast.call() == Call::Held {
                    return Some(beast.calls());
                }
            }
            beast.step(Glyph::Sunwise);
        }
        beast.step(Glyph::Keystone);
    }
    None
}

/// The keystones a temper lit on `fervour` rows can be held by — `[winnowing]`'s
/// ladder, rung for rung.
pub(super) fn keystones(fervour: u32) -> &'static [Humour] {
    use Humour::{Eschew, Heed, Mirror, Oppose, Spurn, Yoke};
    match fervour {
        1 => &[Yoke, Eschew],
        2 => &[Yoke, Eschew, Oppose, Mirror],
        3 | 5 => &[Yoke, Spurn, Heed, Eschew],
        6 => &[Spurn, Heed, Oppose, Mirror],
        7 => &[Spurn, Heed],
        _ => &Humour::ALL,
    }
}

/// `[winnowing]`, line for line: for each keystone the ladder allows, limn it,
/// then sweep widdershins six times under sunwise at `yoke`, `heed` and
/// `oppose` — two steps apart — stopping when held.
pub(super) fn winnowing(puzzle: Puzzle) -> Option<u32> {
    let mut beast = Beast::arriving(Shape::Whole(puzzle));
    for keystone in keystones(puzzle.temper.fervour()) {
        beast.limn(Glyph::Keystone, *keystone);
        for _ in 0..3 {
            for _ in 0..Humour::ALL.len() {
                beast.step(Glyph::Widdershins);
                if beast.call() == Call::Held {
                    return Some(beast.calls());
                }
            }
            beast.step(Glyph::Sunwise);
            beast.step(Glyph::Sunwise);
        }
    }
    None
}

/// A count as a `u64`, for totals.
pub(super) fn wide(count: u32) -> u64 {
    u64::from(count)
}
