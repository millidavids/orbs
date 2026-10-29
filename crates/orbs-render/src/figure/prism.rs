//! The prism: sigils aligned in their sockets, and a focus searching the rest.

use super::rite::Rite;
use crate::pulse::{TICKS_PER_CYCLE, shared_tick, whole_share};
use crate::style::Depiction;

/// Ticks the focus takes to climb one cell while a ward is open.
///
/// Travelling, so the shared clock: a staggered one lands the step a tick early
/// on half the cells and the motion washes out. Slower than the mortar's two
/// and the bath's three, so a lens reads as searching rather than boiling.
pub const FOCUS_EVERY: u16 = 4;

/// Beats in a cycle, a whole number by the check below.
const BEATS: u16 = TICKS_PER_CYCLE / FOCUS_EVERY;
const _: () = assert!(TICKS_PER_CYCLE.is_multiple_of(FOCUS_EVERY));

/// One cell: an aligned sigil in the gauge's hue, or an open socket the focus
/// passes over.
pub(super) fn cell(
    _lane: u16,
    step: u16,
    filled: u16,
    total: u16,
    rite: Rite,
) -> (char, Depiction) {
    if step < filled {
        return ('█', Depiction::gauge(rite.fill));
    }
    if rite.working {
        // One mark climbing the open sockets and starting again at the face, so
        // the aligned part never carries anything but its own fill. Each climb
        // takes a whole share of the cycle and rests at the top for what is
        // over, so the cycle's wrap never cuts one short.
        let open = total.saturating_sub(filled).max(1);
        let at = shared_tick(rite.phase) / FOCUS_EVERY % whole_share(BEATS, open);
        if step - filled == at {
            return ('▒', Depiction::None);
        }
    }
    ('░', Depiction::None)
}
