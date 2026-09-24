//! The line the player is typing.
//!
//! The buffer, the caret, the history, and what Tab and the ghost make of them.
//! Nothing here reads a keystroke — each frontend's input path does that and
//! calls these — so the editor is testable without a window.
//!
//! The caret counts characters, not bytes: CP437 admits multi-byte glyphs that
//! occupy one cell (`░`, `Σ`), so a byte caret would split one and panic.
//! Counting goes through [`orbs_render::char_index`], shared with `orbs-sim`'s
//! completer so the two agree about where a word starts.

use bevy_ecs::prelude::*;
use orbs_render::char_index;

/// The parser's own limit (§6). A shorter cap would silently change what a
/// command means; a longer one lets the player type into a void.
const MAX_INPUT: usize = 512;

/// How many lines of history are kept.
///
/// Far more than a session reaches, and it bounds the memory. Not persisted —
/// a home for it arrives with Phase 13's settings.
const MAX_HISTORY: usize = 100;

/// The command line as it currently stands.
#[derive(Resource, Debug, Default)]
pub struct Line {
    text: String,
    /// Where the next character goes, in **characters** from the start.
    caret: usize,
    /// Submitted lines, oldest first.
    history: Vec<String>,
    /// How far back a recall has walked, as an index into `history`.
    recalled: Option<usize>,
    /// The prefix a history search is anchored to.
    ///
    /// Held for the whole search rather than re-read each press: after one
    /// recall the line *is* the recalled command, so an unanchored second `Up`
    /// would search on that. fish and zsh both hold the original prefix.
    search: Option<String>,
    /// What was being typed before a recall started, restored by walking back.
    draft: String,
    /// A Tab cycle in progress, if the last thing pressed was Tab.
    cycle: Option<crate::tabbing::Cycle>,
}

impl Line {
    /// A line with `text` already in it and the caret at the end.
    ///
    /// For `ORBS_LINE` — the only way a dump can show a partly-typed line.
    #[must_use]
    pub fn typed(text: &str) -> Self {
        let text: String = text
            .chars()
            .filter(|c| orbs_render::is_renderable(*c))
            .collect();
        let caret = text.chars().count();
        Self {
            text,
            caret,
            ..Self::default()
        }
    }

    /// What is being typed.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Where the visible window starts, as a **byte** offset into the line.
    ///
    /// The companion to [`viewport`](Self::viewport), for one caller:
    /// highlighting. `parser::lex` classifies partly by *position*, so lexing
    /// the visible slice of a scrolled line would read a mid-line fragment as a
    /// line start. The prompt lexes the whole line and maps the runs through
    /// this.
    #[must_use]
    pub fn window_starts(&self, width: u16) -> usize {
        // `viewport(0)` returns the literal `""`, not a subslice of this line,
        // so the pointer arithmetic below would give nonsense. The 80×22 floor
        // keeps the prompt from asking, but this is `pub`.
        if width == 0 {
            return 0;
        }
        let (visible, _) = self.viewport(width);
        // A subslice of `self.text`, so its offset is the difference of the two
        // pointers — the same arithmetic `parser::lexeme` uses.
        (visible.as_ptr() as usize).saturating_sub(self.text.as_ptr() as usize)
    }

    /// The window of the line that fits `width` cells, and the caret's column
    /// within it.
    ///
    /// Without this the line has no viewport: at the 80×22 floor the prompt
    /// leaves 72 cells, past which `put_str` clips silently *and*
    /// [`Frame::set_cursor`](orbs_render::Frame::set_cursor) refuses an off-grid
    /// position — so the player types into a dead line with no caret. A
    /// 44-character `sift` is ordinary, and the magnified prompt halves the
    /// budget again.
    ///
    /// The window follows the *caret*, so it scrolls left as well as right, and
    /// one cell is always reserved for the caret at the end of a full line.
    #[must_use]
    pub fn viewport(&self, width: u16) -> (&str, u16) {
        if width == 0 {
            return ("", 0);
        }
        let width = usize::from(width);
        let count = self.text.chars().count();

        // The whole line fits: no window at all.
        if count < width {
            return (&self.text, to_col(self.caret));
        }

        // Otherwise keep the caret inside a window one cell short of the pane,
        // sliding only as far as it must. `visible` is that cell count.
        let visible = width - 1;
        let start = if self.caret <= visible {
            0
        } else {
            (self.caret - visible).min(count.saturating_sub(visible))
        };
        let from = char_index(&self.text, start);
        let to = char_index(&self.text, (start + visible).min(count));
        (&self.text[from..to], to_col(self.caret - start))
    }

