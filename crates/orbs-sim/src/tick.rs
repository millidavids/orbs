//! World time.
//!
//! A tick is the atomic unit of world time — the clock for drift, upkeep,
//! triggers, timers, aberrations, and accrual. It is **not** a resource the
//! player spends. See DESIGN.md §5.0.
//!
//! One tick is one real second while the window is open.

use bevy_ecs::prelude::*;

/// Ticks elapsed since the world began.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tick(pub u64);

impl Tick {
    /// Real seconds per tick. Durations in DESIGN.md §11.5 are expressed in
    /// seconds and convert 1:1.
    pub const SECONDS: f64 = 1.0;

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0 + 1)
    }

    /// Elapsed wall-clock seconds this tick count represents.
    #[must_use]
    pub fn as_seconds(self) -> f64 {
        self.0 as f64 * Self::SECONDS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_advances_by_one() {
        assert_eq!(Tick(41).next(), Tick(42));
    }

    #[test]
    fn seconds_convert_one_to_one() {
        assert!((Tick(90).as_seconds() - 90.0).abs() < f64::EPSILON);
    }
}
