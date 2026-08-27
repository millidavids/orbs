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
//! - **Amber by default** — warm, classic, and the tube a scrying orb ought to
//!   be. Muted violet was the original default and is still on the list.
//! - **The base hue carries all ordinary text through intensity alone.**
//! - **A small accent set is reserved strictly for meaning**, and every theme
//!   defines its own triad so contrast holds against its own base.
//!
//! [`Presentation`](orbs_render::Presentation) deliberately has no entry here:
//! it selects a *face* in the glyph atlas, not a colour. §14 forbids colour as
//! the sole carrier of meaning, and a tonal register that existed only as a hue
//! would be exactly that.

use bevy::prelude::*;
use orbs_render::{Depiction, Intensity, Lexeme, Role, Style, Wash};

use super::ember;

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
    /// The parts of speech a spell is drawn in, indexed by [`syntax_slot`].
    ///
    /// # `None` is a real answer, and `monochrome` gives it
    ///
    /// A theme named for having one colour cannot sprout five, and a player who
    /// wants a spell drawn in weight alone now has somewhere to go — which is
    /// the §14 answer for anyone who reads hue poorly, and the same escape the
    /// tube's own themes already offer for the phosphor.
    ///
    /// Weight carries the reading either way: `Lexeme::weight` is applied to the
    /// `Style` before this is consulted, so a monochrome spell reads exactly as
    /// every spell did before hue existed.
    syntax: Option<[Srgba; SYNTAX]>,
    /// Behind everything. Never fully black — a dead screen and an idle one
    /// should not look the same (see `main.rs`).
    pub(crate) background: Srgba,
}

/// How many parts of speech depart from the base hue.
///
/// Five, not eight. `Name` is most of a spell and keeps the phosphor — colouring
/// it would leave the departures with nothing to depart from — and `Comment` and
/// `Filler` already recede on the weight axis, which survives a theme with no
/// syntax palette at all.
pub(crate) const SYNTAX: usize = 5;

/// Where a lexeme sits in a theme's [`Phosphor::syntax`] table.
///
/// `None` for the three that keep the base hue. **`Call` shares `Control`'s
/// slot**: both are the file's own scaffolding, and what tells a block keyword
/// from a call to this file's own part is the `()`, not the colour.
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
/// **One table rather than three, for now, and that is an admission.** The
/// accents are tuned per theme because each sits against a different background;
/// these are not, because they were authored without a window to judge them in.
/// The field is per-theme so a pass with eyes on the tube can split them, which
/// is the shape §15 asks for — build the instrument, then use it.
///
/// Chosen to sit *away* from all three phosphors rather than within them: the
/// point of a hue here is to be told apart from the base, and amber's base is
/// already gold.
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
/// `pub(super)` so [`super::ember`] shares it rather than declaring an
/// identical one — it had, and two copies of a three-token helper is two places
/// for the alpha channel to be spelled differently.
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

