//! The orb's voice, loaded from data rather than written in Rust.
//!
//! CLAUDE.md rule 6: prose lives in hot-reloadable data files, never as string
//! literals. DESIGN.md §12 budgets ~88k words, and a sentence spelled out at the
//! call site is a sentence no writer can find.
//!
//! Not a record formatter: records stay structured (rule 4), and `sift`, pipes
//! and §14's linearisation all read fields rather than sentences. A line from
//! here goes into [`FieldName::Message`](orbs_render::FieldName::Message)
//! *alongside* the fields it was built from.
//!
//! The sim parses content but never watches a file, because rule 8 forbids async
//! here and rule 3 makes a frontend a caller rather than a host — and a watcher
//! inside the sim would make every headless test touch the filesystem. So this
//! crate parses a string and a frontend calls
//! [`Sim::set_prose`](crate::Sim::set_prose) at a tick boundary.
//!
//! Swapping prose is replay-safe, because no line here reaches a decision — it
//! is presentation over a record already built. Recipes are not, and will need
//! content versioned into the submission log.

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

    /// A line whose wording depends on how many, with `{count}` filled in.
    ///
    /// A mastery deed's sentence sits beside a live `3 of 5` counter, so it says
    /// the number — but `"{count} charms laid"` reads "1 charms laid", and six
    /// of the twenty-five deeds ask for exactly one. A key that reads wrong in
    /// the singular is authored twice, `key` and `key_one`; every other key
    /// falls straight through.
    ///
    /// Not a pluralisation rule: English inflects more than the noun (*"the
    /// stacks walked"* against *"3 walks of the stacks"*).
    #[must_use]
    pub fn counted(&self, key: &str, count: u32) -> String {
        self.line(
            &self.counted_key(key, u64::from(count)),
            &[("count", &count.to_string())],
        )
    }

    /// Which key a line about `count` things reads from: `{key}_one` for a
    /// single one and `{key}_none` for nought, each when it is authored, and
    /// `key` otherwise.
    ///
    /// The singular rule, once: [`counted`](Self::counted) fills `{count}` and
    /// nothing else, so a line naming the things themselves needs only the key.
    /// The circle's painter chose it by hand, which was this rule's second copy.
    ///
    /// `_none` covers what `_one` cannot: *"the last call balked at rows "* with
    /// nothing after it. Only a key that authors it changes.
    #[must_use]
    pub fn counted_key(&self, key: &str, count: u64) -> String {
        let form = match count {
            0 => format!("{key}_none"),
            1 => format!("{key}_one"),
            _ => return key.to_owned(),
        };
        if self.has(&form) {
            form
        } else {
            key.to_owned()
        }
    }

    /// Every subject the manual can answer on, from the `recall_` keys.
    ///
    /// Derived rather than listed, so authoring a manual entry is all it takes
    /// to make the subject nameable — otherwise a writer adds `recall_warding`
    /// and the parser has never heard of it.
    ///
    /// The prefix is load-bearing and templates must not share it: every key
    /// under it becomes a parser noun. `grimoire_route`, `grimoire_step` and
    /// friends registered `route`, `step` and `heat` as subjects the manual
    /// could not answer; they are `route_` now.
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
    fn a_deed_that_asks_for_one_reads_as_one() {
        // Fifteen mastery keys went from spelled-out counts to `{count}`, right
        // for nine and wrong for six — *"1 charms laid"* for *"a charm laid"*.
        // Nothing asserts English, so a `dumps.sh` diff caught it, not a test.
        let prose = Prose::builtin();
        assert_eq!(prose.counted("mastery_forge_1", 1), "a charm laid");
        assert_eq!(prose.counted("mastery_archive_1", 1), "the stacks walked");

        // ...and above one it is the plural, from the same key.
        assert_eq!(prose.counted("mastery_forge_1", 5), "5 charms laid");

        // `archive_2` can stop being one: it sits second on its line, so a long
        // game stretches it and the plural is then right.
        assert_eq!(prose.counted("mastery_archive_2", 1), "a scroll assembled");
        assert_eq!(prose.counted("mastery_archive_2", 2), "2 scrolls assembled");

        // A key with no singular form falls straight through, which is what
        // keeps the other nine as one authored line each.
        assert_eq!(prose.counted("mastery_laboratory_2", 1), "1 potions brewed");
    }

    /// One rule for the key, whether or not the count is in the sentence: a line
    /// naming the rows picks its key the same way, and `_none` says something
    /// true of an empty list instead of trailing off after *"rows"*.
    #[test]
    fn a_key_is_chosen_for_none_one_and_many_and_falls_through_when_unauthored() {
        let prose = Prose::builtin();
        assert_eq!(
            prose.counted_key("circle_balks_spoken", 0),
            "circle_balks_spoken_none"
        );
        assert_eq!(
            prose.counted_key("circle_balks_spoken", 1),
            "circle_balks_spoken_one"
        );
        assert_eq!(
            prose.counted_key("circle_balks_spoken", 3),
            "circle_balks_spoken"
        );
        assert_eq!(prose.counted("circle_balks", 0), "every row agrees");
        // Unauthored forms fall through to the key itself.
        assert_eq!(
            prose.counted_key("mastery_laboratory_2", 0),
            "mastery_laboratory_2"
        );
    }

    #[test]
    fn every_deed_that_can_ask_for_one_has_a_singular() {
        // Derived from the content, not a list here: a deed authored with no
        // `times` asks for one, and `1 walks of the stacks` is a sentence
        // nobody wrote.
        let prose = Prose::builtin();
        for stone in super::super::Progression::builtin().mastery() {
            if stone.done.times() != 1 {
                continue;
            }
            let key = format!("mastery_{}", stone.id);
            let singular = prose.counted(&key, 1);
            assert!(
                !singular.starts_with("1 "),
                "`{key}` asks for one and reads {singular:?} — author `{key}_one`",
            );
        }
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
        // The `recall_` prefix makes a key a parser noun, so a *template* filed
        // under it becomes a subject answered with its own braces: `recall
        // road` printed `{name} line: {count} of {quantity} reached`. The route
        // templates were renamed; this is the lint that was missing.
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
        // as `?` — the first draft of `work_busy` had one. A writer pasting a
        // smart quote should get a failing test, not a `?` in the tube.
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
    fn every_settings_row_is_padded_to_the_same_width() {
        // `menu_setting_row` puts `[value]` straight after this text, so a row
        // one cell short puts a bracket out of line down the whole page —
        // `voice` came out at 28 against everything else's 27, and it reads as
        // a rendering bug rather than a typo in a string.
        let prose = Prose::builtin();
        let rows: Vec<(&String, usize)> = prose
            .lines
            .iter()
            .filter(|(key, _)| key.starts_with("menu_set_"))
            .map(|(key, template)| (key, template.chars().count()))
            .collect();
        assert!(rows.len() > 1, "no settings rows to measure");
        let (first, width) = rows[0];
        for (key, found) in &rows {
            assert_eq!(
                *found, width,
                "{key} is {found} cells and {first} is {width}; the values will not line up",
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
