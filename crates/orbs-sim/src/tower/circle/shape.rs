//! Which circle a beast is held by: the keystone alone, or all three glyphs.
//!
//! # Composition, as the genre teaches it
//!
//! Every logic-gate game starts with one gate — *whose table is this?* — and
//! builds circuits after (§19). **The lesser circle is that first lesson**: a
//! beast with two senses and a four-row temper, held by the keystone alone,
//! which is the question *"which humour is this?"* asked five ways. A sealed
//! tower draws nothing else until `menagerie_2` opens the whole circle, and an
//! open tower — every test, dump and balance policy — never draws one at all.
//!
//! **A shape of beast rather than a second kind of circle**, so the circle's
//! verbs, readings, board and save stay one set: `limn` asks whether a glyph is
//! lit, the board draws the lines it is given, and a save says which by the
//! length of the temper.

use super::circuit::{self, Puzzle, Turned, Wiring};
use super::temper::{self, OPENING, PAR, Temper};
use super::{Glyph, Humour};
use crate::rng::Rngs;

/// How many senses a lesser beast has — the keystone's two.
pub const LESSER_SENSES: usize = 2;

/// How many calls a lesser beast may take and still pay in full: one.
///
/// **A four-row table read is a hold in one call**, and there is nothing to
/// check a guess against but the call itself — so PAR is the reading, and a
/// player stepping through the five is searching rather than reading.
pub const LESSER_PAR: u32 = 1;

/// Troops a lesser hold brings, within par or past it.
pub const LESSER_TROOPS: u32 = 1;

/// A lesser hold earns a quarter of what a whole one does.
///
/// **A lesson, priced as one.** It is a quarter of the work — one glyph of
/// three, four rows of eight — and a player on it is learning the six rather
/// than feeding a siege; five holds open the whole circle, and it is the whole
/// circle the economy is measured on.
pub const LESSER_SHARE: u64 = 4;

/// The circle a beast is held by, and what it answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Shape {
    /// The keystone alone, given the two senses directly. A four-row temper.
    Lesser(Temper),
    /// All three glyphs, wired as drawn. An eight-row temper.
    Whole(Puzzle),
}

impl Shape {
    /// Draw a beast's shape: a whole puzzle, or a lesser temper.
    ///
    /// **One value from the stream either way** — the forge's rule, kept in
    /// `circuit::pick` for both draws: a tower opening the whole circle moves no
    /// later draw by more than the one it already took.
    ///
    /// **A lesser beast is never given a turned wire.** A turned input to a lone
    /// keystone answers something none of the six humours does over the plain
    /// senses, so *"which humour is this?"* would stop having an answer.
    #[must_use]
    pub fn draw(rngs: &mut Rngs, whole: bool) -> Self {
        if whole {
            Self::Whole(circuit::draw(rngs))
        } else {
            Self::Lesser(circuit::pick(rngs, &lessers()))
        }
    }

    /// What every row must answer.
    #[must_use]
    pub const fn temper(self) -> Temper {
        match self {
            Self::Lesser(temper) => temper,
            Self::Whole(puzzle) => puzzle.temper,
        }
    }

    /// How the outer glyphs are wired, if there are outer glyphs.
    #[must_use]
    pub const fn wiring(self) -> Option<Wiring> {
        match self {
            Self::Lesser(_) => None,
            Self::Whole(puzzle) => Some(puzzle.wiring),
        }
    }

    /// Which wires into the outer glyphs are turned — none at a lesser circle,
    /// which has no outer glyphs.
    #[must_use]
    pub const fn turned(self) -> Turned {
        match self {
            Self::Lesser(_) => Turned::NONE,
            Self::Whole(puzzle) => puzzle.turned,
        }
    }

    /// How many senses the beast has.
    #[must_use]
    pub const fn senses(self) -> usize {
        self.temper().senses()
    }

    /// How many rows its temper has.
    #[must_use]
    pub const fn rows(self) -> usize {
        self.temper().rows()
    }

    /// How many calls pay in full.
    #[must_use]
    pub const fn par(self) -> u32 {
        match self {
            Self::Lesser(_) => LESSER_PAR,
            Self::Whole(_) => PAR,
        }
    }

    /// Whether `glyph` is part of this circle — whether it may be limned, and
    /// whether it carries a reading.
    #[must_use]
    pub const fn lights(self, glyph: Glyph) -> bool {
        match self {
            Self::Lesser(_) => matches!(glyph, Glyph::Keystone),
            Self::Whole(_) => true,
        }
    }