/// **The default.** Warm, classic, and the tube a scrying orb ought to be.
///
/// Amber is what a real phosphor terminal looked like when it was not green, and
/// it is warmer than either alternative — which suits a thing a wizard stares
/// into by candlelight better than violet does.
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
/// Every other theme is a single hue at three weights, which is what a real tube
/// did and what §4 asks for. This is a modern terminal instead: neutral text,
/// and the accents carrying all of the colour there is.
///
/// It earns its place on accessibility rather than taste. A monochrome base is
/// the highest contrast the game can offer, and it is the only theme where a
/// player with a colour vision deficiency loses nothing at all — the base ramp
/// has no hue to lose, and §14's accent triad is separable by luminance anyway.
///
/// The values are **solved, not picked.** A light base leaves very little
/// luminance headroom above it, and the first attempt put `success` 1.22:1 from
/// body text — inside the greyscale-separation margin the tests below demand.
/// Lowering the base to 0.74 is what bought the accents room to be accents.
pub(crate) const MONOCHROME: Phosphor = Phosphor {
    name: "monochrome",
    base: [
        rgb(0.38, 0.38, 0.41),
        rgb(0.74, 0.74, 0.78),
        rgb(1.00, 1.00, 1.00),
    ],
    danger: rgb(0.95, 0.25, 0.24),
    // **Lifted in green, and only in green** — `0.62` was `1.06:1` from `danger`
    // under deuteranopia, which is the same brightness to the player this theme
    // is most for. `danger` is untouched and so are red and blue here: the solve
    // moved the one channel that could move.
    //
    // **Squeezed from both sides, which is the thing to know before touching
    // it.** `cost` here is pinned between the ≥4.5:1 floor against the
    // background below and the ≥1.2:1 floor against body text above, and the
    // deficiency floor pushes *up* into the second of those. This value clears
    // deuteranopia at 1.27:1 and body text at 1.23:1 — both with room, and
    // there is not much more to be had: buying another 0.06 of deficiency
    // margin spends body text down to 1.20 exactly.
    //
    // That is the *"very little luminance headroom"* this theme's own doc
    // comment warns about, met a second time; `success` was re-solved for the
    // same reason once already. **Solve it, do not pick it** — and solve it in
    // floats, because a value chosen on a 0–255 grid lands a rounding step away
    // and this margin is smaller than one step.
    cost: rgb(0.30, 0.70, 1.00),
    success: rgb(0.62, 1.00, 0.60),
    // **The one theme that declines**, and the reason it is worth having: a
    // theme named for one colour cannot sprout five, and a player who reads hue
    // poorly now has a setting rather than a complaint. A spell here draws in
    // weight alone, exactly as every spell did before hue existed.
    syntax: None,
    // Cooler and darker than the phosphors', because there is no warm hue in the
    // text to sit against. Still not pure black — §4, and `main.rs`: a dead
    // screen and an idle one must not look the same.
    background: rgb(0.020, 0.020, 0.025),
};

