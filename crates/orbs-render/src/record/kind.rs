//! What a record *is* — and, from that, which of them the eldritch renderer may
//! touch.
//!
//! # The exemption
//!
//! DESIGN.md §3: *"Script listings, schedule listings, and log output are
//! corruption-exempt surfaces. The renderer never applies eldritch presentation
//! to them. These are the diagnostic surfaces; they must always be trustworthy
//! as **renderings**, even when their contents are not."*
//!
//! That last clause is the whole rule, and it cuts in two directions that are
//! easy to collapse into one:
//!
//! | Treatment | On a diagnostic surface | Why |
//! |---|---|---|
//! | [`Presentation::Eldritch`] | **never** | It is tonal. A player auditing a log at peak threat must be able to believe their eyes |
//! | [`Presentation::Tampered`] | **always allowed** | It is diagnostic. §8.1's structural tell *is* the sabotage on that surface, and suppressing it would delete the signal |
//!
//! §3 requires the two vocabularies be disjoint, and this asymmetry is what
//! disjointness buys: the tonal system cannot jam the diagnostic system on the
//! exact surfaces used to diagnose.
//!
//! Enforcement lives in [`RecordKind::allows`], and the only presentation
//! accessor on a record ([`Record::presentation`](super::Record::presentation))
//! is filtered through it. There is no unfiltered path, so no call site can get
//! this wrong.

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
    /// `[reagent]`, `[place]` — the shape a `.toml` table header has, because
    /// that is the shape a player who has seen one config file already knows.
    ///
    /// **It stacks while the rows tile.** A heading packed into a column beside
    /// the things it heads would head nothing; the whole point is that it owns
    /// its row and the entries fill in underneath.
    ///
    /// It also carries the `kind` that used to sit on every row. `sage reagent`
    /// repeated the word once per line for no gain, and the repetition is what
    /// made a listing read as a wall rather than as a table.
    Section,
    /// One row of a filesystem listing.
    Entry,
    /// One line of a log. **Diagnostic surface** (§3).
    LogLine,
    /// One line of a script listing. **Diagnostic surface** (§3).
    ScriptLine,
    /// One row of the schedule. **Diagnostic surface** (§3).
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
    /// Exactly the three the design names, and no more. `ls` output reads like a
    /// diagnostic surface and is deliberately *not* one here — extending the
    /// exemption is a design decision for §19, not an inference to make in a
    /// match arm.
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
    /// A listing is a **set**: the reader wants to find one name in it, and the
    /// order between two of them carries nothing. Stacked one per line, ten
    /// belongings eat ten rows of an 80-column pane to use eighteen of its
    /// columns — and the boot report, which is sixteen such rows, pushed its own
    /// first line off the top of the screen before anyone had typed anything.
    ///
    /// Every other kind is a **sequence**. A log line, a script line and a
    /// schedule row are read in order and their neighbours are their context;
    /// tiling those would scramble the thing being read. A status row is a
    /// `label: value` pair that wants its column to line up with the pair above
    /// it. So exactly one kind tiles, and it is the one whose rows are unordered.
    ///
    /// This costs a screen-reader nothing: [`RecordView`](super::RecordView)
    /// speaks each record separately and in stream order, so the linear form is
    /// identical whether or not the visual one wraps.
    #[must_use]
    pub const fn tiles(self) -> bool {
        matches!(self, Self::Entry)
    }

    /// The glyph a prompt draws in front of a record that named no
    /// [`Outcome`](super::Outcome).
    ///
    /// Only a completion has one. A duration action finishing is the one event
    /// the player did not just cause — §14 announces completions and suppresses
    /// progress for the same reason — and without a mark it reads as another
    /// line of echo rather than as the world answering.
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
    /// A total function rather than an argument at each emit site. Every record
    /// of a kind linearises the same way by construction, which is what stops
    /// the stream from depending on which command happened to build the row.
    #[must_use]
    pub const fn utterance(self) -> UtteranceKind {
        match self {
            // Fielded rows. §14: a reader must never have to reconstruct
            // columns from spacing, so these speak as `label: value`.
            Self::Entry | Self::LogLine | Self::Schedule | Self::Status => UtteranceKind::TableRow,
            // A heading, because that is what it is: §14 gives a reader the
            // same grouping the columns give everyone else.
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
        // The asymmetry is the point. A log line is exactly where a player looks
        // for tampering; suppressing the tell there would delete §8.1's visual
        // channel on the surface it matters most.
        for kind in RecordKind::ALL {
            assert!(kind.allows(Presentation::Tampered), "{kind:?}");
            assert!(kind.allows(Presentation::Plain), "{kind:?}");
        }
    }

    #[test]
    fn only_unordered_rows_tile() {
        // Tiling reorders rows across a line, so it is safe exactly where order
        // carries nothing. The three diagnostic surfaces are read in sequence
        // and a status row wants its value column aligned with the one above.
        for kind in RecordKind::ALL {
            assert_eq!(kind.tiles(), kind == RecordKind::Entry, "{kind:?}");
            assert!(!(kind.tiles() && kind.is_diagnostic()), "{kind:?}");
        }
    }

    #[test]
    fn completions_linearise_as_completions() {
        // §14 announces completions and suppresses progress. That filter works
        // off the utterance kind, so this mapping is load-bearing.
        assert_eq!(
            RecordKind::Completion.utterance(),
            UtteranceKind::Completion
        );
        assert_eq!(RecordKind::Echo.utterance(), UtteranceKind::Echo);
        assert_eq!(RecordKind::Input.utterance(), UtteranceKind::Input);
    }
}
