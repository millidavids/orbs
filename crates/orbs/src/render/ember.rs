//! What fire looks like.
//!
//! The colours a [`Depiction`] resolves to. Split from [`palette`](super::palette)
//! because that file was already at CLAUDE.md's ~300-line limit and this is a
//! feature of its own rather than more of the same table — but it is the same
//! rule 2 boundary: `orbs-render` says *this cell is flame, and this hot*, and
//! nothing below decides anything but the hue.
//!
//! # One fire, on every tube
//!
//! **Fire is orange, and the phosphor does not get a vote.** This shipped as four
//! ramps — amber burned orange, green burned green, violet burned violet,
//! monochrome burned white — on the argument that §4's premise is a single curved
//! CRT whose base hue carries the picture, so an orange fire on the green
//! phosphor is a colour that tube cannot make.
//!
//! That argument was about the *tube*. The thing being drawn is a **fire**, and a
//! green fire does not read as one — it reads as the meter having changed colour.
//! The whole reason [`Depiction`] exists is that a picture is worth more than
//! consistency with the surrounding hue, and four ramps was that concession made
//! and then taken back. One ramp is also one thing to solve for rather than four,
//! which is what the contrast table below now checks *across* the themes instead
//! of within each.
//!
//! It costs monochrome its no-hue promise, and that is a real cost recorded in
//! DESIGN.md §19: the accessibility theme now has exactly one coloured object on
//! it. Nothing *informational* moves — [`Depiction`] carries no meaning by
//! construction, and the meter's value is read off the glyph boundary — so a
//! colourblind player still loses nothing they could have used.
//!
//! # Smoke is cool, and that is the whole of it
//!
//! Two steps, low and near the background. Smoke is what is left when the fire
//! is not there, so it reads by being dimmer than everything around it rather
//! than by being a colour.

use bevy::prelude::*;
use orbs_render::{Depiction, Heat};

