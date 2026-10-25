//! What fire looks like.
//!
//! The colours a [`Depiction`] resolves to, beside [`palette`](super::palette).
//!
//! Fire is orange on every tube, since a green fire reads as a changed meter;
//! it costs monochrome its no-hue promise and no information (§19). Smoke is
//! two cool steps near the background, read by being dim.

use bevy::prelude::*;
use orbs_render::{Depiction, Heat};

/// The colours fire burns in. One table, shared by every theme.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Ember {
    /// [`Heat::Ember`], [`Heat::Flame`], [`Heat::Blaze`], [`Heat::Core`].
    ///
    /// Read hottest-first at the *base* of the bar, not at the flame front: a
    /// fire is brightest where the fuel is and mellows toward its tip.
    pub(crate) flame: [Srgba; 4],
    /// [`Depiction::SmokeThin`], [`Depiction::SmokeThick`].
    pub(crate) smoke: [Srgba; 2],
}

use super::palette::rgb;

/// Where a [`Heat`] sits on the ramp.
const fn step(heat: Heat) -> usize {
    match heat {
        Heat::Ember => 0,
        Heat::Flame => 1,
        Heat::Blaze => 2,
        Heat::Core => 3,
    }
}

/// Fire: the literal orange into yellow a hearth is expected to be.
///
/// Amber's ramp, now everyone's. It kept its own numbers rather than being
/// re-tuned toward a compromise — there is nothing left to compromise with, and
/// it clears the floor against all four backgrounds with room to spare (5.5:1
/// at the coolest ember against the tightest background, on a floor of 3.0).
pub(crate) const FIRE: Ember = Ember {
    flame: [
        rgb(0.80, 0.44, 0.14),
        rgb(0.92, 0.58, 0.16),
        rgb(1.00, 0.72, 0.22),
        rgb(1.00, 0.89, 0.46),
    ],
    smoke: [rgb(0.30, 0.27, 0.26), rgb(0.45, 0.41, 0.39)],
};

/// The balneum mariae's liquid, at [`Depiction::LiquidStill`],
/// [`Depiction::LiquidStirred`], [`Depiction::LiquidRolling`].
///
/// Teal, the fire's opposite, since the two sit side by side. 4.7:1 at the
/// stillest step on every tube.
pub(crate) const LIQUID: [Srgba; 3] = [
    rgb(0.28, 0.52, 0.58),
    rgb(0.40, 0.70, 0.76),
    rgb(0.60, 0.88, 0.92),
];

/// What a run leaves settled in the bottom of a vessel.
///
/// Dimmer than the stillest liquid, deliberately: settled waste should recede,
/// and a sediment that read as bright as the tincture would make a jammed
/// instrument look like a working one. Held to the smoke's 1.5 floor rather than
/// the liquid's 3.0, for the same reason — it is chrome, not the reading.
pub(crate) const SEDIMENT: Srgba = rgb(0.36, 0.32, 0.26);

/// The colour of a depicted cell, or `None` if it is not a picture of anything.
///
/// A free function, not a method, since the fire stopped being per-theme: there
/// is one of each table now, so a `&self` was a receiver with one value.
pub(crate) const fn resolve(depiction: Depiction) -> Option<Srgba> {
    match depiction {
        Depiction::Sediment => Some(SEDIMENT),
        Depiction::LiquidStill => Some(LIQUID[0]),
        Depiction::LiquidStirred => Some(LIQUID[1]),
        Depiction::LiquidRolling => Some(LIQUID[2]),
        Depiction::GaugeFaint => Some(GAUGE[0]),
        Depiction::GaugeLow => Some(GAUGE[1]),
        Depiction::GaugeMiddle => Some(GAUGE[2]),
        Depiction::GaugeHigh => Some(GAUGE[3]),
        Depiction::GaugeNear => Some(GAUGE[4]),
        Depiction::GaugeWhole => Some(GAUGE[5]),
        other => FIRE.resolve(other),
    }
}

/// A gauge's fill, red through yellow to green — see [`Fill`](orbs_render::Fill).
///
/// Peaks in brightness at yellow, allowed since the length carries the reading
/// (§14). Even sixths, off full saturation, and a dim red end so it reads as
/// early, not `Role::Danger` — held by `no_step_of_the_gauge_reads_as_an_accent`
/// and `every_step_of_the_gauge_is_visible_on_every_tube`.
const GAUGE: [Srgba; 6] = [
    rgb(0.64, 0.25, 0.05),
    rgb(0.78, 0.42, 0.10),
    rgb(0.86, 0.56, 0.12),
    rgb(0.88, 0.80, 0.16),
    rgb(0.60, 0.80, 0.20),
    rgb(0.34, 0.78, 0.34),
];

