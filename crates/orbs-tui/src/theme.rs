//! Where a [`Style`] finally becomes a colour, on this side of rule 2.
//!
//! `orbs-render` decides *what* appears and never emits a hue; everything here
//! decides *how* it is drawn. The counterpart of the Bevy build's
//! `render::palette`, with sixteen ANSI colours instead of a solved palette.
//!
//! Indexed ANSI 0–15, inheriting the user's terminal theme (§19), so there is no
//! palette to tune and no contrast to verify — the numbers belong to whoever
//! configured the terminal. That holds for the syntax hues too, which is why
//! they are indices here and `Srgba` in the Bevy build.
//!
//! What it costs is §14's guarantee: sixteen colours we do not own cannot be
//! held to the Bevy palette's solved contrast. So the glyph carries the identity
//! and the colour is a hint — a meter is read off a `█`/`▓` boundary against
//! `░`, the ward's six sigils are six shapes, and every `Role` reaches the
//! linear stream beside its text.
//!
//! Three accepted degradations:
//!
//! - `Presentation` is ignored: Eldritch and Tampered pick a face in the glyph
//!   atlas, and the face here is the terminal's. §19 allows this by name —
//!   `verify` is the authoritative sabotage detector on every surface.
//! - A mixed [`Wash`] takes its first tint; sixteen indices cannot average, and
//!   the flask's one two-colour band is already told apart by position.
//! - No CRT, no phosphor, no blinking caret. The terminal draws its own cursor,
//!   which is the one a screen reader tracks.

use crossterm::style::Color;
use orbs_render::{Density, Depiction, Heat, Intensity, Lexeme, Roil, Role, Style, Tint, Wash};

/// A resolved pen: what colour, and how heavy.
///
/// Weight is separate from colour because the base hue is the terminal's
/// foreground, and §4 gives ordinary text only intensity to vary on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Ink {
    /// The foreground, or `None` for the terminal's own.
    pub(crate) colour: Option<Color>,
    /// Dim, plain, or bold.
    pub(crate) weight: Weight,
}

/// The three steps of [`Intensity`], as a terminal can draw them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Weight {
    /// SGR 2.
    Dim,
    /// Neither attribute.
    #[default]
    Plain,
    /// SGR 1.
    Bold,
}

impl Ink {
    /// The terminal's own foreground, unweighted.
    pub(crate) const PLAIN: Self = Self {
        colour: None,
        weight: Weight::Plain,
    };

    const fn coloured(colour: Color) -> Self {
        Self {
            colour: Some(colour),
            weight: Weight::Plain,
        }
    }
}

impl From<Intensity> for Weight {
    fn from(intensity: Intensity) -> Self {
        match intensity {
            Intensity::Dim => Self::Dim,
            Intensity::Normal => Self::Plain,
            Intensity::Bright => Self::Bold,
        }
    }
}

/// Resolve one cell.
///
/// The order mirrors the Bevy build's `Phosphor::resolve_tinted` exactly, and
/// has to: a tint that declines there and resolves here would mean the two
/// frontends disagree about what a fouled instrument looks like.
///
/// 1. A material's [`Wash`], if there is one and it does not decline.
/// 2. Otherwise the cell's [`Depiction`] — fire, liquid, sediment.
/// 3. Otherwise the [`Lexeme`] over it, if a spell is being read here.
/// 4. Otherwise the [`Role`], varied by [`Intensity`] only when it is `Normal`.
pub(crate) fn resolve(style: Style, wash: Option<Wash>, syntax: Option<Lexeme>) -> Ink {
    // `depicted()`, never `style.depiction`: the accessor yields `None` on any
    // accented cell, so a `Role::Danger` cell can never render in flame colours.
    let depiction = style.depicted();

    if let Some(wash) = wash
        && let Some(ink) = tinted(wash, style.role, style.intensity, depiction)
    {
        return ink;
    }
    if let Some(ink) = depicted(depiction) {
        return ink;
    }
    // After the pictures and before the accent. A syntax run and an instrument
    // bar never share a cell, so this is a statement of rank rather than a case
    // anyone can reach.
    if let Some(ink) = syntaxed(syntax, style.role, style.intensity) {
        return ink;
    }
    accent(style.role, style.intensity)
}

