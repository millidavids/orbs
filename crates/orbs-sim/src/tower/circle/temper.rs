//! A beast's temper — what it answers on every row — and the numbers every
//! circle shares.
//!
//! How a temper comes to be drawn is `circuit`'s: a circuit is drawn and the
//! temper is what it answers. This module is the table itself, which the lesser
//! circle and the whole one both use.

use super::{Humour, LESSER_SENSES};

/// How many senses a beast at the whole circle has. A lesser one has
/// [`LESSER_SENSES`].
pub const SENSES: usize = 3;

/// How many rows its temper has: one for every way the senses can be lit.
pub const ROWS: usize = 1 << SENSES;

/// How many rows a lesser beast's temper has, on [`ROWS`]' rule.
const LESSER_ROWS: usize = 1 << LESSER_SENSES;

/// How every glyph stands when a beast arrives.
///
/// Reset per beast, never carried over, so a spell always knows where it starts.
/// A circle left as the last beast held it would make every solver's first moves
/// depend on history the spell cannot read.
pub const OPENING: [Humour; 3] = [Humour::Yoke; 3];

/// How many calls a beast may take and still pay in full.
///
/// Three, because the temper is visible: a player who reads the table needs one
/// call, and PAR leaves room for two that check a guess. Past it a hold pays
/// three quarters, never nothing — the lens's `yield_of`, on the grounds that
/// the tick cost of a search already does the real work.
pub const PAR: u32 = 3;

/// Whether `sense` is lit on `row` of a table over `senses`.
///
/// The first sense is the row's highest bit, so row nought is every sense dark
/// and the last every sense lit — the order a truth table is written in, and the
/// order the board draws its columns.
#[must_use]
pub const fn sense_lit(row: usize, sense: usize, senses: usize) -> bool {
    (row >> (senses - 1 - sense)) & 1 == 1
}

/// A beast's answer on every row — or a circle's.
///
/// One byte, a row a bit, row nought lowest, plus the number of senses the rows
/// are over — three for the whole circle, two for the lesser (see
/// [`Shape`](super::Shape)). Eight rows fit a byte exactly; a four-row temper
/// masks the high half, so [`agreeing`](Self::agreeing) never counts rows the
/// table does not have.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Temper {
    bits: u8,
    senses: usize,
}

impl Temper {
    /// A temper from its bits over `senses`, row nought lowest. Bits past the
    /// last row are dropped.
    #[must_use]
    pub const fn from_bits(bits: u8, senses: usize) -> Self {
        Self {
            bits: bits & mask(1 << senses),
            senses,
        }
    }

    /// Its bits, row nought lowest.
    #[must_use]
    pub const fn bits(self) -> u8 {
        self.bits
    }

    /// How many senses its rows are over.
    #[must_use]
    pub const fn senses(self) -> usize {
        self.senses
    }

    /// How many rows it has.
    #[must_use]
    pub const fn rows(self) -> usize {
        1 << self.senses
    }

    /// Whether `row` is lit.
    #[must_use]
    pub const fn lit(self, row: usize) -> bool {
        row < self.rows() && (self.bits >> row) & 1 == 1
    }

    /// How many rows are lit — what the circle publishes as
    /// [`FERVOUR`](super::FERVOUR).
    #[must_use]
    pub const fn fervour(self) -> u32 {
        self.bits.count_ones()
    }

    /// How many rows two answers agree on, of this temper's rows.
    #[must_use]
    pub const fn agreeing(self, other: Self) -> u32 {
        (!(self.bits ^ other.bits) & mask(self.rows())).count_ones()
    }

    /// The rows as a save writes them: `1` lit, `0` dark, row one first.
    #[must_use]
    pub fn to_rows(self) -> String {
        (0..self.rows())
            .map(|row| if self.lit(row) { '1' } else { '0' })
            .collect()
    }

    /// Read rows back: four `0`s and `1`s for a lesser temper, eight for a whole
    /// one, and anything else refused.
    ///
    /// Both lengths derived from their circle's senses, so moving
    /// `LESSER_SENSES` moves what a save reads rather than refusing every lesser
    /// beast on load.
    #[must_use]
    pub fn from_rows(rows: &str) -> Option<Self> {
        let senses = match rows.chars().count() {
            LESSER_ROWS => LESSER_SENSES,
            ROWS => SENSES,
            _ => return None,
        };
        let mut bits = 0u8;
        for (row, cell) in rows.chars().enumerate() {
            match cell {
                '1' => bits |= 1 << row,
                '0' => {}
                _ => return None,
            }
        }
        Some(Self::from_bits(bits, senses))
    }

    /// Whether turning any one sense over changes some row's answer.
    #[must_use]
    pub fn turns_on_every_sense(self) -> bool {
        (0..self.senses).all(|sense| {
            let flip = 1 << (self.senses - 1 - sense);
            (0..self.rows()).any(|row| self.lit(row) != self.lit(row ^ flip))
        })
    }
}

/// The bits of a byte that `rows` rows occupy.
const fn mask(rows: usize) -> u8 {
    if rows >= 8 { u8::MAX } else { (1 << rows) - 1 }
}
