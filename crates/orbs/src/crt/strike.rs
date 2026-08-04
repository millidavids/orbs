//! The tube coming on: one soft flash, decaying.
//!
//! # The flash budget, computed
//!
//! This is the one thing in the game that can hurt somebody, so the arithmetic
//! is here rather than in a comment somewhere and it is asserted by a test.
//!
//! §14 and §19 both fix the band: **3–30 Hz provokes photosensitive reactions**,
//! and §19 records the tube already shipping a 19.1 Hz strobe once, found by
//! looking rather than by reasoning. WCAG 2.3.1 puts the ceiling at **three
//! general flashes in any one second.**
//!
//! The background is near-black (`palette`, relative luminance ≈ 0.003), so a
//! full-screen brightening is unambiguously a *general flash*. This spends
//! **exactly one — one rise, one fall, once per launch.**
//!
//! # The limit is per window, not per second of effect
//!
//! An earlier version of this file divided the flash count by the stage's length
//! and called the result a rate, which made a short strike look unsafe: one pair
//! over 0.3 s "computes" to 3.33/s and appears to breach the ceiling. **That is
//! the wrong arithmetic.** WCAG counts flashes occurring *within* any one-second
//! window, and a single non-repeating flash is one flash in that window however
//! briefly it lasts. A 0.25 s strike and a 0.9 s strike are both 1.
//!
//! So the stage's length is a **taste** decision, and the safety property is a
//! different one: the flash must rise once, fall once, and not recur. That is
//! what [`flash_at`]'s monotone decay gives and what the test below asserts.
//! Getting this wrong in the cautious direction still cost something — it was
//! the stated reason the strike had been slowed until it stopped reading as a
//! tube striking at all.
//!
//! An earlier version also swept a bright band down the tube, which *was* a
//! second flash at every pixel it crossed. That one is gone.
//!
//! # There is no persisted way to turn this off yet
//!
//! §14 makes the tube disableable, `F3` does it, and nothing persists that
//! choice — settings arrive with §15's Phase 5 screen, and the health warning
//! §14 records this product inheriting arrives with them. So the off switch
//! **cannot be the safety mechanism**, and the effect is inside the limit by
//! construction instead.

use bevy::prelude::*;

use super::settings::CrtSettings;
use crate::boot::{Boot, Stage};

/// How much of the tube's brightness the flash adds at its peak.
///
/// Additive on top of a near-black screen, so this *is* the flash's amplitude.
/// Deliberately gentle: a tube warming up, not a camera flash. The value it
/// replaced was more than twice this and read as a strobe rather than a strike.
const PEAK: f32 = 0.30;

/// Drive the tube through the boot strike.
///
/// Runs only during [`Stage::Strike`]; every other stage leaves the settings
/// alone, so a player who pressed `F3` is not overridden by an animation.
pub(super) fn drive(
    boot: Res<Boot>,
    theme: Res<crate::render::Theme>,
    settings: Option<Single<&mut CrtSettings>>,
) {
    let Some(mut settings) = settings else {
        return;
    };
    let strike = match boot.stage() {
        Stage::Strike => boot.progress(),
        // Anything before the strike is a dark tube; anything after has the
        // picture in it and no business being flashed.
        Stage::Dark => 0.0,
        _ => {
            if settings.flash != LinearRgba::NONE {
                settings.flash = LinearRgba::NONE;
            }
            return;
        }
    };

    // A tube that was turned off stays off. §14's switch is not something an
    // animation may talk over — and with `enabled` now explicit state rather
    // than `settings != OFF`, writing `flash` here can no longer switch it back
    // on by itself.
    if !settings.on {
        return;
    }

    // The active phosphor, not white: all three themes are §4's art direction,
    // and a violet tube that flashes white is a different machine for a moment.
    settings.flash = LinearRgba {
        alpha: flash_at(strike),
        ..theme.0.glow()
    };
}

/// The flash's alpha at `progress` through the strike.
///
/// One rise and one fall — a quick attack and a long decay, which is what a tube
/// striking does, and which is one flash pair rather than a strobe.
fn flash_at(progress: f32) -> f32 {
    let progress = progress.clamp(0.0, 1.0);
    let remaining = 1.0 - progress;
    PEAK * remaining * remaining
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The most general flashes any one-second window of the strike contains.
    ///
    /// This is what WCAG 2.3.1 actually bounds. Because the curve is monotone
    /// and runs once, the answer is one for every window that overlaps it and
    /// zero for every window that does not — **independent of how long the stage
    /// is**, which is the correction the module docs describe.
    fn flashes_in_the_worst_second() -> usize {
        let steps = 200u16;
        let stage = Stage::Strike.duration().as_secs_f32();
        let mut rises = 0usize;
        let mut previous = 0.0;
        for step in 0..=steps {
            let alpha = flash_at(f32::from(step) / f32::from(steps));
            if alpha > previous {
                rises += 1;
            }
            previous = alpha;
        }
        // Every rise is one flash; the whole stage fits inside a window whenever
        // it is under a second, and is only ever more spread out if it is not.
        let _ = stage;
        rises
    }

    #[test]
    fn the_strike_is_one_flash_and_stays_one_however_fast_it_runs() {
        // WCAG 2.3.1: no more than three general flashes in any one-second
        // window. On a near-black background a full-screen brightening is a
        // general flash, so what has to be bounded is how many of them a window
        // can contain — not how much of a second the effect occupies.
        //
        // Asserted over the curve rather than over a constant, because the thing
        // that would make this unsafe is the curve gaining a second rise. §19
        // records the 19.1 Hz strobe that got through by being split across two
        // constants that each looked fine on its own.
        assert!(
            flashes_in_the_worst_second() <= 3,
            "the strike flashes more than three times in a second",
        );
        assert_eq!(flashes_in_the_worst_second(), 1, "it should be exactly one");
    }

    #[test]
    fn it_is_one_pair_rather_than_a_strobe() {
        // Monotone decay: the alpha never rises again once it has started
        // falling, so there is exactly one rise and one fall however long the
        // stage is made.
        let mut previous = f32::MAX;
        for step in 0..=100u16 {
            let alpha = flash_at(f32::from(step) / 100.0);
            assert!(alpha <= previous, "the flash brightened again at {step}");
            previous = alpha;
        }
        assert_eq!(flash_at(1.0), 0.0, "the strike never finished");
        assert!(flash_at(0.0) > 0.0, "the strike never struck");
    }

    #[test]
    fn the_flash_is_a_tube_warming_rather_than_a_camera() {
        // The amplitude is additive on a near-black screen, so it *is* how
        // bright the flash gets. Anything approaching 1.0 whites out the tube.
        //
        // Asserted through `flash_at` rather than against `PEAK`, so this
        // measures the brightness the driver actually writes — a curve that
        // overshot its own peak would pass a check on the constant.
        let brightest = flash_at(0.0);
        assert!(brightest <= 0.4, "{brightest} is a strobe, not a strike");
        assert!(brightest >= 0.1, "{brightest} is not visible");
    }
}
