//! How far the next herb has grown toward flowering (§19, *Supply*).
//!
//! Verdant is cultivation: each draw of its supply grows the next herb the
//! laboratory has never held, and at the authored threshold it flowers on the
//! shelf for good. The growth is the tower's, not the supply's, so a supply
//! that runs dry keeps what it grew.

use bevy_ecs::prelude::*;

/// Growth toward the next herb. Saved; absent reads as nought, so no `FORMAT`
/// bump.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Cultivation(u32);

impl Cultivation {
    /// How far the next herb has grown.
    #[must_use]
    pub const fn grown(self) -> u32 {
        self.0
    }

    /// Put growth back, for a save.
    pub const fn restore(&mut self, grown: u32) {
        self.0 = grown;
    }

    /// Grow by `worth`, and say how many herbs that flowers at `flowers_at`.
    /// What is left over carries to the next.
    pub const fn grow(&mut self, worth: u32, flowers_at: u32) -> u32 {
        let total = self.0.saturating_add(worth);
        if flowers_at == 0 {
            self.0 = total;
            return 0;
        }
        self.0 = total % flowers_at;
        total / flowers_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn growth_carries_past_a_flowering() {
        let mut grown = Cultivation::default();
        assert_eq!(grown.grow(2, 3), 0);
        assert_eq!(grown.grow(2, 3), 1);
        assert_eq!(grown.grown(), 1);
    }
}
