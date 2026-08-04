//! A run of content: what is drawn, what it means, and what is spoken.
//!
//! The visual text and the spoken text travel together to the same call site.
//! That coupling is the whole point, and it is why this exists instead of a
//! plain `&str` argument: DESIGN.md §3 requires that *every* eldritch message
//! carry an authored linear variant, and a separate "and now register the
//! spoken form" call would be forgotten exactly on the lines that need it. Once
//! a few hundred call sites exist, adding the second argument stops being
//! feasible — which is what makes this a Phase 0 decision rather than a later
//! one.

use crate::linear::UtteranceKind;
use crate::record::Outcome;
use crate::style::Style;

/// A run of text with its semantic style and its spoken form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span<'a> {
    text: &'a str,
    spoken: Option<&'a str>,
    style: Style,
    kind: UtteranceKind,
    outcome: Option<Outcome>,
}

impl<'a> Span<'a> {
    /// Body text, spoken exactly as it is drawn.
    #[must_use]
    pub const fn new(text: &'a str) -> Self {
        Self {
            text,
            spoken: None,
            style: Style::NORMAL,
            kind: UtteranceKind::Text,
            outcome: None,
        }
    }

    /// This span with a semantic style.
    #[must_use]
    pub const fn with_style(self, style: Style) -> Self {
        Self { style, ..self }
    }

    /// This span with an authored spoken form that differs from the drawn text.
    ///
    /// Required whenever the drawn form is not a sentence: eldritch text whose
    /// effect is carried by damaged presentation, an abbreviation, a bar, a
    /// column of a table.
    #[must_use]
    pub const fn with_spoken(self, spoken: &'a str) -> Self {
        Self {
            spoken: Some(spoken),
            ..self
        }
    }

    /// This span reclassified for the linear stream.
    #[must_use]
    pub const fn with_kind(self, kind: UtteranceKind) -> Self {
        Self { kind, ..self }
    }

    /// This span tagged with how its command concluded.
    ///
    /// The prompt draws that as a marker glyph and a brightness. A listener has
    /// neither channel, so it travels here too — the same coupling that makes
    /// [`Span::with_spoken`] exist at the call site rather than as a second,
    /// forgettable call.
    #[must_use]
    pub const fn with_outcome(self, outcome: Outcome) -> Self {
        Self {
            outcome: Some(outcome),
            ..self
        }
    }

    /// How the command that produced this concluded, if it said.
    #[must_use]
    pub const fn outcome(&self) -> Option<Outcome> {
        self.outcome
    }

    /// The text as drawn.
    #[must_use]
    pub const fn text(&self) -> &'a str {
        self.text
    }

    /// The text as spoken — the authored variant if there is one, otherwise the
    /// drawn text.
    #[must_use]
    pub const fn spoken_text(&self) -> &'a str {
        match self.spoken {
            Some(spoken) => spoken,
            None => self.text,
        }
    }

    /// Whether an authored spoken variant was supplied.
    #[must_use]
    pub const fn has_spoken_variant(&self) -> bool {
        self.spoken.is_some()
    }

    /// The semantic style.
    #[must_use]
    pub const fn style(&self) -> Style {
        self.style
    }

    /// How the linear stream classifies this span.
    #[must_use]
    pub const fn kind(&self) -> UtteranceKind {
        self.kind
    }
}

impl<'a> From<&'a str> for Span<'a> {
    fn from(text: &'a str) -> Self {
        Self::new(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::{Presentation, Role};

    #[test]
    fn spoken_defaults_to_the_drawn_text() {
        let span = Span::new("the ward holds");
        assert_eq!(span.spoken_text(), "the ward holds");
        assert!(!span.has_spoken_variant());
    }

    #[test]
    fn an_authored_variant_replaces_the_drawn_text() {
        let span = Span::new("t h e   d o o r   i s   o p e n")
            .with_style(Style::NORMAL.with_presentation(Presentation::Eldritch))
            .with_spoken("the door is open");

        assert_eq!(span.spoken_text(), "the door is open");
        assert!(span.has_spoken_variant());
    }

    #[test]
    fn builders_compose_in_any_order() {
        let a = Span::new("x")
            .with_style(Style::DANGER)
            .with_kind(UtteranceKind::Completion);
        let b = Span::new("x")
            .with_kind(UtteranceKind::Completion)
            .with_style(Style::DANGER);

        assert_eq!(a, b);
        assert_eq!(a.style().role, Role::Danger);
        assert_eq!(a.kind(), UtteranceKind::Completion);
    }
}
