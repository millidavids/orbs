//! The tube coming on: one flash, then one roll pass.
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
//! The background is near-black (`palette`, relative luminance ≈ 0.003), so
//! *both* effects here are general flashes rather than one being a flash and the
//! other being motion:
//!
//! | | Pairs |
//! |---|---|
//! | The flash — up, then down | 1 |
//! | The roll — one bright band crossing, so up-then-down at every pixel it passes | 1 |
//! | **Total** | **2** |
//!
//! Two pairs in 0.6 s would be **3.33/s — over the limit.** Over
//! [`Stage::Strike`](crate::boot::Stage)'s 0.9 s it is **2.22/s**, inside it with
//! margin. That is why the stage is 0.9 s and not the 0.6 s that would feel
//! snappier, and `boot::stage` carries a test saying so from the other side.
//!
//! They are also **separated in time** within the stage — the flash decays
//! before the roll begins — so no pixel sees both inside the 200 ms that would
//! make them read as one faster pair.
//!
//! # There is no persisted way to turn this off yet
//!
//! §14 makes the tube disableable, `F3` does it, and nothing persists that
//! choice — settings arrive with §15's Phase 5 screen, and the health warning
//! §14 records this product inheriting arrives with them. So the off switch
//! **cannot be the safety mechanism**, and the effect is inside the limit by
//! construction instead. That is the whole adjustment; it is not a reason to
//! soften the numbers above later.

use bevy::prelude::*;

use super::settings::CrtSettings;
use crate::boot::{Boot, Stage};

/// Fraction of the strike spent on the flash, before the roll begins.
///
/// The two must not overlap: separated, a pixel sees one pair then the other;
/// together, it sees one brighter pair twice as fast.
const FLASH_SHARE: f32 = 0.35;

/// How much of the tube's brightness the flash adds at its peak.
///
/// Additive on top of a near-black screen, so this *is* the flash's amplitude.
const PEAK: f32 = 0.75;

/// Passes the roll band makes across the tube during its share of the stage.
///
/// **One.** Two would double the flash count and put the strike over the WCAG
/// ceiling — see the module docs.
const ROLL_PASSES: f32 = 1.0;

/// Drive the tube through the boot strike.
///
/// Runs only during [`Stage::Strike`]; every other stage leaves the settings
/// alone, so a player who pressed `F3` during a previous run is not overridden
/// by an animation.
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
            if settings.flash != LinearRgba::NONE || settings.sweep != 0.0 {
                settings.flash = LinearRgba::NONE;
                settings.sweep = 0.0;
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
    // and a violet tube that flashes white is a different machine for a fifth
    // of a second.
    settings.flash = LinearRgba {
        alpha: flash_at(strike),
        ..theme.0.glow()
    };
    settings.sweep = sweep_at(strike);
}

/// Where the sweep band sits at `progress` through the strike, 0 → 1 down the
/// tube. Zero means no band.
///
/// Begins only once the flash has decayed: [`FLASH_SHARE`] is the seam, and
/// keeping them apart is what makes this two pairs at 2.22/s instead of one
/// brighter pair at twice the rate.
fn sweep_at(progress: f32) -> f32 {
    let progress = progress.clamp(0.0, 1.0);
    if progress < FLASH_SHARE {
        return 0.0;
    }
    let through = (progress - FLASH_SHARE) / (1.0 - FLASH_SHARE);
    // Never exactly zero once it has started, because zero is the shader's
    // "no band" sentinel and the band starts at the top of the tube.
    (through * ROLL_PASSES).max(f32::EPSILON)
}

