//! The caret's blink.
//!
//! Purely how a cell is drawn, never what appears: the caret's *position* is
//! carried by the `Frame`, and a blink adds no information a static block does
//! not already have. That is what architectural rule 2 permits a frontend to do
//! on its own — `orbs-tui` gets the terminal's own cursor and is none the
//! poorer for it.
//!
//! # The rate is a safety constraint, not a taste one
//!
//! §14 and the CRT port (§19) both landed on the same number: **3–30 Hz is the
//! photosensitive band**, and the tube's original "60Hz-ish" flicker turned out
//! to be 19.1 Hz sitting squarely inside it. A caret is small and dim where that
//! was the whole screen, but the rule is not worth relearning, so this blinks at
//! well under 1 Hz — slower than a VT100 and comfortably below the floor.
//!
//! Driven from `Time`, never from the tick: the sim must not be able to observe
//! it, or replay would depend on how long a frame took.

use bevy::prelude::*;

/// How long the caret spends solid, and then hidden.
///
/// 0.53 s each way — a ~0.94 Hz cycle, the shape a modern terminal uses. Below
/// the 3 Hz floor of the photosensitive band with a wide margin.
const HALF_CYCLE: f32 = 0.53;

/// The caret's blink phase.
#[derive(Resource, Debug)]
pub(crate) struct Blink {
    elapsed: f32,
    showing: bool,
}

impl Default for Blink {
    fn default() -> Self {
        // Starts solid: the first thing a player sees at the prompt should be a
        // caret, not the half of the cycle where there isn't one.
        Self {
            elapsed: 0.0,
            showing: true,
        }
    }
}

impl Blink {
    /// Whether the caret is drawn this frame.
    pub(crate) const fn showing(&self) -> bool {
        self.showing
    }

    /// Put the caret back on, solid, and restart the cycle.
    ///
    /// Typing must not hide what is being typed. Every terminal does this, and
    /// the reason is that a caret which happens to be mid-blink when a key lands
    /// reads as dropped input.
    pub(crate) const fn wake(&mut self) {
        self.elapsed = 0.0;
        self.showing = true;
    }

    /// Carry the phase forward by `seconds`.
    ///
    /// **Divided, not looped.** Draining whole half-cycles with
    /// `while elapsed >= HALF_CYCLE { elapsed -= HALF_CYCLE; … }` iterates once
    /// per half-cycle of elapsed time, so a frame arriving after a long stall — a
    /// breakpoint, a laptop lid, `Time`'s `max_delta` raised as `sim::clock` does
    /// — spins proportionally to the stall to reach a phase two arithmetic
    /// operations give directly. Repeated float subtraction also accumulates the
    /// error a single remainder does not.
    ///
    /// Separate from [`tick`] so it is testable without an `App`: the test that
    /// covered this used to re-run the loop inline and would have passed whatever
    /// the system did.
    pub(crate) fn advance(&mut self, seconds: f32) {
        self.elapsed += seconds;
        let cycles = (self.elapsed / HALF_CYCLE).floor();
        if cycles >= 1.0 {
            self.elapsed -= cycles * HALF_CYCLE;
            // Only the parity matters: two flips are no flip.
            if (cycles / 2.0).fract() != 0.0 {
                self.showing = !self.showing;
            }
        }
    }
}

/// Advance the blink. See [`Blink::advance`].
pub(crate) fn tick(time: Res<Time>, mut blink: ResMut<Blink>) {
    blink.advance(time.delta_secs());
}

/// Keep the caret solid while the player is typing.
pub(crate) fn wake(mut blink: ResMut<Blink>) {
    blink.wake();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rate_is_below_the_photosensitive_band() {
        // §14, and the lesson the CRT port paid for: 3–30 Hz is the band, and
        // "it looks about right" is how the tube ended up at 19.1 Hz.
        let hz = 1.0 / (HALF_CYCLE * 2.0);
        assert!(hz < 3.0, "{hz} Hz is inside the photosensitive band");
        assert!(hz > 0.5, "{hz} Hz is slow enough to look broken");
    }

    #[test]
    fn it_starts_solid_and_typing_keeps_it_solid() {
        let mut blink = Blink::default();
        assert!(blink.showing(), "the prompt opened with no caret");

        blink.showing = false;
        blink.wake();
        assert!(blink.showing(), "typing hid what was being typed");
    }

    #[test]
    fn a_long_frame_does_not_desynchronise_the_phase() {
        // A stall must not leave the caret stuck: the phase is caught up in
        // whole half-cycles rather than clamped to one flip per frame.
        //
        // This calls `advance` now. It used to re-run the drain loop *inline*,
        // which meant it asserted its own copy of the arithmetic and would have
        // passed whatever the real code did.
        let mut blink = Blink::default();
        assert!(blink.showing());

        // Four whole half-cycles and a bit: an even number of flips, so the
        // caret is back where it started, with the remainder carried.
        blink.advance(HALF_CYCLE * 4.5);
        assert!(blink.showing(), "four flips should land back on solid");
        assert!(blink.elapsed < HALF_CYCLE, "the remainder was not carried");

        // One more takes it to five: odd, so it flips.
        blink.advance(HALF_CYCLE);
        assert!(!blink.showing(), "the fifth half-cycle did not flip it");
    }

    #[test]
    fn one_half_cycle_at_a_time_and_all_at_once_agree() {
        // The property the divide replaced a loop to keep: catching up in one
        // call must land on the same phase as arriving there a frame at a time.
        let mut stepped = Blink::default();
        for _ in 0..7 {
            stepped.advance(HALF_CYCLE);
        }
        let mut jumped = Blink::default();
        jumped.advance(HALF_CYCLE * 7.0);

        assert_eq!(stepped.showing(), jumped.showing());
    }
}