    /// Put `text` in at the caret.
    ///
    /// Takes a whole string, not a character at a time: winit documents a
    /// Windows dead key yielding two characters from one press, and inserting
    /// them one at a time at a moving caret would land them reversed.
    pub fn insert(&mut self, text: &str) {
        // Truncated to what is left, not dropped whole: refusing the insert
        // would make a paste longer than the remaining room vanish silently.
        let room = MAX_INPUT.saturating_sub(self.text.chars().count());
        let glyphs: String = text
            .chars()
            .filter(|c| orbs_render::is_renderable(*c))
            .take(room)
            .collect();
        if glyphs.is_empty() {
            return;
        }
        let at = char_index(&self.text, self.caret);
        self.text.insert_str(at, &glyphs);
        self.caret += glyphs.chars().count();
        self.end_search();
    }

    /// Delete the character before the caret.
    pub fn backspace(&mut self) {
        if self.caret == 0 {
            return;
        }
        let from = char_index(&self.text, self.caret - 1);
        let to = char_index(&self.text, self.caret);
        self.text.replace_range(from..to, "");
        self.caret -= 1;
        self.end_search();
    }

    /// Move the caret, clamped to the line.
    pub const fn left(&mut self) {
        self.caret = self.caret.saturating_sub(1);
    }

    /// Move the caret right, clamped to the end of the line.
    pub fn right(&mut self) {
        self.caret = (self.caret + 1).min(self.text.chars().count());
    }

    /// Caret to the start.
    pub const fn home(&mut self) {
        self.caret = 0;
    }

    /// Caret to the end.
    pub fn end(&mut self) {
        self.caret = self.text.chars().count();
    }

    /// Throw the line away.
    pub fn clear(&mut self) {
        self.text.clear();
        self.caret = 0;
        self.recalled = None;
        self.search = None;
        self.draft.clear();
    }

    /// Take the finished line, resetting everything, and remember it.
    ///
    /// `remember` is false for a bare digit answering §6's numbered prompt: it
    /// is an answer, not a phrasing, and recalling one would submit it as a
    /// command and land in `ParseLog` as a failed phrasing.
    pub fn take(&mut self, remember: bool) -> String {
        let finished = std::mem::take(&mut self.text);
        self.clear();
        if remember && !finished.trim().is_empty() {
            // Consecutive duplicates collapse — bash's `HISTCONTROL=ignoredups`.
            // Running `survey` five times should not cost five presses of `Up`.
            if self.history.last().map(String::as_str) != Some(finished.as_str()) {
                self.history.push(finished.clone());
                if self.history.len() > MAX_HISTORY {
                    self.history.remove(0);
                }
            }
        }
        finished
    }

    /// Walk back through history; `Up`.
    ///
    /// Filtered by what is already typed, which is fish's default and zsh's
    /// `history-beginning-search-backward`. Against a `move / wield / siphon`
    /// loop, typing `wield ` and pressing `Up` should walk the wields.
    pub fn earlier(&mut self) {
        let anchor = self.anchor();
        let upto = self.recalled.unwrap_or(self.history.len());
        let Some(found) = self.history[..upto]
            .iter()
            .rposition(|line| line.starts_with(&anchor))
        else {
            return;
        };
        if self.recalled.is_none() {
            self.draft = self.text.clone();
        }
        self.recalled = Some(found);
        self.replace_with(self.history[found].clone());
    }

    /// Walk forward through history; `Down`. Past the newest, the draft returns.
    pub fn later(&mut self) {
        let Some(from) = self.recalled else {
            return;
        };
        let anchor = self.anchor();
        match self.history[from + 1..]
            .iter()
            .position(|line| line.starts_with(&anchor))
        {
            Some(offset) => {
                let found = from + 1 + offset;
                self.recalled = Some(found);
                self.replace_with(self.history[found].clone());
            }
            None => {
                self.recalled = None;
                let draft = std::mem::take(&mut self.draft);
                self.replace_with(draft);
                self.search = None;
            }
        }
    }

    /// The prefix this search is anchored to, taken once at the first `Up`.
    fn anchor(&mut self) -> String {
        self.search.get_or_insert_with(|| self.text.clone()).clone()
    }

    /// Swap the line's contents, caret to the end.
    fn replace_with(&mut self, text: String) {
        self.text = text;
        self.caret = self.text.chars().count();
    }

    /// Typing ends a history search — the anchor was the *old* prefix.
    fn end_search(&mut self) {
        self.recalled = None;
        self.search = None;
    }

    /// What the line would become if the ghost were accepted.
    ///
    /// A chain, history first, which is fish's order and zsh's default. History
    /// is the cheap half and usually the right one: a player repeating a brew
    /// loop is about to type what they typed before.
    ///
    /// Only offered with the caret at the end — a suggestion trailing text being
    /// edited mid-line would sit in the wrong place.
    ///
    /// Called once per change rather than per frame; see
    /// [`Ghost`](super::offering::Ghost) for what owns the result.
    #[must_use]
    pub fn ghost(&self, scene: &orbs_sim::parser::Scene, prompt_open: bool) -> String {
        if self.text.is_empty() || self.caret != self.text.chars().count() {
            return String::new();
        }
        if let Some(earlier) = self
            .history
            .iter()
            .rev()
            .find(|line| line.starts_with(&self.text) && line.as_str() != self.text)
        {
            return earlier[self.text.len()..].to_owned();
        }
        // The same `expect` and `common` Tab reads: a second opinion would have
        // the ghost drawing text the key then does not take.
        let found = orbs_sim::parser::expect(
            &self.text,
            self.caret,
            &orbs_sim::parser::Situation {
                scene,
                spell: false,
                prompt_open,
                // The prompt has no lines above it, cannot run a block, and
                // cannot run a `for each`.
                open: &[],
                sets: &[],
            },
        );
        let common = found.common();
        let typed = &self.text[found.replaces];
        common.strip_prefix(typed).unwrap_or_default().to_owned()
    }

