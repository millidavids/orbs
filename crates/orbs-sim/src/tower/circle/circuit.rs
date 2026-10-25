//! How a beast is generated: a circuit is drawn from the seed, and its temper is
//! what that circuit answers.
//!
//! By construction, never invented and checked: a circuit is a [`Wiring`], its
//! [`Turned`] wires and a limning, and the temper is computed from it, so every
//! beast has a circle that holds it.
//!
//! Refused, each a puzzle with a question missing: lit on every row or none; a
//! sense that never changes the answer, making its wire a decoy; and what the
//! opening already answers, a free troop.
//!
//! [`drawable`] is walked once in a fixed order and a draw is one `u64` over
//! it. The distribution is by circuit, chosen rather than corrected (§19).

use std::sync::OnceLock;

use rand::RngExt;

use super::temper::{OPENING, ROWS, SENSES, Temper, sense_lit};
use super::{Glyph, Humour};
use crate::rng::{RngStream, Rngs};

/// Which two senses each outer glyph is wired to.
///
/// The two outer glyphs never share a pair, so between them all three senses
/// reach the keystone — and since the order of a glyph's two wires does not
/// matter to a symmetric humour, there are exactly six wirings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Wiring {
    /// The senses the sunwise glyph is given, lower first.
    pub sunwise: [usize; 2],
    /// The senses the widdershins glyph is given, lower first.
    pub widdershins: [usize; 2],
}

/// The three pairs of senses a glyph can be wired to.
const PAIRS: [[usize; 2]; 3] = [[0, 1], [0, 2], [1, 2]];

impl Wiring {
    /// All six, in a fixed order.
    #[must_use]
    pub fn all() -> Vec<Self> {
        let mut all = Vec::with_capacity(6);
        for sunwise in PAIRS {
            for widdershins in PAIRS {
                if sunwise != widdershins {
                    all.push(Self {
                        sunwise,
                        widdershins,
                    });
                }
            }
        }
        all
    }

    /// The senses a glyph is given, or `None` for the keystone, which is given
    /// what the other two answer.
    #[must_use]
    pub const fn senses_of(self, glyph: Glyph) -> Option<[usize; 2]> {
        match glyph {
            Glyph::Keystone => None,
            Glyph::Sunwise => Some(self.sunwise),
            Glyph::Widdershins => Some(self.widdershins),
        }
    }

    /// Read one back from its two pairs, refusing anything that is not a wiring.
    #[must_use]
    pub fn from_pairs(sunwise: [usize; 2], widdershins: [usize; 2]) -> Option<Self> {
        let wanted = Self {
            sunwise,
            widdershins,
        };
        Self::all().into_iter().find(|one| *one == wanted)
    }
}

/// Which wires into the outer glyphs are turned, each handing its glyph the sense
/// turned over.
///
/// Four wires, in the order a save writes them: sunwise's first and second, then
/// widdershins's first and second — *first* being the lower sense of the glyph's
/// pair, as [`Wiring`] writes it.
///
/// At most one a glyph: both turned is an unturned glyph wearing another humour
/// (De Morgan), a `~` that changes nothing. Nine masks, each showing something
/// real.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Turned(u8);

impl Turned {
    /// No wire turned — every beast before turned wires existed.
    pub const NONE: Self = Self(0);

    /// Every mask a circuit may carry, in a fixed order: none, one or the other
    /// for each glyph.
    pub const ALL: [Self; 9] = [
        Self(0b0000),
        Self(0b0001),
        Self(0b0010),
        Self(0b0100),
        Self(0b0101),
        Self(0b0110),
        Self(0b1000),
        Self(0b1001),
        Self(0b1010),
    ];

    /// Whether `glyph`'s `input`th wire — nought for its lower sense, one for its
    /// higher — is turned. The keystone's wires never are.
    #[must_use]
    pub const fn wire(self, glyph: Glyph, input: usize) -> bool {
        let bit = match glyph {
            Glyph::Keystone => return false,
            Glyph::Sunwise => input,
            Glyph::Widdershins => 2 + input,
        };
        input < 2 && (self.0 >> bit) & 1 == 1
    }

    /// Whether no wire is turned.
    #[must_use]
    pub const fn is_none(self) -> bool {
        self.0 == 0
    }

    /// Whether this is a mask a circuit may carry — see the type's doc.
    #[must_use]
    pub const fn is_allowed(self) -> bool {
        self.0 < 16 && self.0 & 0b11 != 0b11 && (self.0 >> 2) & 0b11 != 0b11
    }

    /// The four wires as a save writes them: `1` turned, `0` not, in the type
    /// doc's order.
    #[must_use]
    pub fn to_wires(self) -> String {
        (0..4)
            .map(|bit| if (self.0 >> bit) & 1 == 1 { '1' } else { '0' })
            .collect()
    }

    /// Read four wires back, refusing anything that is not four `0`s and `1`s
    /// naming an allowed mask. `0000` is accepted and is [`NONE`](Self::NONE).
    #[must_use]
    pub fn from_wires(wires: &str) -> Option<Self> {
        if wires.chars().count() != 4 {
            return None;
        }
        let mut bits = 0u8;
        for (bit, cell) in wires.chars().enumerate() {
            match cell {
                '1' => bits |= 1 << bit,
                '0' => {}
                _ => return None,
            }
        }
        Some(Self(bits)).filter(|turned| turned.is_allowed())
    }
}

