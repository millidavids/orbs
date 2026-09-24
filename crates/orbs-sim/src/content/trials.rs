//! Lines a spell reader is tested on and never taught (§19, *the scrivener*).
//!
//! Authored in `content/spell_trials.toml`. This module knows the shape of a
//! trial and how one is scored; the lines themselves are data.
//!
//! Kept apart from the holdout in `spellings.toml`, which is worth less than it
//! looks: written beside the corpus by the same hand from the same templates,
//! so a reader that learned the *template* passes it. The trials are literal
//! lines with no `{slots}`, written to be unlike the corpus — contractions,
//! capitals, typos, short and unseen names, digits and number words, archaic
//! and terse registers. A test holds that none is a line anything was taught.
//!
//! A trial fails if the orb reads it wrongly, or leaves alone a line it should
//! have read. It *betrays* the player if the reading is one listed under
//! `never` — a dropped `not`, a dropped `or` clause, a changed number. Those
//! are scored apart, and the only acceptable count is zero.
//!
//! Like [`Phrasings`](super::Phrasings), nothing in a running tower reads this.
//! It is the fixture the scrivener is measured against.

use serde::Deserialize;

/// The compiled-in trials, so a measurement needs no filesystem.
const BUILTIN: &str = include_str!("../../content/spell_trials.toml");

/// The file's name, for an error a writer can act on.
const FILE: &str = "spell_trials.toml";

/// One line, and what the orb should make of it.
#[derive(Debug, Clone, Deserialize)]
pub struct Trial {
    /// What the player wrote.
    pub said: String,
    /// What the orb should hear. Absent means the line must come back exactly
    /// as written — a comment, a canonical line, a sentence asking nothing.
    #[serde(default)]
    pub reads: Option<String>,
    /// Whether leaving the line alone also passes.
    ///
    /// A spell runs with nobody watching, so a line left alone is recoverable
    /// where a line read wrongly is not.
    #[serde(default)]
    pub lenient: bool,
    /// Readings that must never come back — the ones that would run a spell
    /// meaning the opposite of what was written.
    #[serde(default)]
    pub never: Vec<String>,
    /// What the line is testing: exactly one shape, and any number of styles.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// A whole spell written loosely, and the spell it should become.
///
/// The scrivener reads a line at a time, so this tests the file rather than any
/// one line: comments and blanks kept, canonical lines untouched, and the whole
/// thing something the room can compile.
#[derive(Debug, Clone, Deserialize)]
pub struct Script {
    /// What to call it in a report.
    pub name: String,
    /// The room it is written for, which is what its names resolve against.
    pub domain: String,
    /// The spell as a player might write it.
    pub loose: Vec<String>,
    /// What each line should read as, line for line. A line the orb should
    /// leave alone is written here exactly as it is in [`loose`](Self::loose).
    pub reads: Vec<String>,
}

/// Every authored trial.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Trials {
    #[serde(default, rename = "trial")]
    lines: Vec<Trial>,
    #[serde(default, rename = "script")]
    scripts: Vec<Script>,
}

/// Every shape a trial can test — one per [`Trial`], exactly.
///
/// The spell register's eleven templates, plus `command`, `alone` for a line
/// that must come back untouched, and `hazard` for one whose wrong reading
/// would invert the spell.
pub const SHAPES: [&str; 14] = [
    "if-idle",
    "if-empty",
    "if-has",
    "else",
    "end",
    "repeat",
    "repeat-until",
    "bide",
    "let",
    "pull",
    "for-each",
    "command",
    "alone",
    "hazard",
];

/// Every style a trial can be tagged with, so a typo in a tag is an error rather
/// than a row of its own in the report.
pub const STYLES: [&str; 24] = [
    "plain",
    "terse",
    "casual",
    "archaic",
    "polite",
    "contraction",
    "caps",
    "punctuation",
    "typo",
    "digits",
    "number-words",
    "unseen-name",
    "short-name",
    "plural",
    "reordered",
    "negation",
    "question",
    "comment",
    "blank",
    "canonical",
    "nothing",
    "no-count",
    "clause",
    "two-at-once",
];

impl Trials {
    /// The trials compiled into the binary.
    ///
    /// # Panics
    ///
    /// If the built-in file is malformed — a build-time authoring error, covered
    /// by `the_trials_file_parses`.
    #[must_use]
    pub fn builtin() -> Self {
        super::load::builtin(FILE, BUILTIN)
    }

    /// Every single-line trial, in the order they were written.
    #[must_use]
    pub fn lines(&self) -> &[Trial] {
        &self.lines
    }

    /// Every whole-spell trial.
    #[must_use]
    pub fn scripts(&self) -> &[Script] {
        &self.scripts
    }
}

impl Trial {
    /// The one shape this trial tests.
    #[must_use]
    pub fn shape(&self) -> Option<&str> {
        self.tags
            .iter()
            .map(String::as_str)
            .find(|tag| SHAPES.contains(tag))
    }