    /// Take a completion: replace the partial word with `insert`.
    fn apply(&mut self, replaces: core::ops::Range<usize>, insert: &str) {
        self.text.replace_range(replaces.clone(), insert);
        let upto = replaces.start + insert.len();
        self.caret = self.text[..upto].chars().count();
        self.end_search();
    }

    /// Tab: extend as far as every candidate agrees, then cycle.
    ///
    /// The rules are [`tabbing::tab`](crate::tabbing::tab)'s, shared with the
    /// spell editor. Applying the decision stays here because a caret is
    /// characters at the prompt and a row and column in the editor.
    ///
    /// Returns the candidate list while a cycle is running, so the caller can
    /// show it with [`cycling`](Self::cycling) marking where the player is.
    pub fn tab(&mut self, scene: &orbs_sim::parser::Scene, prompt_open: bool) -> Vec<String> {
        let found = orbs_sim::parser::expect(
            &self.text,
            self.caret,
            &orbs_sim::parser::Situation {
                scene,
                spell: false,
                prompt_open,
                // The prompt has no lines above it, cannot run a block, and
                // cannot run a `for each`.
                open: &[],
                sets: &[],
            },
        );
        match crate::tabbing::tab(&self.text, &found, &mut self.cycle) {
            crate::tabbing::Tabbed::Nothing => Vec::new(),
            crate::tabbing::Tabbed::Listed(candidates) => candidates,
            crate::tabbing::Tabbed::Wrote {
                replaces,
                text,
                candidates,
            } => {
                self.apply(replaces, &text);
                candidates
            }
        }
    }

    /// Which candidate the cycle has put in the line, for the list to mark.
    ///
    /// `None` while the list is merely being shown, which is the first press.
    #[must_use]
    pub fn cycling(&self) -> Option<usize> {
        self.cycle.as_ref().and_then(crate::tabbing::Cycle::at)
    }

    /// Abandon any Tab cycle. Anything that is not another Tab ends it.
    pub fn end_cycle(&mut self) {
        self.cycle = None;
    }
}

fn to_col(count: usize) -> u16 {
    u16::try_from(count).unwrap_or(u16::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_of(text: &str) -> Line {
        Line::typed(text)
    }

    #[test]
    fn a_short_line_is_shown_whole_with_the_caret_after_it() {
        let line = line_of("survey");
        assert_eq!(line.viewport(40), ("survey", 6));
    }

    #[test]
    fn a_long_line_scrolls_and_keeps_its_caret_on_screen() {
        // Past the pane width the text stops appearing and `set_cursor` refuses
        // an off-grid position, so the player types into a dead line.
        let line = line_of("abcdefghij");
        let (visible, caret) = line.viewport(5);
        assert_eq!(visible, "ghij", "the tail must stay visible");
        assert_eq!(caret, 4, "the caret must stay inside the viewport");
        assert!(usize::from(caret) < 5);
    }

    #[test]
    fn the_caret_never_leaves_the_grid_at_any_length() {
        for length in 0..200usize {
            let line = line_of(&"x".repeat(length));
            for width in [1u16, 8, 72] {
                let (visible, caret) = line.viewport(width);
                assert!(caret < width, "len {length} width {width} caret {caret}");
                assert!(visible.chars().count() < usize::from(width) + 1);
            }
        }
    }

    #[test]
    fn a_zero_width_viewport_is_empty_rather_than_a_panic() {
        assert_eq!(line_of("survey").viewport(0), ("", 0));
    }

    #[test]
    fn a_multibyte_glyph_is_not_split() {
        // `░` is three bytes and one cell; slicing by byte offset would panic
        // mid-character.
        let line = line_of("░░░░░");
        let (visible, caret) = line.viewport(3);
        assert_eq!(visible, "░░", "one cell is reserved for the caret");
        assert_eq!(caret, 2);
    }

    #[test]
    fn an_over_long_insert_fills_the_line_rather_than_doing_nothing() {
        // The cap used to drop the whole insert, so a paste longer than the
        // remaining room vanished.
        let mut line = Line::typed(&"x".repeat(MAX_INPUT - 2));
        line.insert("abcdef");
        assert_eq!(line.text().chars().count(), MAX_INPUT);
    }
}
