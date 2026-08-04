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
//! **one pair — one rise and one fall — across the whole of
//! [`Stage::Strike`](crate::boot::Stage)**, which at that stage's length is well
//! under a quarter of a flash per second.
//!
//! An earlier version also swept a bright band down the tube, which was a second
//! pair at every pixel it crossed and took the strike to 2.22/s — inside the
//! limit, but only because the stage had been lengthened to make it so. The
//! sweep is gone, and with it the only reason the stage length was a safety
//! constraint rather than a taste one.
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

    /// Luminance transitions the strike produces at one pixel, per second.
    ///
    /// The number the whole module exists to keep under three. Counted as
    /// *pairs* — a rise and its matching fall — because that is what WCAG 2.3.1
    /// counts.
    fn flashes_per_second() -> f32 {
        const PAIRS: f32 = 1.0;
        PAIRS / Stage::Strike.duration().as_secs_f32()
    }

    #[test]
    fn the_strike_stays_well_under_the_flash_limit() {
        // WCAG 2.3.1: no more than three general flashes in any one second. On a
        // near-black background a full-screen brightening is a general flash, so
        // this counts every source of one — analysing two effects separately is
        // exactly how §19's 19.1 Hz strobe got through, split across two
        // constants that each looked fine.
        let rate = flashes_per_second();
        assert!(rate <= 3.0, "{rate} general flashes per second");
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
