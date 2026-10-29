//! The circle: a beast's temper, rows that agree standing still and rows that
//! do not pulling at the wires.

use super::rite::Rite;
use crate::pulse::{noise, tick_of};
use crate::style::Depiction;

/// Ticks a balking row holds its pull.
pub const PULL_EVERY: u16 = 6;

/// One cell: an agreeing row in the gauge's hue, or a balking one that pulls in
/// place. In place, never rising: rising means heat (`liquid`).
pub(super) fn cell(lane: u16, step: u16, filled: u16, rite: Rite) -> (char, Depiction) {
    if step < filled {
        return ('█', Depiction::gauge(rite.fill));
    }
    if rite.working {
        let beat = tick_of(rite.phase, lane, step) / PULL_EVERY;
        if noise(lane, step, beat) >> 6 == 3 {
            return ('▒', Depiction::None);
        }
    }
    ('░', Depiction::None)
}
