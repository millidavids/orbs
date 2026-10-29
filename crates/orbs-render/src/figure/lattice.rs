//! The lattice: snapped columns burning, catching while a charm is bound and
//! cooling while it anneals.

use super::rite::Rite;
use crate::pulse::{noise, tick_of};
use crate::style::{Depiction, Heat};

/// Ticks a lit cell holds its heat while a charm is being bound.
pub const BREATHE_EVERY: u16 = 6;

/// Ticks a cell holds its heat while the lattice anneals — slower, a settle.
pub const COOL_EVERY: u16 = 8;

/// One cell: a snapped column on the flame's ramp, or an unsnapped one dark.
///
/// One cell in eight lifts a step, on the staggered clock and a fresh draw a
/// beat, so the forge flickers without the column turning over at once.
pub(super) fn cell(lane: u16, step: u16, filled: u16, rite: Rite) -> (char, Depiction) {
    if step >= filled {
        return ('░', Depiction::None);
    }
    let (rest, lift, every) = if rite.settling {
        (Heat::Ember, Heat::Flame, COOL_EVERY)
    } else {
        (Heat::Flame, Heat::Blaze, BREATHE_EVERY)
    };
    if !rite.working {
        return ('█', Depiction::flame(rest));
    }
    let beat = tick_of(rite.phase, lane, step) / every;
    let heat = if noise(lane, step, beat) >> 5 == 7 {
        lift
    } else {
        rest
    };
    ('█', Depiction::flame(heat))
}
