//! The orb's voice, loaded from data rather than written in Rust.
//!
//! CLAUDE.md rule 6: prose lives in hot-reloadable data files, never as string
//! literals. DESIGN.md §12 budgets ~88k words, and a sentence spelled out at the
//! call site is a sentence no writer can find.
//!
//! # What this is not
//!
//! It is **not** a record formatter. Records stay structured (rule 4) — `sift`,
//! pipes and §14's linearisation all read fields, not sentences. A line from
//! here goes into [`FieldName::Message`](orbs_render::FieldName::Message),
//! *alongside* the fields it was built
//! from, so a machine still reads the facts and a player reads the sentence.
//!
//! # Why the sim parses content but never watches a file
//!
//! Rule 8 forbids async here, and rule 3 makes a frontend a caller rather than a
//! host. A watcher inside the sim would also make every headless test touch the
//! filesystem. So this crate parses a string; a frontend owns the watcher and
//! calls [`Sim::set_prose`](crate::Sim::set_prose) at a tick boundary.
//!
//! # Replay
//!
//! Swapping prose is **replay-safe**, because no line here reaches a decision —
//! it is presentation over a record that was already built. Recipes are not, and
//! when they arrive they need content versioned into the submission log; that is
//! a deliberately separate problem from this one.

use std::collections::BTreeMap;

use bevy_ecs::resource::Resource;
use serde::Deserialize;

/// The compiled-in default, so a headless `Sim` needs no filesystem.
const BUILTIN: &str = include_str!("../../content/prose.toml");

/// Everything the orb can say, keyed by an in-world situation.
///
/// `BTreeMap` rather than `HashMap`: nothing iterates this today, but a hash
/// map's order varies per process, and this crate's whole contract is that two
/// runs of the same seed produce the same bytes.
#[derive(Debug, Clone, Deserialize, Resource)]
pub struct Prose {
    lines: BTreeMap<String, String>,
}

/// A raw content file, before it is turned into a [`Prose`].
#[derive(Debug, Deserialize)]
struct File {
    lines: BTreeMap<String, String>,
}

/// The file's name, for an error a writer can act on.
const FILE: &str = "prose.toml";

impl Prose {
    /// The prose compiled into the binary.
    ///
    /// # Panics
    ///
    /// If the *built-in* file is malformed — which is a build-time authoring
    /// error, not a runtime condition, and is covered by
    /// `the_builtin_file_parses`.
    #[must_use]
    pub fn builtin() -> Self {
        let file: File = super::load::builtin(FILE, BUILTIN);
        Self { lines: file.lines }
    }

    /// Parse a replacement content file.
    ///
    /// # Errors
    ///
    /// [`ContentError`](super::ContentError) if the text is not valid TOML of the
    /// expected shape. A frontend hot-reloading a file the writer just broke
    /// should keep the prose it already has and report this, never fall back to
    /// silence.
    pub fn parse(text: &str) -> Result<Self, super::ContentError> {
        let file: File = super::load::parse(FILE, text)?;
        Ok(Self { lines: file.lines })
    }

    /// The line for `key`, with `{placeholder}`s filled from `fields`.
    ///
    /// A missing key yields the key itself, and an unmatched placeholder is left
    /// as written. Both are deliberately *visible* rather than empty: §15's
    /// practice is that work is done when it has been looked at, and a silently
    /// blank line is the one failure looking cannot catch.
    #[must_use]
    pub fn line(&self, key: &str, fields: &[(&str, &str)]) -> String {
        let Some(template) = self.lines.get(key) else {
            return key.to_owned();
        };
        interpolate(template, fields)
    }

    /// Whether `key` has a line, without building one.
    #[must_use]
    pub fn has(&self, key: &str) -> bool {
        self.lines.contains_key(key)
    }

    /// Every subject the manual can answer on, from the `recall_` keys.
    ///
    /// Derived rather than listed, so authoring a manual entry in the content
    /// file is all it takes to make the subject **nameable** — otherwise a
    /// writer adds `recall_warding`, and the parser has never heard of it.
    ///
    /// # The prefix is load-bearing, so templates do not share it
    ///
    /// This makes every key under the prefix a **parser noun**, which is a
    /// strong thing for a content file to be able to do by accident. The route
    /// templates were `grimoire_route`, `grimoire_step`, `grimoire_step_or` and
    /// `grimoire_heat`, so the scene registered `route`, `step`, `step_or` and
    /// `heat` as subjects a player could ask about and the manual could not
    /// answer. They are `route_` now, and a template that wants a new name
    /// should take any prefix but this one.
    #[must_use]
    pub fn topics(&self) -> Vec<&str> {
        self.lines
            .keys()
            .filter_map(|key| key.strip_prefix("recall_"))
            .collect()
    }
}

impl Default for Prose {
    fn default() -> Self {
        Self::builtin()
    }
}

