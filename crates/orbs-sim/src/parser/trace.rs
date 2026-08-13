//! Instrumentation — what the Phase 0 gate actually measures.
//!
//! DESIGN.md §6: *"Every resolution reproducible — full input, resolution, and
//! candidate scores logged and exportable."* §15 sets the bar this feeds:
//!
//! > ≥ 85% of inputs resolve to the intended action on first attempt, **and**
//! > ≥ 95% of initially-unresolved inputs reach the intended action within two
//! > further attempts, with zero dead ends.
//! >
//! > Act on the per-input failure *clustering*, not the aggregate.
//!
//! Clustering is why every candidate is kept and not just the winner: a miss
//! caused by an unknown verb and a miss caused by an argument that did not exist
//! look identical in an aggregate and need opposite fixes.
//!
//! # Why TSV
//!
//! The consumer is a spreadsheet or a five-line script, run once per playtest by
//! one person. TSV needs no dependency, survives `grep`, and pastes into
//! anything. Records carry no timing — wall-clock in a sim record would make two
//! runs of the same seed differ.

use core::fmt::Write as _;

use bevy_ecs::prelude::*;

use super::intent::{Candidate, Mode, Resolution};
use super::resolve::Analysis;
use super::verb::Verb;
use super::vocabulary::Register;

/// What became of one line of input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// A command ran, chosen outright.
    Resolved,
    /// A command ran, chosen under siege pressure with correction offered.
    Forced,
    /// The player was asked which of several readings they meant.
    Ambiguous,
    /// The verb was understood but a free-text or numeric argument was absent.
    Incomplete,
    /// Nothing scored; suggestions were offered.
    Unresolved,
}

impl Outcome {
    const fn label(self) -> &'static str {
        match self {
            Self::Resolved => "resolved",
            Self::Forced => "forced",
            Self::Ambiguous => "ambiguous",
            Self::Incomplete => "incomplete",
            Self::Unresolved => "unresolved",
        }
    }
}

/// One resolution, in full.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseRecord {
    /// World time when the input arrived.
    pub tick: u64,
    /// Exactly what the player typed, before normalisation.
    pub input: String,
    /// Whether a blocking prompt was permitted.
    pub mode: Mode,
    /// What became of it.
    pub outcome: Outcome,
    /// The canonical echo, if a command was chosen.
    pub echo: Option<String>,
    /// Which dialect the player reached for, if one was identified.
    pub register: Option<Register>,
    /// Every scored reading, best first. The raw material for clustering.
    pub candidates: Vec<Candidate>,
    /// Verbs offered when nothing resolved.
    pub suggestions: Vec<Verb>,
}

impl ParseRecord {
    /// Record a resolution, keeping every reading that was scored.
    ///
    /// Takes an [`Analysis`] rather than a [`Resolution`] because §6 wants the
    /// candidate scores for *successful* resolutions too — a command that won by
    /// four points and one that won by four hundred are the same `Resolved` and
    /// very different data.
    #[must_use]
    pub fn new(tick: u64, input: &str, mode: Mode, analysis: &Analysis) -> Self {
        let (outcome, echo, register, suggestions) = match &analysis.resolution {
            Resolution::Resolved { intent, confidence } => (
                match confidence {
                    super::intent::Confidence::Clear => Outcome::Resolved,
                    super::intent::Confidence::Forced => Outcome::Forced,
                },
                Some(intent.echo()),
                Some(intent.register),
                Vec::new(),
            ),
            Resolution::Ambiguous { candidates } => (
                Outcome::Ambiguous,
                None,
                candidates.first().map(|c| c.intent.register),
                Vec::new(),
            ),
            Resolution::Incomplete {
                verb,
                register,
                missing,
                filled,
            } => {
                // The echo shows how far the parser got, so the gate can tell
                // "did not know the verb" from "knew the verb, wanted an
                // argument" — opposite fixes, identical in an aggregate.
                let mut echo = String::from(verb.canonical());
                for argument in filled {
                    echo.push(' ');
                    echo.push_str(&argument.value);
                }
                echo.push_str(&format!(" <{missing:?}?>"));
                (Outcome::Incomplete, Some(echo), Some(*register), Vec::new())
            }
            // Traced as unresolved, because for §15's gate it *is* a line the
            // player typed that ran nothing. The echo names the verb, so a
            // cluster of these reads as "they tried to brew in the archive"
            // rather than as a parser failure — which is a design signal about
            // where the domains sit, not a phrasing one.
            Resolution::Elsewhere { verb } => (
                Outcome::Unresolved,
                Some(verb.canonical().to_owned()),
                None,
                Vec::new(),
            ),
            Resolution::InSpell { word } => (
                Outcome::Unresolved,
                Some(word.canonical().to_owned()),
                None,
                Vec::new(),
            ),
            Resolution::Unresolved { suggestions } => {
                (Outcome::Unresolved, None, None, suggestions.clone())
            }
        };
        let candidates = analysis.candidates.clone();

        Self {
            tick,
            input: input.to_owned(),
            mode,
            outcome,
            echo,
            register,
            candidates,
            suggestions,
        }
    }
}

