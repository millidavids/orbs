//! The line on the wall: who stands on it, and the room it has (§19,
//! *Supply*).
//!
//! What a levy puts there, and what a refill, a heal or a fortify supply does
//! to it between rounds. `resolve` wounds it; this is everything else.

use super::{Band, Siege, VIGOUR};

impl Siege {
    /// Put the levied garrison on the wall, with room for at most `room`.
    ///
    /// After `begin_against` rather than inside it, so the draws stay where
    /// every replay has them.
    pub const fn muster(&mut self, levied: u32, room: u32) {
        self.garrison = Band::new(levied);
        self.mustered = levied;
        self.levied = levied;
        self.room = if room > levied { room } else { levied };
    }

    /// The most fight the garrison can hold: what it mustered, and what vigour
    /// raised it by. Not what [`hurt`](Self::hurt) measures against, or a
    /// potion would read as a wound.
    #[must_use]
    pub const fn full(&self) -> u32 {
        self.mustered * VIGOUR + self.fortified
    }

    /// Add bodies to the line, never past [`room`](Self::room). Returns how
    /// many joined.
    ///
    /// `mustered` rises with them, so reinforcing a worn line does not make it
    /// read as freshly wounded.
    pub const fn reinforce(&mut self, count: u32) -> u32 {
        let joined = if self.room > self.garrison.count {
            let space = self.room - self.garrison.count;
            if count < space { count } else { space }
        } else {
            0
        };
        self.garrison.count += joined;
        self.garrison.vigour += joined * VIGOUR;
        self.mustered += joined;
        joined
    }

    /// Put fight back into the garrison — what `quaff` does.
    pub fn heal(&mut self, points: u32) {
        self.garrison.mend(points, self.full(), self.room);
    }

    /// Raise what the garrison can hold, without healing it — vigour's gift.
    /// Mending and succour fill the room it makes.
    pub const fn fortify(&mut self, points: u32) {
        self.fortified += points;
    }

    /// Take away the room a fortify supply held, before this round's draws
    /// hold it again. [`clamp_line`](Self::clamp_line) after the draws settles
    /// what no longer fits. A [`kept`](Self::kept) room stays.
    pub const fn reset_room(&mut self) {
        self.fortified = self.kept;
    }

    /// Fight past what the line can now hold is lost, and the bodies with it —
    /// a fortified room is a level, not a gift.
    pub const fn clamp_line(&mut self) {
        let full = self.full();
        if self.garrison.vigour > full {
            self.garrison.vigour = full;
        }
        let standing = self.garrison.vigour.div_ceil(VIGOUR);
        if self.garrison.count > standing {
            self.garrison.count = standing;
        }
    }
}
