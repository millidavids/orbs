//! Lines a spell reader is **tested** on and never taught (§19, *the scrivener*).
//!
//! Authored in `content/spell_trials.toml`. This module knows the shape of a
//! trial and how one is scored; the lines themselves are data.
//!
//! # Not the holdout, and kept apart from it
//!
//! `spellings.toml` holds back a few phrasings per shape, and they are worth
//! less than they look. They are written beside the corpus they are held back
//! from, by the same hand, from the same templates, expanded over the same
//! nouns — so a reader that has learned the *template* passes them without
//! having learned the language. Two shapes are measured on four lines each.
//!
//! The trials are the other instrument. **Literal lines**, no `{slots}`, written
//! to be unlike the corpus: contractions and capitals, punctuation and typos,
//! short names a player actually uses, names the reader has never seen, digits
//! and number words, archaic and terse registers. A test holds that none of them
//! is a line anything was taught.
//!
//! # Three ways to fail, and one of them is worse
//!
//! A trial fails if the orb reads it wrongly, or leaves alone a line it should
//! have read. It **betrays** the player if the reading is one listed under
//! `never` — the readings that silently invert what was written: a dropped
//! `not`, a dropped `or` clause, a number that changed. Those are scored apart,
//! and the only acceptable count is zero.
//!
//! # Not loaded by a `Sim`
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
    /// What the orb should hear. **Absent means the line must come back exactly
    /// as written** — a comment, a line already in canonical form, a sentence
    /// that asks for nothing.
    #[serde(default)]
    pub reads: Option<String>,
    /// Whether leaving the line alone also passes.
    ///
    /// For lines where refusing is as honest as reading: a spell runs with
    /// nobody watching, so a line left alone is recoverable in a way a line read
    /// wrongly is not.
    #[serde(default)]
    pub lenient: bool,
    /// Readings that must **never** come back — the ones that would run a spell
    /// that means the opposite of what was written.
    #[serde(default)]
    pub never: Vec<String>,
    /// What the line is testing: exactly one shape, and any number of styles.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// A whole spell written loosely, and the spell it should become.
///
/// The scrivener reads a line at a time, so a script is not a harder test of
/// any one line. What it tests is the file: that a player's spell comes out as
/// a spell — comments and blanks kept, canonical lines untouched, and the whole
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
/// The spell register's eleven templates, plus `command` for a loose command
/// line, `alone` for a line that must come back untouched, and `hazard` for a
/// line whose wrong reading would invert the spell.
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
    /// Compared with case and spacing folded: the spell language reads `if
    /// ALEMBIC is idle` exactly as it reads `if alembic is idle`, and a trial is
    /// about what a line means rather than how the orb capitalised it.
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
        // Exactly one shape, so the report can put it in exactly one row — and
        // every tag a known one, so a typo is an error rather than a new row.
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
        // **The holdout's own failing, not repeated.** `else` and `end` were
        // measured on four lines each, where one line is twenty-five points.
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
        // **The whole point of a separate file.** A trial that is also a corpus
        // line measures memory, and the reader's memory is excellent. Checked
        // against every expansion of both corpora, holdouts and refusals
        // included — a trial copied out of a holdout is a trial the holdout
        // already runs.
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
        // A trial expecting a line the spell language cannot parse can never
        // pass, and the report would blame the reader for the file's mistake.
        // Statements must stand on their own; a command must resolve against
        // everything the content tables name, to exactly itself.
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
        // A line the spell language already parses never reaches the reader —
        // `Scribe` leaves it alone by its first rule — so a trial expecting the
        // reader to change it is a trial about the parser, and cannot pass.
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
