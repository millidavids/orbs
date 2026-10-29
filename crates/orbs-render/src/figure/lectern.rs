//! The lectern: four fragments knitting into one scroll.

use super::rite::Rite;
use crate::pulse::{TICKS_PER_CYCLE, shared_tick};
use crate::style::Depiction;

/// Ticks between one seam knitting and the next while a scroll is assembled —
/// the slowest picture in the tower, for the slowest work in it. Eighteen, so a
/// cycle is a whole number of passes: sixteen gave nine beats, and the wrap
/// held the open seams twice as long once every twenty-four seconds.
pub const KNIT_EVERY: u16 = 18;

/// The fragments a scroll is made of, and so the segments the bar shows.
const FRAGMENTS: u16 = 4;

const _: () = assert!(
    TICKS_PER_CYCLE.is_multiple_of(KNIT_EVERY)
        && (TICKS_PER_CYCLE / KNIT_EVERY).is_multiple_of(FRAGMENTS)
);

/// One cell: the scroll so far, its seams open until they knit, or the empty
/// lectern. No depiction, so the fragments' own colour reaches it.
pub(super) fn cell(step: u16, filled: u16, total: u16, rite: Rite) -> (char, Depiction) {
    if step >= filled {
        return ('░', Depiction::None);
    }
    if rite.working {
        // The seams between the four, at the bar's quarters, close one at a
        // time from the base and open again for the next pass.
        let knit = shared_tick(rite.phase) / KNIT_EVERY % FRAGMENTS;
        let seam = (1..FRAGMENTS).find(|quarter| step == total * quarter / FRAGMENTS);
        if seam.is_some_and(|quarter| quarter > knit) {
            return ('▓', Depiction::None);
        }
    }
    ('█', Depiction::None)
}
