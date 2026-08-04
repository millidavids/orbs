//! How a command concluded.
//!
//! An outcome is **output vocabulary**, not parser vocabulary — every command
//! concludes somehow, and the parser is merely the first thing that had to say
//! so. It lives here for the same reason [`RecordKind`](super::RecordKind) and
//! [`FieldName`](super::FieldName) do: a view has to be able to draw the
//! difference, and a view that had to recognise string literals defined in
//! `orbs-sim` would be reaching across the boundary to do it.
//!
//! # Why a view must be able to tell these apart
//!
//! All six carry one canonical command form in
//! [`FieldName::Message`](super::FieldName::Message), so as *text* they are
//! identical. What differs is what the player is being asked to do:
//!
//! - [`Outcome::Candidate`] is **selectable** — DESIGN.md §6 numbers the tied
//!   readings and the player answers with a number.
//! - [`Outcome::Suggestion`] is not. It is the orb offering somewhere to go,
//!   and numbering it would promise an interaction that does nothing.
//! - [`Outcome::Forced`] needs a correction affordance; [`Outcome::Resolved`]
//!   does not.
//!
//! Drawn without that distinction, an unresolved input renders as a column of
//! bare words and the first ambiguous phrase a player meets looks like the
//! parser malfunctioning.

use crate::style::Intensity;

/// How a command concluded.
///
/// Not `#[non_exhaustive]`, for the reason given on [`Role`](crate::Role).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Outcome {
    /// One reading won outright. This is what will run.
    Resolved,
    /// A siege took the best of several close readings rather than blocking on a
    /// prompt (§6), so the echo must offer correction.
    Forced,
    /// The verb is known and a required slot is empty.
    Incomplete,
    /// One of several tied readings. **Selectable.**
    Candidate,
    /// Nothing resolved.
    Unresolved,
    /// Somewhere to go. Not selectable.
    Suggestion,
}

impl Outcome {
    /// Every outcome, in declaration order.
    pub const ALL: [Self; 6] = [
        Self::Resolved,
        Self::Forced,
        Self::Incomplete,
        Self::Candidate,
        Self::Unresolved,
        Self::Suggestion,
    ];

    /// The value stored in [`FieldName::Outcome`](super::FieldName::Outcome).
    ///
    /// A record field holds text, so the enum round-trips through this. It is
    /// never spoken and never drawn — [`FieldName::Outcome`](super::FieldName::Outcome) is an annotation
    /// (see [`FieldName::is_annotation`](super::FieldName::is_annotation)).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Resolved => "resolved",
            Self::Forced => "forced",
            Self::Incomplete => "incomplete",
            Self::Candidate => "candidate",
            Self::Unresolved => "unresolved",
            Self::Suggestion => "suggestion",
        }
    }

    /// Recover the outcome from a stored field value.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.as_str() == value)
    }

    /// The glyph a prompt draws in front of the line.
    ///
    /// A glyph, not a word: prose belongs in content files (§12, rule 6), and a
    /// marker has to survive being read at a glance during a siege. All six are
    /// in the CP437 repertoire, which a test asserts.
    ///
    /// | Outcome | Marker | Reads as |
    /// |---|---|---|
    /// | [`Outcome::Resolved`] | `→` | this is what runs |
    /// | [`Outcome::Forced`] | `≈` | approximately this — correct me |
    /// | [`Outcome::Incomplete`] | `¿` | I need one more thing |
    /// | [`Outcome::Candidate`] | `»` | pick one of these |
    /// | [`Outcome::Unresolved`] | `!` | I do not know this |
    /// | [`Outcome::Suggestion`] | `·` | you could try |
    #[must_use]
    pub const fn marker(self) -> char {
        match self {
            Self::Resolved => '→',
            Self::Forced => '≈',
            Self::Incomplete => '¿',
            Self::Candidate => '»',
            Self::Unresolved => '!',
            Self::Suggestion => '·',
        }
    }

    /// How strongly the line is drawn.
    ///
    /// The second channel, because a marker is one glyph and §14 forbids any
    /// single channel carrying meaning alone. A selectable prompt is brightest
    /// because it is the only one waiting on the player; an offer is dimmest
    /// because ignoring it is the common case.
    #[must_use]
    pub const fn intensity(self) -> Intensity {
        match self {
            Self::Candidate => Intensity::Bright,
            Self::Resolved | Self::Forced | Self::Incomplete | Self::Unresolved => {
                Intensity::Normal
            }
            Self::Suggestion => Intensity::Dim,
        }
    }

    /// Whether §6 lets the player answer this with a number.
    #[must_use]
    pub const fn is_selectable(self) -> bool {
        matches!(self, Self::Candidate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cp437::is_renderable;

    #[test]
    fn every_marker_is_drawable() {
        // A marker outside the CP437 repertoire draws nothing at all, and the
        // line silently loses its only visual classification.
        for outcome in Outcome::ALL {
            assert!(
                is_renderable(outcome.marker()),
                "{outcome:?} marker {:?} is not in the repertoire",
                outcome.marker(),
            );
        }
    }

    #[test]
    fn markers_are_distinct() {
        for (index, outcome) in Outcome::ALL.into_iter().enumerate() {
            for other in Outcome::ALL.into_iter().skip(index + 1) {
                assert_ne!(outcome.marker(), other.marker(), "{outcome:?}/{other:?}");
            }
        }
    }

    #[test]
    fn every_outcome_round_trips_through_its_stored_form() {
        for outcome in Outcome::ALL {
            assert_eq!(Outcome::parse(outcome.as_str()), Some(outcome));
        }
        assert_eq!(Outcome::parse("nonsense"), None);
    }

    #[test]
    fn only_a_candidate_is_selectable() {
        // §6 numbers tied readings and has the player pick one. Numbering
        // anything else promises an interaction that does nothing.
        for outcome in Outcome::ALL {
            assert_eq!(
                outcome.is_selectable(),
                outcome == Outcome::Candidate,
                "{outcome:?}",
            );
        }
    }

    #[test]
    fn a_choice_outranks_an_offer() {
        // The one waiting on the player must not be dimmer than the one that is
        // safe to ignore.
        assert!(Outcome::Candidate.intensity() > Outcome::Suggestion.intensity());
        assert!(Outcome::Resolved.intensity() > Outcome::Suggestion.intensity());
    }
}