    /// Whether `got` passes — `got` being the reader's answer, [`None`] for
    /// *left as written*.
    ///
    /// Case and spacing folded: a trial is about what a line means, not how the
    /// orb capitalised it.
    #[must_use]
    pub fn passes(&self, got: Option<&str>) -> bool {
        match (&self.reads, got) {
            (Some(wanted), Some(got)) => fold(wanted) == fold(got),
            (Some(_), None) => self.lenient,
            (None, None) => true,
            (None, Some(_)) => false,
        }
    }

    /// Whether `got` is one of the readings this line must never produce.
    #[must_use]
    pub fn betrayed_by(&self, got: Option<&str>) -> bool {
        got.is_some_and(|got| self.never.iter().any(|never| fold(never) == fold(got)))
    }
}

/// A line with its case and spacing folded, for comparing readings.
#[must_use]
pub fn fold(line: &str) -> String {
    line.split_whitespace()
        .map(str::to_lowercase)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::{Phrasings, corpus_scene};

    #[test]
    fn the_trials_file_parses() {
        let trials = Trials::builtin();
        println!(
            "{} lines and {} scripts",
            trials.lines().len(),
            trials.scripts().len()
        );
        assert!(
            trials.lines().len() >= 250,
            "too few trials to call a suite"
        );
        assert!(!trials.scripts().is_empty());
    }

    #[test]
    fn every_trial_says_what_it_tests() {
        // One shape per trial, so it lands in one row; unknown tags are typos.
        for trial in Trials::builtin().lines() {
            let shapes = trial
                .tags
                .iter()
                .filter(|tag| SHAPES.contains(&tag.as_str()))
                .count();
            assert_eq!(shapes, 1, "{:?} tests {shapes} shapes", trial.said);
            for tag in &trial.tags {
                assert!(
                    SHAPES.contains(&tag.as_str()) || STYLES.contains(&tag.as_str()),
                    "{:?} is tagged {tag:?}, which is no shape or style",
                    trial.said,
                );
            }
        }
    }

    #[test]
    fn every_shape_is_tried_often_enough_to_mean_something() {
        // The holdout measured `else` and `end` on four lines each, where one
        // line is twenty-five points.
        let trials = Trials::builtin();
        for shape in SHAPES {
            let tried = trials
                .lines()
                .iter()
                .filter(|trial| trial.shape() == Some(shape))
                .count();
            assert!(tried >= 10, "{shape} is tried {tried} times");
        }
    }

    #[test]
    fn no_trial_is_a_line_anything_was_taught() {
        // A trial that is also a corpus line measures memory, and the reader's
        // memory is excellent. Checked against every expansion of both corpora,
        // holdouts and refusals included.
        let scene = corpus_scene();
        let mut taught: std::collections::HashSet<String> = std::collections::HashSet::new();
        for phrasings in [Phrasings::builtin(), Phrasings::spellings()] {
            taught.extend(phrasings.corpus(&scene).into_iter().map(|e| fold(&e.said)));
            taught.extend(phrasings.holdout(&scene).into_iter().map(|e| fold(&e.said)));
            taught.extend(phrasings.refused(&scene).iter().map(|line| fold(line)));
            taught.extend(
                phrasings
                    .refused_holdout(&scene)
                    .iter()
                    .map(|line| fold(line)),
            );
        }
        let trials = Trials::builtin();
        let copied: Vec<&str> = trials
            .lines()
            .iter()
            .filter(|trial| !trial.said.trim().is_empty())
            .filter(|trial| trial.reads.is_some())
            .filter(|trial| taught.contains(&fold(&trial.said)))
            .map(|trial| trial.said.as_str())
            .collect();
        assert!(
            copied.is_empty(),
            "{} trials are lines the corpus teaches: {copied:?}",
            copied.len()
        );
    }

    #[test]
    fn every_expected_reading_is_one_the_language_reads() {
        // A trial expecting a line the language cannot parse can never pass,
        // and the report would blame the reader for the file's mistake.
        use crate::parser::{Mode, Resolution, resolve};
        let scene = corpus_scene();
        for trial in Trials::builtin().lines() {
            let Some(reads) = &trial.reads else { continue };
            if trial.shape() == Some("command") {
                assert!(
                    matches!(
                        resolve(reads, &scene, Mode::Calm),
                        Resolution::Resolved { ref intent, .. } if intent.echo() == *reads
                    ),
                    "{:?} expects {reads:?}, which is no command the orb echoes",
                    trial.said,
                );
            } else {
                assert!(
                    crate::tower::spell::reads_cleanly(reads),
                    "{:?} expects {reads:?}, which is no statement",
                    trial.said,
                );
            }
        }
    }

    #[test]
    fn a_trial_expecting_a_reading_is_one_the_orb_cannot_read_alone() {
        // `Scribe` leaves a line the language already parses alone, so a trial
        // expecting the reader to change it is a trial about the parser.
        for trial in Trials::builtin().lines() {
            if trial.reads.is_some() {
                assert!(
                    !crate::tower::spell::reads_cleanly(trial.said.trim()),
                    "{:?} already reads as a statement; the reader never sees it",
                    trial.said,
                );
            }
        }
    }
}
