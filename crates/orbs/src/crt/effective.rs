//! What the shader is handed: the tube's state, as the player has tempered it.
//!
//! The one place the tube's inputs meet, so no second path can combine them
//! differently.

use super::picture::{Effect, Picture, Strength};
use super::settings::CrtSettings;

/// The tube as it is drawn.
///
/// Each effect is scaled toward *its own* off value, not toward zero: the
/// vignette's radius is `1.0` when off, so halving it would widen the
/// darkness. `still` is §14's motion switch, and the hum band is the only term
/// that moves, so stillness takes it and leaves the shape.
pub(crate) fn effective(settings: CrtSettings, picture: Picture, still: bool) -> CrtSettings {
    let off = CrtSettings::OFF;
    // The ends exactly, so `full` is the tube as it was to the bit.
    let toward = |state: f32, rest: f32, effect: Effect| match picture.of(effect) {
        Strength::Off => rest,
        Strength::Full => state,
        soft @ Strength::Soft => (state - rest).mul_add(soft.share(), rest),
    };
    let mut drawn = settings;
    drawn.barrel = toward(settings.barrel, off.barrel, Effect::Curve);
    drawn.scanline = toward(settings.scanline, off.scanline, Effect::Lines);
    drawn.glow = toward(settings.glow, off.glow, Effect::Glow);
    drawn.flicker = if still {
        off.flicker
    } else {
        toward(settings.flicker, off.flicker, Effect::Flicker)
    };
    drawn.mask = toward(settings.mask, off.mask, Effect::Mask);
    drawn.aberration = toward(settings.aberration, off.aberration, Effect::Split);
    drawn.vignette = toward(settings.vignette, off.vignette, Effect::Edge);
    drawn.vignette_radius = toward(settings.vignette_radius, off.vignette_radius, Effect::Edge);
    // The glass's rim and its answer to the tower are one row: `edge off`
    // means the glass never answers (§19, *the tower speaks up*).
    drawn.desaturation = toward(settings.desaturation, off.desaturation, Effect::Edge);
    drawn
}

#[cfg(test)]
mod tests {
    use super::*;

    const PRESETS: [CrtSettings; 2] = [CrtSettings::DEFAULT, CrtSettings::PEAK_THREAT];

    /// Every field an effect governs, read off a tube.
    fn fields(tube: CrtSettings, effect: Effect) -> Vec<f32> {
        match effect {
            Effect::Curve => vec![tube.barrel],
            Effect::Lines => vec![tube.scanline],
            Effect::Glow => vec![tube.glow],
            Effect::Flicker => vec![tube.flicker],
            Effect::Mask => vec![tube.mask],
            Effect::Split => vec![tube.aberration],
            Effect::Edge => vec![tube.vignette, tube.vignette_radius, tube.desaturation],
        }
    }

    #[test]
    fn off_is_each_effects_own_off_at_every_preset() {
        // Including peak: a ceiling the threat could push through is not one.
        for preset in PRESETS {
            for effect in Effect::ALL {
                let mut picture = Picture::FULL;
                picture.set(effect, Strength::Off);
                assert_eq!(
                    fields(effective(preset, picture, false), effect),
                    fields(CrtSettings::OFF, effect),
                    "`{} off` left some of it on",
                    effect.word(),
                );
            }
        }
    }

    #[test]
    fn a_ceiling_never_adds_to_the_tube() {
        // Every strength sits between the effect's off value and the preset —
        // which for the vignette's radius is *above* the preset.
        for preset in PRESETS {
            for effect in Effect::ALL {
                for strength in Strength::ALL {
                    let mut picture = Picture::FULL;
                    picture.set(effect, strength);
                    let drawn = fields(effective(preset, picture, false), effect);
                    let ends = fields(preset, effect)
                        .into_iter()
                        .zip(fields(CrtSettings::OFF, effect));
                    for (value, (full, rest)) in drawn.into_iter().zip(ends) {
                        let (low, high) = (full.min(rest), full.max(rest));
                        assert!(
                            (low..=high).contains(&value),
                            "`{} {}` drew {value}, outside {low}..={high}",
                            effect.word(),
                            strength.word(),
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn full_is_the_tube_as_it_was() {
        for preset in PRESETS {
            assert_eq!(effective(preset, Picture::FULL, false), preset);
        }
    }

    #[test]
    fn stillness_takes_the_hum_band_and_leaves_the_shape() {
        for preset in PRESETS {
            let drawn = effective(preset, Picture::FULL, true);
            assert_eq!(
                drawn.flicker, 0.0,
                "the band still rolls for a still player"
            );
            assert_eq!(drawn.barrel, preset.barrel, "stillness flattened the glass");
            assert!(drawn.on, "stillness turned the tube off");
        }
    }
}
