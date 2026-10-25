//! Phosphor themes — where a [`Style`] finally becomes a colour.
//!
//! Rule 2's Bevy side. Curated themes, amber by default; the base hue carries
//! text by intensity, and each theme's accent triad is tuned to its base (§4).
//! [`Presentation`](orbs_render::Presentation) selects a face, not a colour
//! (§14).

use bevy::prelude::*;
use orbs_render::{Depiction, Intensity, Lexeme, Role, Style, Wash};

use super::ember;

/// A hand-tuned harmony: one base hue at three weights, plus the accent triad.
///
/// "Hand-tuned" turned out to mean *solved*: the first pass was picked by eye
/// and failed its own accessibility tests — muted violet's cost and success
/// accents were 1.19:1 apart, the same colour in greyscale.
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
    /// The parts of speech a spell is drawn in, indexed by [`syntax_slot`].
    ///
    /// `None` for `monochrome`, somewhere to go for anyone who reads hue poorly
    /// (§14); `Lexeme::weight` carries the reading either way.
    syntax: Option<[Srgba; SYNTAX]>,
    /// Behind everything. Never fully black — a dead screen and an idle one
    /// should not look the same (see `main.rs`).
    pub(crate) background: Srgba,
}

/// How many parts of speech depart from the base hue.
///
/// Five, not eight. `Name` is most of a spell and keeps the phosphor, or the
/// departures would have nothing to depart from; `Comment` and `Filler` already
/// recede on the weight axis, which survives a theme with no palette at all.
pub(crate) const SYNTAX: usize = 5;

/// Where a lexeme sits in a theme's [`Phosphor::syntax`] table.
///
/// `None` for the three that keep the base hue. `Call` shares `Control`'s slot:
/// both are the file's own scaffolding, and what tells a block keyword from a
/// call to this file's own part is the `()`, not the colour.
const fn syntax_slot(kind: Lexeme) -> Option<usize> {
    match kind {
        Lexeme::Control | Lexeme::Call => Some(0),
        Lexeme::Verb => Some(1),
        Lexeme::Number => Some(2),
        Lexeme::Grammar => Some(3),
        Lexeme::State => Some(4),
        Lexeme::Name | Lexeme::Comment | Lexeme::Filler | Lexeme::None => None,
    }
}

/// The syntax palette the three coloured themes share.
///
/// One table, authored without a window; per-theme so a pass with eyes on the
/// tube can split it. Away from all three phosphors.
const SPELL: [Srgba; SYNTAX] = [
    // Control and call — arcane, the register §14 already reads as magical.
    rgb(0.78, 0.62, 1.00),
    // Verb — what the tower answers to.
    rgb(1.00, 0.78, 0.36),
    // Number.
    rgb(0.95, 0.62, 0.36),
    // Grammar — what a question turns on.
    rgb(0.40, 0.74, 0.76),
    // State — what the answer may be.
    rgb(0.46, 0.88, 0.94),
];

/// An opaque colour, written the way the tables below read best.
///
/// `pub(super)` so [`super::ember`] shares it — two copies of a three-token
/// helper is two places for the alpha channel to be spelled differently.
pub(super) const fn rgb(red: f32, green: f32, blue: f32) -> Srgba {
    Srgba::new(red, green, blue, 1.0)
}

/// Distinctive, and reads arcane rather than computer.
///
/// §4's original default, and now one of four. It stayed on the list because it
/// is the one theme nobody mistakes for a real terminal.
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
    syntax: Some(SPELL),
    background: rgb(0.055, 0.035, 0.075),
};

/// The default. Warm, classic, and the tube a scrying orb ought to be.
///
/// What a real phosphor terminal looked like when it was not green, and warmer
/// than either alternative — which suits a thing stared into by candlelight.
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
    syntax: Some(SPELL),
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
    syntax: Some(SPELL),
    background: rgb(0.020, 0.055, 0.030),
};

