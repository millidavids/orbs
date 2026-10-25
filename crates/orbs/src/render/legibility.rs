//! `contrast high` — the palette lifted, whatever the theme.
//!
//! A filter rather than a theme, for the reason the colour-vision box gives: an
//! accommodation should not cost a player the phosphor they like. Applied where
//! a cell becomes a colour, so it reaches nothing the Frame did not carry.
//!
//! The measures are WCAG's. They were test-only until this needed them at run
//! time, and one copy is what keeps the lift and the tests agreeing.

use bevy::prelude::*;
use orbs_shell::settings::Pair;

use super::palette::Phosphor;

/// What `contrast high` holds ordinary text and spell text to, against the
/// background. WCAG AAA for body text.
pub(crate) const HIGH: f32 = 7.0;

/// The least two adjacent weights may differ by, once lifted, so §4's ramp —
/// dim recedes, bright advances — still reads.
pub(crate) const LADDER: f32 = 1.25;

crate::shell::pair! {
    /// Whether the player asked for more contrast — `settings` → `access`.
    ///
    /// Anything but `high` is normal: the palette as solved is the fallback
    /// that is always legible.
    pub(crate) struct Contrast(bool) = "contrast", ["normal", "high"];
}

/// Relative luminance, per WCAG.
pub(crate) fn luminance(colour: Srgba) -> f32 {
    fn channel(value: f32) -> f32 {
        if value <= 0.039_28 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    }
    0.2126 * channel(colour.red) + 0.7152 * channel(colour.green) + 0.0722 * channel(colour.blue)
}

/// WCAG contrast ratio, `1.0..=21.0`.
pub(crate) fn contrast(a: Srgba, b: Srgba) -> f32 {
    let (high, low) = {
        let (x, y) = (luminance(a), luminance(b));
        if x > y { (x, y) } else { (y, x) }
    };
    (high + 0.05) / (low + 0.05)
}

/// Move `colour` toward `toward` until `enough` holds, as little as it takes.
///
/// Toward the theme's own bright weight for the ramp, so a lifted amber is
/// still amber; toward white only where there is nowhere brighter.
fn lift(colour: Srgba, toward: Srgba, enough: impl Fn(Srgba) -> bool) -> Srgba {
    if enough(colour) {
        return colour;
    }
    if !enough(toward) {
        return toward;
    }
    let (mut low, mut high) = (0.0_f32, 1.0_f32);
    // Twenty halvings is finer than an 8-bit channel can show.
    for _ in 0..20 {
        let middle = f32::midpoint(low, high);
        if enough(colour.mix(&toward, middle)) {
            high = middle;
        } else {
            low = middle;
        }
    }
    colour.mix(&toward, high)
}

/// The last theme lifted, and what it lifted to.
///
/// The lift is a search, so it is solved once per theme rather than per frame.
#[derive(Debug, Default)]
pub(crate) struct Lifted(Option<(Phosphor, Phosphor)>);

impl Lifted {
    /// `theme` as drawn under `contrast`.
    pub(crate) fn drawn(&mut self, theme: Phosphor, contrast: Contrast) -> Phosphor {
        if !contrast.raised() {
            return theme;
        }
        match self.0 {
            Some((from, to)) if from == theme => to,
            _ => {
                let to = theme.drawn(contrast);
                self.0 = Some((theme, to));
                to
            }
        }
    }
}

/// A base ramp and a spell palette, lifted against `background`.
///
/// Dim first, then normal above it, then bright above that, so the ramp keeps
/// its order and its steps. The accents are left alone: lifting danger to 7:1
/// on amber puts it 1.23:1 from cost, under the triad's floor, and the triad is
/// what carries meaning (§19).
pub(crate) fn raise<const SYNTAX: usize>(
    base: [Srgba; 3],
    syntax: Option<[Srgba; SYNTAX]>,
    background: Srgba,
) -> ([Srgba; 3], Option<[Srgba; SYNTAX]>) {
    let white = Srgba::WHITE;
    let [dim, normal, bright] = base;
    let dim = lift(dim, bright, |colour| contrast(colour, background) >= HIGH);
    let normal = lift(normal, bright, |colour| {
        contrast(colour, background) >= HIGH
            && luminance(colour) > luminance(dim)
            && contrast(colour, dim) >= LADDER
    });
    let bright = lift(bright, white, |colour| {
        luminance(colour) > luminance(normal) && contrast(colour, normal) >= LADDER
    });
    let syntax = syntax.map(|palette| {
        palette.map(|hue| lift(hue, white, |colour| contrast(colour, background) >= HIGH))
    });
    ([dim, normal, bright], syntax)
}

#[cfg(test)]
mod tests {
    use orbs_render::{Intensity, Lexeme, Role, Style};

    use super::super::palette::EVERY;
    use super::*;

    const WEIGHTS: [Intensity; 3] = [Intensity::Dim, Intensity::Normal, Intensity::Bright];

    #[test]
    fn high_contrast_holds_every_theme_to_seven_to_one() {
        for theme in EVERY {
            let raised = theme.raised();
            let ramp = WEIGHTS.map(|weight| raised.base_at(weight));
            for (weight, colour) in WEIGHTS.iter().zip(ramp) {
                let ratio = contrast(colour, theme.background);
                assert!(
                    ratio >= HIGH,
                    "{}: {weight:?} is {ratio:.2}:1 under contrast high",
                    theme.name,
                );
            }
            // §4: dim recedes and bright advances, by a step a reader can see.
            for pair in ramp.windows(2) {
                assert!(
                    luminance(pair[0]) < luminance(pair[1]),
                    "{}: the ramp inverted",
                    theme.name
                );
                let step = contrast(pair[0], pair[1]);
                assert!(
                    step >= LADDER - 0.001,
                    "{}: two weights are {step:.2}:1 apart under contrast high",
                    theme.name,
                );
            }
            // Spell text is text.
            for lexeme in [
                Lexeme::Control,
                Lexeme::Verb,
                Lexeme::Number,
                Lexeme::Grammar,
                Lexeme::State,
            ] {
                let colour = Srgba::from(raised.resolve_tinted(Style::NORMAL, None, Some(lexeme)));
                let ratio = contrast(colour, theme.background);
                assert!(ratio >= HIGH, "{}: {lexeme:?} is {ratio:.2}:1", theme.name);
            }
            // The triad is solved for meaning and is left as it is.
            for role in [Role::Danger, Role::Cost, Role::Success] {
                let accent = Style::NORMAL.with_role(role);
                assert_eq!(
                    raised.resolve(accent),
                    theme.resolve(accent),
                    "{}: {role:?} moved",
                    theme.name
                );
            }
        }
    }

    #[test]
    fn normal_contrast_is_the_palette_as_solved() {
        for theme in EVERY {
            assert_eq!(theme.drawn(Contrast::named("normal")), theme);
        }
    }

    #[test]
    fn a_word_this_build_does_not_know_is_normal() {
        assert!(!Contrast::named("extreme").raised());
        assert!(Contrast::named("high").raised());
    }

    #[test]
    fn a_lift_moves_only_as_far_as_it_must() {
        let dark = Srgba::new(0.05, 0.05, 0.05, 1.0);
        let grey = Srgba::new(0.3, 0.3, 0.3, 1.0);
        let lifted = lift(grey, Srgba::WHITE, |colour| contrast(colour, dark) >= HIGH);
        assert!(contrast(lifted, dark) >= HIGH);
        assert!(
            contrast(lifted, dark) < HIGH + 0.05,
            "it overshot to {:.2}:1",
            contrast(lifted, dark),
        );
    }
}
