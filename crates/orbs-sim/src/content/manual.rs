//! The manual: chapters a player reads, rather than sentences the orb says.
//!
//! Not `prose.toml`, for two reasons. The width lint
//! (`every_authored_line_fits_the_worst_case_width`) holds those lines to 70
//! cells because they are drawn *as written*; a manual is paragraphs the reader
//! wraps, so the same rule would mean thousands of fragment keys. And
//! `Prose::topics` makes every `recall_` key a `NounKind::Topic` — a content
//! file once registered four route templates as subjects that way (§19), where
//! a separate resource cannot do it by accident at all.
//!
//! Everything else it shares: `include_str!` for a headless `Sim`, `parse` for
//! a replacement, and `ORBS_CONTENT` hot-reload through the shell — rule 6 is
//! not satisfied by `include_str!` alone.

use bevy_ecs::resource::Resource;
use serde::Deserialize;

/// The compiled-in default, so a headless `Sim` needs no filesystem.
const BUILTIN: &str = include_str!("../../content/manual.toml");

/// The file's name, for an error a writer can act on.
const FILE: &str = "manual.toml";

/// One chapter of the manual.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Chapter {
    /// The word a player types to open it.
    ///
    /// One word, lowercase and unique — held by `the_manual_reads_as_a_book` in
    /// the shell, which also checks no two share a first letter, because the
    /// contents page is prefix-matched like every other list in the game.
    pub name: String,
    /// What the contents page calls it.
    pub title: String,
    /// The chapter itself, a paragraph per entry.
    ///
    /// An empty entry is a blank line. The rest the reader wraps to whatever
    /// width the pane is, which is why these are not in `prose.toml`.
    pub lines: Vec<String>,
}

/// Every authored chapter, in the order the contents page lists them.
#[derive(Debug, Clone, Deserialize, Resource)]
pub struct Manual {
    chapters: Vec<Chapter>,
}

/// A raw content file, before it is turned into a [`Manual`].
#[derive(Debug, Deserialize)]
struct File {
    chapters: Vec<Chapter>,
}

impl Manual {
    /// The manual compiled into the binary.
    ///
    /// # Panics
    ///
    /// If the *built-in* file is malformed — a build-time authoring error rather
    /// than a runtime condition, covered by `the_builtin_file_parses`.
    #[must_use]
    pub fn builtin() -> Self {
        let file: File = super::load::builtin(FILE, BUILTIN);
        Self {
            chapters: file.chapters,
        }
    }

    /// Parse a replacement manual.
    ///
    /// # Errors
    ///
    /// [`ContentError`](super::ContentError) if the text is not valid TOML of
    /// the expected shape. A frontend hot-reloading a file the writer has just
    /// broken keeps the manual it has and reports this.
    pub fn parse(text: &str) -> Result<Self, super::ContentError> {
        let file: File = super::load::parse(FILE, text)?;
        Ok(Self {
            chapters: file.chapters,
        })
    }

    /// Every chapter, in order.
    #[must_use]
    pub fn chapters(&self) -> &[Chapter] {
        &self.chapters
    }
}

impl Default for Manual {
    fn default() -> Self {
        Self::builtin()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_builtin_file_parses() {
        // The build-time authoring error `builtin` panics on: a stray bracket
        // would otherwise take the game down at startup, in front of a player.
        let manual = Manual::builtin();
        assert!(
            !manual.chapters().is_empty(),
            "the manual has no chapters in it",
        );
    }

    #[test]
    fn every_chapter_is_named_by_one_lowercase_word() {
        // The contents page is typed at, like every other list in the game, so a
        // chapter called "The Screen" is one nobody can open.
        for chapter in Manual::builtin().chapters() {
            assert!(
                !chapter.name.is_empty()
                    && chapter
                        .name
                        .chars()
                        .all(|glyph| glyph.is_ascii_lowercase() || glyph == '-'),
                "{:?} is not a word a player can type",
                chapter.name,
            );
            assert!(
                !chapter.title.is_empty(),
                "{} has no title for the contents page",
                chapter.name,
            );
            assert!(
                !chapter.lines.is_empty(),
                "{} is a chapter with nothing in it",
                chapter.name,
            );
        }
    }

    #[test]
    fn no_two_chapters_share_a_name_or_a_first_letter() {
        // Prefix-matched like the menu's words, the lengths and the settings
        // pages — `w` has to mean one chapter.
        let chapters = Manual::builtin();
        let mut names: Vec<&str> = chapters
            .chapters()
            .iter()
            .map(|chapter| chapter.name.as_str())
            .collect();
        names.sort_unstable();
        let before = names.len();
        names.dedup();
        assert_eq!(before, names.len(), "two chapters share a name");

        let mut firsts: Vec<char> = chapters
            .chapters()
            .iter()
            .filter_map(|chapter| chapter.name.chars().next())
            .collect();
        firsts.sort_unstable();
        let before = firsts.len();
        firsts.dedup();
        assert_eq!(before, firsts.len(), "two chapters share a first letter");
    }

    #[test]
    fn the_manual_is_written_as_a_book_and_never_shouts() {
        // Two registers, on purpose: `prose.toml` is the orb speaking and is
        // held to lower case by `authored_lines_do_not_shout` (§3), while the
        // manual is a book written about the game and capitalises its
        // sentences. A second lint rather than the first widened, then — but
        // neither may shout, and that lint never saw `manual.toml` at all.
        for chapter in Manual::builtin().chapters() {
            for line in &chapter.lines {
                let letters = line.chars().filter(|glyph| glyph.is_alphabetic());
                let shouted =
                    letters.clone().count() > 3 && letters.clone().all(char::is_uppercase);
                assert!(!shouted, "{} shouts a line: {line:?}", chapter.name,);
            }
            // A chapter's own name and title are typed and drawn in a list, and
            // are lower case like every other word a player types.
            assert!(
                !chapter.title.chars().next().is_some_and(char::is_uppercase),
                "{}'s title opens upper case; the contents page is the orb's",
                chapter.name,
            );
        }
    }

    #[test]
    fn every_line_is_drawable() {
        // CP437 is the intersection of what both frontends can draw, so an
        // em-dash or a smart quote here is a glyph the game cannot render —
        // `screens` found exactly that in DESIGN.md's own boot text once.
        for chapter in Manual::builtin().chapters() {
            for line in &chapter.lines {
                assert_eq!(
                    orbs_render::cp437::first_unrenderable(line),
                    None,
                    "{} has a glyph the game cannot draw: {line:?}",
                    chapter.name,
                );
            }
            assert_eq!(
                orbs_render::cp437::first_unrenderable(&chapter.title),
                None,
                "{}'s title has a glyph the game cannot draw",
                chapter.name,
            );
        }
    }
}
