//! What a figure is and what it is doing, and the one dispatch to its picture.

use crate::style::{Depiction, Fill};

/// Which of the five instruments the picture is of.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Figure {
    /// The lens's prism: sigils aligned in their sockets.
    #[default]
    Prism,
    /// The forge's lattice: columns snapped and burning.
    Lattice,
    /// The sanctum's pylon: the wall as it stands.
    Pylon,
    /// The archive's lectern: four fragments becoming a scroll.
    Lectern,
    /// The menagerie's circle: a beast's temper, rows agreeing.
    Circle,
}

impl Figure {
    /// Every figure, for the tests that sweep them.
    pub const ALL: [Self; 5] = [
        Self::Prism,
        Self::Lattice,
        Self::Pylon,
        Self::Lectern,
        Self::Circle,
    ];
}

/// What a figure is doing, beyond how full it is.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Rite {
    /// Elapsed seconds from the frontend's own clock.
    pub phase: f32,
    /// Which instrument.
    pub figure: Figure,
    /// Whether it is at work. Nothing moves otherwise: an instrument that has
    /// not begun must not move (§10.1's at-rest rule), and motion off sets
    /// this false.
    pub working: bool,
    /// Whether the work in hand is a settle — the lattice annealing, cooling
    /// rather than catching.
    pub settling: bool,
    /// The fill's step on the gauge ramp, from the truth. The painter sets it,
    /// so no caller can hand a hue that disagrees with the length.
    pub fill: Fill,
}

/// One cell of a figure's bar.
///
/// `step` counts from the end the bar fills from; `filled` is the painter's
/// `filled_of`, so the picture and the plain meter agree where the value is.
/// Every figure keeps that join: a filled cell is `█` or `▓`, an empty one `░`
/// or `▒`, so the value survives greyscale whatever moves.
pub(crate) fn cell(lane: u16, step: u16, filled: u16, total: u16, rite: Rite) -> (char, Depiction) {
    if step >= total {
        return (' ', Depiction::None);
    }
    match rite.figure {
        Figure::Prism => super::prism::cell(lane, step, filled, total, rite),
        Figure::Lattice => super::lattice::cell(lane, step, filled, rite),
        Figure::Pylon => super::pylon::cell(lane, step, filled, rite),
        Figure::Lectern => super::lectern::cell(step, filled, total, rite),
        Figure::Circle => super::circle::cell(lane, step, filled, rite),
    }
}
