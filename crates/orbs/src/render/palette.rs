//! Phosphor themes — where a [`Style`] finally becomes a colour.
//!
//! This module is the whole of architectural rule 2 on the Bevy side.
//! `orbs-render` decides *what* appears and never emits a hue; everything below
//! decides *how* it is drawn, and nothing here can add information the Frame did
//! not carry.
//!
//! DESIGN.md §4:
//!
//! - **Curated themes, not a hue slider.** Each is a hand-tuned harmony.
//! - **Muted violet by default** — distinctive, and reads arcane rather than
//!   computer.
//! - **The base hue carries all ordinary text through intensity alone.**
//! - **A small accent set is reserved strictly for meaning**, and every theme
//!   defines its own triad so contrast holds against its own base.
//!
//! [`Presentation`](orbs_render::Presentation) deliberately has no entry here:
//! it selects a *face* in the glyph atlas, not a colour. §14 forbids colour as
//! the sole carrier of meaning, and a tonal register that existed only as a hue
//! would be exactly that.

use bevy::prelude::*;
use orbs_render::{Intensity, Role, Style};

/// A hand-tuned harmony: one base hue at three weights, plus the accent triad.
///
/// "Hand-tuned" turned out to mean *solved*: the first pass was picked by eye
/// and failed its own accessibility tests — muted violet's cost and success
/// accents were 1.19:1 apart, which is the same colour in greyscale. The values
/// below satisfy every constraint the tests below assert.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Phosphor {
    /// What the theme is called in the settings screen.
    pub(crate) name: &'static str,
    /// The base hue at [`Intensity::Dim`], [`Intensity::Normal`],
    /// [`Intensity::Bright`].
    base: [Srgba; 3],
    /// Failure, breach, hostile action.
    danger: Srgba,
    /// Mana and arcane expenditure.
    cost: Srgba,
    /// Completion.
    success: Srgba,
    /// Behind everything. Never fully black — a dead screen and an idle one
    /// should not look the same (see `main.rs`).
    pub(crate) background: Srgba,
}

const fn rgb(red: f32, green: f32, blue: f32) -> Srgba {
    Srgba::new(red, green, blue, 1.0)
}

/// **The default.** Distinctive, and reads arcane rather than computer.
pub(crate) const MUTED_VIOLET: Phosphor = Phosphor {
    name: "muted violet",
    base: [
        rgb(0.41, 0.34, 0.56),
        rgb(0.67, 0.59, 0.84),
        rgb(0.92, 0.88, 1.00),
    ],
    danger: rgb(0.88, 0.22, 0.26),
    cost: rgb(0.48, 0.79, 1.00),
    success: rgb(0.56, 1.00, 0.55),
    background: rgb(0.055, 0.035, 0.075),
};

/// Warm, classic. §4's second default.
pub(crate) const AMBER: Phosphor = Phosphor {
    name: "amber",
    base: [
        rgb(0.55, 0.44, 0.32),
        rgb(0.86, 0.72, 0.55),
        rgb(1.00, 0.91, 0.80),
    ],
    danger: rgb(0.85, 0.29, 0.26),
    cost: rgb(0.19, 0.72, 0.95),
    success: rgb(0.60, 1.00, 0.50),
    background: rgb(0.070, 0.045, 0.020),
};

/// The canonical terminal.
pub(crate) const GREEN: Phosphor = Phosphor {
    name: "green",
    base: [
        rgb(0.32, 0.55, 0.33),
        rgb(0.55, 0.86, 0.56),
        rgb(0.80, 1.00, 0.81),
    ],
    danger: rgb(0.95, 0.09, 0.15),
    cost: rgb(0.45, 0.68, 0.90),
    success: rgb(1.00, 0.92, 0.50),
    background: rgb(0.020, 0.055, 0.030),
};

/// Every theme, in the order the settings screen offers them.
pub(crate) const ALL: [Phosphor; 3] = [MUTED_VIOLET, AMBER, GREEN];

impl Phosphor {
    /// The colour a cell of this style is drawn in.
    pub(crate) fn resolve(&self, style: Style) -> Color {
        let srgba = match style.role {
            // Ordinary text varies on intensity alone (§4).
            Role::Normal => self.base[weight(style.intensity)],
            // An accent is already a signal; intensity must not dilute it into
            // something a player has to compare against a neighbour to read.
            Role::Danger => self.danger,
            Role::Cost => self.cost,
            Role::Success => self.success,
        };
        srgba.into()
    }
}

const fn weight(intensity: Intensity) -> usize {
    match intensity {
        Intensity::Dim => 0,
        Intensity::Normal => 1,
        Intensity::Bright => 2,
    }
}