    /// What the circle answers, limned `glyphs`, on every row.
    ///
    /// A lesser circle reads only the keystone; the others stand wherever they
    /// stand and change nothing.
    #[must_use]
    pub fn answer(self, glyphs: [Humour; 3]) -> Temper {
        match self {
            Self::Lesser(_) => lesser_answer(glyphs[Glyph::Keystone.index()]),
            Self::Whole(puzzle) => circuit::answer(puzzle.wiring, puzzle.turned, glyphs),
        }
    }

    /// What a hold worth `full` earns in this circle.
    #[must_use]
    pub const fn priced(self, full: u64) -> u64 {
        match self {
            Self::Lesser(_) => full / LESSER_SHARE,
            Self::Whole(_) => full,
        }
    }

    /// Whether this is a beast the circle could have drawn — what a restore
    /// checks.
    #[must_use]
    pub fn is_drawable(self) -> bool {
        match self {
            Self::Lesser(temper) => lessers().contains(&temper),
            Self::Whole(puzzle) => circuit::is_puzzle(puzzle),
        }
    }
}

/// What a keystone limned `humour` answers over two senses.
#[must_use]
pub fn lesser_answer(humour: Humour) -> Temper {
    let rows = 1 << LESSER_SENSES;
    let mut bits = 0u8;
    for row in 0..rows {
        let one = temper::sense_lit(row, 0, LESSER_SENSES);
        let other = temper::sense_lit(row, 1, LESSER_SENSES);
        if humour.answer(one, other) {
            bits |= 1 << row;
        }
    }
    Temper::from_bits(bits, LESSER_SENSES)
}

/// Every lesser temper, in [`Humour::ALL`]'s order: the six less the one the
/// opening keystone already answers, which would be a beast held uncalled.
#[must_use]
pub fn lessers() -> Vec<Temper> {
    let opening = OPENING[Glyph::Keystone.index()];
    Humour::ALL
        .into_iter()
        .filter(|humour| *humour != opening)
        .map(lesser_answer)
        .collect()
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::*;
    use crate::rng::RngStream;

    /// **Five, all different, and each held by exactly one humour** — so a
    /// lesser beast *is* the question *"which humour is this?"*, with one answer.
    #[test]
    fn five_lesser_tempers_each_held_by_one_humour_and_none_by_the_opening() {
        let tempers = lessers();
        assert_eq!(tempers.len(), 5);
        for temper in &tempers {
            let holding: Vec<Humour> = Humour::ALL
                .into_iter()
                .filter(|humour| lesser_answer(*humour) == *temper)
                .collect();
            assert_eq!(holding.len(), 1, "{temper:?} is held by {holding:?}");
            assert_ne!(holding[0], OPENING[Glyph::Keystone.index()]);
            assert_eq!(temper.rows(), 4);
        }
    }

    #[test]
    fn a_lesser_draw_takes_one_value_and_draws_every_temper() {
        let mut drawn = std::collections::BTreeSet::new();
        for seed in 0..60 {
            let mut rngs = Rngs::from_seed(seed);
            let _ = Shape::draw(&mut rngs, false);
            let mut stepped = Rngs::from_seed(seed);
            let _: u64 = stepped.stream(RngStream::Menagerie).random();
            let after_draw: u64 = rngs.stream(RngStream::Menagerie).random();
            let after_step: u64 = stepped.stream(RngStream::Menagerie).random();
            assert_eq!(after_draw, after_step, "seed {seed} took more than one");
            drawn.insert(Shape::draw(&mut Rngs::from_seed(seed), false));
        }
        assert_eq!(drawn.len(), 5, "60 seeds drew {drawn:?}");
    }

    #[test]
    fn a_lesser_circle_reads_only_the_keystone() {
        let shape = Shape::Lesser(lesser_answer(Humour::Oppose));
        assert!(shape.lights(Glyph::Keystone));
        assert!(!shape.lights(Glyph::Sunwise));
        assert!(!shape.lights(Glyph::Widdershins));
        let held = [Humour::Oppose, Humour::Mirror, Humour::Heed];
        assert_eq!(shape.answer(held), shape.temper());
        assert_eq!(shape.par(), 1);
        assert_eq!(shape.priced(8), 2);
    }
}
