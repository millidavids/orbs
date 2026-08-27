//! Colour vision deficiency, simulated — **an instrument, not an effect.**
//!
//! Nothing here draws. This module exists so the palette's accessibility claim
//! can be *measured* rather than asserted: push each accent through a
//! deficiency and check the triad is still separable on the other side.
//!
//! # Why this is not a correction filter
//!
//! ROADMAP Phase 11 and DESIGN.md §19 specify three **daltonisation** filters —
//! simulate the deficiency, take the error, redistribute it into the channels
//! the player can still see. That was implemented numerically against this
//! palette before any of it was written, and it makes the game **worse**:
//!
//! | channel | unfiltered | daltonised |
//! |---|---|---|
//! | accent triad (luminance) | 1.05–2.10 | 1.00–1.39 |
//! | material tints (hue) | 6–16 | 4–29 |
//! | spell syntax (hue) | 43–53 | 37–44 |
//!
//! Worse in eleven of twelve theme × deficiency combinations, worst case violet
//! under protanopia going 1.41 → **1.00** — danger and cost at identical
//! brightness.
//!
//! The reason is structural rather than a tuning failure. This game does not
//! carry meaning in hue: [`the accent
//! triad`](super::palette::Phosphor) is *solved* so danger, cost and success
//! differ in **luminance**, and every screen duplicates colour with a glyph or a
//! word (§14). Daltonisation redistributes hue, which moves luminance around —
//! so it fights the one property the design actually relies on, in exchange for
//! improving a channel §19 already permits to collapse.
//!
//! So the correction is not shipped, and the simulation is kept and pointed at
//! the tests. §19 records the measurement.
//!
//! # The matrices
//!
//! Machado, Oliveira and Fernandes (2009), severity 1.0, as used by
//! `court_wizard`. They operate on **linear** RGB.

use bevy::color::Srgba;

/// A deficiency to look at the palette through.
///
/// Only the three dichromacies. Anomalous trichromacy is a severity below 1.0 of
/// the same matrices, and testing the severe end is what bounds the mild one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Deficiency {
    /// Red-blind.
    Protanopia,
    /// Green-blind. The common one, and the one this palette had to be re-solved for.
    Deuteranopia,
    /// Blue-blind.
    Tritanopia,
}

impl Deficiency {
    /// All three, for a test that must cover every one.
    pub(crate) const ALL: [Self; 3] = [Self::Protanopia, Self::Deuteranopia, Self::Tritanopia];

    /// The simulation matrix, row-major.
    const fn matrix(self) -> [f32; 9] {
        match self {
            Self::Protanopia => [
                0.152_286, 1.052_583, -0.204_868, //
                0.114_503, 0.786_281, 0.099_216, //
                -0.003_882, -0.048_116, 1.051_998,
            ],
            Self::Deuteranopia => [
                0.367_322, 0.860_646, -0.227_968, //
                0.280_085, 0.672_501, 0.047_413, //
                -0.011_820, 0.042_940, 0.968_881,
            ],
            Self::Tritanopia => [
                1.255_528, -0.076_749, -0.178_779, //
                -0.078_411, 0.930_809, 0.147_602, //
                0.004_733, 0.691_367, 0.303_900,
            ],
        }
    }

    /// What this deficiency is called, for a failure message.
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Protanopia => "protanopia",
            Self::Deuteranopia => "deuteranopia",
            Self::Tritanopia => "tritanopia",
        }
    }
}

/// What `colour` looks like to someone with `deficiency`.
///
/// **Written as three dot products rather than a `Mat3`**, deliberately.
/// `court_wizard`'s shader builds `mat3x3(row0, row1, row2)` — and WGSL's
/// constructor takes **columns**, so it is applying the transpose of every one
/// of these. Machado's matrices are not symmetric, so that is not a subtle
/// difference. Spelling the rows out is what makes the convention unmistakable
/// if this is ever transliterated to a shader.
pub(crate) fn simulated(deficiency: Deficiency, colour: Srgba) -> Srgba {
    let m = deficiency.matrix();
    let (r, g, b) = (
        linear(colour.red),
        linear(colour.green),
        linear(colour.blue),
    );
    Srgba::new(
        encoded(m[0].mul_add(r, m[1].mul_add(g, m[2] * b))),
        encoded(m[3].mul_add(r, m[4].mul_add(g, m[5] * b))),
        encoded(m[6].mul_add(r, m[7].mul_add(g, m[8] * b))),
        colour.alpha,
    )
}

/// sRGB → linear, matching [`palette::luminance`](super::palette::luminance)'s
/// own channel curve. Two transfer functions in one crate is how they stop
/// agreeing.
fn linear(value: f32) -> f32 {
    if value <= 0.039_28 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

/// linear → sRGB, and clamped: a simulation can leave the cube.
fn encoded(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    if value <= 0.003_130_8 {
        12.92 * value
    } else {
        1.055f32.mul_add(value.powf(1.0 / 2.4), -0.055)
    }
}