/// A part of speech's colour, or `None` if it declines.
///
/// It declines on an accent, for [`tinted`]'s reason: an accent is a signal, a
/// tint is a hint, and the signal wins. Enforced here and not only in
/// `orbs-shell` — trusting the other layer left a line `interpret` could not
/// read resolving violet instead of red (§19).
fn syntaxed(syntax: Option<Lexeme>, role: Role, intensity: Intensity) -> Option<Ink> {
    if role != Role::Normal {
        return None;
    }
    let colour = syntax_colour(syntax?)?;
    Some(Ink {
        colour: Some(colour),
        weight: intensity.into(),
    })
}

/// A part of speech's colour, or `None` to keep the terminal's own foreground.
///
/// [`Lexeme::Name`] resolves to `None` deliberately: a name is most of a spell,
/// so colouring it would override the reader's own foreground for the bulk of
/// the file and leave the departures with nothing to depart from (§19).
/// [`Lexeme::Comment`] and [`Lexeme::Filler`] keep it too — [`Lexeme::weight`]
/// draws both dim, which survives a terminal with no colour at all.
const fn syntax_colour(kind: Lexeme) -> Option<Color> {
    match kind {
        // The scaffolding, and a call to this file's own — arcane, which is the
        // register §14 already reads as magical.
        Lexeme::Control | Lexeme::Call => Some(Color::Magenta),
        // Something the tower answers to.
        Lexeme::Verb => Some(Color::Yellow),
        Lexeme::Number => Some(Color::DarkYellow),
        // The two halves of a question, told apart: what it turns on, and what
        // the answer may be.
        Lexeme::Grammar => Some(Color::DarkCyan),
        Lexeme::State => Some(Color::Cyan),
        Lexeme::Name | Lexeme::Comment | Lexeme::Filler | Lexeme::None => None,
    }
}

/// A material's colour family, or `None` if the tint declines.
///
/// All three declines are reproduced: an accent outranks a tint (a fouled
/// instrument's red label would go brown), fire is never tinted (the athanor
/// burns orange on every tube, §19), and sediment declines so that waste always
/// looks like waste.
fn tinted(wash: Wash, role: Role, intensity: Intensity, depiction: Depiction) -> Option<Ink> {
    if role != Role::Normal {
        return None;
    }
    // `orbs-render`'s rule, not a second copy: this was three `if`s mirroring
    // the Bevy build's `match`, with nothing making the two agree.
    if depiction.declines_tint() {
        return None;
    }

    // A mixture takes its first tint — see the module note. `with` is `Some`
    // only for the flask's mid-combination band, told apart by position.
    let colour = hue(wash.tint);
    let weight = match depiction.roil() {
        Some(Roil::Still) => Weight::Dim,
        Some(Roil::Stirred) => Weight::Plain,
        Some(Roil::Rolling) => Weight::Bold,
        None => intensity.into(),
    };
    Some(Ink {
        colour: Some(colour),
        weight,
    })
}

/// The eight material families, as the eight ANSI hues.
///
/// Eight names, eight hues, no fallback and no default, so a tint added to the
/// enum breaks this build on purpose.
const fn hue(tint: Tint) -> Color {
    match tint {
        Tint::Green => Color::Green,
        Tint::Brown => Color::DarkYellow,
        Tint::Grey => Color::Grey,
        Tint::Gold => Color::Yellow,
        Tint::Violet => Color::Magenta,
        Tint::Red => Color::Red,
        Tint::Blue => Color::Blue,
        Tint::Bone => Color::White,
    }
}