/// Every resolution this session, in order.
///
/// A `Resource` because it is world-adjacent bookkeeping the sim owns: §6 makes
/// explaining itself part of the parser's contract, and the Phase 0 gate acts on
/// the *clustering* of failures rather than on the aggregate, which means every
/// scored reading has to be kept as it happens. Reconstructing it afterwards
/// from the scrollback is not possible — the losing candidates are gone.
#[derive(Resource, Debug, Default, Clone)]
pub struct ParseLog {
    records: Vec<ParseRecord>,
}

impl ParseLog {
    /// An empty log.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a record.
    pub fn push(&mut self, record: ParseRecord) {
        self.records.push(record);
    }

    /// Everything recorded.
    #[must_use]
    pub fn records(&self) -> &[ParseRecord] {
        &self.records
    }

    /// How many inputs resolved on the first attempt.
    ///
    /// The numerator of the gate's headline number. Forced resolutions count —
    /// they ran the intended command — but they are reported separately by
    /// [`ParseLog::forced`] because a high forced rate means siege play is
    /// guessing.
    #[must_use]
    pub fn resolved(&self) -> usize {
        self.records
            .iter()
            .filter(|record| matches!(record.outcome, Outcome::Resolved | Outcome::Forced))
            .count()
    }

    /// How many resolutions were taken under siege pressure.
    #[must_use]
    pub fn forced(&self) -> usize {
        self.count(Outcome::Forced)
    }

    /// How many inputs produced a numbered prompt.
    #[must_use]
    pub fn ambiguous(&self) -> usize {
        self.count(Outcome::Ambiguous)
    }

    /// How many inputs named a verb but left a free-text slot empty.
    #[must_use]
    pub fn incomplete(&self) -> usize {
        self.count(Outcome::Incomplete)
    }

    /// How many inputs scored nothing.
    #[must_use]
    pub fn unresolved(&self) -> usize {
        self.count(Outcome::Unresolved)
    }

    fn count(&self, outcome: Outcome) -> usize {
        self.records
            .iter()
            .filter(|record| record.outcome == outcome)
            .count()
    }

    /// Every record as tab-separated rows, with a header.
    ///
    /// One row per *candidate*, so a spreadsheet can pivot on why a reading lost
    /// rather than only on whether it won. Inputs with no candidates still get a
    /// row, or unresolved inputs — the ones the gate cares about most — would
    /// vanish from the export.
    #[must_use]
    pub fn to_tsv(&self) -> String {
        let mut out = String::from(
            "tick\tinput\tmode\toutcome\techo\tregister\trank\tcandidate\tscore\tverb_score\targ_score\tsuggestions\n",
        );

        for record in &self.records {
            let mode = match record.mode {
                Mode::Calm => "calm",
                Mode::Siege => "siege",
            };
            // Escaped like every other field: argument values come from scene
            // noun names, which from Phase 1 are player-authored script and file
            // names. One tab in one of those would add a column to the row.
            let echo = escape(record.echo.as_deref().unwrap_or(""));
            let register = record.register.map_or("", register_label);
            let suggestions = record
                .suggestions
                .iter()
                .map(|verb| verb.canonical())
                .collect::<Vec<_>>()
                .join(",");

            if record.candidates.is_empty() {
                let _ = writeln!(
                    out,
                    "{}\t{}\t{mode}\t{}\t{echo}\t{register}\t\t\t\t\t\t{suggestions}",
                    record.tick,
                    escape(&record.input),
                    record.outcome.label(),
                );
                continue;
            }

            for (rank, candidate) in record.candidates.iter().enumerate() {
                let _ = writeln!(
                    out,
                    "{}\t{}\t{mode}\t{}\t{echo}\t{register}\t{rank}\t{}\t{}\t{}\t{}\t{suggestions}",
                    record.tick,
                    escape(&record.input),
                    record.outcome.label(),
                    escape(&candidate.intent.echo()),
                    candidate.score,
                    candidate.verb_score,
                    candidate.argument_score,
                );
            }
        }
        out
    }
}