/// Light grey on black — the one theme that is not a phosphor.
///
/// Here for accessibility: the highest contrast, and a base ramp with no hue to
/// lose. Solved, not picked — a light base leaves little headroom, so it sits
/// at 0.74.
pub(crate) const MONOCHROME: Phosphor = Phosphor {
    name: "monochrome",
    base: [
        rgb(0.38, 0.38, 0.41),
        rgb(0.74, 0.74, 0.78),
        rgb(1.00, 1.00, 1.00),
    ],
    danger: rgb(0.95, 0.25, 0.24),
    // Lifted in green only, to clear `danger` under deuteranopia (1.27:1)
    // while keeping body text at 1.23:1. Solve it in floats: the margin is
    // smaller than a 0–255 step (§19).
    cost: rgb(0.30, 0.70, 1.00),
    success: rgb(0.62, 1.00, 0.60),
    // The one theme that declines: a theme named for one colour cannot sprout
    // five, and a player who reads hue poorly gets a setting rather than a
    // complaint. A spell here draws in weight alone.
    syntax: None,
    // Cooler and darker than the phosphors', because there is no warm hue in the
    // text to sit against. Still not pure black — §4, and `main.rs`: a dead
    // screen and an idle one must not look the same.
    background: rgb(0.020, 0.020, 0.025),
};

/// A radar tube: blue-white, with a long afterglow nothing here draws.
///
/// Hidden — never offered, cycled or reached by a prefix; `theme p7` finds it
/// (§3: discovered, never pushed). Held to every floor the offered themes are:
/// the tests walk [`EVERY`]. `cost` is the warm one here, because the base is
/// already the blue `cost` is elsewhere.
pub(crate) const P7: Phosphor = Phosphor {
    name: "p7",
    base: [
        rgb(0.30, 0.38, 0.52),
        rgb(0.60, 0.72, 0.88),
        rgb(0.88, 0.95, 1.00),
    ],
    danger: rgb(0.98, 0.30, 0.26),
    cost: rgb(0.98, 0.76, 0.32),
    // Paler than the other themes' greens, and solved, not picked: at
    // `(0.62, 1.00, 0.50)` it sat 1.21:1 from `cost` under deuteranopia.
    success: rgb(0.75, 1.00, 0.75),
    syntax: Some(SPELL),
    background: rgb(0.015, 0.022, 0.045),
};

/// Every theme, in the order the settings screen offers them.
///
/// The default comes first, so `F2` starts by showing what the alternatives are
/// alternatives *to*.
pub(crate) const ALL: [Phosphor; 4] = [AMBER, GREEN, MUTED_VIOLET, MONOCHROME];

/// Every theme there is, offered or not. What a name is looked up in, and what
/// the tests hold to their floors.
pub(crate) const EVERY: [Phosphor; 5] = [AMBER, GREEN, MUTED_VIOLET, MONOCHROME, P7];

/// Phosphor part numbers, as the orb knows them: `theme p3` is amber.
///
/// Real JEDEC designations, so no brand. P7 is its own theme; the rest name
/// the offered ones.
pub(crate) const NUMBERED: [(&str, &str); 5] = [
    ("p1", "green"),
    ("p3", "amber"),
    ("p4", "monochrome"),
    ("p7", "p7"),
    ("p31", "green"),
];

/// The theme called `name`, offered or not.
///
/// The one lookup the settings row, `Theme::default` and `F2` share, so a
/// hidden theme cannot be set by one and lost by another on relaunch.
pub(crate) fn named(name: &str) -> Option<Phosphor> {
    EVERY.into_iter().find(|phosphor| phosphor.name == name)
}

impl Phosphor {
    /// The colour a cell of this style is drawn in, with no material tint.
    ///
    /// Most of the screen. [`Phosphor::resolve_tinted`] is the same thing for
    /// the handful of cells inside an instrument's bar.
    pub(crate) fn resolve(&self, style: Style) -> Color {
        self.resolve_tinted(style, None, None)
    }

    /// The colour a cell draws in, given the colour family of whatever region it
    /// falls in.
    ///
    /// The tint is tried first and may decline — on an accent (§4 keeps the
    /// triad for meaning) and on the fire (§19 makes it one orange ramp), so the
    /// channels below are reached when a material has nothing to say.
    pub(crate) fn resolve_tinted(
        &self,
        style: Style,
        wash: Option<Wash>,
        syntax: Option<Lexeme>,
    ) -> Color {
        if let Some(wash) = wash
            && let Some(tinted) =
                super::tint::resolve(wash, style.role, style.intensity, style.depicted())
        {
            return tinted.into();
        }
        if let Some(lit) = self.lexed(syntax, style.role) {
            return lit.into();
        }
        self.untinted(style)
    }