impl Ember {
    /// The colour of a depicted cell, or `None` if it is not one of the fire's.
    pub(crate) const fn resolve(&self, depiction: Depiction) -> Option<Srgba> {
        match depiction {
            Depiction::None => None,
            Depiction::SmokeThin => Some(self.smoke[0]),
            Depiction::SmokeThick => Some(self.smoke[1]),
            // Flame and spark both: a spark is a piece of the fire that got
            // away, so it draws from the fire's own ramp. `heat()` answers
            // `None` for everything else — smoke above, and the bath's liquid
            // and sediment, which the free `resolve` handles before this.
            other => match other.heat() {
                Some(heat) => Some(self.flame[step(heat)]),
                None => None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::palette::{EVERY, contrast, luminance};
    use super::*;
    use orbs_render::{Role, Style};

    #[test]
    fn the_flame_ramp_climbs() {
        // The whole point of a heat ramp: hotter is brighter. A ramp that sagged
        // in the middle would make the fire's shimmer read as noise, and one
        // that inverted would put the coolest embers at the flame front.
        for pair in FIRE.flame.windows(2) {
            assert!(
                luminance(pair[0]) < luminance(pair[1]),
                "the flame ramp is not monotonic",
            );
        }
    }

    #[test]
    fn smoke_is_dimmer_than_the_fire_that_made_it() {
        let coolest = luminance(FIRE.flame[0]);
        for (index, step) in FIRE.smoke.iter().enumerate() {
            assert!(
                luminance(*step) < coolest,
                "smoke step {index} is brighter than the embers",
            );
        }
        assert!(
            luminance(FIRE.smoke[0]) < luminance(FIRE.smoke[1]),
            "thick smoke is not denser than thin",
        );
    }

    #[test]
    fn the_one_fire_is_visible_on_every_tube() {
        // One ramp against all four backgrounds, which is what makes it safe.
        // Smoke is chrome, held to 3.0, not body text's 4.5.
        for theme in EVERY {
            for (index, step) in FIRE.flame.iter().enumerate() {
                let ratio = contrast(*step, theme.background);
                assert!(
                    ratio >= 3.0,
                    "{}: flame step {index} contrasts {ratio:.1}:1",
                    theme.name,
                );
            }
            for (index, step) in FIRE.smoke.iter().enumerate() {
                let ratio = contrast(*step, theme.background);
                assert!(
                    ratio >= 1.5,
                    "{}: smoke step {index} contrasts {ratio:.1}:1 — invisible",
                    theme.name,
                );
            }
        }
    }

    #[test]
    fn the_hottest_flame_reads_against_body_text_on_every_tube() {
        // Against `Normal`, not `Bright`: monochrome's `Bright` is pure white
        // and nothing is hotter, so a rule phrased against it is unsatisfiable
        // for the one theme the game most needs to keep.
        for theme in EVERY {
            let body = theme.base_at(orbs_render::Intensity::Normal);
            let ratio = contrast(FIRE.flame[3], body);
            assert!(
                ratio >= 1.2,
                "{}: the flame front is indistinguishable from body text",
                theme.name,
            );
        }
    }

    #[test]
    fn monochromes_own_text_is_still_hueless() {
        // Narrower than it was (§19): monochrome's text is grey.
        let monochrome = EVERY
            .iter()
            .find(|theme| theme.name == "monochrome")
            .expect("the grey theme");
        let hueless = |colour: Srgba| {
            let spread = [colour.red, colour.green, colour.blue];
            let low = spread.iter().copied().fold(f32::MAX, f32::min);
            let high = spread.iter().copied().fold(f32::MIN, f32::max);
            high - low <= 0.05
        };
        for weight in [
            orbs_render::Intensity::Dim,
            orbs_render::Intensity::Normal,
            orbs_render::Intensity::Bright,
        ] {
            assert!(
                hueless(monochrome.base_at(weight)),
                "monochrome's base carries hue at {weight:?}",
            );
        }
        assert!(hueless(monochrome.background), "its background does");

        // ...and the fire is deliberately not. Asserted rather than merely
        // allowed, so "fire is orange" stays a decision someone made rather
        // than something that drifts back to white unnoticed.
        assert!(
            !hueless(FIRE.flame[0]),
            "the fire went hueless again — see this module's header",
        );
    }

    #[test]
    fn the_liquid_reads_on_every_tube_and_climbs() {
        // The same bar the fire is held to, for the same reason: one ramp for
        // every theme means it has to clear the floor against four backgrounds
        // rather than sit inside one family.
        for pair in LIQUID.windows(2) {
            assert!(
                luminance(pair[0]) < luminance(pair[1]),
                "the roil ramp is not monotonic — a bath would shimmer as noise",
            );
        }
        for theme in EVERY {
            for (index, step) in LIQUID.iter().enumerate() {
                let ratio = contrast(*step, theme.background);
                assert!(
                    ratio >= 3.0,
                    "{}: liquid step {index} contrasts {ratio:.1}:1",
                    theme.name,
                );
            }
            let ratio = contrast(SEDIMENT, theme.background);
            assert!(
                ratio >= 1.5,
                "{}: sediment contrasts {ratio:.1}:1 — invisible",
                theme.name,
            );
        }
    }

    #[test]
    fn the_bath_never_reads_as_the_hearth() {
        // The two heated instruments sit next to each other and both draw solid
        // blocks, so the glyph cannot say which is which and the hue has to.
        //
        // On the blue-yellow axis rather than luminance, which is exactly what
        // they are *allowed* to share: a bright roll and a mid flame land
        // within 1.2:1 of each other and should.
        let axis = |colour: Srgba| colour.blue - colour.red;
        for (index, liquid) in LIQUID.iter().enumerate() {
            for (other, flame) in FIRE.flame.iter().enumerate() {
                assert!(
                    axis(*liquid) - axis(*flame) > 0.5,
                    "liquid {index} and flame {other} are too close in hue",
                );
            }
        }
    }

    #[test]
    fn sediment_recedes_rather_than_reading_as_a_reading() {
        // A jammed instrument must not look like a working one. The bath's
        // sediment is the counterpart to the mortar's husks: both are what a run
        // left behind, and both have to be visibly *less* than the thing the
        // instrument holds when it is doing its job.
        assert!(
            luminance(SEDIMENT) < luminance(LIQUID[0]),
            "sediment is brighter than the stillest liquid",
        );
    }

    /// The claim the other ramps each make, and the one this ramp did not. A
    /// step that sank into the background would draw a bar that stops part way
    /// along its own fill — read as a *shorter* bar, not an unstyled one.
    ///
    /// Held to chrome's 3.0 rather than body text's 4.5: a gauge is furniture,
    /// like smoke, and the reading it carries is its length.
    #[test]
    fn every_step_of_the_gauge_is_visible_on_every_tube() {
        for theme in EVERY {
            for (index, step) in GAUGE.iter().enumerate() {
                let ratio = contrast(*step, theme.background);
                assert!(
                    ratio >= 3.0,
                    "{}: gauge step {index} contrasts {ratio:.1}:1",
                    theme.name,
                );
            }
        }
    }

    /// No step reads as an accent, which is why this ramp is a depiction.
    ///
    /// Measured through `resolve`, the frontend's path, where accent and
    /// depiction are different arms.
    #[test]
    fn no_step_of_the_gauge_reads_as_an_accent() {
        let gauges: Vec<_> = Depiction::ALL
            .into_iter()
            .filter(|depiction| depiction.is_gauge())
            .collect();
        assert_eq!(gauges.len(), 6, "the ramp is not six steps");

        for theme in EVERY {
            for role in [Role::Danger, Role::Cost, Role::Success] {
                let accent = Srgba::from(theme.resolve(Style::default().with_role(role)));
                for depiction in &gauges {
                    let fill =
                        Srgba::from(theme.resolve(Style::default().with_depiction(*depiction)));
                    // Distance in RGB rather than luminance: an accent and a
                    // gauge step are *allowed* to share a brightness, and on a
                    // red-to-green ramp several do. What they may not share is
                    // the colour itself.
                    let apart = (fill.red - accent.red).abs()
                        + (fill.green - accent.green).abs()
                        + (fill.blue - accent.blue).abs();
                    // 0.25 is the floor; the ramp clears it by 0.36. A
                    // red-to-green ramp runs through *three* of the triad's own
                    // hues, so every step is near something and the separation
                    // has to be deliberate at each, not only at the ends.
                    assert!(
                        apart > 0.25,
                        "{}: {depiction:?} is {apart:.2} from {role:?} — a bar \
                         drawn in an accent's own colour",
                        theme.name,
                    );
                }
            }
        }
    }

    #[test]
    fn a_cell_that_is_a_picture_of_nothing_resolves_to_nothing() {
        // What keeps the channel inert for the whole rest of the screen.
        assert_eq!(resolve(Depiction::None), None);
        assert!(resolve(Depiction::flame(Heat::Blaze)).is_some());

        // Walked off `Depiction::ALL`, not a hand-written array: the bath's
        // four variants arrived and the old list went on passing while covering
        // none of them. `ALL` is exhaustiveness-checked at its definition.
        for depiction in Depiction::ALL {
            let resolved = resolve(depiction).is_some();
            assert_eq!(
                resolved,
                depiction != Depiction::None,
                "{depiction:?} resolved to {resolved:?}, which is not what it is",
            );
        }
    }
}
