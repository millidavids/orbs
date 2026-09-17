//! A beast waiting at the circle, and the circle being limned for it.

use bevy_ecs::prelude::*;

use super::circuit;
use super::shape::{LESSER_TROOPS, Shape};
use super::temper::{self, OPENING, Temper};
use super::{Glyph, Humour};
use crate::rng::Rngs;

/// Troops a hold within par brings. See [`Beast::troops`].
pub const TROOPS: u32 = 4;

/// Troops a hold past par brings.
pub const TROOPS_PAST_PAR: u32 = 3;

/// What calling a beast into the circle came to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Call {
    /// Every row agreed. The beast is held.
    Held,
    /// Some rows did not.
    Balked {
        /// How many of the temper's rows the circle answered differently.
        rows: u32,
    },
}

/// A beast at the circle: its temper, how it is wired, and how the glyphs stand.
///
/// **There is no clock in here, and that is the design.** A beast waits for
/// ever; nothing about it moves on a tick, so the circle has no system in the
/// schedule and a player may take an hour over one glyph. §10.1's *"never a
/// reflex"* holds in every room again, and §19 records the rhythm game this
/// replaced.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Beast {
    /// Which circle holds it, and what it answers.
    shape: Shape,
    /// How the glyphs stand, in [`Glyph::ALL`]'s order.
    ///
    /// **All three even at a lesser circle**, standing at the opening: the
    /// dark two are never read, and one shape of array keeps a save and a
    /// search the same whichever circle it is.
    glyphs: [Humour; 3],
    /// How many times the beast has been called in.
    ///
    /// **The draw is not a call.** Only a `summon` with a beast already waiting
    /// counts, and every one does — calling in a circle nobody changed tells a
    /// player nothing they did not know, and it is theirs to spend.
    calls: u32,
    /// What the circle answered the last time it was called, if it has been.
    ///
    /// **The answer, not a count of rows**, because the board draws which rows
    /// balked and a count cannot say which.
    answer: Option<Temper>,
}

impl Beast {
    /// A beast arriving as `shape`, every glyph at the [`OPENING`].
    #[must_use]
    pub const fn arriving(shape: Shape) -> Self {
        Self {
            shape,
            glyphs: OPENING,
            calls: 0,
            answer: None,
        }
    }

    /// Draw a beast from the menagerie's stream — for the whole circle, or the
    /// lesser one while the tower has not opened it.
    #[must_use]
    pub fn draw(rngs: &mut Rngs, whole: bool) -> Self {
        Self::arriving(Shape::draw(rngs, whole))
    }

    /// Put one back as a save left it, or `None` if it is not a beast the
    /// circle could have drawn.
    ///
    /// **Refused rather than repaired**, on `Chant::from_save`'s precedent: a
    /// hand-edited temper no circle can answer would be unholdable, and no beast
    /// at all is a `summon` away from fine. The draw is not repeated either — a
    /// restore reads the beast rather than rolling one, or loading a tower would
    /// move every later draw in the session.
    ///
    /// An answer of the wrong length is forgotten rather than refused: it only
    /// draws the last call's row, and the next call writes it again.
    #[must_use]
    pub fn restored(
        shape: Shape,
        glyphs: [Humour; 3],
        calls: u32,
        answer: Option<Temper>,
    ) -> Option<Self> {
        let answer = answer.filter(|answer| answer.senses() == shape.senses());
        shape.is_drawable().then_some(Self {
            shape,
            glyphs,
            calls,
            answer,
        })
    }

    /// Write one out for a save.
    pub(crate) fn to_save(&self) -> crate::save::BeastSave {
        let wiring = self.shape.wiring();
        let turned = self.shape.turned();
        crate::save::BeastSave {
            sunwise: wiring.map(|wiring| wiring.sunwise),
            widdershins: wiring.map(|wiring| wiring.widdershins),
            // **Absent when nothing is turned**, so a beast with no turned wire
            // writes exactly what a format-13 document held.
            turned: (!turned.is_none()).then(|| turned.to_wires()),
            temper: self.shape.temper().to_rows(),
            glyphs: self
                .glyphs
                .iter()
                .map(|humour| humour.word().to_owned())
                .collect(),
            calls: self.calls,
            answer: self.answer.map(Temper::to_rows),
        }
    }