    /// A part of speech's colour under this theme, or `None` if it declines.
    ///
    /// An accent outranks it (§4), refused here as well as in `orbs-shell`.
    /// `monochrome` has no palette.
    fn lexed(&self, syntax: Option<Lexeme>, role: Role) -> Option<Srgba> {
        if role != Role::Normal {
            return None;
        }
        let slot = syntax_slot(syntax?)?;
        self.syntax.map(|palette| palette[slot])
    }

    /// The colour a cell of this style is drawn in.
    fn untinted(&self, style: Style) -> Color {
        // Per cell per frame, so skip the common case. The raw field is safe
        // only to skip: `depicted()` never turns `None` into a depiction.
        if !matches!(style.depiction, Depiction::None) {
            // `depicted()`, never `style.depiction`: the accessor enforces §4's
            // "accents are never decorative" by yielding `None` on any accented
            // cell, so a `Role::Danger` cell never comes back in flame colours.
            if let Some(depicted) = ember::resolve(style.depicted()) {
                return depicted.into();
            }
        }

        let srgba = match style.role {
            // Ordinary text varies on intensity alone (§4).
            Role::Normal => self.base[weight(style.intensity)],
            // An accent is already a signal; intensity must not dilute it.
            Role::Danger => self.danger,
            Role::Cost => self.cost,
            Role::Success => self.success,
        };
        srgba.into()
    }

    /// The base hue at one weight. For tests in [`ember`](super::ember), which
    /// have to compare a flame against the body text of the same theme.
    #[cfg(test)]
    pub(crate) const fn base_at(&self, intensity: Intensity) -> Srgba {
        self.base[weight(intensity)]
    }
}

const fn weight(intensity: Intensity) -> usize {
    match intensity {
        Intensity::Dim => 0,
        Intensity::Normal => 1,
        Intensity::Bright => 2,
    }
}

// WCAG's measures, which live with `contrast high` now that it runs on them.
// Here for the tests below and `ember`'s, which hold their ramps to one formula.
#[cfg(test)]
pub(crate) use super::legibility::{contrast, luminance};

impl Phosphor {
    /// This theme under `contrast high`: ordinary and spell text lifted to
    /// 7:1, the ramp's steps kept, the accents untouched. See
    /// [`legibility::raise`](super::legibility::raise).
    pub(crate) fn raised(self) -> Self {
        let (base, syntax) = super::legibility::raise(self.base, self.syntax, self.background);
        Self {
            base,
            syntax,
            ..self
        }
    }

    /// The theme as drawn, with or without the lift.
    pub(crate) fn drawn(self, contrast: super::legibility::Contrast) -> Self {
        if orbs_shell::settings::Pair::raised(contrast) {
            self.raised()
        } else {
            self
        }
    }
}

#[cfg(test)]
mod tests {
    /// A real bar, painted by the real painter, resolved by the real palette.
    ///
    /// The gap every unit test here leaves open: `bath::cell` proves it emits a
    /// depiction and `ember::resolve` proves a depiction becomes a colour, but
    /// neither proves the two are connected — a painter dropping the depiction
    /// on the way into the `Cell` keeps both green while the bar draws in base.
    #[test]
    fn an_instrument_picture_reaches_the_palette_as_its_own_colour() {
        use orbs_render::{Frame, GridSize, Rect, Steep};

        let area = Rect::new(0, 0, 2, 8);
        let mut frame = Frame::new(GridSize::new(2, 8));
        frame.painter(area).bath_meter_upward(
            area,
            8,
            8,
            Steep {
                phase: 3.0,
                motion: orbs_render::Motion::Bubbling,
                ..Steep::default()
            },
        );

        let theme = super::ALL[0];
        let base: Vec<_> = [
            orbs_render::Intensity::Dim,
            orbs_render::Intensity::Normal,
            orbs_render::Intensity::Bright,
        ]
        .iter()
        .map(|weight| theme.base_at(*weight))
        .collect();

        let mut liquid = 0;
        for row in 0..8 {
            for col in 0..2 {
                let cell = frame
                    .cell(orbs_render::Pos::new(col, row))
                    .expect("inside the grid");
                let colour = theme.resolve(cell.style);
                assert!(
                    !base.iter().any(|weight| Color::from(*weight) == colour),
                    "a liquid cell resolved to the base hue — the picture is not \
                     reaching the palette",
                );
                liquid += 1;
            }
        }
        assert_eq!(liquid, 16, "the bar did not fill");
    }

