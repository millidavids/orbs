//! The addressable parts of a record: what a field is called, and what it holds.
//!
//! Fields are named from a **closed set**. DESIGN.md §6 fuzzy-resolves player
//! input against closed vocabularies — verbs, nouns, registers — and a field name
//! is the same kind of thing the moment `sift --field stat feed.log` has to mean
//! something. A closed enum also makes a missed case a compile error in every
//! view, which is the point: a table that silently skips a column it does not
//! recognise is worse than one that will not build.

/// The name of one field of a record.
///
/// Deliberately **not** `#[non_exhaustive]`, for the same reason as
/// [`Role`](crate::Role): every consumer is in this workspace, and exhaustive
/// matching is what makes adding a field a reviewed act rather than a silent one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FieldName {
    /// What the thing is called. `nightshade`, `feed.log`, `north_gate`.
    Name,
    /// Where it lives. `/laboratory/reagents/nightshade`.
    Path,
    /// What sort of thing it is. `reagent`, `potion`, `script`, `directory`.
    Kind,
    /// Its condition. `ready`, `brewing`, `spoiled`, `bound`.
    State,
    /// How many. A number, kept as a number — see [`Value`].
    Quantity,
    /// When, in world time (DESIGN.md §5.0). Ticks, not wall-clock.
    Tick,
    /// Ticks left on a duration action.
    Remaining,
    /// Which subsystem produced this. `laboratory`, `battlements`, `lens`.
    ///
    /// **A domain, not a place.** `read_file` filters a domain's log by this, so
    /// putting a *room* in it silently drops the record from the log of the
    /// domain it happened in. Where a thing came from is [`FieldName::Origin`].
    Source,
    /// Where a thing was before it moved. `dispensary`, `mortar_and_pestle`.
    ///
    /// Distinct from [`FieldName::Source`] precisely because that one is
    /// load-bearing for log filtering. §10.1's `move` needs to name both ends,
    /// and a record about the laboratory must still appear in the laboratory's
    /// log while saying it came from the dispensary.
    Origin,
    /// Free prose — a log line's text, the orb speaking.
    Message,
    /// Secondary prose, subordinate to [`FieldName::Message`].
    Detail,
    /// How the command that emitted this record concluded.
    ///
    /// The one **annotation** field: written for views and machines, never for a
    /// reader. See [`FieldName::is_annotation`].
    Outcome,
    /// Which numbered option this is, when the orb is asking the player to pick.
    ///
    /// DESIGN.md §6's disambiguation prompt is numbered and the player answers
    /// with a digit, so the number is a *fact about the option* rather than a
    /// position a view invents — a view cannot count reliably anyway, because it
    /// draws a window onto a scrollback and may not hold the whole list.
    ///
    /// Not [`FieldName::Quantity`]: an ordinal is not an amount, and a `qty`
    /// column reading `1` for the first of two would be a lie the closed field
    /// set exists to prevent.
    Choice,
}

impl FieldName {
    /// Every field name, in declaration order.
    pub const ALL: [Self; 13] = [
        Self::Name,
        Self::Path,
        Self::Kind,
        Self::State,
        Self::Quantity,
        Self::Tick,
        Self::Remaining,
        Self::Source,
        Self::Origin,
        Self::Message,
        Self::Detail,
        Self::Outcome,
        Self::Choice,
    ];

    /// The word a player sees as a column header and hears in a linearised row.
    ///
    /// Lower case throughout: the tower's whole surface is a terminal, and a
    /// capitalised header would be the only thing on screen shouting.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Path => "path",
            Self::Kind => "kind",
            Self::State => "state",
            Self::Quantity => "qty",
            Self::Tick => "tick",
            Self::Remaining => "left",
            Self::Source => "source",
            Self::Origin => "from",
            Self::Message => "message",
            Self::Detail => "detail",
            Self::Outcome => "outcome",
            Self::Choice => "choice",
        }
    }

    /// Whether this field is written for views and machines rather than for a
    /// player.
    ///
    /// The same split [`Painter`](crate::Painter) already draws one layer up,
    /// where content speaks and structure does not. An annotation classifies a
    /// record so a view can decide how to draw it — is this prompt selectable,
    /// does this echo need a correction affordance — and speaking it would read
    /// an internal token aloud.
    ///
    /// [`Record::speak`](crate::Record::speak) skips annotations.
    /// [`Record::fields`](crate::Record::fields) does not: a view that names one
    /// as a column asked for it, and `sift` searching them is how a future
    /// pipeline filters on outcome.
    #[must_use]
    pub const fn is_annotation(self) -> bool {
        matches!(self, Self::Outcome)
    }

    /// Resolve a label back to its field, exactly.
    ///
    /// Exact only. Fuzzy resolution belongs to the parser, which already owns
    /// the scoring, the thresholds, and the echo that tells a player what their
    /// word became (§6); a second, quieter fuzzy matcher here would produce
    /// corrections nothing ever echoes.
    #[must_use]
    pub fn from_label(label: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|name| name.label() == label)
    }
}