/// What a circuit wired `wiring`, with `turned` wires, limned `glyphs` answers on
/// every row.
///
/// `glyphs` is in [`Glyph::ALL`]'s order: keystone, sunwise, widdershins.
#[must_use]
pub fn answer(wiring: Wiring, turned: Turned, glyphs: [Humour; 3]) -> Temper {
    let [keystone, sunwise, widdershins] = glyphs;
    let mut bits = 0u8;
    for row in 0..ROWS {
        let given = |glyph: Glyph, [one, other]: [usize; 2]| {
            (
                sense_lit(row, one, SENSES) != turned.wire(glyph, 0),
                sense_lit(row, other, SENSES) != turned.wire(glyph, 1),
            )
        };
        let (a, b) = given(Glyph::Sunwise, wiring.sunwise);
        let (c, d) = given(Glyph::Widdershins, wiring.widdershins);
        if keystone.answer(sunwise.answer(a, b), widdershins.answer(c, d)) {
            bits |= 1 << row;
        }
    }
    Temper::from_bits(bits, SENSES)
}

/// A beast to hold at the whole circle: how the circle is wired for it, which
/// wires are turned, and what it answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Puzzle {
    /// Which senses reach which glyph.
    pub wiring: Wiring,
    /// Which of those wires are turned.
    pub turned: Turned,
    /// What every row must answer.
    pub temper: Temper,
}

/// One circuit: everything a beast is built from, the limning included.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Circuit {
    /// Which senses reach which glyph.
    pub wiring: Wiring,
    /// Which of those wires are turned.
    pub turned: Turned,
    /// A limning that holds the beast, in [`Glyph::ALL`]'s order.
    pub glyphs: [Humour; 3],
}

/// Every way of limning the three glyphs — 216 — keystone first, each in
/// [`Humour::ALL`]'s order.
pub fn limnings() -> impl Iterator<Item = [Humour; 3]> {
    Humour::ALL.into_iter().flat_map(|keystone| {
        Humour::ALL.into_iter().flat_map(move |sunwise| {
            Humour::ALL
                .into_iter()
                .map(move |widdershins| [keystone, sunwise, widdershins])
        })
    })
}

impl Circuit {
    /// Every circuit, in a fixed order: wiring, then turned wires, then the
    /// limning in [`limnings`]' order.
    pub fn all() -> impl Iterator<Item = Self> {
        Wiring::all().into_iter().flat_map(|wiring| {
            Turned::ALL.into_iter().flat_map(move |turned| {
                limnings().map(move |glyphs| Self {
                    wiring,
                    turned,
                    glyphs,
                })
            })
        })
    }

    /// The beast this circuit makes, or `None` if it is refused — see the module
    /// doc for the three refusals.
    #[must_use]
    pub fn puzzle(self) -> Option<Puzzle> {
        let temper = answer(self.wiring, self.turned, self.glyphs);
        (self.turned.is_allowed() && asks_something(self.wiring, self.turned, temper)).then_some(
            Puzzle {
                wiring: self.wiring,
                turned: self.turned,
                temper,
            },
        )
    }
}

/// Whether a temper is a question worth drawing under this wiring and these
/// turned wires: it turns on every sense, and the opening does not already hold it.
fn asks_something(wiring: Wiring, turned: Turned, temper: Temper) -> bool {
    temper.turns_on_every_sense() && temper != answer(wiring, turned, OPENING)
}

/// Every beast a draw can make, one entry a valid circuit, in [`Circuit::all`]'s
/// order.
///
/// One entry per circuit, not per distinct puzzle, which is what makes the
/// draw's distribution by circuit: a beast six limnings hold appears six times.
/// Built once in a fixed order, so a given draw means the same on every run and
/// every machine. The proofs pin its length and its contents.
#[must_use]
pub fn drawable() -> &'static [Puzzle] {
    static TABLE: OnceLock<Vec<Puzzle>> = OnceLock::new();
    TABLE.get_or_init(|| Circuit::all().filter_map(Circuit::puzzle).collect())
}

/// Whether a wiring, turned wires and temper make a beast some circuit could
/// have made — what a restored beast is checked against.
///
/// Asked of the rules, not looked up, so a hand-edited save is judged by what
/// makes a beast a beast: an allowed mask, a question worth asking, and a
/// limning that holds it.
#[must_use]
pub fn is_puzzle(puzzle: Puzzle) -> bool {
    puzzle.turned.is_allowed()
        && asks_something(puzzle.wiring, puzzle.turned, puzzle.temper)
        && limnings().any(|glyphs| answer(puzzle.wiring, puzzle.turned, glyphs) == puzzle.temper)
}

/// Draw a beast for the whole circle.
///
/// One value from the stream whatever the list's size, through `pick`. The
/// `fervour` a spell reads is never nought, which the proofs hold.
#[must_use]
pub fn draw(rngs: &mut Rngs) -> Puzzle {
    pick(rngs, drawable())
}

/// One entry of `table`, for exactly one value from the menagerie's stream.
///
/// The one copy of the rule both draws cite. Not `random_range`, which rejects
/// and redraws; one `u64` modulo a table this small is biased below one part in
/// 10^15. The tables are never empty, which the proofs hold.
pub(super) fn pick<T: Copy>(rngs: &mut Rngs, table: &[T]) -> T {
    let value: u64 = rngs.stream(RngStream::Menagerie).random();
    let len = u64::try_from(table.len()).unwrap_or(u64::MAX).max(1);
    let index = usize::try_from(value % len).unwrap_or(0);
    table[index]
}