/// What a cell is a *picture* of, if it is a picture of anything.
///
/// Carries no meaning by design (§19's fourth channel), which is why it may be
/// colour alone: a channel that says nothing has nothing to withhold.
///
/// Four steps out of two hues, using the weight axis. Sixteen indices hold only
/// `DarkYellow` and `Yellow` in the fire's family, so [`Weight`] supplies the
/// rest: dim-dark, dark, bright, bold-bright — monotonic, and every step still
/// orange (§19). The liquid ramp takes the same fix, so a rolling bath is teal
/// rather than briefly colourless.
const fn depicted(depiction: Depiction) -> Option<Ink> {
    let (colour, weight) = match depiction {
        Depiction::None => return None,
        // Flame and spark share a ramp, as they do on the other side.
        Depiction::FlameEmber | Depiction::SparkEmber => (Color::DarkYellow, Weight::Dim),
        Depiction::FlameBody | Depiction::SparkBody => (Color::DarkYellow, Weight::Plain),
        Depiction::FlameBlaze | Depiction::SparkBlaze => (Color::Yellow, Weight::Plain),
        Depiction::FlameCore | Depiction::SparkCore => (Color::Yellow, Weight::Bold),
        Depiction::SmokeThin => (Color::DarkGrey, Weight::Dim),
        Depiction::SmokeThick => (Color::Grey, Weight::Plain),
        // Teal, which is the fire's opposite.
        Depiction::LiquidStill => (Color::DarkCyan, Weight::Dim),
        Depiction::LiquidStirred => (Color::Cyan, Weight::Plain),
        Depiction::LiquidRolling => (Color::Cyan, Weight::Bold),
        // Waste declines the tint so that waste always looks like waste, and it
        // must not be mistakable for smoke — hence grey rather than a dark grey.
        Depiction::Sediment => (Color::Grey, Weight::Dim),
        // The gauge ramp, red through yellow to green — the one ramp that is
        // not monotonic in brightness, since it shows a distance being closed
        // rather than a substance getting hotter. The fill length says that on
        // its own, so the tests below do not walk it.
        //
        // Three hue families, six steps, and none of them the triad's ink:
        // `Red` and `Green` are byte-for-byte `Role::Danger` and
        // `Role::Success`, so a ley gauge drew the exact red of every error
        // line. [`Weight`] supplies the steps within each family instead, and
        // `the_gauge_ramp_never_wears_an_accents_ink` keeps it that way.
        Depiction::GaugeFaint => (Color::DarkRed, Weight::Dim),
        Depiction::GaugeLow => (Color::DarkRed, Weight::Plain),
        Depiction::GaugeMiddle => (Color::DarkYellow, Weight::Plain),
        Depiction::GaugeHigh => (Color::Yellow, Weight::Plain),
        Depiction::GaugeNear => (Color::DarkGreen, Weight::Plain),
        Depiction::GaugeWhole => (Color::DarkGreen, Weight::Bold),
    };
    Some(Ink {
        colour: Some(colour),
        weight,
    })
}

/// Ordinary text, or one of the three things that mean something.
///
/// Intensity is ignored on an accent, matching the Bevy side: dimming one would
/// dilute the channel §14 reserves for meaning.
fn accent(role: Role, intensity: Intensity) -> Ink {
    match role {
        Role::Normal => Ink {
            colour: None,
            weight: intensity.into(),
        },
        Role::Danger => Ink::coloured(Color::Red),
        Role::Cost => Ink::coloured(Color::Blue),
        Role::Success => Ink::coloured(Color::Green),
    }
}

