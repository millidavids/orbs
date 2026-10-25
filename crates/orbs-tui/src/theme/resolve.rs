//! One cell's style, region and lexeme, resolved to a pen.

use crossterm::style::Color;
use orbs_render::{Depiction, Intensity, Lexeme, Roil, Role, Style, Tint, Wash};

use super::ink::{Ink, Weight};

/// Resolve one cell.
///
/// In `Phosphor::resolve_tinted`'s order, so both builds agree.
///
/// 1. A material's [`Wash`], if there is one and it does not decline.
/// 2. Otherwise the [`Lexeme`] over it, if a spell is being read here.
/// 3. Otherwise the cell's [`Depiction`] — fire, liquid, sediment.
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
    // A syntax run and an instrument bar never share a cell, so this rank is a
    // statement of agreement with the window rather than a case anyone reaches.
    if let Some(ink) = syntaxed(syntax, style.role, style.intensity) {
        return ink;
    }
    if let Some(ink) = depicted(depiction) {
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
pub(super) const fn syntax_colour(kind: Lexeme) -> Option<Color> {
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
pub(super) fn tinted(
    wash: Wash,
    role: Role,
    intensity: Intensity,
    depiction: Depiction,
) -> Option<Ink> {
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
pub(super) const fn hue(tint: Tint) -> Color {
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
/// Meaningless by design, so colour alone (§19). Four steps from two hues,
/// with [`Weight`] supplying the rest, for fire and liquid alike.
pub(super) const fn depicted(depiction: Depiction) -> Option<Ink> {
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
        // The gauge, red to green, never the triad's own ink
        // (`the_gauge_ramp_never_wears_an_accents_ink`); [`Weight`] steps
        // within each family.
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
pub(super) fn accent(role: Role, intensity: Intensity) -> Ink {
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