/// Every theme, in the order the settings screen offers them.
///
/// The default comes first, so `F2` starts by showing what the alternatives are
/// alternatives *to*.
pub(crate) const ALL: [Phosphor; 4] = [AMBER, GREEN, MUTED_VIOLET, MONOCHROME];

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
    /// **The tint is tried first and may decline.** It declines on an accent
    /// (§4 keeps the triad for meaning) and on the fire (§19 makes it one orange
    /// ramp everywhere), so the two channels below it are reached exactly when a
    /// material has nothing to say about the cell.
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
    /// # Two declines, and the first is §4's
    ///
    /// **An accent outranks a syntax colour**, exactly as it outranks a tint:
    /// *"an accent is a signal; a tint is a hint. The signal wins."* A line
    /// `interpret` could not read is drawn `Role::Danger`, and the fault is the
    /// thing the eye must go to — highlighting over it would be decoration
    /// beating meaning, which §4 forbids in one sentence. `orbs-shell` already
    /// refuses to register a run on an accented row and this refuses it again,
    /// because the equivalent hole in `orbs-tui` was real and its test caught it.
    ///
    /// **And a theme may have no palette at all** — `monochrome` does not, which
    /// is what makes hue a setting rather than something imposed.
    fn lexed(&self, syntax: Option<Lexeme>, role: Role) -> Option<Srgba> {
        if role != Role::Normal {
            return None;
        }
        let slot = syntax_slot(syntax?)?;
        self.syntax.map(|palette| palette[slot])
    }

    /// The colour a cell of this style is drawn in.
    fn untinted(&self, style: Style) -> Color {
        // **The overwhelmingly common case, taken first.** This runs once per
        // cell per frame — 7,040 times at the worst-case 160×44 — and a picture
        // occupies about thirty cells of one panel. Everything else on screen
        // pays two matches and a call to be told it is not a fire.
        //
        // **Reading the field directly is safe *only* to skip work.**
        // `depicted()` can turn a depiction into `None`; it can never turn
        // `None` into one, so a cell that fails this test would have resolved to
        // no colour anyway. That asymmetry is the whole argument, and it is why
        // the branch below still goes through the accessor rather than the
        // field.
        if !matches!(style.depiction, Depiction::None) {
            // **`depicted()`, never `style.depiction`.** The accessor is where
            // §4's "accents are never decorative" is enforced: it yields `None`
            // on any cell carrying an accent, so a `Role::Danger` cell can never
            // come back painted in flame colours. Reading the field directly
            // *here* is the one way to get this wrong, and `orbs-tui` will have
            // to resolve it too.
            if let Some(depicted) = ember::resolve(style.depicted()) {
                return depicted.into();
            }
        }

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

#[cfg(test)]
/// Relative luminance, per WCAG.
///
/// Used to check that meaning survives without hue — the cheap, honest proxy for
/// §14's colourblind-safety requirement. Two accents that differ only in hue are
/// the same colour to a substantial minority of players.
///
/// `pub(crate)` so [`ember`](super::ember) holds its ramps to the same bar. Two
/// copies of a luminance formula is how one of them quietly stops matching.
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

#[cfg(test)]
/// WCAG contrast ratio, `1.0..=21.0`. `pub(crate)` for the same reason
/// [`luminance`] is.
pub(crate) fn contrast(a: Srgba, b: Srgba) -> f32 {
    let (high, low) = {
        let (x, y) = (luminance(a), luminance(b));
        if x > y { (x, y) } else { (y, x) }
    };
    (high + 0.05) / (low + 0.05)
}

#[cfg(test)]
mod tests {
    /// A real bar, painted by the real painter, resolved by the real palette.
    ///
    /// **The gap every unit test in this crate leaves open.** `bath::cell`
    /// proves it emits a depiction, `ember::resolve` proves a depiction becomes
    /// a colour, and neither proves the two are *connected* — a painter that
    /// dropped the depiction on the way into the `Cell` would keep both green
    /// while the bar drew in plain base hue.
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
    fn the_tube_comes_up_amber() {
        // The default is `ALL[0]` by construction, so this is really an
        // assertion about the *order* — `F2` cycles the list, and a default that
        // is not where the cycle starts makes the first keypress do nothing.
        assert_eq!(ALL[0].name, "amber");
        assert_eq!(super::super::plugin::Theme::default().0.name, "amber");
    }

    #[test]
    fn the_monochrome_theme_has_no_hue_in_its_base() {
        // What makes it the accessible one rather than a fourth colour: a player
        // who cannot separate hues loses nothing from the base ramp, because
        // there is nothing there to lose. An "almost grey" base would be a
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
            // The proxy: all the hue gone at once. A real deficiency takes one
            // axis and leaves the others, which is a different picture — see
            // `the_accent_triad_survives_every_deficiency`, which is the
            // measurement this approximates and which found what it missed.
            triad_separates(theme, "greyscale", |colour| colour);
        }
    }

    /// §14's claim, measured instead of approximated.
    ///
    /// [`the_accent_triad_is_separable_without_hue`] takes *all* the hue away,
    /// which is the cheap proxy — a real deficiency takes away one axis and
    /// leaves the others, so it is a different picture and it can fail where
    /// greyscale passes. This is the honest version, and it is the reason
    /// [`deficiency`](super::super::deficiency) exists.
    ///
    /// **It failed on its first run**, at exactly one pair: monochrome's danger
    /// and cost sat 1.05:1 apart under deuteranopia. §19 records the re-solve,
    /// and the shape of it is the lesson — there was no scalar fix, because that
    /// theme's `cost` is pinned between the ≥4.5:1 background floor below and
    /// the ≥1.2:1 body-text floor above. The margin only ever reaches ~1.36
    /// however far the colour is moved, which is the *"very little luminance
    /// headroom"* its own doc comment warned about.
    #[test]
    fn the_accent_triad_survives_every_deficiency() {
        use super::super::deficiency::{Deficiency, simulated};

        for theme in ALL {
            for deficiency in Deficiency::ALL {
                triad_separates(theme, deficiency.name(), |colour| {
                    simulated(deficiency, colour)
                });
            }
        }
    }

    /// The triad, pairwise, through whatever `seen` does to it.
    ///
    /// **One walk, because there were two.** The greyscale proxy and the
    /// deficiency measurement built the same three-tuple, deduped pairs with the
    /// same `a_name >= b_name`, and used the same 1.25 floor — forty lines
    /// duplicated for one call. The floor is the thing most likely to be
    /// revisited, and two copies of it silently disagreeing is exactly how the
    /// proxy and the measurement would stop meaning the same thing.
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