    /// Put one back, or `None` if the document does not describe a beast the
    /// circle could have drawn.
    ///
    /// **The temper's length says which circle**: four rows and no wiring is a
    /// lesser beast, eight rows and both pairs a whole one, and anything else —
    /// a lesser temper with wiring, a whole one without — is no beast.
    ///
    /// **Turned wires belong to the whole circle.** Absent is none turned, which
    /// is what every beast before turned wires was; a mask that is not four
    /// `0`s and `1`s naming an allowed one is no beast, and so is any mask on a
    /// lesser beast — even `0000`, since the lesser circle has no wires to turn.
    pub(crate) fn from_save(save: &crate::save::BeastSave) -> Option<Self> {
        let temper = Temper::from_rows(&save.temper)?;
        let shape = match (temper.senses(), save.sunwise, save.widdershins) {
            (super::LESSER_SENSES, None, None) if save.turned.is_none() => Shape::Lesser(temper),
            (temper::SENSES, Some(sunwise), Some(widdershins)) => {
                let wiring = circuit::Wiring::from_pairs(sunwise, widdershins)?;
                let turned = match save.turned.as_deref() {
                    None => circuit::Turned::NONE,
                    Some(wires) => circuit::Turned::from_wires(wires)?,
                };
                Shape::Whole(circuit::Puzzle {
                    wiring,
                    turned,
                    temper,
                })
            }
            _ => return None,
        };
        let glyphs: Vec<Humour> = save
            .glyphs
            .iter()
            .map(|word| Humour::from_word(word))
            .collect::<Option<_>>()?;
        let glyphs: [Humour; 3] = glyphs.try_into().ok()?;
        let answer = save.answer.as_deref().and_then(Temper::from_rows);
        Self::restored(shape, glyphs, save.calls, answer)
    }

    /// Which circle holds it, and what it answers.
    #[must_use]
    pub const fn shape(&self) -> Shape {
        self.shape
    }

    /// How every glyph stands, in [`Glyph::ALL`]'s order.
    #[must_use]
    pub const fn glyphs(&self) -> [Humour; 3] {
        self.glyphs
    }

    /// How one glyph stands.
    #[must_use]
    pub const fn humour(&self, glyph: Glyph) -> Humour {
        self.glyphs[glyph.index()]
    }

    /// How many times the beast has been called in.
    #[must_use]
    pub const fn calls(&self) -> u32 {
        self.calls
    }

    /// What the circle answered the last time, if it has been called.
    #[must_use]
    pub const fn answer(&self) -> Option<Temper> {
        self.answer
    }

    /// How many rows the temper has — four at a lesser circle, eight at the
    /// whole one.
    #[must_use]
    pub const fn rows(&self) -> usize {
        self.shape.rows()
    }

    /// How many rows the last call agreed on — nought before the first. What
    /// the rail's meter counts, so its remainder is the rows still balking.
    #[must_use]
    pub fn agreeing(&self) -> u32 {
        self.answer
            .map_or(0, |answer| answer.agreeing(self.shape.temper()))
    }

    /// Whether a hold now would pay in full.
    #[must_use]
    pub const fn within_par(&self) -> bool {
        self.calls <= self.shape.par()
    }

    /// How many troops a hold now would bring to the arsenal.
    ///
    /// **Four within par, three past it** — the chant's clean figure and its
    /// scraped one, so the siege's supply is what it was for a player who reads
    /// the table, and a search pays the same three a scraped chant did. First
    /// pass: `orbs-balance` measures what a bound search supplies against what a
    /// siege spends, and this is the number that moves if it falls short.
    ///
    /// **One at a lesser circle, either way** — it is a lesson, and the siege is
    /// fed by the whole circle (see [`LESSER_TROOPS`]).
    #[must_use]
    pub const fn troops(&self) -> u32 {
        match self.shape {
            Shape::Lesser(_) => LESSER_TROOPS,
            Shape::Whole(_) if self.within_par() => TROOPS,
            Shape::Whole(_) => TROOPS_PAST_PAR,
        }
    }

    /// Limn a glyph with a humour.
    pub const fn limn(&mut self, glyph: Glyph, humour: Humour) {
        self.glyphs[glyph.index()] = humour;
    }

    /// Step a glyph to the next humour, wrapping, and say which it stands at now.
    ///
    /// **What makes a search writable.** `for each humour` binds the set's own
    /// word, so a spell cannot nest it three deep; three literal `repeat 6`
    /// loops around a bare `limn <glyph>` can, and that is the lens's bare
    /// `dial <socket>` one room over.
    pub const fn step(&mut self, glyph: Glyph) -> Humour {
        let next = self.glyphs[glyph.index()].next();
        self.glyphs[glyph.index()] = next;
        next
    }

    /// Call the beast into the circle as it stands.
    pub fn call(&mut self) -> Call {
        self.calls = self.calls.saturating_add(1);
        let answered = self.shape.answer(self.glyphs);
        self.answer = Some(answered);
        let rows = u32::try_from(self.shape.rows()).unwrap_or(u32::MAX);
        let agreeing = answered.agreeing(self.shape.temper());
        if agreeing == rows {
            Call::Held
        } else {
            Call::Balked {
                rows: rows.saturating_sub(agreeing),
            }
        }
    }

    /// One way of limning the circle that holds this beast.
    ///
    /// **The first in `circuit::limnings`' order** — the one the proofs walk, so
    /// the two cannot disagree about which solution comes first — and it is the
    /// same answer on every run. At a lesser circle that leaves the dark glyphs
    /// at the opening, where they stand, since the opening's humour leads. Only `debug_circle` asks — a tester's door past
    /// the puzzle to what a hold does, as `debug_ward` is one room over — and
    /// nothing a player or a spell can reach reads it.
    #[must_use]
    pub fn solution(&self) -> Option<[Humour; 3]> {
        circuit::limnings().find(|glyphs| self.shape.answer(*glyphs) == self.shape.temper())
    }
}