const fn register_label(register: Register) -> &'static str {
    match register {
        Register::Arcane => "arcane",
        Register::Shell => "shell",
        Register::Plain => "plain",
    }
}

/// Keep one record on one row. Players type tabs and newlines by accident.
fn escape(field: &str) -> String {
    field.replace(['\t', '\n', '\r'], " ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{NounKind, Scene, analyse};

    fn tower() -> Scene {
        // Recipes are `Topic` nouns (§6.1), which is what a bare `brew` now
        // enumerates — `decoct` is retired and its words point at the manual.
        Scene::new()
            .with(NounKind::Topic, "clarity")
            .with(NounKind::Topic, "warding")
    }

    fn log_of(inputs: &[&str], mode: Mode) -> ParseLog {
        let scene = tower();
        let mut log = ParseLog::new();
        for (tick, input) in inputs.iter().enumerate() {
            let analysis = analyse(input, &scene, mode);
            let tick = u64::try_from(tick).expect("small");
            log.push(ParseRecord::new(tick, input, mode, &analysis));
        }
        log
    }

    #[test]
    fn outcomes_are_tallied_for_the_gate() {
        let log = log_of(&["recall clarity", "brew", "xyzzy"], Mode::Calm);
        assert_eq!(log.resolved(), 1);
        assert_eq!(log.ambiguous(), 1);
        assert_eq!(log.unresolved(), 1);
    }

    #[test]
    fn forced_resolutions_are_counted_separately() {
        // A high forced rate means siege play is guessing, which the aggregate
        // "resolved" number would hide.
        let log = log_of(&["brew"], Mode::Siege);
        assert_eq!(log.resolved(), 1);
        assert_eq!(log.forced(), 1);
    }

    #[test]
    fn every_candidate_survives_into_the_export() {
        // Clustering needs the losers, not just the winner.
        let log = log_of(&["brew"], Mode::Calm);
        let tsv = log.to_tsv();
        assert!(tsv.contains("recall clarity"), "{tsv}");
        assert!(tsv.contains("recall warding"), "{tsv}");
    }

    #[test]
    fn unresolved_inputs_still_appear() {
        // These are the rows the gate cares about most; dropping them would
        // silently flatter the numbers.
        let tsv = log_of(&["xyzzy"], Mode::Calm).to_tsv();
        assert!(tsv.contains("xyzzy"), "{tsv}");
        assert!(tsv.contains("unresolved"), "{tsv}");
    }

    #[test]
    fn the_export_is_rectangular() {
        let tsv = log_of(&["recall clarity", "brew", "xyzzy"], Mode::Calm).to_tsv();
        let mut lines = tsv.lines();
        let columns = lines.next().expect("header").split('\t').count();
        for line in lines {
            assert_eq!(line.split('\t').count(), columns, "ragged row: {line:?}");
        }
    }

    #[test]
    fn tabs_and_newlines_in_input_cannot_break_a_row() {
        let scene = tower();
        let input = "decoct\tclarity\nrm -rf";
        let analysis = analyse(input, &scene, Mode::Calm);
        let mut log = ParseLog::new();
        log.push(ParseRecord::new(0, input, Mode::Calm, &analysis));

        // **Every row has the header's shape**, which is the property — not the
        // number of rows. Counting them made this a test of how many candidates
        // the parser happened to find, so it broke the day a verb stopped taking
        // an argument and could complete on its own.
        let tsv = log.to_tsv();
        let mut lines = tsv.lines();
        let columns = lines.next().expect("no header").matches('\t').count();
        for line in lines {
            assert_eq!(
                line.matches('\t').count(),
                columns,
                "a row broke the shape: {line:?}",
            );
        }
    }
}
