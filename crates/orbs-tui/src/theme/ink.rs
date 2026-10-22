//! A resolved pen, as a terminal can draw one.

use crossterm::style::Color;
use orbs_render::Intensity;

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

    pub(super) const fn coloured(colour: Color) -> Self {
        Self {
            colour: Some(colour),
            weight: Weight::Plain,
        }
    }

    /// This pen with its hue taken out — `sight greyscale`.
    ///
    /// The colour becomes one of the terminal's own greys by how light it is,
    /// and the weight stays: the window keeps luminance when it takes the hue
    /// out, and weight is what this build's ramps were built to survive on.
    /// Never black, which vanishes on the dark background most terminals have.
    pub(crate) const fn without_hue(self) -> Self {
        Self {
            colour: match self.colour {
                Some(colour) => Some(grey(colour)),
                None => None,
            },
            weight: self.weight,
        }
    }
}

/// The grey a hue reads as, by its ordinary ANSI lightness: white for the
/// brightest, grey for the middle, dark grey for the rest.
///
/// The named sixteen are the only colours `theme` emits; anything else is left
/// as it is.
const fn grey(colour: Color) -> Color {
    match colour {
        Color::White | Color::Yellow => Color::White,
        Color::Grey
        | Color::Green
        | Color::Cyan
        | Color::DarkGreen
        | Color::DarkYellow
        | Color::DarkCyan => Color::Grey,
        Color::DarkGrey
        | Color::Black
        | Color::Red
        | Color::Blue
        | Color::Magenta
        | Color::DarkRed
        | Color::DarkBlue
        | Color::DarkMagenta => Color::DarkGrey,
        other => other,
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
