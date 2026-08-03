//! World time.
//!
//! A tick is the atomic unit of world time — the clock for drift, upkeep,
//! triggers, timers, aberrations, and accrual. It is **not** a resource the
//! player spends. See `docs/DESIGN.md` §5.0.
//!
//! One tick is one real second while the window is open.

use bevy_ecs::prelude::*;

/// Ticks elapsed since the world began.
///
/// The inner counter is private so the only ways to advance are [`Tick::next`]
/// and [`crate::Sim::step`]. Arbitrary mutation of world time would break replay
/// and offline/online parity.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tick(u64);

impl Tick {
    /// Real seconds per tick.
    ///
    /// Durations in `docs/DESIGN.md` §11.5 are expressed in seconds and convert
    /// 1:1.
    pub const SECONDS: f64 = 1.0;

    /// Largest tick that converts to `f64` seconds without precision loss.
    ///
    /// `2^53` ticks is roughly 285 million years, so this bound is theoretical.
    const EXACT_F64_LIMIT: u64 = 1 << 53;

    /// A tick count. Prefer [`Sim::step`](crate::Sim::step) for advancing time;
    /// this exists for construction in tests and for loading a save.
    #[must_use]
    pub const fn new(ticks: u64) -> Self {
        Self(ticks)
    }

    /// The elapsed tick count.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    /// The next tick.
    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0 + 1)
    }

    /// Elapsed wall-clock seconds this tick count represents.
    ///
    /// # Panics
    ///
    /// Debug builds only, if the count exceeds `2^53` ticks and the conversion
    /// would lose precision.
    #[must_use]
    pub fn as_seconds(self) -> f64 {
        debug_assert!(
            self.0 < Self::EXACT_F64_LIMIT,
            "tick count exceeds exact f64 range"
        );
        // Bounded above by the assertion, so the conversion is exact.
        #[allow(clippy::cast_precision_loss)]
        let ticks = self.0 as f64;
        ticks * Self::SECONDS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_advances_by_one() {
        assert_eq!(Tick::new(41).next(), Tick::new(42));
    }

    #[test]
    fn seconds_convert_one_to_one() {
        assert!((Tick::new(90).as_seconds() - 90.0).abs() < f64::EPSILON);
    }

    #[test]
    fn default_starts_at_zero() {
        assert_eq!(Tick::default().get(), 0);
    }

    #[test]
    fn ordering_follows_elapsed_time() {
        assert!(Tick::new(1) < Tick::new(2));
    }
}
