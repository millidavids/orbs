//! What a record *is* — and, from that, which of them the eldritch renderer may
//! touch.
//!
//! DESIGN.md §3 makes script listings, schedule listings and log output
//! corruption-exempt: they must be trustworthy as *renderings* even when their
//! contents are not. That cuts two ways. [`Presentation::Eldritch`] is tonal
//! and never reaches them — a player auditing a log at peak threat must be able
//! to believe their eyes. [`Presentation::Tampered`] is diagnostic and always
//! allowed: §8.1's structural tell *is* the sabotage on that surface.
//!
//! Enforcement is in [`RecordKind::allows`], and the only presentation accessor
//! ([`Record::presentation`](super::Record::presentation)) is filtered through
//! it, so no call site can get it wrong.

use crate::linear::UtteranceKind;
use crate::style::Presentation;

/// What a record is, which decides how it linearises and whether the eldritch
/// register may touch it.
///
/// Not `#[non_exhaustive]`, for the reason given on [`Role`](crate::Role).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RecordKind {
    /// What the rows under it are, in a listing that groups them.
    ///
    /// `[reagent]`, `[place]` — the shape a `.toml` table header has, which a
    /// player who has seen one config file already knows.
    ///
    /// It stacks while the rows tile: a heading packed into a column beside the
    /// things it heads would head nothing. It also carries the `kind` that used
    /// to sit on every row, whose repetition made a listing read as a wall.
    Section,
    /// One row of a filesystem listing.
    Entry,
    /// One line of a log. Diagnostic surface (§3).
    LogLine,
    /// One line of a script listing. Diagnostic surface (§3).
    ScriptLine,
    /// One row of the schedule. Diagnostic surface (§3).
    Schedule,
    /// One labelled reading of world state — a status row, a meter's numbers.
    Status,
    /// The orb speaking. The register the eldritch treatment exists for.
    #[default]
    Message,
    /// A duration action finishing (§14 announces completions, never progress).
    Completion,
    /// The parser's canonical restatement of what the player meant (§6).
    Echo,
    /// What the player typed.
    Input,
}

impl RecordKind {
    /// Every kind, in declaration order.
    pub const ALL: [Self; 10] = [
        Self::Section,
        Self::Entry,
        Self::LogLine,
        Self::ScriptLine,
        Self::Schedule,
        Self::Status,
        Self::Message,
        Self::Completion,
        Self::Echo,
        Self::Input,
    ];

    /// Whether this record sits on one of §3's corruption-exempt surfaces.
    ///
    /// Exactly the three the design names. `ls` output reads like one and
    /// deliberately is not — extending the exemption is a §19 decision, not an
    /// inference to make in a match arm.
    #[must_use]
    pub const fn is_diagnostic(self) -> bool {
        matches!(self, Self::LogLine | Self::ScriptLine | Self::Schedule)
    }

    /// Whether `presentation` is permitted on this kind.
    ///
    /// See the module docs for why [`Presentation::Tampered`] survives the
    /// exemption that [`Presentation::Eldritch`] does not.
    #[must_use]
    pub const fn allows(self, presentation: Presentation) -> bool {
        match presentation {
            Presentation::Plain | Presentation::Tampered => true,
            Presentation::Eldritch => !self.is_diagnostic(),
        }
    }

    /// Whether a run of these packs across the pane instead of stacking down it.
    ///
    /// A listing is a set: the reader wants one name in it and the order
    /// carries nothing. Stacked one per line, ten belongings eat ten rows of an
    /// 80-column pane to use eighteen columns, and the boot report pushed its
    /// own first line off the screen. Every other kind is a sequence — a log
    /// line's neighbours are its context, a status row wants its column aligned
    /// with the pair above — so exactly one kind tiles.
    ///
    /// This costs a screen reader nothing: [`RecordView`](super::RecordView)
    /// speaks each record separately and in stream order.
    #[must_use]
    pub const fn tiles(self) -> bool {
        matches!(self, Self::Entry)
    }

    /// The glyph a prompt draws in front of a record that named no
    /// [`Outcome`](super::Outcome).
    ///
    /// Only a completion has one: a duration action finishing is the one event
    /// the player did not just cause, and unmarked it reads as more echo.
    #[must_use]
    pub const fn marker(self) -> Option<char> {
        match self {
            Self::Completion => Some('√'),
            Self::Section
            | Self::Entry
            | Self::LogLine
            | Self::ScriptLine
            | Self::Schedule
            | Self::Status
            | Self::Message
            | Self::Echo
            | Self::Input => None,
        }
    }

    /// How this record enters the screen-reader stream.
    ///
    /// A total function rather than an argument at each emit site, so the
    /// stream cannot depend on which command happened to build the row.
    #[must_use]
    pub const fn utterance(self) -> UtteranceKind {
        match self {
            // Fielded rows: §14 says a reader must never reconstruct columns
            // from spacing, so these speak as `label: value`.
            Self::Entry | Self::LogLine | Self::Schedule | Self::Status => UtteranceKind::TableRow,
            // A heading: §14 gives a reader the grouping the columns give.
            Self::Section => UtteranceKind::Heading,
            Self::ScriptLine | Self::Message => UtteranceKind::Text,
            Self::Completion => UtteranceKind::Completion,
            Self::Echo => UtteranceKind::Echo,
            Self::Input => UtteranceKind::Input,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_design_names_three_diagnostic_surfaces() {
        let diagnostic: Vec<_> = RecordKind::ALL
            .into_iter()
            .filter(|kind| kind.is_diagnostic())
            .collect();
        assert_eq!(
            diagnostic,
            [
                RecordKind::LogLine,
                RecordKind::ScriptLine,
                RecordKind::Schedule
            ],
        );
    }

    #[test]
    fn eldritch_never_reaches_a_diagnostic_surface() {
        for kind in RecordKind::ALL {
            assert_eq!(
                kind.allows(Presentation::Eldritch),
                !kind.is_diagnostic(),
                "{kind:?}",
            );
        }
    }

    #[test]
    fn sabotage_tells_survive_the_exemption() {
        // A log line is exactly where a player looks for tampering; suppressing
        // the tell there deletes §8.1's channel where it matters most.
        for kind in RecordKind::ALL {
            assert!(kind.allows(Presentation::Tampered), "{kind:?}");
            assert!(kind.allows(Presentation::Plain), "{kind:?}");
        }
    }

    #[test]
    fn only_unordered_rows_tile() {
        // Tiling reorders rows across a line, so it is safe only where order
        // carries nothing.
        for kind in RecordKind::ALL {
            assert_eq!(kind.tiles(), kind == RecordKind::Entry, "{kind:?}");
            assert!(!(kind.tiles() && kind.is_diagnostic()), "{kind:?}");
        }
    }

    #[test]
    fn completions_linearise_as_completions() {
        // §14's completion filter works off the utterance kind.
        assert_eq!(
            RecordKind::Completion.utterance(),
            UtteranceKind::Completion
        );
        assert_eq!(RecordKind::Echo.utterance(), UtteranceKind::Echo);
        assert_eq!(RecordKind::Input.utterance(), UtteranceKind::Input);
    }
}