/// The colours fire burns in. One table, shared by every theme.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Ember {
    /// [`Heat::Ember`], [`Heat::Flame`], [`Heat::Blaze`], [`Heat::Core`].
    ///
    /// **Read hottest-first at the *base* of the bar**, not at the flame front:
    /// a fire is brightest where the fuel is, and mellows toward its tip.
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
/// **This was amber's ramp**, and it is now everyone's. It kept the numbers it
/// was solved for rather than being re-tuned toward a compromise, because there
/// is nothing left to compromise with — it now has to clear the contrast floor
/// against four backgrounds rather than sit inside one theme's family, and it
/// does so with room to spare (5.5:1 at the coolest ember against the tightest
/// background, on a floor of 3.0).
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
/// **Teal, which is the fire's opposite**, and that is the whole selection
/// criterion beyond the contrast floor. The two heated instruments sit next to
/// each other on the panel, and the one thing a glance has to answer is which is
/// which — so the bath is as far from orange as the repertoire allows while
/// still reading as a liquid.
///
/// Solved against all four backgrounds like the fire: 4.7:1 at the stillest step
/// against the tightest background, on a floor of 3.0.
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
/// **A free function, not a method**, since the fire stopped being per-theme:
/// there is one of each table now, so threading a `&self` through was carrying a
/// receiver that only ever had one value.
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
/// **The only ramp here that is not monotonic in brightness.** Fire, liquid and
/// smoke all climb from dim to bright and their tests assert it; this one peaks
/// in the middle, where yellow is. That is what a red-to-green ramp *is*, and it
/// is allowed because the fill's **length** carries the reading — take every
/// colour away and the bar still says how full it is, which is the test §14 sets
/// for anything decorative.
///
/// Six steps, walked in even sixths, so the bar warms as it fills rather than
/// switching like a traffic light. The hues are pulled off full saturation and
/// lifted off the floor so each sits clear of the ground the tube paints, and so
/// the red end reads as *early* rather than as `Role::Danger` — an accent this
/// must never be mistaken for, which is also why a gauge is a depiction and
/// never an accent.
///
/// **The red end is dark, and that is the whole of what separates it from
/// `Role::Danger`.** Every theme's danger is a *bright* saturated red — 0.85 to
/// 0.95 on the red channel — so brightness is the axis with room in it, and a
/// bar barely begun reading dim is what it should look like anyway.
///
/// The first pass ran the low end at `0.78, 0.24, 0.20` and sat 0.18 away from
/// danger on three of the four tubes, which is to say it *was* the accent this
/// doc claims it is not. Nothing caught it, because the ramp shipped without the
/// test every other ramp in this file has. It now clears every accent on every
/// tube by 0.36 and every background by 3.06:1, both measured rather than
/// judged — `no_step_of_the_gauge_reads_as_an_accent` and
/// `every_step_of_the_gauge_is_visible_on_every_tube` hold them.
///
/// **The top half turns green by losing red, not by gaining green**, which is
/// why luminance peaks at `GaugeHigh` and falls after it. A red-to-green ramp
/// has no monotonic-brightness reading to preserve; the fill's length is the
/// reading.
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
            // Flame and spark both. **A spark is a piece of the fire that got
            // away**, so it draws from the fire's own ramp; a separate one would
            // be four more colours to solve for a difference nobody could name.
            //
            // `heat()` answers `None` for everything that is not the fire — the
            // two smoke arms above, and the bath's liquid and sediment, which
            // the free `resolve` handles before this is reached.
            other => match other.heat() {
                Some(heat) => Some(self.flame[step(heat)]),
                None => None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::palette::{ALL, contrast, luminance};
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
        // **The test that got harder when the ramps merged**, and the reason the
        // merge is safe. Four ramps each had to clear this against one
        // background; one ramp has to clear it against all four, which is a
        // stronger claim and the only thing standing between "fire is orange
        // everywhere" and a bar nobody can see on the green phosphor.
        //
        // Smoke is chrome rather than text, so it is held to
        // `even_dim_text_stays_visible`'s 3.0 rather than to body text's 4.5.
        for theme in ALL {
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
        // **Compared against `Normal`, deliberately, not against `Bright`.**
        // Monochrome's `Bright` is pure white and nothing is hotter than that,
        // so a rule phrased against it is unsatisfiable for the one theme the
        // game most needs to keep.
        for theme in ALL {
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
        // **What is left of monochrome's promise, and what is not.**
        //
        // The theme used to guarantee that a colourblind player lost nothing:
        // its base carried no hue, and it burned white so the fire carried none
        // either. Two decisions retired that — one fire ramp on every tube, and
        // material tints — and the guarantee **moved to a Phase 13 roadmap item**
        // (colour-vision filters and a true greyscale mode), which is a better
        // home for it: a theme made the accessible option also an aesthetic
        // choice, so a player who wanted amber had to give up the accommodation
        // to get it. DESIGN.md §19 records the trade.
        //
        // What still holds here is narrower and worth keeping: monochrome's
        // *text* is grey. It is a grey aesthetic, and a grey aesthetic with a
        // faintly warm body hue would just be a bad amber.
        let monochrome = ALL
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
        for theme in ALL {
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
        // **The two heated instruments sit next to each other on the panel**, and
        // the one thing a glance has to answer is which is which. Both draw solid
        // blocks, so the glyph cannot settle it and the hue has to.
        //
        // Measured on the blue-yellow axis rather than by luminance, because
        // luminance is exactly what they are *allowed* to share: a bright roll
        // and a mid flame land within 1.2:1 of each other and should.
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

    /// **Every step of the gauge is visible on every tube.**
    ///
    /// The claim the other ramps each make, and the one this ramp did not. A
    /// step that sank into the background would draw as a bar that stops part
    /// way along its own fill — worse than a bar with no colour at all, because
    /// it reads as a *shorter* bar rather than as an unstyled one.
    ///
    /// Held to chrome's 3.0 rather than body text's 4.5: a gauge is furniture,
    /// like smoke, and the reading it carries is its length.
    #[test]
    fn every_step_of_the_gauge_is_visible_on_every_tube() {
        for theme in ALL {
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

    /// **No step of the gauge reads as an accent**, which is this ramp's whole
    /// reason for being a depiction.
    ///
    /// The doc above says the red end must read as *early* rather than as
    /// `Role::Danger`, and until now nothing checked it. The terminal build
    /// made exactly that mistake with the same ramp — its low step was
    /// `Color::Red`, byte-for-byte Danger's ink, and its full step was
    /// Success's — so the property is worth asserting on the side that got it
    /// right, not only on the side that did not.
    ///
    /// **Measured through `resolve`**, the path the frontend actually takes,
    /// rather than against the constants: an accent and a depiction reach the
    /// screen by different arms of the same function and comparing the arms is
    /// the point.
    #[test]
    fn no_step_of_the_gauge_reads_as_an_accent() {
        let gauges: Vec<_> = Depiction::ALL
            .into_iter()
            .filter(|depiction| depiction.is_gauge())
            .collect();
        assert_eq!(gauges.len(), 6, "the ramp is not six steps");

        for theme in ALL {
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
                    // 0.25 is the floor; the ramp clears it by 0.36. The margin
                    // is not generous by accident — a red-to-green ramp runs
                    // through *three* of the triad's own hues, so every step is
                    // near something and the separation has to be deliberate at
                    // every one of them rather than only at the ends.
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

        // **Walked off `Depiction::ALL`, not a list written out here.** This was
        // a hand-written array, which is a list that falls silently behind the
        // enum the moment a variant is added — and it had: the bath's four
        // arrived and the walk went on passing while covering none of them.
        // `ALL` is exhaustiveness-checked at its definition.
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