/// What a field holds.
///
/// Numbers stay numbers. A quantity rendered to `"3"` at emit time cannot be
/// right-aligned by a table, compared by a future `sift --above`, or summed by
/// the balance harness without being parsed back out of its own presentation —
/// which is the exact inversion architectural rule 4 exists to prevent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Value<'a> {
    /// Text, as emitted.
    Text(&'a str),
    /// A count of things.
    Count(u64),
    /// A point in world time, in ticks.
    Tick(u64),
}

impl Value<'_> {
    /// Call `f` with this value's canonical text form.
    ///
    /// The single place a value becomes characters, so drawing it, measuring it,
    /// and matching it against a pattern can never disagree. Numbers format onto
    /// the stack, because measuring a table means touching every value of every
    /// row on every frame.
    pub fn with_str<R>(&self, f: impl FnOnce(&str) -> R) -> R {
        match *self {
            Self::Text(text) => f(text),
            Self::Count(number) | Self::Tick(number) => f(Decimal::new(number).as_str()),
        }
    }

    /// Append the canonical text form to `out`.
    pub fn write(&self, out: &mut String) {
        self.with_str(|text| out.push_str(text));
    }

    /// How many cells the canonical text form occupies.
    ///
    /// Characters, not bytes: the CP437 repertoire is one cell per glyph, and
    /// several of its glyphs — `░`, `é`, the box-drawing set — are multi-byte in
    /// UTF-8 (see [`cp437`](crate::cp437)).
    #[must_use]
    pub fn width(&self) -> usize {
        self.with_str(|text| text.chars().count())
    }

    /// Whether a view should right-align this value in a column.
    #[must_use]
    pub const fn is_numeric(&self) -> bool {
        matches!(*self, Self::Count(_) | Self::Tick(_))
    }
}

/// A `u64` rendered onto the stack.
struct Decimal {
    digits: [u8; Self::MAX],
    start: usize,
}

impl Decimal {
    /// `u64::MAX` is 20 digits.
    const MAX: usize = 20;

    fn new(value: u64) -> Self {
        let mut digits = [b'0'; Self::MAX];
        let mut start = Self::MAX;
        let mut rest = value;
        loop {
            start -= 1;
            // `rest % 10` is 0..=9, so the fallback is unreachable.
            digits[start] = b'0' + u8::try_from(rest % 10).unwrap_or(9);
            rest /= 10;
            if rest == 0 {
                break;
            }
        }
        Self { digits, start }
    }

    fn as_str(&self) -> &str {
        // Every byte written is an ASCII digit.
        core::str::from_utf8(&self.digits[self.start..]).unwrap_or("?")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_are_distinct() {
        // Two fields sharing a label would make `from_label` — and a column
        // header — ambiguous.
        for (index, name) in FieldName::ALL.into_iter().enumerate() {
            for other in FieldName::ALL.into_iter().skip(index + 1) {
                assert_ne!(name.label(), other.label(), "{name:?} and {other:?}");
            }
        }
    }

    #[test]
    fn every_label_resolves_back_to_its_field() {
        for name in FieldName::ALL {
            assert_eq!(FieldName::from_label(name.label()), Some(name));
        }
        assert_eq!(FieldName::from_label("nonsense"), None);
    }

    #[test]
    fn numbers_render_without_allocating_a_wrong_answer() {
        for number in [0, 1, 9, 10, 99, 1_000, u64::MAX] {
            let rendered = Value::Count(number).with_str(str::to_owned);
            assert_eq!(rendered, number.to_string());
            assert_eq!(Value::Count(number).width(), rendered.len());
        }
    }

    #[test]
    fn width_counts_cells_not_bytes() {
        // `░` is three bytes and one cell. A table measuring bytes would leave a
        // two-cell hole in every column containing one.
        assert_eq!(Value::Text("░░░").width(), 3);
        assert_eq!(Value::Text("ready").width(), 5);
    }

    #[test]
    fn only_numbers_right_align() {
        assert!(Value::Count(3).is_numeric());
        assert!(Value::Tick(1247).is_numeric());
        assert!(!Value::Text("3").is_numeric());
    }
}