/// The flash's alpha at `progress` through the strike.
///
/// Rises fast and decays over the rest of its share — one pair, not a strobe.
fn flash_at(progress: f32) -> f32 {
    let progress = progress.clamp(0.0, 1.0);
    if progress >= FLASH_SHARE {
        return 0.0;
    }
    let through = progress / FLASH_SHARE;
    // A quick attack and a longer fall, which is what a tube striking does.
    PEAK * (1.0 - through) * (1.0 - through)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::time::Duration;

    /// Luminance transitions the strike produces at one pixel, per second.
    ///
    /// The number the whole module exists to keep under three. Counted as
    /// *pairs* — an up and its matching down — because that is what WCAG 2.3.1
    /// counts, and counted across **both** effects together because a pixel does
    /// not care which of them brightened it.
    fn flashes_per_second() -> f32 {
        const FLASH_PAIRS: f32 = 1.0;
        let pairs = FLASH_PAIRS + ROLL_PASSES;
        pairs / Stage::Strike.duration().as_secs_f32()
    }

    #[test]
    fn the_strike_stays_under_the_flash_limit() {
        // WCAG 2.3.1: no more than three general flashes in any one second. On a
        // near-black background both the flash and the roll are general flashes,
        // so they are counted together — analysing them separately is exactly
        // how §19's 19.1 Hz strobe got through, split across two constants that
        // each looked fine.
        let rate = flashes_per_second();
        assert!(rate <= 3.0, "{rate} general flashes per second");
        // And not so slow it stops reading as a strike.
        assert!(rate >= 1.0, "{rate}/s is not a tube coming on");
    }

    #[test]
    fn shortening_the_strike_breaks_the_budget_loudly() {
        // The number someone will reach for to make boot feel snappier. 0.6s is
        // 3.33/s, which is over — so this records that the stage length is a
        // safety constraint rather than a taste call.
        let pairs = 1.0 + ROLL_PASSES;
        let snappier = Duration::from_millis(600);
        assert!(
            pairs / snappier.as_secs_f32() > 3.0,
            "0.6s is supposed to be the value that fails",
        );
    }

    #[test]
    fn the_flash_and_the_roll_do_not_overlap() {
        // Separated in time, so no pixel sees both pairs inside the ~200ms that
        // would make them read as one pair at twice the rate.
        assert_eq!(flash_at(FLASH_SHARE), 0.0);
        assert_eq!(flash_at(1.0), 0.0);
        assert!(flash_at(0.0) > 0.0, "the strike never struck");

        assert_eq!(sweep_at(0.0), 0.0, "the sweep started during the flash");
        assert_eq!(sweep_at(FLASH_SHARE - 0.01), 0.0);
        assert!(sweep_at(1.0) > 0.0, "the sweep never crossed");

        for step in 0..=100u16 {
            let at = f32::from(step) / 100.0;
            assert!(
                flash_at(at) == 0.0 || sweep_at(at) == 0.0,
                "both were lit at {at}",
            );
        }
    }

    #[test]
    fn the_sweep_crosses_the_tube_exactly_once() {
        // Two passes would double the flash count and put the strike over the
        // WCAG ceiling. Monotone and ending at one pass is what "once" means.
        assert!((sweep_at(1.0) - ROLL_PASSES).abs() < 0.01);
        let mut previous = 0.0;
        for step in 0..=100u16 {
            let at = sweep_at(f32::from(step) / 100.0);
            assert!(at >= previous, "the band went back up the tube");
            previous = at;
        }
    }

    #[test]
    fn it_is_one_pair_rather_than_a_strobe() {
        // Monotone decay: the alpha never rises again once it has started
        // falling, so there is exactly one up and one down.
        let mut previous = f32::MAX;
        for step in 0..=100u16 {
            let alpha = flash_at(f32::from(step) / 100.0);
            assert!(alpha <= previous, "the flash brightened again at {step}");
            previous = alpha;
        }
    }

    #[test]
    fn the_roll_is_far_below_the_photosensitive_floor() {
        // One pass over the strike's roll share. §14's band starts at 3 Hz and
        // the tube's own hum sits at 0.22 Hz for the same reason.
        let roll_seconds = Stage::Strike.duration().as_secs_f32() * (1.0 - FLASH_SHARE);
        let hz = ROLL_PASSES / roll_seconds;
        assert!(hz < 3.0, "{hz} Hz is inside the photosensitive band");
    }
}
