//! The pylon: the wall as it stands, its top course settling while a course
//! is being raised.

use super::rite::Rite;
use crate::pulse::tick_of;
use crate::style::Depiction;

/// Ticks the top course holds a shade while a course is drawn.
pub const RAISE_EVERY: u16 = 9;

/// One cell: the wall in the gauge's hue, or open air above it.
///
/// Only the face moves, `█` to `▓` and back: a course is laid at the top of a
/// wall, and the rest of it is standing.
pub(super) fn cell(lane: u16, step: u16, filled: u16, rite: Rite) -> (char, Depiction) {
    if step >= filled {
        return ('░', Depiction::None);
    }
    let face = step + 1 == filled;
    if face && rite.working && (tick_of(rite.phase, lane, step) / RAISE_EVERY) % 2 == 1 {
        return ('▓', Depiction::gauge(rite.fill));
    }
    ('█', Depiction::gauge(rite.fill))
}