#[cfg(test)]
/// Relative luminance, per WCAG.
///
/// Used to check that meaning survives without hue — the cheap, honest proxy for
/// §14's colourblind-safety requirement. Two accents that differ only in hue are
/// the same colour to a substantial minority of players.
fn luminance(colour: Srgba) -> f32 {
    fn channel(value: f32) -> f32 {
        if value <= 0.039_28 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    }
    0.2126 * channel(colour.red) + 0.7152 * channel(colour.green) + 0.0722 * channel(colour.blue)
}

#[cfg(test)]
/// WCAG contrast ratio, `1.0..=21.0`.
fn contrast(a: Srgba, b: Srgba) -> f32 {
    let (high, low) = {
        let (x, y) = (luminance(a), luminance(b));
        if x > y { (x, y) } else { (y, x) }
    };
    (high + 0.05) / (low + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Body text against its own background, at the weight most of the game is
    /// drawn in. 4.5:1 is WCAG AA for normal text.
    #[test]
    fn body_text_is_legible_against_its_background() {
        for theme in ALL {
            let ratio = contrast(theme.base[weight(Intensity::Normal)], theme.background);
            assert!(
                ratio >= 4.5,
                "{}: body text contrasts {ratio:.1}:1 against its background",
                theme.name
            );
        }
    }

    /// Dim is for chrome the eye should skip, but §4 still draws pane borders
    /// with it — invisible borders are not a design, they are a bug.
    #[test]
    fn even_dim_text_stays_visible() {
        for theme in ALL {
            let ratio = contrast(theme.base[weight(Intensity::Dim)], theme.background);
            assert!(
                ratio >= 3.0,
                "{}: dim text contrasts only {ratio:.1}:1",
                theme.name
            );
        }
    }

    #[test]
    fn intensity_is_monotonic() {
        // The whole point of the base ramp: dim recedes, bright advances. A
        // theme where bright is darker than normal would invert every emphasis.
        for theme in ALL {
            let [dim, normal, bright] = theme.base;
            assert!(
                luminance(dim) < luminance(normal) && luminance(normal) < luminance(bright),
                "{}: the base ramp is not monotonic",
                theme.name
            );
        }
    }

    /// §14: no meaning conveyed by colour alone.
    ///
    /// The accents are the one place colour *is* meaning, so they have to be
    /// separable without hue. Requiring a luminance gap is what makes them
    /// survive greyscale, and therefore most colour vision deficiencies.
    #[test]
    fn the_accent_triad_is_separable_without_hue() {
        for theme in ALL {
            let accents = [
                ("danger", theme.danger),
                ("cost", theme.cost),
                ("success", theme.success),
            ];
            for (a_name, a) in accents {
                for (b_name, b) in accents {
                    if a_name >= b_name {
                        continue;
                    }
                    let ratio = contrast(a, b);
                    assert!(
                        ratio >= 1.25,
                        "{}: {a_name} and {b_name} differ by {ratio:.2}:1 — \
                         indistinguishable in greyscale",
                        theme.name
                    );
                }
            }
        }
    }

    #[test]
    fn every_accent_is_legible_against_its_own_background() {
        for theme in ALL {
            for (name, accent) in [
                ("danger", theme.danger),
                ("cost", theme.cost),
                ("success", theme.success),
            ] {
                let ratio = contrast(accent, theme.background);
                assert!(
                    ratio >= 4.5,
                    "{}: {name} contrasts {ratio:.1}:1 against its background",
                    theme.name
                );
            }
        }
    }

    /// An accent that reads as ordinary text is not an accent.
    #[test]
    fn accents_are_distinguishable_from_body_text() {
        for theme in ALL {
            let body = theme.base[weight(Intensity::Normal)];
            for (name, accent) in [
                ("danger", theme.danger),
                ("cost", theme.cost),
                ("success", theme.success),
            ] {
                assert!(
                    contrast(accent, body) >= 1.2,
                    "{}: {name} is too close to body text",
                    theme.name
                );
            }
        }
    }

    #[test]
    fn presentation_never_changes_the_colour() {
        // Presentation picks a face in the atlas. If it also moved the hue, a
        // tonal register would exist as colour alone, which §14 forbids.
        use orbs_render::Presentation;
        for presentation in [
            Presentation::Plain,
            Presentation::Eldritch,
            Presentation::Tampered,
        ] {
            let style = Style::DANGER.with_presentation(presentation);
            assert_eq!(
                MUTED_VIOLET.resolve(style),
                MUTED_VIOLET.resolve(Style::DANGER)
            );
        }
    }

    #[test]
    fn the_background_is_never_true_black() {
        // An empty screen that is exactly #000000 is indistinguishable from a
        // crashed one.
        for theme in ALL {
            assert!(luminance(theme.background) > 0.0, "{}", theme.name);
        }
    }
}