/// Substitute `{name}` in `template` from `fields`.
///
/// Single pass, no regex, no allocation per placeholder. An unclosed `{` is
/// copied through rather than treated as an error — a writer's typo should show
/// on screen, not stop the tower.
fn interpolate(template: &str, fields: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;

    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];

        let Some(close) = after.find('}') else {
            // No closing brace: the rest is literal, including this `{`.
            out.push_str(&rest[open..]);
            return out;
        };

        let name = &after[..close];
        match fields.iter().find(|(field, _)| *field == name) {
            Some((_, value)) => out.push_str(value),
            // Leave it as the writer typed it, braces and all.
            None => {
                out.push('{');
                out.push_str(name);
                out.push('}');
            }
        }
        rest = &after[close + 1..];
    }

    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_builtin_file_parses() {
        // Guards the `expect` in `builtin`, which is otherwise a panic shipped
        // to players.
        let prose = Prose::builtin();
        assert!(prose.has("work_busy"), "the busy refusal is authored");
    }

    #[test]
    fn a_line_interpolates_its_fields() {
        let prose = Prose::builtin();
        let line = prose.line(
            "work_busy",
            &[("source", "laboratory"), ("state", "decoct")],
        );
        assert!(line.contains("laboratory"), "{line}");
        assert!(line.contains("decoct"), "{line}");
        assert!(
            !line.contains('{'),
            "a filled line has no braces left: {line}"
        );
    }

    #[test]
    fn every_manual_subject_is_a_page_rather_than_a_template() {
        // **The `recall_` prefix makes a key a parser noun**, so a *template*
        // filed under it becomes a subject the manual answers with its own
        // braces: `recall_road` shipped as a topic called `road`, and `recall
        // road` printed `{name} line: {count} of {quantity} reached`. The route
        // templates paid for this once already and were renamed; this is the
        // lint that was missing, and it costs nothing.
        let prose = Prose::builtin();
        for topic in prose.topics() {
            let page = prose.line(&format!("recall_{topic}"), &[]);
            assert!(
                !page.contains('{'),
                "`{topic}` is a manual subject and its page is a template: {page}",
            );
        }
    }

    #[test]
    fn a_missing_key_yields_the_key_rather_than_an_empty_line() {
        // Visible on screen, so it is caught by looking (§15).
        let prose = Prose::builtin();
        assert_eq!(prose.line("no_such_line", &[]), "no_such_line");
    }

    #[test]
    fn an_unmatched_placeholder_is_left_as_written() {
        assert_eq!(interpolate("the {thing} waits", &[]), "the {thing} waits");
    }

    #[test]
    fn an_unclosed_brace_is_copied_through() {
        assert_eq!(interpolate("the {thing waits", &[]), "the {thing waits");
    }

    #[test]
    fn text_without_placeholders_is_unchanged() {
        assert_eq!(interpolate("the orb cools", &[("a", "b")]), "the orb cools");
    }

    #[test]
    fn adjacent_placeholders_both_fill() {
        assert_eq!(
            interpolate("{a}{b}", &[("a", "one"), ("b", "two")]),
            "onetwo"
        );
    }

    #[test]
    fn a_broken_replacement_is_an_error_rather_than_a_panic() {
        // A writer saving a half-edited file must not take the tower down.
        assert!(Prose::parse("this is not toml [[[").is_err());
    }

    #[test]
    fn a_replacement_overrides_the_builtin() {
        let prose = Prose::parse("[lines]\nwork_busy = \"the {source} is occupied\"\n")
            .expect("valid toml");
        assert_eq!(
            prose.line("work_busy", &[("source", "athanor")]),
            "the athanor is occupied"
        );
    }

    #[test]
    fn every_authored_line_fits_the_worst_case_width() {
        // §4's floor is 80×22; a pane's usable width is ~70 cells once the
        // border and the outcome glyph are drawn. A line wider than this wraps,
        // which at the floor costs a transcript row that is already scarce.
        const USABLE: usize = 70;
        let prose = Prose::builtin();
        for (key, template) in &prose.lines {
            assert!(
                template.chars().count() <= USABLE,
                "{key} is {} cells; the floor gives ~{USABLE}",
                template.chars().count()
            );
        }
    }

    #[test]
    fn every_authored_line_is_drawable() {
        // The grid draws CP437 and nothing else, so an em-dash arrives on screen
        // as `?`. This is not hypothetical: the first draft of `work_busy` used
        // one, and CLAUDE.md already records the same defect being found in
        // DESIGN.md's own boot text. A writer typing a smart quote from a word
        // processor should get a failing test, not a question mark in the tube.
        let prose = Prose::builtin();
        for (key, template) in &prose.lines {
            assert_eq!(
                orbs_render::cp437::first_unrenderable(template),
                None,
                "{key} has a glyph the grid cannot draw"
            );
        }
    }

    #[test]
    fn authored_lines_do_not_shout() {
        // §3: the tower is a terminal; a capitalised line would be the only
        // thing on screen shouting.
        let prose = Prose::builtin();
        for (key, template) in &prose.lines {
            let first = template.chars().next().expect("no empty lines");
            assert!(!first.is_uppercase(), "{key} opens upper case");
            assert!(!template.ends_with('.'), "{key} ends with a full stop");
        }
    }
}
