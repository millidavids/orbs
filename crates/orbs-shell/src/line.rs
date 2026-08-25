//! The line the player is typing.
//!
//! The buffer, the caret, the history, and what Tab and the ghost make of them.
//! Nothing here reads a keystroke — each frontend's own input path does that and calls
//! these — so the whole editor is testable without a window, which is what the
//! `Line`-only tests below rely on.
//!
//! # The caret counts characters, not bytes
//!
//! CP437 admits multi-byte glyphs that occupy exactly one cell (`░`, `Σ`), so a
//! byte caret would split one and panic on the slice. Every helper here counts
//! characters for the same reason, through [`orbs_render::char_index`] — shared
//! with `orbs-sim`'s completer, because the two have to agree about where a word
//! starts or Tab overwrites the wrong bytes.

use bevy_ecs::prelude::*;
use orbs_render::char_index;

/// The parser's own limit (§6). Matching it here means the buffer never holds
/// something the parser would truncate — a shorter cap would silently change
/// what a command means, and a longer one would let the player type into a void.
const MAX_INPUT: usize = 512;

/// How many lines of history are kept.
///
/// Far more than a session reaches, and it bounds the memory. History is not
/// persisted — it dies with the process, and giving it a home arrives with
/// Phase 11's settings, where §4's sticky skip is already waiting.
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
    /// Held for the whole search rather than re-read from the line each press:
    /// after one recall the line *is* the recalled command, so an unanchored
    /// second `Up` would search on that and behave differently from the first.
    /// fish and zsh both hold the original prefix; this is that.
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
    /// The companion to [`viewport`](Self::viewport), and it exists for exactly
    /// one caller: highlighting. `parser::lex` classifies partly by *position* —
    /// the first word of a line is the verb — so lexing the visible slice of a
    /// scrolled line would read a mid-line fragment as a line start, and mis-hue
    /// precisely the long lines that scrolled far enough to need the help.
    ///
    /// So the prompt lexes the whole line and maps the runs through this.
    #[must_use]
    pub fn window_starts(&self, width: u16) -> usize {
        // **`viewport(0)` is the one answer that is not a subslice.** It returns
        // the literal `""`, which lives in rodata with no relation to this
        // line's buffer, so the pointer arithmetic below would give 0 or a large
        // arbitrary number depending on where the allocator put things. The
        // 80×22 floor and `paint_too_small` keep the prompt from ever asking,
        // but this is `pub` and its doc promises an offset into the line.
        if width == 0 {
            return 0;
        }
        let (visible, _) = self.viewport(width);
        // A subslice of `self.text`, so its offset is the difference of the two
        // pointers — the same arithmetic `parser::lexeme` uses to keep two
        // identical words apart, and the only way to get it back out of a `&str`
        // without threading it through the return type of a function five tests
        // assert the shape of.
        (visible.as_ptr() as usize).saturating_sub(self.text.as_ptr() as usize)
    }

    /// The window of the line that fits `width` cells, and the caret's column
    /// within it.
    ///
    /// Without this the line has no viewport: at the 80×22 floor the prompt
    /// leaves 72 cells, past which `put_str` clips silently *and*
    /// [`Frame::set_cursor`](orbs_render::Frame::set_cursor) refuses an off-grid
    /// position — so the player types into a dead line with no caret and no
    /// explanation. `sift "march north" /tower/laboratory/feed.log` is 44
    /// characters, so 72 is not a theoretical limit, and the magnified prompt
    /// halves the budget again.
    ///
    /// It used to show the **tail** and put the caret at the end. With a caret
    /// that can sit anywhere, the window has to follow the *caret* — so it
    /// scrolls left as well as right, and one cell is always reserved so the
    /// caret at the end of a full line still has somewhere to be.
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
    /// Takes the whole string rather than a character at a time: winit documents
    /// a Windows dead key yielding **two** characters from one press, and
    /// inserting them one at a time at a moving caret would land them reversed.
    pub fn insert(&mut self, text: &str) {
        // **Truncated to what is left, not dropped whole.** Refusing the entire
        // insert when it would cross `MAX_INPUT` reintroduces the void this cap
        // exists to prevent: with 510 characters typed, a dead key yielding two
        // would do nothing at all, and a paste of any length over the remainder
        // would silently vanish rather than filling the line.
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
    /// is an answer, not a phrasing, and recalling one later would submit it as
    /// a command and land in `ParseLog` as a failed phrasing — diluting the very
    /// metric `answering_is_not_counted_as_a_phrasing` exists to protect.
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
    /// **Filtered by what is already typed**, which is fish's default and zsh's
    /// `history-beginning-search-backward`. Against a loop that runs
    /// `move / wield / siphon / purge` over and over, typing `wield ` and
    /// pressing `Up` should walk the wields — bash's positional walk is the
    /// worse of the two here.
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
    /// **A chain, history first**, which is fish's order and zsh's default
    /// (`ZSH_AUTOSUGGEST_STRATEGY=(history)`, with completion opt-in and
    /// documented as slow). History is the cheap half and usually the right one:
    /// a twelve-command brew loop is a player repeating themselves, and what
    /// they are about to type is nearly always what they typed before.
    ///
    /// Only ever offered with the caret at the end — a suggestion trailing text
    /// the player is editing in the middle would sit in the wrong place.
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
        // **The same `expect` Tab reads, and the same `common`.** The ghost is a
        // promise about what Tab will do, so a second opinion about either would
        // have it drawing text the key then does not take.
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
    /// spell editor. What stays here is applying the decision, because a caret
    /// is counted in characters at the prompt and in a row and a column in the
    /// editor, and a shared function that moved both would need to know both.
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
        // The failure this exists to prevent: past the pane width the text stops
        // appearing and `set_cursor` refuses an off-grid position, so the player
        // types into a dead line.
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
        // `░` is three bytes and one cell. Slicing the tail by byte offset would
        // panic mid-character; the viewport has to seek by `char_indices`.
        let line = line_of("░░░░░");
        let (visible, caret) = line.viewport(3);
        assert_eq!(visible, "░░", "one cell is reserved for the caret");
        assert_eq!(caret, 2);
    }

    #[test]
    fn an_over_long_insert_fills_the_line_rather_than_doing_nothing() {
        // The cap used to drop the whole insert, which reintroduces the void it
        // exists to prevent — a paste longer than the remaining room vanished.
        let mut line = Line::typed(&"x".repeat(MAX_INPUT - 2));
        line.insert("abcdef");
        assert_eq!(line.text().chars().count(), MAX_INPUT);
    }
}
