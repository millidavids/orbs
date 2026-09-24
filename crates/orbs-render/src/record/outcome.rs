//! How a command concluded.
//!
//! Output vocabulary, not parser vocabulary — every command concludes somehow.
//! It lives here for the same reason [`RecordKind`](super::RecordKind) and
//! [`FieldName`](super::FieldName) do: a view that had to recognise string
//! literals defined in `orbs-sim` would be reaching across the boundary.
//!
//! All six carry one canonical command form in
//! [`FieldName::Message`](super::FieldName::Message), so as *text* they are
//! identical; what differs is what the player is asked to do.
//! [`Outcome::Candidate`] is selectable and §6 has the player answer with a
//! number; [`Outcome::Suggestion`] is not, and numbering it would promise an
//! interaction that does nothing; [`Outcome::Forced`] needs a correction
//! affordance where [`Outcome::Resolved`] does not. Drawn without the
//! distinction, an ambiguous input reads as the parser malfunctioning.

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
    /// One of several tied readings. Selectable.
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
    /// an [annotation](super::FieldName::is_annotation), never spoken or drawn.
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
    /// marker has to be readable at a glance during a siege. All six are in the
    /// CP437 repertoire, which a test asserts.
    ///
    /// `→` this is what runs, `≈` approximately this — correct me, `¿` I need
    /// one more thing, `»` pick one of these, `!` I do not know this, `·` you
    /// could try.
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
    /// The second channel, because §14 forbids any single channel carrying
    /// meaning alone. A selectable prompt is brightest because it is the only
    /// one waiting on the player; an offer is dimmest.
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
        // A marker outside CP437 draws nothing, and the line loses its class.
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
        // Numbering anything but a tie promises an interaction that does
        // nothing (§6).
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
        // The one waiting on the player must not be dimmer than the safe one.
        assert!(Outcome::Candidate.intensity() > Outcome::Suggestion.intensity());
        assert!(Outcome::Resolved.intensity() > Outcome::Suggestion.intensity());
    }
}