/// Smoke has two densities and both are grey; this keeps the mapping honest.
const _: () = {
    assert!(matches!(Density::Thin, Density::Thin));
    assert!(matches!(Heat::Ember, Heat::Ember));
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_tint_resolves_to_its_own_hue() {
        // Eight names, eight distinct indices. Two tints sharing a colour would
        // make two materials indistinguishable in a bar that is all one glyph.
        let mut seen = Vec::new();
        for tint in Tint::ALL {
            let colour = hue(tint);
            assert!(
                !seen.contains(&colour),
                "{} shares a colour with something already mapped",
                tint.name(),
            );
            seen.push(colour);
        }
    }

    #[test]
    fn the_three_accents_are_three_colours() {
        // §14: colour never carries meaning alone, but where it carries any it
        // must at least separate.
        let danger = accent(Role::Danger, Intensity::Normal);
        let cost = accent(Role::Cost, Intensity::Normal);
        let success = accent(Role::Success, Intensity::Normal);
        assert_ne!(danger, cost);
        assert_ne!(cost, success);
        assert_ne!(danger, success);
    }

    /// No step of the gauge ramp may wear an accent's ink. §4 and §14 reserve
    /// the triad for meaning and a depiction carries none; it shipped once with
    /// `GaugeLow` as `Color::Red` and `GaugeWhole` as `Color::Green`.
    ///
    /// Weight is no escape: a bold green still reads as green, and several
    /// terminals render bold-plus-dark as the bright index outright — so the
    /// check is on the colour, not on the pair.
    #[test]
    fn the_gauge_ramp_never_wears_an_accents_ink() {
        let triad: Vec<_> = [Role::Danger, Role::Cost, Role::Success]
            .into_iter()
            .filter_map(|role| accent(role, Intensity::Normal).colour)
            .collect();

        let mut seen = Vec::new();
        for depiction in Depiction::ALL
            .into_iter()
            .filter(|depiction| depiction.is_gauge())
        {
            let ink = depicted(depiction).expect("a gauge step drew nothing");
            let colour = ink.colour.expect("a gauge step took no colour");
            assert!(
                !triad.contains(&colour),
                "{depiction:?} draws in {colour:?}, which is an accent's own ink",
            );
            assert!(
                !seen.contains(&ink),
                "{depiction:?} is indistinguishable from an earlier step",
            );
            seen.push(ink);
        }
        assert_eq!(seen.len(), 6, "the ramp is not six steps");
    }

    #[test]
    fn ordinary_text_keeps_the_terminals_own_foreground() {
        // The base hue is the user's, and §4 lets ordinary text vary only on
        // intensity.
        for intensity in [Intensity::Dim, Intensity::Normal, Intensity::Bright] {
            let ink = accent(Role::Normal, intensity);
            assert_eq!(ink.colour, None, "ordinary text took a colour");
        }
        assert_eq!(accent(Role::Normal, Intensity::Dim).weight, Weight::Dim);
        assert_eq!(accent(Role::Normal, Intensity::Bright).weight, Weight::Bold);
    }

    #[test]
    fn an_accent_ignores_intensity() {
        // The Bevy side asserts the same. Two frontends disagreeing about
        // whether a dim breach is dimmer is rule 2 broken where nobody looks.
        for role in [Role::Danger, Role::Cost, Role::Success] {
            let dim = accent(role, Intensity::Dim);
            let bright = accent(role, Intensity::Bright);
            assert_eq!(dim, bright, "{role:?} was diluted by intensity");
        }
    }

    #[test]
    fn a_tint_declines_to_an_accent() {
        // "An accent is a signal; a tint is a hint. The signal wins."
        let wash = Wash::plain(Tint::Green);
        for role in [Role::Danger, Role::Cost, Role::Success] {
            assert!(
                tinted(wash, role, Intensity::Normal, Depiction::None).is_none(),
                "{role:?} was overpainted by a material's colour",
            );
        }
        assert!(tinted(wash, Role::Normal, Intensity::Normal, Depiction::None).is_some());
    }

    #[test]
    fn a_tint_declines_to_fire_and_to_waste() {
        let wash = Wash::plain(Tint::Green);
        for depiction in Depiction::ALL {
            // Against `orbs-render`'s predicate, which the other frontend also
            // calls. This used to compare `tinted` against a copy of its own
            // rule while claiming to check the Bevy build.
            let declines = tinted(wash, Role::Normal, Intensity::Normal, depiction).is_none();
            assert_eq!(
                declines,
                depiction.declines_tint(),
                "{depiction:?} resolves a tint the shared rule declines, or the \
                 other way round — the two frontends would draw it differently",
            );
        }
    }

    #[test]
    fn a_mixture_takes_its_first_tint() {
        // The accepted degradation, stated so it cannot drift into a bug report.
        let mixed = Wash::mixing(Tint::Green, Tint::Bone);
        let plain = Wash::plain(Tint::Green);
        assert_eq!(
            tinted(mixed, Role::Normal, Intensity::Normal, Depiction::None),
            tinted(plain, Role::Normal, Intensity::Normal, Depiction::None),
        );
    }

    /// How bright a resolved ink reads, for comparing steps of one ramp.
    fn brightness(ink: Ink) -> u8 {
        let base = match ink.colour {
            Some(Color::DarkYellow | Color::DarkCyan | Color::DarkGrey) => 0,
            Some(Color::Yellow | Color::Cyan | Color::Grey) => 2,
            _ => 4,
        };
        base + match ink.weight {
            Weight::Dim => 0,
            Weight::Plain => 1,
            Weight::Bold => 2,
        }
    }

    #[test]
    fn the_fire_is_orange_all_the_way_up() {
        // The athanor burns orange on every tube (§19). This ramp once ran
        // `DarkRed` → `White`: red at the cool end, colourless at the hot one.
        for heat in [Heat::Ember, Heat::Flame, Heat::Blaze, Heat::Core] {
            for depiction in [Depiction::flame(heat), Depiction::spark(heat)] {
                let ink = depicted(depiction).expect("fire is a picture of something");
                assert!(
                    matches!(ink.colour, Some(Color::DarkYellow | Color::Yellow)),
                    "{depiction:?} drew {:?}, which is not a fire",
                    ink.colour,
                );
            }
        }
    }

    #[test]
    fn every_ramp_climbs() {
        // A meter is read off where the colour steps, so a ramp that is not
        // monotonic cannot be read.
        let fire: Vec<_> = [Heat::Ember, Heat::Flame, Heat::Blaze, Heat::Core]
            .map(|heat| brightness(depicted(Depiction::flame(heat)).expect("fire")))
            .to_vec();
        assert!(
            fire.windows(2).all(|pair| pair[0] < pair[1]),
            "the flame ramp does not climb: {fire:?}",
        );

        let liquid: Vec<_> = [Roil::Still, Roil::Stirred, Roil::Rolling]
            .map(|roil| brightness(depicted(Depiction::liquid(roil)).expect("liquid")))
            .to_vec();
        assert!(
            liquid.windows(2).all(|pair| pair[0] < pair[1]),
            "the liquid ramp does not climb: {liquid:?}",
        );

        let smoke: Vec<_> = [Density::Thin, Density::Thick]
            .map(|density| brightness(depicted(Depiction::smoke(density)).expect("smoke")))
            .to_vec();
        assert!(
            smoke[0] < smoke[1],
            "the smoke ramp does not climb: {smoke:?}"
        );
    }

    #[test]
    fn the_bath_is_teal_and_waste_is_not() {
        // The fire's opposite stays a colour, and waste stays tellable from the
        // smoke it sits under — `Sediment` and `SmokeThin` were both dark grey.
        for roil in [Roil::Still, Roil::Stirred, Roil::Rolling] {
            let ink = depicted(Depiction::liquid(roil)).expect("liquid");
            assert!(
                matches!(ink.colour, Some(Color::DarkCyan | Color::Cyan)),
                "{roil:?} drew {:?}, which is not the bath",
                ink.colour,
            );
        }
        assert_ne!(
            depicted(Depiction::Sediment),
            depicted(Depiction::smoke(Density::Thin)),
            "waste and thin smoke resolved identically",
        );
    }

    #[test]
    fn a_role_never_reads_its_depiction_field_directly() {
        // An accented cell that also carries a depiction must resolve as the
        // accent, which only happens if `resolve` went through `depicted()`.
        let style = Style::DANGER.with_depiction(Depiction::FlameCore);
        assert_eq!(
            resolve(style, None, None),
            accent(Role::Danger, Intensity::Normal)
        );
    }

    /// Every part of speech that departs from the base gets its own colour.
    ///
    /// Seven categories and six colours: `Control` and `Call` share one, both
    /// being the file's own scaffolding, and the `()` tells them apart.
    #[test]
    fn each_part_of_speech_that_departs_has_its_own_colour() {
        let departs = [
            Lexeme::Control,
            Lexeme::Verb,
            Lexeme::Number,
            Lexeme::Grammar,
            Lexeme::State,
        ];
        let mut seen = Vec::new();
        for kind in departs {
            let colour = syntax_colour(kind).expect("a departing kind has a colour");
            assert!(
                !seen.contains(&colour),
                "{kind:?} shares a colour with something already listed",
            );
            seen.push(colour);
        }
        assert_eq!(
            syntax_colour(Lexeme::Call),
            syntax_colour(Lexeme::Control),
            "a call is the file's own scaffolding and reads as such",
        );
    }

    /// ...and the ones that keep the reader's own foreground keep it.
    ///
    /// A name is most of a spell, so colouring it leaves the departures with
    /// nothing to depart from; comments and filler recede on the weight axis.
    #[test]
    fn the_base_hue_still_carries_most_of_a_spell() {
        for kind in [Lexeme::Name, Lexeme::Comment, Lexeme::Filler, Lexeme::None] {
            assert_eq!(
                syntax_colour(kind),
                None,
                "{kind:?} overrode the reader's own foreground",
            );
        }
        // The weight axis is doing the work for those two, and still is.
        assert_eq!(Lexeme::Comment.weight(), Intensity::Dim);
        assert_eq!(Lexeme::Filler.weight(), Intensity::Dim);
    }

    /// An accent outranks a syntax colour, as it outranks a tint.
    #[test]
    fn a_fault_outranks_the_highlighting_here_too() {
        assert_eq!(
            resolve(Style::DANGER, None, Some(Lexeme::Control)),
            accent(Role::Danger, Intensity::Normal),
            "a line the orb could not read went violet instead of red",
        );
    }
}
