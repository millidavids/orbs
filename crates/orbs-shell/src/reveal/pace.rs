//! How fast text types itself.

/// How fast text types itself — `settings` → `habits` → `pace`, in baud.
///
/// A teletype ran ~10 characters a second and is unusable; a 9600-baud VT100
/// managed ~960 and is indistinguishable from instant. The default was settled
/// by looking at 420, and is 4800 baud now that it has a name.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Pace {
    /// 120 characters a second: a line visibly typing.
    Baud1200,
    /// 480 characters a second.
    #[default]
    Baud4800,
    /// 960 characters a second, which reads as nearly instant.
    Baud9600,
    /// No reveal at all — what stillness forces.
    Instant,
}

impl Pace {
    /// All four, in the order the row cycles them.
    pub const ALL: [Self; 4] = [
        Self::Baud1200,
        Self::Baud4800,
        Self::Baud9600,
        Self::Instant,
    ];

    /// The word for it, on the page and in the file.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Baud1200 => "1200",
            Self::Baud4800 => "4800",
            Self::Baud9600 => "9600",
            Self::Instant => "instant",
        }
    }

    /// The pace `word` names, whole.
    #[must_use]
    pub fn named(word: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|pace| pace.word() == word)
    }

    /// Characters a second, or `None` for no reveal.
    ///
    /// A baud is a bit a second and a character ten of them on the wire.
    pub(super) const fn characters(self) -> Option<f32> {
        match self {
            Self::Baud1200 => Some(120.0),
            Self::Baud4800 => Some(480.0),
            Self::Baud9600 => Some(960.0),
            Self::Instant => None,
        }
    }
}