    use super::*;

    /// Body text against its own background, at the weight most of the game is
    /// drawn in. 4.5:1 is WCAG AA for normal text.
    #[test]
    fn body_text_is_legible_against_its_background() {
        for theme in EVERY {
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
        for theme in EVERY {
            let ratio = contrast(theme.base[weight(Intensity::Dim)], theme.background);
            assert!(
                ratio >= 3.0,
                "{}: dim text contrasts only {ratio:.1}:1",
                theme.name
            );
        }
    }

    #[test]
    fn the_tube_comes_up_amber() {
        // Really an assertion about the *order*: `F2` cycles the list, and a
        // default that is not where the cycle starts makes F2 do nothing.
        assert_eq!(ALL[0].name, "amber");
        assert_eq!(super::super::plugin::Theme::default().0.name, "amber");
    }

    #[test]
    fn the_monochrome_theme_has_no_hue_in_its_base() {
        // What makes it the accessible one rather than a fourth colour: there is
        // no hue in the base ramp to lose. An "almost grey" base would be a
        // colour theme wearing the name.
        for weight in MONOCHROME.base {
            let spread = [weight.red, weight.green, weight.blue];
            let (low, high) = (
                spread.iter().copied().fold(f32::MAX, f32::min),
                spread.iter().copied().fold(f32::MIN, f32::max),
            );
            assert!(
                high - low <= 0.05,
                "monochrome's base has {:.2} of hue in it",
                high - low,
            );
        }
    }

    #[test]
    fn intensity_is_monotonic() {
        // The whole point of the base ramp: dim recedes, bright advances. A
        // theme where bright is darker than normal would invert every emphasis.
        for theme in EVERY {
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
    /// The accents are the one place colour *is* meaning, so a luminance gap is
    /// what makes them survive greyscale and most colour vision deficiencies.
    #[test]
    fn the_accent_triad_is_separable_without_hue() {
        for theme in EVERY {
            // The proxy: all the hue gone at once. A real deficiency takes one
            // axis and leaves the others — see
            // `the_accent_triad_survives_every_deficiency`, which found the gap.
            triad_separates(theme, "greyscale", |colour| colour);
        }
    }

    /// §14's claim, measured instead of approximated.
    ///
    /// A real deficiency takes one axis and can fail where greyscale passes
    /// ([`deficiency`](super::super::deficiency)); monochrome's margin is
    /// bounded near 1.36 (§19).
    #[test]
    fn the_accent_triad_survives_every_deficiency() {
        use super::super::deficiency::{Deficiency, simulated};

        for theme in EVERY {
            for deficiency in Deficiency::ALL {
                triad_separates(theme, deficiency.name(), |colour| {
                    simulated(deficiency, colour)
                });
            }
        }
    }

    /// The triad, pairwise, through whatever `seen` does to it.
    ///
    /// One walk, because there were two: forty duplicated lines for one call,
    /// and two copies of the 1.25 floor silently disagreeing is how the proxy
    /// and the measurement would stop meaning the same thing.
    fn triad_separates(theme: Phosphor, through: &str, seen: impl Fn(Srgba) -> Srgba) {
        let accents = [
            ("danger", seen(theme.danger)),
            ("cost", seen(theme.cost)),
            ("success", seen(theme.success)),
        ];
        for (a_name, a) in accents {
            for (b_name, b) in accents {
                if a_name >= b_name {
                    continue;
                }
                let ratio = contrast(a, b);
                assert!(
                    ratio >= 1.25,
                    "{}: through {through}, {a_name} and {b_name} differ by \
                     {ratio:.2}:1 — the same brightness to a player who cannot \
                     separate them by hue either",
                    theme.name,
                );
            }
        }
    }

    #[test]
    fn every_accent_is_legible_against_its_own_background() {
        for theme in EVERY {
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
        for theme in EVERY {
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
        for theme in EVERY {
            assert!(luminance(theme.background) > 0.0, "{}", theme.name);
        }
    }
}
